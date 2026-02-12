mod health;
mod markets;
mod orders;
mod accounts;
mod history;
mod auto_trade;

use std::sync::Arc;

use axum::{
    routing::{delete, get},
    Router,
};
use tower_http::cors::CorsLayer;

use super::state::AppState;

pub fn create_router(state: AppState) -> Router {
    let state = Arc::new(state);
    Router::new()
        .route("/api/health", get(health::health_check))
        .route("/api/opportunities", get(markets::get_opportunities))
        .route("/api/markets", get(markets::get_markets))
        .route("/api/orders", get(orders::get_orders).post(orders::place_order))
        .route("/api/orders/:id", delete(orders::cancel_order))
        .route("/api/accounts/balance", get(accounts::get_balance))
        .route("/api/accounts/positions", get(accounts::get_positions))
        .route("/api/history", get(history::get_history))
        .route("/api/auto-trade", get(auto_trade::get_auto_trade).post(auto_trade::set_auto_trade))
        .route("/ws", get(crate::api::ws::ws_handler))
        .layer(CorsLayer::permissive())
        .with_state(state)
}
