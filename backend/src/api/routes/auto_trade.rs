use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::{error, info, instrument};

use crate::api::state::AppState;

#[derive(Debug, Serialize)]
pub struct AutoTradeStatus {
    pub enabled: bool,
    pub trade_count: u32,
    pub max_trade_count: u32,
    pub max_amount: String,
}

#[instrument(skip(state))]
pub async fn get_auto_trade(
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    match state.storage.get_settings() {
        Ok(settings) => {
            match state.storage.get_auto_trade_count_today() {
                Ok(count) => Json(AutoTradeStatus {
                    enabled: settings.auto_trade_enabled,
                    trade_count: count,
                    max_trade_count: settings.auto_trade_max_count,
                    max_amount: settings.auto_trade_max_amount.to_string(),
                }),
                Err(_) => Json(AutoTradeStatus {
                    enabled: settings.auto_trade_enabled,
                    trade_count: 0,
                    max_trade_count: settings.auto_trade_max_count,
                    max_amount: settings.auto_trade_max_amount.to_string(),
                }),
            }
        }
        Err(e) => {
            error!("Failed to get settings: {}", e);
            Json(AutoTradeStatus {
                enabled: false,
                trade_count: 0,
                max_trade_count: 0,
                max_amount: "0".to_string(),
            })
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct SetAutoTradeRequest {
    pub enabled: bool,
    #[serde(default)]
    pub max_trade_count: Option<u32>,
    #[serde(default)]
    pub max_amount: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct SetAutoTradeResponse {
    pub success: bool,
    pub message: String,
}

#[instrument(skip(state))]
pub async fn set_auto_trade(
    State(state): State<Arc<AppState>>,
    Json(req): Json<SetAutoTradeRequest>,
) -> Response {
    let mut settings = match state.storage.get_settings() {
        Ok(s) => s,
        Err(e) => {
            error!("Failed to get settings: {}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(SetAutoTradeResponse {
                    success: false,
                    message: format!("Failed to get settings: {}", e),
                }),
            )
                .into_response();
        }
    };

    settings.auto_trade_enabled = req.enabled;
    if let Some(count) = req.max_trade_count {
        settings.auto_trade_max_count = count;
    }
    if let Some(amount_str) = req.max_amount {
        if let Ok(amount) = amount_str.parse::<Decimal>() {
            settings.auto_trade_max_amount = amount;
        }
    }

    match state.storage.update_settings(&settings) {
        Ok(_) => {
            info!("Auto trade {}", if req.enabled { "enabled" } else { "disabled" });
            Json(SetAutoTradeResponse {
                success: true,
                message: if req.enabled { "Auto trade enabled" } else { "Auto trade disabled" }.to_string(),
            })
            .into_response()
        }
        Err(e) => {
            error!("Failed to update settings: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(SetAutoTradeResponse {
                    success: false,
                    message: format!("Failed to update settings: {}", e),
                }),
            )
                .into_response()
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct ExclusionRequest {
    pub market_key: String,
}

#[derive(Debug, Serialize)]
pub struct ExclusionResponse {
    pub success: bool,
    pub message: String,
    pub market_key: String,
}

#[instrument(skip(state))]
pub async fn exclude_market(
    State(state): State<Arc<AppState>>,
    Json(req): Json<ExclusionRequest>,
) -> impl IntoResponse {
    match state.storage.exclude_market(&req.market_key) {
        Ok(_) => Json(ExclusionResponse {
            success: true,
            message: "Market excluded".to_string(),
            market_key: req.market_key,
        }),
        Err(e) => {
            error!("Failed to exclude market: {}", e);
            Json(ExclusionResponse {
                success: false,
                message: format!("Failed to exclude market: {}", e),
                market_key: req.market_key,
            })
        }
    }
}

#[instrument(skip(state))]
pub async fn unexclude_market(
    State(state): State<Arc<AppState>>,
    Json(req): Json<ExclusionRequest>,
) -> impl IntoResponse {
    match state.storage.unexclude_market(&req.market_key) {
        Ok(_) => Json(ExclusionResponse {
            success: true,
            message: "Market unexcluded".to_string(),
            market_key: req.market_key,
        }),
        Err(e) => {
            error!("Failed to unexclude market: {}", e);
            Json(ExclusionResponse {
                success: false,
                message: format!("Failed to unexclude market: {}", e),
                market_key: req.market_key,
            })
        }
    }
}
