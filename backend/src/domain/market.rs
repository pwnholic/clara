use crate::domain::types::{Amount, MarketStatus, Price};
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KalshiMarket {
    pub ticker: String,
    pub event_ticker: String,
    pub title: String,
    pub subtitle: Option<String>,
    pub status: MarketStatus,
    #[serde(with = "rust_decimal::serde::arbitrary_precision")]
    pub yes_price: Decimal,
    #[serde(with = "rust_decimal::serde::arbitrary_precision")]
    pub no_price: Decimal,
    #[serde(with = "rust_decimal::serde::arbitrary_precision")]
    pub yes_bid: Decimal,
    #[serde(with = "rust_decimal::serde::arbitrary_precision")]
    pub yes_ask: Decimal,
    #[serde(with = "rust_decimal::serde::arbitrary_precision")]
    pub no_bid: Decimal,
    #[serde(with = "rust_decimal::serde::arbitrary_precision")]
    pub no_ask: Decimal,
    #[serde(with = "rust_decimal::serde::arbitrary_precision")]
    pub volume: Decimal,
    #[serde(with = "rust_decimal::serde::arbitrary_precision")]
    pub open_interest: Decimal,
    pub close_time: DateTime<Utc>,
    pub created_time: DateTime<Utc>,
    pub category: String,
}

impl KalshiMarket {
    pub fn yes_price(&self) -> Price {
        Price::new(self.yes_price).unwrap_or(Price::ZERO)
    }

    pub fn no_price(&self) -> Price {
        Price::new(self.no_price).unwrap_or(Price::ZERO)
    }

    pub fn yes_bid_price(&self) -> Price {
        Price::new(self.yes_bid).unwrap_or(Price::ZERO)
    }

    pub fn yes_ask_price(&self) -> Price {
        Price::new(self.yes_ask).unwrap_or(Price::MAX)
    }

    pub fn no_bid_price(&self) -> Price {
        Price::new(self.no_bid).unwrap_or(Price::ZERO)
    }

    pub fn no_ask_price(&self) -> Price {
        Price::new(self.no_ask).unwrap_or(Price::MAX)
    }

    pub fn volume_amount(&self) -> Amount {
        Amount::new(self.volume)
    }

    pub fn open_interest_amount(&self) -> Amount {
        Amount::new(self.open_interest)
    }

    pub fn is_tradable(&self) -> bool {
        self.status.is_tradable()
    }

    pub fn spread(&self) -> Decimal {
        self.yes_ask - self.yes_bid
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolymarketMarket {
    pub condition_id: String,
    pub token_id_yes: String,
    pub token_id_no: String,
    pub question: String,
    pub description: Option<String>,
    pub status: MarketStatus,
    #[serde(with = "rust_decimal::serde::arbitrary_precision")]
    pub yes_price: Decimal,
    #[serde(with = "rust_decimal::serde::arbitrary_precision")]
    pub no_price: Decimal,
    #[serde(with = "rust_decimal::serde::arbitrary_precision")]
    pub yes_bid: Decimal,
    #[serde(with = "rust_decimal::serde::arbitrary_precision")]
    pub yes_ask: Decimal,
    #[serde(with = "rust_decimal::serde::arbitrary_precision")]
    pub no_bid: Decimal,
    #[serde(with = "rust_decimal::serde::arbitrary_precision")]
    pub no_ask: Decimal,
    #[serde(with = "rust_decimal::serde::arbitrary_precision")]
    pub volume: Decimal,
    #[serde(with = "rust_decimal::serde::arbitrary_precision")]
    pub liquidity: Decimal,
    pub end_date: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub category: String,
    pub active: bool,
}

impl PolymarketMarket {
    pub fn yes_price(&self) -> Price {
        Price::new(self.yes_price).unwrap_or(Price::ZERO)
    }

    pub fn no_price(&self) -> Price {
        Price::new(self.no_price).unwrap_or(Price::ZERO)
    }

    pub fn yes_bid_price(&self) -> Price {
        Price::new(self.yes_bid).unwrap_or(Price::ZERO)
    }

    pub fn yes_ask_price(&self) -> Price {
        Price::new(self.yes_ask).unwrap_or(Price::MAX)
    }

    pub fn no_bid_price(&self) -> Price {
        Price::new(self.no_bid).unwrap_or(Price::ZERO)
    }

    pub fn no_ask_price(&self) -> Price {
        Price::new(self.no_ask).unwrap_or(Price::MAX)
    }

    pub fn volume_amount(&self) -> Amount {
        Amount::new(self.volume)
    }

    pub fn liquidity_amount(&self) -> Amount {
        Amount::new(self.liquidity)
    }

    pub fn is_tradable(&self) -> bool {
        self.active && self.status.is_tradable()
    }

    pub fn spread(&self) -> Decimal {
        self.yes_ask - self.yes_bid
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MatchedMarket {
    pub kalshi_ticker: String,
    pub polymarket_condition_id: String,
    pub kalshi_market: KalshiMarket,
    pub polymarket_market: PolymarketMarket,
    pub match_confidence: Decimal,
    pub matched_at: DateTime<Utc>,
}

impl MatchedMarket {
    pub fn new(
        kalshi_market: KalshiMarket,
        polymarket_market: PolymarketMarket,
        match_confidence: Decimal,
    ) -> Self {
        MatchedMarket {
            kalshi_ticker: kalshi_market.ticker.clone(),
            polymarket_condition_id: polymarket_market.condition_id.clone(),
            kalshi_market,
            polymarket_market,
            match_confidence,
            matched_at: Utc::now(),
        }
    }

    pub fn price_discrepancy(&self) -> Decimal {
        (self.kalshi_market.yes_price - self.polymarket_market.yes_price).abs()
    }

    pub fn has_arbitrage_opportunity(&self, min_profit: Decimal) -> bool {
        let kalshi_yes = self.kalshi_market.yes_price;
        let kalshi_no = self.kalshi_market.no_price;
        let poly_yes = self.polymarket_market.yes_price;
        let poly_no = self.polymarket_market.no_price;

        let arb_yes = kalshi_yes + poly_yes;
        let arb_no = kalshi_no + poly_no;

        arb_yes < Decimal::ONE - min_profit || arb_no < Decimal::ONE - min_profit
    }

    pub fn is_tradable(&self) -> bool {
        self.kalshi_market.is_tradable() && self.polymarket_market.is_tradable()
    }

    pub fn calculate_max_position(&self, max_position: Decimal) -> Decimal {
        let kalshi_volume = self.kalshi_market.volume;
        let poly_liquidity = self.polymarket_market.liquidity;

        max_position
            .min(kalshi_volume / Decimal::from(10))
            .min(poly_liquidity / Decimal::from(10))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketSummary {
    pub platform: String,
    pub identifier: String,
    pub title: String,
    pub yes_price: Price,
    pub no_price: Price,
    pub spread: Decimal,
    pub volume: Amount,
    pub status: MarketStatus,
}

impl From<&KalshiMarket> for MarketSummary {
    fn from(market: &KalshiMarket) -> Self {
        MarketSummary {
            platform: "kalshi".to_string(),
            identifier: market.ticker.clone(),
            title: market.title.clone(),
            yes_price: market.yes_price(),
            no_price: market.no_price(),
            spread: market.spread(),
            volume: market.volume_amount(),
            status: market.status,
        }
    }
}

impl From<&PolymarketMarket> for MarketSummary {
    fn from(market: &PolymarketMarket) -> Self {
        MarketSummary {
            platform: "polymarket".to_string(),
            identifier: market.condition_id.clone(),
            title: market.question.clone(),
            yes_price: market.yes_price(),
            no_price: market.no_price(),
            spread: market.spread(),
            volume: market.volume_amount(),
            status: market.status,
        }
    }
}
