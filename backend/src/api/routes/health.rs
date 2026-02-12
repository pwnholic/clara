use axum::{extract::State, Json};
use serde::Serialize;
use std::sync::Arc;

use crate::api::state::AppState;

#[derive(Serialize)]
pub struct HealthResponse {
    status: &'static str,
    version: &'static str,
    service_running: bool,
}

pub async fn health_check(State(state): State<Arc<AppState>>) -> Json<HealthResponse> {
    let service_running = state.service.read().await.is_running().await;
    
    Json(HealthResponse {
        status: "ok",
        version: env!("CARGO_PKG_VERSION"),
        service_running,
    })
}
