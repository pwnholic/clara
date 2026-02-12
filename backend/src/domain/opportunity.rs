use crate::domain::market::MatchedMarket;
use crate::domain::types::{Amount, Price, Side};
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ArbitrageType {
    PriceDiscrepancy,
    SpreadArbitrage,
    CrossPlatform,
}

impl ArbitrageType {
    pub fn as_str(&self) -> &'static str {
        match self {
            ArbitrageType::PriceDiscrepancy => "price_discrepancy",
            ArbitrageType::SpreadArbitrage => "spread_arbitrage",
            ArbitrageType::CrossPlatform => "cross_platform",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OpportunityStatus {
    Detected,
    Validated,
    Executing,
    Completed,
    Failed,
    Expired,
}

impl OpportunityStatus {
    pub fn is_actionable(&self) -> bool {
        matches!(
            self,
            OpportunityStatus::Detected | OpportunityStatus::Validated
        )
    }

    pub fn is_terminal(&self) -> bool {
        matches!(
            self,
            OpportunityStatus::Completed | OpportunityStatus::Failed | OpportunityStatus::Expired
        )
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArbitrageOpportunity {
    pub id: String,
    pub opportunity_type: ArbitrageType,
    pub status: OpportunityStatus,
    pub matched_market: MatchedMarket,
    pub buy_platform: String,
    pub buy_side: Side,
    pub buy_market_id: String,
    #[serde(with = "rust_decimal::serde::arbitrary_precision")]
    pub buy_price: Decimal,
    pub sell_platform: String,
    pub sell_side: Side,
    pub sell_market_id: String,
    #[serde(with = "rust_decimal::serde::arbitrary_precision")]
    pub sell_price: Decimal,
    #[serde(with = "rust_decimal::serde::arbitrary_precision")]
    pub position_size: Decimal,
    #[serde(with = "rust_decimal::serde::arbitrary_precision")]
    pub gross_profit: Decimal,
    #[serde(with = "rust_decimal::serde::arbitrary_precision")]
    pub estimated_fees: Decimal,
    #[serde(with = "rust_decimal::serde::arbitrary_precision")]
    pub net_profit: Decimal,
    #[serde(with = "rust_decimal::serde::arbitrary_precision")]
    pub profit_percentage: Decimal,
    #[serde(with = "rust_decimal::serde::arbitrary_precision")]
    pub confidence_score: Decimal,
    pub detected_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
    pub executed_at: Option<DateTime<Utc>>,
}

impl ArbitrageOpportunity {
    pub fn new(
        matched_market: MatchedMarket,
        position_size: Decimal,
        estimated_fees: Decimal,
    ) -> Self {
        let kalshi = &matched_market.kalshi_market;
        let polymarket = &matched_market.polymarket_market;

        let (
            buy_platform,
            buy_side,
            buy_market_id,
            buy_price,
            sell_platform,
            sell_side,
            sell_market_id,
            sell_price,
        ) = Self::determine_optimal_route(kalshi, polymarket);

        let gross_profit = Self::calculate_gross_profit(buy_price, sell_price, position_size);
        let net_profit = gross_profit - estimated_fees;
        let profit_percentage = if position_size > Decimal::ZERO {
            (net_profit / position_size) * Decimal::ONE_HUNDRED
        } else {
            Decimal::ZERO
        };

        let id = format!(
            "{}-{}-{}",
            matched_market.kalshi_ticker,
            matched_market.polymarket_condition_id,
            Utc::now().timestamp_millis()
        );

        let confidence_score = matched_market.match_confidence;

        ArbitrageOpportunity {
            id,
            opportunity_type: ArbitrageType::CrossPlatform,
            status: OpportunityStatus::Detected,
            matched_market,
            buy_platform,
            buy_side,
            buy_market_id,
            buy_price,
            sell_platform,
            sell_side,
            sell_market_id,
            sell_price,
            position_size,
            gross_profit,
            estimated_fees,
            net_profit,
            profit_percentage,
            confidence_score,
            detected_at: Utc::now(),
            expires_at: Some(Utc::now() + chrono::Duration::seconds(30)),
            executed_at: None,
        }
    }

    fn determine_optimal_route(
        kalshi: &crate::domain::market::KalshiMarket,
        polymarket: &crate::domain::market::PolymarketMarket,
    ) -> (String, Side, String, Decimal, String, Side, String, Decimal) {
        let kalshi_yes = kalshi.yes_price;
        let kalshi_no = kalshi.no_price;
        let poly_yes = polymarket.yes_price;
        let poly_no = polymarket.no_price;

        let yes_sum = kalshi_yes + poly_yes;
        let no_sum = kalshi_no + poly_no;

        if yes_sum < no_sum {
            let cheaper_platform = if kalshi_yes < poly_yes {
                "kalshi"
            } else {
                "polymarket"
            };
            let expensive_platform = if kalshi_yes < poly_yes {
                "polymarket"
            } else {
                "kalshi"
            };

            let (buy_market_id, sell_market_id) = if kalshi_yes < poly_yes {
                (kalshi.ticker.clone(), polymarket.condition_id.clone())
            } else {
                (polymarket.condition_id.clone(), kalshi.ticker.clone())
            };

            let (buy_price, sell_price) = if kalshi_yes < poly_yes {
                (kalshi_yes, Decimal::ONE - poly_no)
            } else {
                (poly_yes, Decimal::ONE - kalshi_no)
            };

            (
                cheaper_platform.to_string(),
                Side::Yes,
                buy_market_id,
                buy_price,
                expensive_platform.to_string(),
                Side::No,
                sell_market_id,
                sell_price,
            )
        } else {
            let cheaper_platform = if kalshi_no < poly_no {
                "kalshi"
            } else {
                "polymarket"
            };
            let expensive_platform = if kalshi_no < poly_no {
                "polymarket"
            } else {
                "kalshi"
            };

            let (buy_market_id, sell_market_id) = if kalshi_no < poly_no {
                (kalshi.ticker.clone(), polymarket.condition_id.clone())
            } else {
                (polymarket.condition_id.clone(), kalshi.ticker.clone())
            };

            let (buy_price, sell_price) = if kalshi_no < poly_no {
                (kalshi_no, Decimal::ONE - poly_yes)
            } else {
                (poly_no, Decimal::ONE - kalshi_yes)
            };

            (
                cheaper_platform.to_string(),
                Side::No,
                buy_market_id,
                buy_price,
                expensive_platform.to_string(),
                Side::Yes,
                sell_market_id,
                sell_price,
            )
        }
    }

    fn calculate_gross_profit(
        buy_price: Decimal,
        sell_price: Decimal,
        position_size: Decimal,
    ) -> Decimal {
        let cost = buy_price * position_size;
        let revenue = sell_price * position_size;
        revenue - cost
    }

    pub fn buy_price_value(&self) -> Price {
        Price::new(self.buy_price).unwrap_or(Price::ZERO)
    }

    pub fn sell_price_value(&self) -> Price {
        Price::new(self.sell_price).unwrap_or(Price::ZERO)
    }

    pub fn position_size_value(&self) -> Amount {
        Amount::new(self.position_size)
    }

    pub fn gross_profit_value(&self) -> Amount {
        Amount::new(self.gross_profit)
    }

    pub fn net_profit_value(&self) -> Amount {
        Amount::new(self.net_profit)
    }

    pub fn estimated_fees_value(&self) -> Amount {
        Amount::new(self.estimated_fees)
    }

    pub fn is_profitable(&self, min_profit: Decimal) -> bool {
        self.net_profit > min_profit && self.profit_percentage > Decimal::ZERO
    }

    pub fn is_expired(&self) -> bool {
        self.expires_at.map(|e| Utc::now() > e).unwrap_or(false)
    }

    pub fn mark_validated(&mut self) {
        self.status = OpportunityStatus::Validated;
    }

    pub fn mark_executing(&mut self) {
        self.status = OpportunityStatus::Executing;
    }

    pub fn mark_completed(&mut self) {
        self.status = OpportunityStatus::Completed;
        self.executed_at = Some(Utc::now());
    }

    pub fn mark_failed(&mut self) {
        self.status = OpportunityStatus::Failed;
    }

    pub fn mark_expired(&mut self) {
        self.status = OpportunityStatus::Expired;
    }

    pub fn update_position_size(&mut self, new_size: Decimal, new_fees: Decimal) {
        self.position_size = new_size;
        self.estimated_fees = new_fees;
        self.gross_profit = Self::calculate_gross_profit(self.buy_price, self.sell_price, new_size);
        self.net_profit = self.gross_profit - new_fees;
        self.profit_percentage = if new_size > Decimal::ZERO {
            (self.net_profit / new_size) * Decimal::ONE_HUNDRED
        } else {
            Decimal::ZERO
        };
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpportunitySummary {
    pub id: String,
    pub kalshi_ticker: String,
    pub polymarket_condition_id: String,
    pub profit_percentage: Decimal,
    pub net_profit: Decimal,
    pub position_size: Decimal,
    pub status: OpportunityStatus,
    pub detected_at: DateTime<Utc>,
}

impl From<&ArbitrageOpportunity> for OpportunitySummary {
    fn from(opp: &ArbitrageOpportunity) -> Self {
        OpportunitySummary {
            id: opp.id.clone(),
            kalshi_ticker: opp.matched_market.kalshi_ticker.clone(),
            polymarket_condition_id: opp.matched_market.polymarket_condition_id.clone(),
            profit_percentage: opp.profit_percentage,
            net_profit: opp.net_profit,
            position_size: opp.position_size,
            status: opp.status,
            detected_at: opp.detected_at,
        }
    }
}
