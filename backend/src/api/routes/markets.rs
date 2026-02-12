use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::{debug, error, instrument};

use crate::api::state::AppState;
use crate::domain::{ArbitrageOpportunity, MatchedMarket};

#[derive(Debug, Serialize)]
pub struct OpportunitiesResponse {
    opportunities: Vec<ArbitrageOpportunity>,
    count: usize,
}

#[instrument(skip(state))]
pub async fn get_opportunities(
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    let opportunities = state.opportunities.read().await.clone();
    let count = opportunities.len();
    
    debug!("Returning {} opportunities", count);
    Json(OpportunitiesResponse { opportunities, count })
}

#[derive(Debug, Serialize)]
pub struct MarketsResponse {
    markets: Vec<MatchedMarket>,
    count: usize,
}

#[instrument(skip(state))]
pub async fn get_markets(
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    let service = state.service.read().await;
    let markets = service.get_matched_markets().await;
    let count = markets.len();
    debug!("Returning {} matched markets", count);
    Json(MarketsResponse { markets, count })
}

#[derive(Debug, Deserialize)]
pub struct OrderbookQuery {
    pub kalshi_ticker: Option<String>,
    pub poly_token_id: Option<String>,
}

#[derive(Debug, Serialize, Default)]
pub struct OrderbookLevel {
    #[serde(with = "rust_decimal::serde::arbitrary_precision")]
    pub price: Decimal,
    #[serde(with = "rust_decimal::serde::arbitrary_precision")]
    pub size: Decimal,
}

#[derive(Debug, Serialize, Default)]
pub struct OrderbookSide {
    pub bids: Vec<OrderbookLevel>,
    pub asks: Vec<OrderbookLevel>,
}

#[derive(Debug, Serialize)]
pub struct OrderbookResponse {
    pub kalshi: Option<OrderbookSide>,
    pub polymarket: Option<OrderbookSide>,
}

#[instrument(skip(state))]
pub async fn get_orderbook(
    State(state): State<Arc<AppState>>,
    Query(query): Query<OrderbookQuery>,
) -> impl IntoResponse {
    let kalshi_orderbook = if let Some(ticker) = query.kalshi_ticker {
        match state.kalshi.get_orderbook(&ticker).await {
            Ok(book) => Some(OrderbookSide {
                bids: book.bids.into_iter().map(|l| OrderbookLevel { price: l.price, size: l.size }).collect(),
                asks: book.asks.into_iter().map(|l| OrderbookLevel { price: l.price, size: l.size }).collect(),
            }),
            Err(e) => {
                error!("Failed to get Kalshi orderbook: {}", e);
                None
            }
        }
    } else {
        None
    };

    let poly_orderbook = if let Some(token_id) = query.poly_token_id {
        match state.polymarket.get_orderbook(&token_id).await {
            Ok(book) => Some(OrderbookSide {
                bids: book.bids.into_iter().map(|l| OrderbookLevel { price: l.price, size: l.size }).collect(),
                asks: book.asks.into_iter().map(|l| OrderbookLevel { price: l.price, size: l.size }).collect(),
            }),
            Err(e) => {
                error!("Failed to get Polymarket orderbook: {}", e);
                None
            }
        }
    } else {
        None
    };

    Json(OrderbookResponse {
        kalshi: kalshi_orderbook,
        polymarket: poly_orderbook,
    })
}
