use num_traits::ToPrimitive;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Platform {
    Kalshi,
    Polymarket,
}

impl Platform {
    pub fn as_str(&self) -> &'static str {
        match self {
            Platform::Kalshi => "kalshi",
            Platform::Polymarket => "polymarket",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "kalshi" => Some(Platform::Kalshi),
            "polymarket" => Some(Platform::Polymarket),
            _ => None,
        }
    }
}

impl fmt::Display for Platform {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Side {
    Yes,
    No,
}

impl Side {
    pub fn as_str(&self) -> &'static str {
        match self {
            Side::Yes => "yes",
            Side::No => "no",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "yes" => Some(Side::Yes),
            "no" => Some(Side::No),
            _ => None,
        }
    }

    pub fn opposite(&self) -> Self {
        match self {
            Side::Yes => Side::No,
            Side::No => Side::Yes,
        }
    }

    pub fn to_polymarket_side(&self) -> &'static str {
        match self {
            Side::Yes => "YES",
            Side::No => "NO",
        }
    }

    pub fn to_kalshi_side(&self) -> i32 {
        match self {
            Side::Yes => 1,
            Side::No => -1,
        }
    }
}

impl fmt::Display for Side {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OrderType {
    Market,
    Limit,
    Gtc,
    Fok,
    Gtd,
}

impl OrderType {
    pub fn as_str(&self) -> &'static str {
        match self {
            OrderType::Market => "market",
            OrderType::Limit => "limit",
            OrderType::Gtc => "gtc",
            OrderType::Fok => "fok",
            OrderType::Gtd => "gtd",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "market" => Some(OrderType::Market),
            "limit" => Some(OrderType::Limit),
            "gtc" => Some(OrderType::Gtc),
            "fok" => Some(OrderType::Fok),
            "gtd" => Some(OrderType::Gtd),
            _ => None,
        }
    }
}

impl fmt::Display for OrderType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MarketStatus {
    Open,
    Closed,
    Settled,
    Cancelled,
    Pending,
}

impl MarketStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            MarketStatus::Open => "open",
            MarketStatus::Closed => "closed",
            MarketStatus::Settled => "settled",
            MarketStatus::Cancelled => "cancelled",
            MarketStatus::Pending => "pending",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "open" => Some(MarketStatus::Open),
            "closed" => Some(MarketStatus::Closed),
            "settled" => Some(MarketStatus::Settled),
            "cancelled" => Some(MarketStatus::Cancelled),
            "pending" => Some(MarketStatus::Pending),
            _ => None,
        }
    }

    pub fn is_tradable(&self) -> bool {
        matches!(self, MarketStatus::Open)
    }
}

impl fmt::Display for MarketStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Price {
    #[serde(with = "rust_decimal::serde::arbitrary_precision")]
    pub value: Decimal,
}

impl Price {
    pub const ZERO: Price = Price {
        value: Decimal::ZERO,
    };
    pub const ONE: Price = Price {
        value: Decimal::ONE,
    };
    pub const MAX: Price = Price {
        value: Decimal::ONE,
    };

    pub fn new(value: Decimal) -> Option<Self> {
        if value >= Decimal::ZERO && value <= Decimal::ONE {
            Some(Price { value })
        } else {
            None
        }
    }

    pub fn from_cents(cents: u64) -> Self {
        Price {
            value: Decimal::from(cents) / Decimal::from(100),
        }
    }

    pub fn to_cents(&self) -> u64 {
        (self.value * Decimal::from(100)).to_u64().unwrap_or(0)
    }

    pub fn from_kalshi_price(kalshi_price: i64) -> Self {
        Price {
            value: Decimal::from(kalshi_price) / Decimal::from(100),
        }
    }

    pub fn to_kalshi_price(&self) -> i64 {
        (self.value * Decimal::from(100)).to_i64().unwrap_or(0)
    }

    pub fn from_polymarket_price(polymarket_price: &str) -> Option<Self> {
        Decimal::from_str_exact(polymarket_price)
            .ok()
            .and_then(|d| Self::new(d))
    }

    pub fn to_polymarket_price(&self) -> String {
        self.value.to_string()
    }

    pub fn implied_probability(&self) -> Decimal {
        self.value
    }
}

impl std::ops::Add for Price {
    type Output = Price;

    fn add(self, other: Price) -> Price {
        Price {
            value: (self.value + other.value).min(Decimal::ONE),
        }
    }
}

impl std::ops::Sub for Price {
    type Output = Price;

    fn sub(self, other: Price) -> Price {
        Price {
            value: (self.value - other.value).max(Decimal::ZERO),
        }
    }
}

impl std::ops::Mul<Decimal> for Price {
    type Output = Price;

    fn mul(self, rhs: Decimal) -> Price {
        Price {
            value: (self.value * rhs).min(Decimal::ONE).max(Decimal::ZERO),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Amount {
    #[serde(with = "rust_decimal::serde::arbitrary_precision")]
    pub value: Decimal,
}

impl Amount {
    pub const ZERO: Amount = Amount {
        value: Decimal::ZERO,
    };

    pub fn new(value: Decimal) -> Self {
        Amount {
            value: value.max(Decimal::ZERO),
        }
    }

    pub fn from_cents(cents: i64) -> Self {
        Amount {
            value: Decimal::from(cents) / Decimal::from(100),
        }
    }

    pub fn from_whole(whole: i64) -> Self {
        Amount {
            value: Decimal::from(whole),
        }
    }

    pub fn to_cents(&self) -> i64 {
        (self.value * Decimal::from(100)).to_i64().unwrap_or(0)
    }

    pub fn to_whole(&self) -> i64 {
        self.value.to_i64().unwrap_or(0)
    }
}

impl std::ops::Add for Amount {
    type Output = Amount;

    fn add(self, other: Amount) -> Amount {
        Amount::new(self.value + other.value)
    }
}

impl std::ops::Sub for Amount {
    type Output = Amount;

    fn sub(self, other: Amount) -> Amount {
        Amount::new(self.value - other.value)
    }
}

impl std::ops::Mul<Decimal> for Amount {
    type Output = Amount;

    fn mul(self, rhs: Decimal) -> Amount {
        Amount::new(self.value * rhs)
    }
}

impl std::ops::Div<Decimal> for Amount {
    type Output = Amount;

    fn div(self, rhs: Decimal) -> Amount {
        if rhs == Decimal::ZERO {
            Amount::ZERO
        } else {
            Amount::new(self.value / rhs)
        }
    }
}

impl rust_decimal::prelude::ToPrimitive for Amount {
    fn to_i64(&self) -> Option<i64> {
        self.value.to_i64()
    }

    fn to_u64(&self) -> Option<u64> {
        self.value.to_u64()
    }
}
