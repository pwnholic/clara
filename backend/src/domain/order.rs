use crate::domain::types::{Amount, OrderType, Price, Side};
use chrono::{DateTime, Utc};
use num_traits::ToPrimitive;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OrderStatus {
    Pending,
    Open,
    PartiallyFilled,
    Filled,
    Cancelled,
    Rejected,
    Expired,
}

impl OrderStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            OrderStatus::Pending => "pending",
            OrderStatus::Open => "open",
            OrderStatus::PartiallyFilled => "partially_filled",
            OrderStatus::Filled => "filled",
            OrderStatus::Cancelled => "cancelled",
            OrderStatus::Rejected => "rejected",
            OrderStatus::Expired => "expired",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "pending" => Some(OrderStatus::Pending),
            "open" => Some(OrderStatus::Open),
            "partially_filled" => Some(OrderStatus::PartiallyFilled),
            "filled" => Some(OrderStatus::Filled),
            "cancelled" => Some(OrderStatus::Cancelled),
            "rejected" => Some(OrderStatus::Rejected),
            "expired" => Some(OrderStatus::Expired),
            _ => None,
        }
    }

    pub fn is_terminal(&self) -> bool {
        matches!(
            self,
            OrderStatus::Filled
                | OrderStatus::Cancelled
                | OrderStatus::Rejected
                | OrderStatus::Expired
        )
    }

    pub fn is_active(&self) -> bool {
        matches!(
            self,
            OrderStatus::Pending | OrderStatus::Open | OrderStatus::PartiallyFilled
        )
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderRequest {
    pub platform: String,
    pub market_id: String,
    pub side: Side,
    pub order_type: OrderType,
    #[serde(with = "rust_decimal::serde::arbitrary_precision")]
    pub price: Decimal,
    #[serde(with = "rust_decimal::serde::arbitrary_precision")]
    pub amount: Decimal,
    pub client_order_id: Option<String>,
    pub expiration: Option<DateTime<Utc>>,
}

impl OrderRequest {
    pub fn new(
        platform: String,
        market_id: String,
        side: Side,
        order_type: OrderType,
        price: Decimal,
        amount: Decimal,
    ) -> Self {
        OrderRequest {
            platform,
            market_id,
            side,
            order_type,
            price,
            amount,
            client_order_id: None,
            expiration: None,
        }
    }

    pub fn with_client_order_id(mut self, id: String) -> Self {
        self.client_order_id = Some(id);
        self
    }

    pub fn with_expiration(mut self, expiration: DateTime<Utc>) -> Self {
        self.expiration = Some(expiration);
        self
    }

    pub fn price_value(&self) -> Price {
        Price::new(self.price).unwrap_or(Price::ZERO)
    }

    pub fn amount_value(&self) -> Amount {
        Amount::new(self.amount)
    }

    pub fn total_value(&self) -> Amount {
        Amount::new(self.price * self.amount)
    }

    pub fn to_kalshi_request(&self) -> KalshiOrderRequest {
        KalshiOrderRequest {
            ticker: self.market_id.clone(),
            side: self.side.to_kalshi_side(),
            order_type: self.order_type.as_str().to_string(),
            price: (self.price * Decimal::from(100)).to_i64().unwrap_or(0),
            count: self.amount.to_i64().unwrap_or(0),
            client_order_id: self.client_order_id.clone(),
            expiration_ts: self.expiration.map(|e| e.timestamp()),
        }
    }

    pub fn to_polymarket_request(&self) -> PolymarketOrderRequest {
        PolymarketOrderRequest {
            token_id: self.market_id.clone(),
            side: self.side.to_polymarket_side().to_string(),
            price: self.price.to_string(),
            size: self.amount.to_string(),
            fee_rate_bps: "0".to_string(),
            nonce: chrono::Utc::now().timestamp_millis() as u64,
            expiration: self.expiration.map(|e| e.timestamp() as u64),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KalshiOrderRequest {
    pub ticker: String,
    pub side: i32,
    pub order_type: String,
    pub price: i64,
    pub count: i64,
    pub client_order_id: Option<String>,
    pub expiration_ts: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolymarketOrderRequest {
    pub token_id: String,
    pub side: String,
    pub price: String,
    pub size: String,
    pub fee_rate_bps: String,
    pub nonce: u64,
    pub expiration: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderResponse {
    pub order_id: String,
    pub platform: String,
    pub market_id: String,
    pub side: Side,
    pub order_type: OrderType,
    #[serde(with = "rust_decimal::serde::arbitrary_precision")]
    pub price: Decimal,
    #[serde(with = "rust_decimal::serde::arbitrary_precision")]
    pub amount: Decimal,
    #[serde(with = "rust_decimal::serde::arbitrary_precision")]
    pub filled_amount: Decimal,
    #[serde(with = "rust_decimal::serde::arbitrary_precision")]
    pub remaining_amount: Decimal,
    #[serde(with = "rust_decimal::serde::arbitrary_precision")]
    pub avg_fill_price: Decimal,
    pub status: OrderStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub client_order_id: Option<String>,
}

impl OrderResponse {
    pub fn price_value(&self) -> Price {
        Price::new(self.price).unwrap_or(Price::ZERO)
    }

    pub fn amount_value(&self) -> Amount {
        Amount::new(self.amount)
    }

    pub fn filled_amount_value(&self) -> Amount {
        Amount::new(self.filled_amount)
    }

    pub fn remaining_amount_value(&self) -> Amount {
        Amount::new(self.remaining_amount)
    }

    pub fn fill_percentage(&self) -> Decimal {
        if self.amount == Decimal::ZERO {
            return Decimal::ZERO;
        }
        (self.filled_amount / self.amount) * Decimal::ONE_HUNDRED
    }

    pub fn is_fully_filled(&self) -> bool {
        self.filled_amount >= self.amount
    }

    pub fn total_filled_value(&self) -> Amount {
        Amount::new(self.avg_fill_price * self.filled_amount)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderBookEntry {
    #[serde(with = "rust_decimal::serde::arbitrary_precision")]
    pub price: Decimal,
    #[serde(with = "rust_decimal::serde::arbitrary_precision")]
    pub size: Decimal,
}

impl OrderBookEntry {
    pub fn price_value(&self) -> Price {
        Price::new(self.price).unwrap_or(Price::ZERO)
    }

    pub fn size_value(&self) -> Amount {
        Amount::new(self.size)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderBook {
    pub market_id: String,
    pub platform: String,
    pub bids: Vec<OrderBookEntry>,
    pub asks: Vec<OrderBookEntry>,
    pub timestamp: DateTime<Utc>,
}

impl OrderBook {
    pub fn best_bid(&self) -> Option<&OrderBookEntry> {
        self.bids.first()
    }

    pub fn best_ask(&self) -> Option<&OrderBookEntry> {
        self.asks.first()
    }

    pub fn spread(&self) -> Option<Decimal> {
        match (self.best_ask(), self.best_bid()) {
            (Some(ask), Some(bid)) => Some(ask.price - bid.price),
            _ => None,
        }
    }

    pub fn mid_price(&self) -> Option<Decimal> {
        match (self.best_ask(), self.best_bid()) {
            (Some(ask), Some(bid)) => Some((ask.price + bid.price) / Decimal::from(2)),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Trade {
    pub trade_id: String,
    pub order_id: String,
    pub platform: String,
    pub market_id: String,
    pub side: Side,
    #[serde(with = "rust_decimal::serde::arbitrary_precision")]
    pub price: Decimal,
    #[serde(with = "rust_decimal::serde::arbitrary_precision")]
    pub amount: Decimal,
    #[serde(with = "rust_decimal::serde::arbitrary_precision")]
    pub fee: Decimal,
    pub executed_at: DateTime<Utc>,
}

impl Trade {
    pub fn price_value(&self) -> Price {
        Price::new(self.price).unwrap_or(Price::ZERO)
    }

    pub fn amount_value(&self) -> Amount {
        Amount::new(self.amount)
    }

    pub fn fee_amount(&self) -> Amount {
        Amount::new(self.fee)
    }

    pub fn total_value(&self) -> Amount {
        Amount::new(self.price * self.amount)
    }

    pub fn net_value(&self) -> Amount {
        Amount::new((self.price * self.amount) - self.fee)
    }
}
