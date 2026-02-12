use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

use parking_lot::Mutex;
use rust_decimal::Decimal;
use tokio::sync::{broadcast, RwLock};

use crate::clients::{KalshiClient, PolymarketClient};
use crate::domain::ArbitrageOpportunity;
use crate::services::ArbitrageService;
use crate::storage::Storage;

pub struct AppState {
    pub service: Arc<RwLock<ArbitrageService>>,
    pub storage: Arc<Storage>,
    pub opportunities: Arc<RwLock<Vec<ArbitrageOpportunity>>>,
    pub ws_broadcast: broadcast::Sender<WsMessage>,
    pub metrics: Arc<Metrics>,
    pub kalshi: Arc<KalshiClient>,
    pub polymarket: Arc<PolymarketClient>,
}

#[derive(Debug)]
pub struct AtomicDecimal {
    value: Mutex<Decimal>,
}

impl Clone for AtomicDecimal {
    fn clone(&self) -> Self {
        Self::new(self.load())
    }
}

impl AtomicDecimal {
    pub fn new(value: Decimal) -> Self {
        Self {
            value: Mutex::new(value),
        }
    }

    pub fn load(&self) -> Decimal {
        *self.value.lock()
    }

    pub fn store(&self, value: Decimal) {
        *self.value.lock() = value;
    }
}

impl Default for AtomicDecimal {
    fn default() -> Self {
        Self::new(Decimal::ZERO)
    }
}

pub struct Metrics {
    pub kalshi_balance: AtomicDecimal,
    pub poly_balance: AtomicDecimal,
    pub total_trades: AtomicU64,
    pub total_profit: AtomicDecimal,
}

impl Metrics {
    pub fn new() -> Self {
        Self {
            kalshi_balance: AtomicDecimal::new(Decimal::ZERO),
            poly_balance: AtomicDecimal::new(Decimal::ZERO),
            total_trades: AtomicU64::new(0),
            total_profit: AtomicDecimal::new(Decimal::ZERO),
        }
    }

    pub fn record_trade(&self, profit: Decimal) {
        self.total_trades.fetch_add(1, Ordering::Relaxed);
        let mut current = self.total_profit.value.lock();
        *current += profit;
    }

    pub fn set_kalshi_balance(&self, balance: Decimal) {
        self.kalshi_balance.store(balance);
    }

    pub fn set_poly_balance(&self, balance: Decimal) {
        self.poly_balance.store(balance);
    }

    pub fn get_balances(&self) -> (Decimal, Decimal) {
        (self.kalshi_balance.load(), self.poly_balance.load())
    }
}

impl Default for Metrics {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum WsMessage {
    Opportunity {
        data: ArbitrageOpportunity,
    },
    Opportunities {
        data: Vec<ArbitrageOpportunity>,
    },
    PriceUpdate {
        market_id: String,
        platform: String,
        price: String,
    },
    Metrics {
        kalshi_balance: String,
        poly_balance: String,
        total_trades: u64,
        total_profit: String,
    },
    AutoTradeStatus {
        enabled: bool,
        trade_count: u32,
    },
    Error {
        message: String,
    },
}

impl AppState {
    pub fn new(
        service: ArbitrageService,
        storage: Arc<Storage>,
        kalshi: KalshiClient,
        polymarket: PolymarketClient,
    ) -> Self {
        let (ws_broadcast, _) = broadcast::channel(1000);
        Self {
            service: Arc::new(RwLock::new(service)),
            storage,
            opportunities: Arc::new(RwLock::new(Vec::new())),
            ws_broadcast,
            metrics: Arc::new(Metrics::new()),
            kalshi: Arc::new(kalshi),
            polymarket: Arc::new(polymarket),
        }
    }

    pub fn broadcast(&self, msg: WsMessage) {
        let _ = self.ws_broadcast.send(msg);
    }
}
