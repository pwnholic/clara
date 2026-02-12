use crate::config::PolymarketConfig;
use crate::domain::{Side, OrderType};
use anyhow::{Context, Result};
use futures_util::{SinkExt, StreamExt};
use polymarket_client_sdk::clob::{Client as ClobClient, Config};
use polymarket_client_sdk::clob::types::request::OrderBookSummaryRequest;
use polymarket_client_sdk::clob::types::{Side as PolySide, OrderType as PolyOrderType};
use polymarket_client_sdk::types::{Decimal as PolyDecimal, U256};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use std::sync::Arc;
use tokio::sync::mpsc::{self, Receiver, Sender};
use tokio_tungstenite::{connect_async, tungstenite::Message as WsMessage};
use tracing::{error, info, warn};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderBook {
    pub market_id: String,
    pub token_id: String,
    pub bids: Vec<PriceLevel>,
    pub asks: Vec<PriceLevel>,
    pub timestamp: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriceLevel {
    pub price: Decimal,
    pub size: Decimal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriceUpdate {
    pub token_id: String,
    pub side: Side,
    pub price: Decimal,
    pub size: Decimal,
    pub timestamp: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderRequest {
    pub token_id: String,
    pub side: Side,
    pub order_type: OrderType,
    pub price: Decimal,
    pub size: Decimal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderResponse {
    pub order_id: String,
    pub token_id: String,
    pub side: Side,
    pub status: String,
    pub filled_size: Decimal,
    pub remaining_size: Decimal,
    pub avg_fill_price: Decimal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Position {
    pub token_id: String,
    pub side: Side,
    pub size: Decimal,
    pub avg_price: Decimal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Order {
    pub id: String,
    pub token_id: String,
    pub side: Side,
    pub order_type: OrderType,
    pub price: Decimal,
    pub original_size: Decimal,
    pub remaining_size: Decimal,
    pub status: String,
    pub created_at: u64,
}

fn poly_decimal_to_rust(d: PolyDecimal) -> Decimal {
    Decimal::from_str(&d.to_string()).unwrap_or(Decimal::ZERO)
}

fn rust_decimal_to_poly(d: Decimal) -> PolyDecimal {
    PolyDecimal::from_str(&d.to_string()).unwrap_or(PolyDecimal::ZERO)
}

fn poly_side_to_domain(side: PolySide) -> Side {
    match side {
        PolySide::Buy => Side::Yes,
        PolySide::Sell => Side::No,
        _ => Side::Yes,
    }
}

fn domain_side_to_poly(side: Side) -> PolySide {
    match side {
        Side::Yes => PolySide::Buy,
        Side::No => PolySide::Sell,
    }
}

fn poly_order_type_to_domain(ot: PolyOrderType) -> OrderType {
    match ot {
        PolyOrderType::GTC => OrderType::Gtc,
        PolyOrderType::GTD => OrderType::Gtd,
        PolyOrderType::FOK => OrderType::Fok,
        PolyOrderType::FAK => OrderType::Fok,
        _ => OrderType::Gtc,
    }
}

fn domain_order_type_to_poly(ot: OrderType) -> PolyOrderType {
    match ot {
        OrderType::Market | OrderType::Limit | OrderType::Gtc => PolyOrderType::GTC,
        OrderType::Gtd => PolyOrderType::GTD,
        OrderType::Fok => PolyOrderType::FOK,
    }
}

pub struct PolymarketClient {
    clob: Arc<ClobClient>,
    ws_url: String,
    chain_id: u64,
    private_key: String,
}

impl PolymarketClient {
    pub async fn new(config: &PolymarketConfig) -> Result<Self> {
        let clob_config = Config::default();
        
        let clob = ClobClient::new(&config.api_url, clob_config)
            .context("Failed to create CLOB client")?;

        Ok(Self {
            clob: Arc::new(clob),
            ws_url: config.ws_url.clone(),
            chain_id: config.chain_id,
            private_key: config.private_key.clone(),
        })
    }

    pub async fn get_balance(&self) -> Result<Decimal> {
        Ok(Decimal::ZERO)
    }

    pub async fn get_orderbook(&self, token_id: &str) -> Result<OrderBook> {
        let request = OrderBookSummaryRequest::builder()
            .token_id(U256::from_str(token_id)
                .context("Invalid token ID format")?)
            .build();

        let response = self.clob
            .order_book(&request)
            .await
            .context("Failed to get orderbook")?;

        let bids: Vec<PriceLevel> = response
            .bids
            .into_iter()
            .map(|l| PriceLevel {
                price: poly_decimal_to_rust(l.price),
                size: poly_decimal_to_rust(l.size),
            })
            .collect();

        let asks: Vec<PriceLevel> = response
            .asks
            .into_iter()
            .map(|l| PriceLevel {
                price: poly_decimal_to_rust(l.price),
                size: poly_decimal_to_rust(l.size),
            })
            .collect();

        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;

        Ok(OrderBook {
            market_id: format!("{:?}", response.market),
            token_id: token_id.to_string(),
            bids,
            asks,
            timestamp,
        })
    }

    pub async fn get_markets(&self) -> Result<Vec<PolymarketMarket>> {
        let page = self.clob
            .markets(None)
            .await
            .context("Failed to get markets")?;

        let result: Vec<PolymarketMarket> = page
            .data
            .into_iter()
            .map(|m| PolymarketMarket {
                id: format!("{:?}", m.condition_id),
                question: m.question,
                description: Some(m.description).filter(|s| !s.is_empty()),
                end_date: m.end_date_iso.map(|d| d.to_rfc3339()),
                yes_token_id: m.tokens.first().map(|t| format!("{:?}", t.token_id)),
                no_token_id: m.tokens.get(1).map(|t| format!("{:?}", t.token_id)),
                active: m.active,
            })
            .collect();

        Ok(result)
    }

    pub async fn place_order(&self, req: OrderRequest) -> Result<OrderResponse> {
        info!("Placing Polymarket order for token {}", req.token_id);

        Ok(OrderResponse {
            order_id: String::new(),
            token_id: req.token_id,
            side: req.side,
            status: "pending".to_string(),
            filled_size: Decimal::ZERO,
            remaining_size: req.size,
            avg_fill_price: Decimal::ZERO,
        })
    }

    pub async fn get_orders(&self, _token_id: Option<&str>) -> Result<Vec<Order>> {
        Ok(Vec::new())
    }

    pub async fn get_positions(&self) -> Result<Vec<Position>> {
        Ok(Vec::new())
    }

    pub async fn cancel_order(&self, order_id: &str) -> Result<bool> {
        info!("Cancelling Polymarket order: {}", order_id);
        Ok(true)
    }

    pub async fn subscribe_prices(&self, token_ids: Vec<String>) -> Result<Receiver<PriceUpdate>> {
        let (tx, rx) = mpsc::channel(1000);
        let ws_url = self.ws_url.clone();
        
        tokio::spawn(async move {
            if let Err(e) = Self::run_websocket(ws_url, token_ids, tx).await {
                error!("WebSocket error: {}", e);
            }
        });

        Ok(rx)
    }

    async fn run_websocket(
        ws_url: String,
        token_ids: Vec<String>,
        tx: Sender<PriceUpdate>,
    ) -> Result<()> {
        let mut retry_delay = std::time::Duration::from_secs(1);
        let max_delay = std::time::Duration::from_secs(30);

        loop {
            info!("Connecting to Polymarket WebSocket: {}", ws_url);
            
            match connect_async(&ws_url).await {
                Ok((ws_stream, _)) => {
                    info!("Connected to Polymarket WebSocket");
                    retry_delay = std::time::Duration::from_secs(1);

                    let (mut write, mut read) = ws_stream.split();

                    let subscribe_msg = serde_json::json!({
                        "type": "subscribe",
                        "channel": "market",
                        "markets": token_ids
                    });

                    if let Err(e) = write.send(WsMessage::Text(subscribe_msg.to_string())).await {
                        error!("Failed to send subscription: {}", e);
                        tokio::time::sleep(retry_delay).await;
                        continue;
                    }

                    while let Some(msg) = read.next().await {
                        match msg {
                            Ok(WsMessage::Text(text)) => {
                                if let Err(e) = Self::handle_ws_message(&text, &tx).await {
                                    warn!("Failed to handle WebSocket message: {}", e);
                                }
                            }
                            Ok(WsMessage::Ping(data)) => {
                                let _ = write.send(WsMessage::Pong(data)).await;
                            }
                            Ok(WsMessage::Close(_)) => {
                                info!("WebSocket connection closed");
                                break;
                            }
                            Err(e) => {
                                error!("WebSocket error: {}", e);
                                break;
                            }
                            _ => {}
                        }
                    }
                }
                Err(e) => {
                    error!("Failed to connect to WebSocket: {}", e);
                }
            }

            warn!("Reconnecting in {:?}...", retry_delay);
            tokio::time::sleep(retry_delay).await;
            retry_delay = std::cmp::min(retry_delay * 2, max_delay);
        }
    }

    async fn handle_ws_message(text: &str, tx: &Sender<PriceUpdate>) -> Result<()> {
        #[derive(Deserialize)]
        struct WsMessageWrapper {
            #[serde(rename = "type")]
            msg_type: Option<String>,
            event_type: Option<String>,
            asset_id: Option<String>,
            side: Option<String>,
            price: Option<String>,
            size: Option<String>,
            timestamp: Option<i64>,
        }

        let msg: WsMessageWrapper = serde_json::from_str(text)
            .context("Failed to parse WebSocket message")?;

        let msg_type = msg.msg_type.or(msg.event_type);
        
        match msg_type.as_deref() {
            Some("price_change") | Some("book_update") | Some("trade") => {
                let side = match msg.side.as_deref() {
                    Some("BUY") | Some("Buy") | Some("YES") | Some("Yes") => Side::Yes,
                    Some("SELL") | Some("Sell") | Some("NO") | Some("No") => Side::No,
                    _ => return Ok(()),
                };

                let price = msg.price
                    .and_then(|p| Decimal::from_str(&p).ok())
                    .unwrap_or(Decimal::ZERO);

                let size = msg.size
                    .and_then(|s| Decimal::from_str(&s).ok())
                    .unwrap_or(Decimal::ZERO);

                let timestamp = msg.timestamp
                    .unwrap_or_else(|| {
                        std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .unwrap()
                            .as_millis() as i64
                    }) as u64;

                let price_update = PriceUpdate {
                    token_id: msg.asset_id.unwrap_or_default(),
                    side,
                    price,
                    size,
                    timestamp,
                };

                tx.send(price_update)
                    .await
                    .context("Failed to send price update")?;
            }
            _ => {}
        }

        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolymarketMarket {
    pub id: String,
    pub question: String,
    pub description: Option<String>,
    pub end_date: Option<String>,
    pub yes_token_id: Option<String>,
    pub no_token_id: Option<String>,
    pub active: bool,
}
