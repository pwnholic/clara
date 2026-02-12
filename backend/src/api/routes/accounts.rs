use axum::{
    extract::State,
    response::IntoResponse,
    Json,
};
use rust_decimal::Decimal;
use serde::Serialize;
use std::sync::Arc;
use tracing::instrument;

use crate::api::state::AppState;

#[derive(Debug, Serialize)]
pub struct BalanceResponse {
    #[serde(with = "rust_decimal::serde::arbitrary_precision")]
    pub kalshi_balance: Decimal,
    #[serde(with = "rust_decimal::serde::arbitrary_precision")]
    pub polymarket_balance: Decimal,
    #[serde(with = "rust_decimal::serde::arbitrary_precision")]
    pub total_balance: Decimal,
    pub kalshi_available: bool,
    pub polymarket_available: bool,
}

#[instrument(skip(state))]
pub async fn get_balance(
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    let (kalshi_balance, poly_balance) = state.metrics.get_balances();

    Json(BalanceResponse {
        kalshi_balance,
        polymarket_balance: poly_balance,
        total_balance: kalshi_balance + poly_balance,
        kalshi_available: kalshi_balance > Decimal::ZERO,
        polymarket_available: poly_balance > Decimal::ZERO,
    })
}

#[derive(Debug, Serialize)]
pub struct Position {
    pub platform: String,
    pub market_id: String,
    pub event_name: String,
    pub side: String,
    #[serde(with = "rust_decimal::serde::arbitrary_precision")]
    pub size: Decimal,
    #[serde(with = "rust_decimal::serde::arbitrary_precision")]
    pub avg_price: Decimal,
    #[serde(with = "rust_decimal::serde::arbitrary_precision")]
    pub current_value: Decimal,
}

#[derive(Debug, Serialize)]
pub struct PositionsResponse {
    positions: Vec<Position>,
    count: usize,
}

#[instrument(skip(_state))]
pub async fn get_positions(
    State(_state): State<Arc<AppState>>,
) -> impl IntoResponse {
    let positions: Vec<Position> = Vec::new();
    let count = positions.len();
    Json(PositionsResponse { positions, count })
}
