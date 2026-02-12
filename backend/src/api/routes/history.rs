use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::{error, instrument};

use crate::api::state::AppState;

#[derive(Debug, Deserialize)]
pub struct HistoryQuery {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[derive(Debug, Serialize)]
pub struct HistoryRecord {
    pub id: i64,
    pub market_key: String,
    pub platform: String,
    pub entry_price: String,
    pub exit_price: Option<String>,
    pub profit: Option<String>,
    pub timestamp: String,
}

#[derive(Debug, Serialize)]
pub struct HistoryResponse {
    records: Vec<HistoryRecord>,
    total: usize,
}

#[instrument(skip(state))]
pub async fn get_history(
    State(state): State<Arc<AppState>>,
    Query(query): Query<HistoryQuery>,
) -> Response {
    let limit = query.limit.unwrap_or(100);
    let offset = query.offset.unwrap_or(0);

    match state.storage.get_history(limit, offset) {
        Ok(records) => {
            let history_records: Vec<HistoryRecord> = records
                .into_iter()
                .map(|r| HistoryRecord {
                    id: r.id,
                    market_key: r.market_key,
                    platform: r.platform,
                    entry_price: r.entry_price.to_string(),
                    exit_price: r.exit_price.map(|p| p.to_string()),
                    profit: r.profit.map(|p| p.to_string()),
                    timestamp: r.timestamp.to_rfc3339(),
                })
                .collect();
            let total = history_records.len();
            Json(HistoryResponse { records: history_records, total }).into_response()
        }
        Err(e) => {
            error!("Failed to get history: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(HistoryResponse {
                    records: Vec::new(),
                    total: 0,
                }),
            )
                .into_response()
        }
    }
}
