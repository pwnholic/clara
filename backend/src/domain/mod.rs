pub mod types;
pub mod market;
pub mod order;
pub mod opportunity;

pub use types::{Platform, Side, OrderType, MarketStatus};
pub use market::{KalshiMarket, PolymarketMarket, MatchedMarket};
pub use order::{OrderRequest, OrderResponse, OrderStatus};
pub use opportunity::ArbitrageOpportunity;
