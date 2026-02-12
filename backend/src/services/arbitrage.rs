use std::sync::Arc;
use std::time::Duration;

use anyhow::Result;
use num_traits::ToPrimitive;
use rust_decimal::Decimal;
use tokio::sync::{broadcast, RwLock};
use tracing::{debug, error, info, instrument, trace, warn};

use crate::config::Config;
use crate::domain::{ArbitrageOpportunity, KalshiMarket, MatchedMarket, PolymarketMarket, Side};
use crate::services::{ArbitrageCalculator, MarketMatcher};
use crate::storage::Storage;

#[derive(Debug, Clone)]
pub struct ExecutionResult {
    pub success: bool,
    pub buy_order_id: Option<String>,
    pub sell_order_id: Option<String>,
    pub profit_realized: Decimal,
    pub error_message: Option<String>,
}

#[derive(Debug, Clone)]
pub struct PriceFeed {
    pub platform: String,
    pub market_id: String,
    pub yes_price: Decimal,
    pub no_price: Decimal,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

pub struct KalshiClient {
    api_url: String,
    api_key: String,
    rate_limit: u32,
}

impl KalshiClient {
    pub fn new(api_url: &str, api_key: &str, rate_limit: u32) -> Self {
        Self {
            api_url: api_url.to_string(),
            api_key: api_key.to_string(),
            rate_limit,
        }
    }

    pub async fn get_markets(&self) -> Result<Vec<KalshiMarket>> {
        debug!(api_url = %self.api_url, "Fetching Kalshi markets");
        Ok(Vec::new())
    }

    pub async fn subscribe_prices(
        &self,
        _tickers: Vec<String>,
        _tx: broadcast::Sender<PriceFeed>,
    ) -> Result<()> {
        debug!("Subscribing to Kalshi price feeds");
        Ok(())
    }

    pub async fn place_order(
        &self,
        _ticker: &str,
        _side: &str,
        _outcome: &str,
        _count: i32,
        _price: i32,
    ) -> Result<serde_json::Value> {
        debug!("Placing Kalshi order");
        Ok(serde_json::json!({"order_id": "k-order-123"}))
    }
}

pub struct PolymarketClient {
    api_url: String,
    chain_id: u64,
}

impl PolymarketClient {
    pub fn new(api_url: &str, chain_id: u64) -> Self {
        Self {
            api_url: api_url.to_string(),
            chain_id,
        }
    }

    pub async fn get_markets(&self) -> Result<Vec<PolymarketMarket>> {
        debug!(api_url = %self.api_url, "Fetching Polymarket markets");
        Ok(Vec::new())
    }

    pub async fn subscribe_prices(
        &self,
        _token_ids: Vec<String>,
        _tx: broadcast::Sender<PriceFeed>,
    ) -> Result<()> {
        debug!("Subscribing to Polymarket price feeds");
        Ok(())
    }

    pub async fn place_market_order(
        &self,
        _token_id: &str,
        _side: &str,
        _amount: f64,
    ) -> Result<serde_json::Value> {
        debug!("Placing Polymarket order");
        Ok(serde_json::json!({"order_id": "p-order-456"}))
    }
}

pub struct ArbitrageService {
    pub kalshi: Arc<KalshiClient>,
    pub polymarket: Arc<PolymarketClient>,
    pub calculator: ArbitrageCalculator,
    pub matcher: MarketMatcher,
    pub storage: Arc<Storage>,
    pub matched_markets: Arc<RwLock<Vec<MatchedMarket>>>,
    pub opportunities: Arc<RwLock<Vec<ArbitrageOpportunity>>>,
    pub price_tx: broadcast::Sender<PriceFeed>,
    pub min_profit_threshold: Decimal,
    pub max_position_size: Decimal,
    pub execution_timeout_ms: u64,
    pub retry_attempts: u32,
    pub retry_delay_ms: u64,
    pub running: Arc<RwLock<bool>>,
}

impl ArbitrageService {
    pub fn new(
        kalshi: Arc<KalshiClient>,
        polymarket: Arc<PolymarketClient>,
        storage: Arc<Storage>,
        config: &Config,
    ) -> Self {
        let (price_tx, _) = broadcast::channel(1024);

        Self {
            kalshi,
            polymarket,
            calculator: ArbitrageCalculator::new(
                config.arbitrage.min_profit_threshold,
                Decimal::new(7, 2),
            ),
            matcher: MarketMatcher::new(24),
            storage,
            matched_markets: Arc::new(RwLock::new(Vec::new())),
            opportunities: Arc::new(RwLock::new(Vec::new())),
            price_tx,
            min_profit_threshold: config.arbitrage.min_profit_threshold,
            max_position_size: config.arbitrage.max_position_size,
            execution_timeout_ms: config.arbitrage.execution_timeout_ms,
            retry_attempts: config.arbitrage.retry_attempts,
            retry_delay_ms: config.arbitrage.retry_delay_ms,
            running: Arc::new(RwLock::new(false)),
        }
    }

    pub async fn initialize(&mut self) -> Result<()> {
        info!("Initializing arbitrage service");

        let kalshi_markets = self.kalshi.get_markets().await?;
        let poly_markets = self.polymarket.get_markets().await?;

        info!(
            kalshi_markets = kalshi_markets.len(),
            poly_markets = poly_markets.len(),
            "Fetched markets from both platforms"
        );

        let matched = self.matcher.match_markets(kalshi_markets, poly_markets);

        let mut markets = self.matched_markets.write().await;
        *markets = matched;

        info!(
            matched_count = markets.len(),
            "Arbitrage service initialized"
        );

        Ok(())
    }

    #[instrument(skip(self))]
    pub async fn scan_opportunities(&self) -> Result<Vec<ArbitrageOpportunity>> {
        trace!("Scanning for arbitrage opportunities");

        let markets = self.matched_markets.read().await;
        let mut found_opportunities = Vec::new();

        for mm in markets.iter() {
            if !mm.is_tradable() {
                continue;
            }

            let kalshi_yes = mm.kalshi_market.yes_price;
            let kalshi_no = mm.kalshi_market.no_price;
            let poly_yes = mm.polymarket_market.yes_price;
            let poly_no = mm.polymarket_market.no_price;

            if let Some(calc) = self.calculator.calculate(kalshi_yes, poly_no, Side::Yes) {
                if self.calculator.is_profitable(calc.profit_margin) {
                    let position_size = mm.calculate_max_position(self.max_position_size);
                    let opp = ArbitrageOpportunity::new(mm.clone(), position_size, calc.kalshi_fee);
                    found_opportunities.push(opp);
                }
            }

            if let Some(calc) = self.calculator.calculate(kalshi_no, poly_yes, Side::No) {
                if self.calculator.is_profitable(calc.profit_margin) {
                    let position_size = mm.calculate_max_position(self.max_position_size);
                    let opp = ArbitrageOpportunity::new(mm.clone(), position_size, calc.kalshi_fee);
                    found_opportunities.push(opp);
                }
            }
        }

        found_opportunities.sort_by(|a, b| b.profit_percentage.cmp(&a.profit_percentage));

        let mut opps = self.opportunities.write().await;
        *opps = found_opportunities.clone();

        debug!(
            opportunities_found = found_opportunities.len(),
            "Scan complete"
        );

        Ok(found_opportunities)
    }

    #[instrument(skip(self))]
    pub async fn execute_arbitrage(
        &self,
        opportunity: &ArbitrageOpportunity,
    ) -> Result<ExecutionResult> {
        info!(
            opportunity_id = %opportunity.id,
            profit_percentage = %opportunity.profit_percentage,
            "Executing arbitrage"
        );

        if opportunity.net_profit > self.max_position_size {
            warn!(
                net_profit = %opportunity.net_profit,
                max_position = %self.max_position_size,
                "Position size exceeds maximum"
            );
            return Ok(ExecutionResult {
                success: false,
                buy_order_id: None,
                sell_order_id: None,
                profit_realized: Decimal::ZERO,
                error_message: Some("Position size exceeds maximum".to_string()),
            });
        }

        let buy_result = self
            .place_order_with_retry(
                &opportunity.buy_platform,
                &opportunity.buy_market_id,
                &opportunity.buy_side,
                opportunity.position_size,
                opportunity.buy_price,
            )
            .await;

        let buy_order_id = match buy_result {
            Ok(id) => Some(id),
            Err(e) => {
                error!(error = %e, "Buy order failed");
                return Ok(ExecutionResult {
                    success: false,
                    buy_order_id: None,
                    sell_order_id: None,
                    profit_realized: Decimal::ZERO,
                    error_message: Some(format!("Buy order failed: {}", e)),
                });
            }
        };

        let sell_result = self
            .place_order_with_retry(
                &opportunity.sell_platform,
                &opportunity.sell_market_id,
                &opportunity.sell_side,
                opportunity.position_size,
                opportunity.sell_price,
            )
            .await;

        let sell_order_id = match sell_result {
            Ok(id) => Some(id),
            Err(e) => {
                error!(error = %e, "Sell order failed");
                return Ok(ExecutionResult {
                    success: false,
                    buy_order_id,
                    sell_order_id: None,
                    profit_realized: Decimal::ZERO,
                    error_message: Some(format!("Sell order failed: {}", e)),
                });
            }
        };

        info!(
            buy_order = ?buy_order_id,
            sell_order = ?sell_order_id,
            profit = %opportunity.net_profit,
            "Arbitrage executed successfully"
        );

        Ok(ExecutionResult {
            success: true,
            buy_order_id,
            sell_order_id,
            profit_realized: opportunity.net_profit,
            error_message: None,
        })
    }

    async fn place_order_with_retry(
        &self,
        platform: &str,
        market_id: &str,
        side: &Side,
        amount: Decimal,
        price: Decimal,
    ) -> Result<String> {
        let mut attempts = 0;

        while attempts < self.retry_attempts {
            let result = if platform == "kalshi" {
                let count = amount.to_i32().unwrap_or(100);
                let price_cents = (price * Decimal::ONE_HUNDRED).to_i32().unwrap_or(50);
                self.kalshi
                    .place_order(market_id, "buy", side.as_str(), count, price_cents)
                    .await
            } else {
                let amount_f64 = amount.to_f64().unwrap_or(100.0);
                self.polymarket
                    .place_market_order(market_id, side.as_str(), amount_f64)
                    .await
            };

            match result {
                Ok(response) => {
                    return Ok(response
                        .get("order_id")
                        .and_then(|v| v.as_str())
                        .unwrap_or("unknown")
                        .to_string());
                }
                Err(e) => {
                    attempts += 1;
                    if attempts < self.retry_attempts {
                        warn!(
                            attempt = attempts,
                            max_attempts = self.retry_attempts,
                            error = %e,
                            "Order failed, retrying"
                        );
                        tokio::time::sleep(Duration::from_millis(self.retry_delay_ms)).await;
                    } else {
                        return Err(e);
                    }
                }
            }
        }

        Err(anyhow::anyhow!("Max retry attempts exceeded"))
    }

    pub async fn start_price_feeds(&mut self) -> Result<()> {
        info!("Starting price feeds");

        let mut running = self.running.write().await;
        *running = true;
        drop(running);

        let markets = self.matched_markets.read().await;
        let (kalshi_tickers, poly_tokens) = self.matcher.get_subscription_info(&markets);
        drop(markets);

        info!(
            kalshi_subscriptions = kalshi_tickers.len(),
            poly_subscriptions = poly_tokens.len(),
            "Subscribing to price feeds"
        );

        let kalshi_client = self.kalshi.clone();
        let kalshi_tickers_clone = kalshi_tickers.clone();
        let price_tx_kalshi = self.price_tx.clone();

        tokio::spawn(async move {
            if let Err(e) = kalshi_client
                .subscribe_prices(kalshi_tickers_clone, price_tx_kalshi)
                .await
            {
                error!(error = %e, "Kalshi price feed error");
            }
        });

        let poly_client = self.polymarket.clone();
        let poly_tokens_clone = poly_tokens.clone();
        let price_tx_poly = self.price_tx.clone();

        tokio::spawn(async move {
            if let Err(e) = poly_client
                .subscribe_prices(poly_tokens_clone, price_tx_poly)
                .await
            {
                error!(error = %e, "Polymarket price feed error");
            }
        });

        Ok(())
    }

    pub fn subscribe_prices(&self) -> broadcast::Receiver<PriceFeed> {
        self.price_tx.subscribe()
    }

    pub async fn handle_price_update(&self, feed: PriceFeed) {
        trace!(
            platform = %feed.platform,
            market_id = %feed.market_id,
            yes_price = %feed.yes_price,
            no_price = %feed.no_price,
            "Handling price update"
        );

        let opportunities = self.scan_opportunities().await.unwrap_or_default();

        if !opportunities.is_empty() {
            debug!(
                count = opportunities.len(),
                best_margin = %opportunities.first().map(|o| o.profit_percentage).unwrap_or(Decimal::ZERO),
                "Opportunities after price update"
            );
        }
    }

    pub async fn get_opportunities(&self) -> Vec<ArbitrageOpportunity> {
        self.opportunities.read().await.clone()
    }

    pub async fn get_matched_markets(&self) -> Vec<MatchedMarket> {
        self.matched_markets.read().await.clone()
    }

    pub async fn stop(&self) {
        let mut running = self.running.write().await;
        *running = false;
        info!("Arbitrage service stopped");
    }

    pub async fn is_running(&self) -> bool {
        *self.running.read().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_execution_result_success() {
        let result = ExecutionResult {
            success: true,
            buy_order_id: Some("k123".to_string()),
            sell_order_id: Some("p456".to_string()),
            profit_realized: Decimal::new(5, 1),
            error_message: None,
        };

        assert!(result.success);
        assert_eq!(result.buy_order_id, Some("k123".to_string()));
        assert_eq!(result.profit_realized, Decimal::new(5, 1));
    }

    #[test]
    fn test_execution_result_failure() {
        let result = ExecutionResult {
            success: false,
            buy_order_id: None,
            sell_order_id: None,
            profit_realized: Decimal::ZERO,
            error_message: Some("Order failed".to_string()),
        };

        assert!(!result.success);
        assert!(result.error_message.is_some());
    }

    #[test]
    fn test_price_feed() {
        let feed = PriceFeed {
            platform: "kalshi".to_string(),
            market_id: "KX-NBA-LAL".to_string(),
            yes_price: Decimal::new(45, 2),
            no_price: Decimal::new(55, 2),
            timestamp: chrono::Utc::now(),
        };

        assert_eq!(feed.platform, "kalshi");
        assert_eq!(feed.yes_price, Decimal::new(45, 2));
    }

    #[test]
    fn test_kalshi_client_new() {
        let client = KalshiClient::new("https://api.kalshi.com", "test-key", 10);
        assert_eq!(client.api_url, "https://api.kalshi.com");
        assert_eq!(client.api_key, "test-key");
        assert_eq!(client.rate_limit, 10);
    }

    #[test]
    fn test_polymarket_client_new() {
        let client = PolymarketClient::new("https://clob.polymarket.com", 137);
        assert_eq!(client.api_url, "https://clob.polymarket.com");
        assert_eq!(client.chain_id, 137);
    }
}
