use std::sync::Arc;
use std::time::Duration;

use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State,
    },
    response::IntoResponse,
};
use futures_util::{SinkExt, StreamExt};
use tokio::sync::mpsc;
use tracing::{debug, info, warn};

use super::state::{AppState, WsMessage};

pub async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, state))
}

async fn handle_socket(socket: WebSocket, state: Arc<AppState>) {
    let (mut sender, mut receiver) = socket.split();

    info!("New WebSocket client connected");

    let (tx, mut rx) = mpsc::channel::<String>(256);

    {
        let service = state.service.read().await;
        let opportunities = service.get_opportunities().await;
        if !opportunities.is_empty() {
            let msg = WsMessage::Opportunities {
                data: opportunities,
            };
            if let Ok(json) = serde_json::to_string(&msg) {
                let _ = tx.send(json).await;
            }
        }
    }

    let (kalshi_bal, poly_bal) = state.metrics.get_balances();
    let metrics_msg = WsMessage::Metrics {
        kalshi_balance: kalshi_bal.to_string(),
        poly_balance: poly_bal.to_string(),
        total_trades: state.metrics.total_trades.load(std::sync::atomic::Ordering::Relaxed),
        total_profit: state.metrics.total_profit.load().to_string(),
    };
    if let Ok(json) = serde_json::to_string(&metrics_msg) {
        let _ = tx.send(json).await;
    }

    let mut broadcast_rx = state.ws_broadcast.subscribe();

    let state_periodic = state.clone();
    let tx_periodic = tx.clone();
    let periodic_task = tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(5));
        loop {
            interval.tick().await;

            let (kalshi_bal, poly_bal) = state_periodic.metrics.get_balances();
            let msg = WsMessage::Metrics {
                kalshi_balance: kalshi_bal.to_string(),
                poly_balance: poly_bal.to_string(),
                total_trades: state_periodic.metrics.total_trades.load(std::sync::atomic::Ordering::Relaxed),
                total_profit: state_periodic.metrics.total_profit.load().to_string(),
            };

            if let Ok(json) = serde_json::to_string(&msg) {
                if tx_periodic.send(json).await.is_err() {
                    break;
                }
            }
        }
    });

    let tx_broadcast = tx.clone();
    let broadcast_task = tokio::spawn(async move {
        while let Ok(msg) = broadcast_rx.recv().await {
            if let Ok(json) = serde_json::to_string(&msg) {
                if tx_broadcast.send(json).await.is_err() {
                    break;
                }
            }
        }
    });

    let state_opp = state.clone();
    let tx_opp = tx.clone();
    let opp_task = tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(2));
        loop {
            interval.tick().await;

            let service = state_opp.service.read().await;
            let opportunities = service.get_opportunities().await;
            if !opportunities.is_empty() {
                let msg = WsMessage::Opportunities {
                    data: opportunities,
                };
                if let Ok(json) = serde_json::to_string(&msg) {
                    if tx_opp.send(json).await.is_err() {
                        break;
                    }
                }
            }
        }
    });

    let send_task = tokio::spawn(async move {
        while let Some(json) = rx.recv().await {
            if sender.send(Message::Text(json)).await.is_err() {
                break;
            }
        }
    });

    let recv_task = tokio::spawn(async move {
        while let Some(result) = receiver.next().await {
            match result {
                Ok(Message::Ping(data)) => {
                    debug!("Received ping: {} bytes", data.len());
                }
                Ok(Message::Pong(_)) => {
                    debug!("Received pong");
                }
                Ok(Message::Close(_)) => {
                    info!("Client requested close");
                    break;
                }
                Ok(Message::Text(text)) => {
                    debug!("Received text message: {}", text);
                    if let Ok(cmd) = serde_json::from_str::<serde_json::Value>(&text) {
                        if cmd.get("type").and_then(|t| t.as_str()) == Some("ping") {
                            continue;
                        }
                    }
                }
                Ok(Message::Binary(data)) => {
                    debug!("Received binary message: {} bytes", data.len());
                }
                Err(e) => {
                    warn!("WebSocket error: {}", e);
                    break;
                }
                _ => {}
            }
        }
    });

    let _ = recv_task.await;

    periodic_task.abort();
    broadcast_task.abort();
    opp_task.abort();
    send_task.abort();

    info!("WebSocket client disconnected");
}
