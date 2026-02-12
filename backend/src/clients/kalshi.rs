use crate::config::KalshiConfig;
use crate::domain::{Side, OrderType};
use anyhow::{anyhow, Context, Result};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use futures_util::{SinkExt, StreamExt};
use rsa::{
    pkcs8::DecodePrivateKey,
    Pkcs1v15Sign,
    RsaPrivateKey,
};
use rust_decimal::Decimal;
use rust_decimal::prelude::ToPrimitive;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use rsa::sha2::{Digest, Sha256};
use std::path::Path;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::mpsc::{self, Receiver, Sender};
use tokio_tungstenite::{connect_async, tungstenite::Message as WsMessage};
use tracing::{debug, error, info, warn};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderBook {
    pub market_id: String,
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
    pub market_id: String,
    pub side: Side,
    pub price: Decimal,
    pub size: Decimal,
    pub timestamp: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderRequest {
    pub market_id: String,
    pub side: Side,
    pub order_type: OrderType,
    pub price: Decimal,
    pub size: Decimal,
    pub client_order_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderResponse {
    pub order_id: String,
    pub market_id: String,
    pub side: Side,
    pub status: String,
    pub filled_size: Decimal,
    pub remaining_size: Decimal,
    pub avg_fill_price: Decimal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Position {
    pub market_id: String,
    pub side: Side,
    pub size: Decimal,
    pub avg_price: Decimal,
    pub total_cost: Decimal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Order {
    pub id: String,
    pub market_id: String,
    pub side: Side,
    pub order_type: OrderType,
    pub price: Decimal,
    pub original_size: Decimal,
    pub remaining_size: Decimal,
    pub status: String,
    pub created_at: u64,
}

#[derive(Debug, Clone, Deserialize)]
struct KalshiOrder {
    id: String,
    ticker: String,
    side: i32,
    order_type: String,
    original_quantity: i64,
    remaining_quantity: i64,
    price: i64,
    status: String,
    creation_time: String,
}

pub struct KalshiClient {
    base_url: String,
    ws_url: String,
    http: reqwest::Client,
    api_key: String,
    private_key: RsaPrivateKey,
    rate_limiter: Arc<tokio::sync::Semaphore>,
}

impl KalshiClient {
    pub fn new(config: &KalshiConfig) -> Result<Self> {
        let private_key = if Path::new(&config.private_key_pem).exists() {
            let key_content = std::fs::read_to_string(&config.private_key_pem)
                .context("Failed to read private key file")?;
            RsaPrivateKey::from_pkcs8_pem(&key_content)
                .context("Failed to parse private key from file")?
        } else {
            RsaPrivateKey::from_pkcs8_pem(&config.private_key_pem)
                .context("Failed to parse private key from PEM string")?
        };

        let rate_limiter = Arc::new(tokio::sync::Semaphore::new(
            config.rate_limit_per_second as usize,
        ));

        Ok(Self {
            base_url: config.api_url.clone(),
            ws_url: config.ws_url.clone(),
            http: reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(30))
                .build()
                .context("Failed to create HTTP client")?,
            api_key: config.api_key.clone(),
            private_key,
            rate_limiter,
        })
    }

    fn generate_signature(&self, method: &str, path: &str, timestamp: u64) -> Result<String> {
        let signing_string = format!("{}{}{}", method, path, timestamp);
        
        let mut hasher = Sha256::new();
        hasher.update(signing_string.as_bytes());
        let hash = hasher.finalize();
        
        let padding = Pkcs1v15Sign::new::<Sha256>();
        let signature = self
            .private_key
            .sign(padding, &hash)
            .context("Failed to sign request")?;
        
        Ok(BASE64.encode(&signature))
    }

    fn get_timestamp() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64
    }

    async fn rate_limit(&self) {
        let _permit = self.rate_limiter.acquire().await.unwrap();
    }

    async fn request<T: DeserializeOwned>(
        &self,
        method: reqwest::Method,
        path: &str,
        body: Option<&serde_json::Value>,
    ) -> Result<T> {
        self.rate_limit().await;
        
        let timestamp = Self::get_timestamp();
        let signature = self.generate_signature(method.as_str(), path, timestamp)?;
        
        let url = format!("{}{}", self.base_url, path);
        let mut request = self
            .http
            .request(method.clone(), &url)
            .header("X-Api-Key", &self.api_key)
            .header("X-Timestamp", timestamp.to_string())
            .header("X-Signature", signature)
            .header("Content-Type", "application/json");

        if let Some(b) = body {
            request = request.json(b);
        }

        debug!("Kalshi API request: {} {}", method, path);
        
        let response = request
            .send()
            .await
            .context("HTTP request failed")?;

        let status = response.status();
        let response_text = response
            .text()
            .await
            .context("Failed to read response body")?;

        if !status.is_success() {
            error!("Kalshi API error: {} - {}", status, response_text);
            return Err(anyhow!("Kalshi API error: {} - {}", status, response_text));
        }

        serde_json::from_str(&response_text)
            .context("Failed to parse response JSON")
    }

    pub async fn get_balance(&self) -> Result<Decimal> {
        #[derive(Deserialize)]
        struct BalanceResponse {
            balance: i64,
        }

        let response: BalanceResponse = self
            .request(reqwest::Method::GET, "/portfolio/balance", None)
            .await?;

        Ok(Decimal::from(response.balance) / Decimal::from(100))
    }

    pub async fn get_orderbook(&self, market_id: &str) -> Result<OrderBook> {
        #[derive(Deserialize)]
        struct KalshiOrderbook {
            yes: Vec<KalshiLevel>,
            no: Vec<KalshiLevel>,
        }

        #[derive(Deserialize)]
        struct KalshiLevel {
            price: i64,
            size: i64,
        }

        let path = format!("/markets/{}/orderbook", market_id);
        let response: KalshiOrderbook = self
            .request(reqwest::Method::GET, &path, None)
            .await?;

        let bids: Vec<PriceLevel> = response
            .yes
            .into_iter()
            .map(|l| PriceLevel {
                price: Decimal::from(l.price) / Decimal::from(100),
                size: Decimal::from(l.size),
            })
            .collect();

        let asks: Vec<PriceLevel> = response
            .no
            .into_iter()
            .map(|l| PriceLevel {
                price: Decimal::from(100 - l.price) / Decimal::from(100),
                size: Decimal::from(l.size),
            })
            .collect();

        Ok(OrderBook {
            market_id: market_id.to_string(),
            bids,
            asks,
            timestamp: Self::get_timestamp(),
        })
    }

    pub async fn get_markets(&self) -> Result<Vec<KalshiMarket>> {
        #[derive(Deserialize)]
        struct MarketsResponse {
            markets: Vec<KalshiMarketRaw>,
        }

        #[derive(Deserialize)]
        struct KalshiMarketRaw {
            ticker: String,
            title: String,
            category: String,
            close_date: Option<String>,
            yes_bid: Option<i64>,
            yes_ask: Option<i64>,
            open_interest: Option<i64>,
            status: String,
        }

        let response: MarketsResponse = self
            .request(reqwest::Method::GET, "/markets", None)
            .await?;

        let markets = response
            .markets
            .into_iter()
            .map(|m| KalshiMarket {
                ticker: m.ticker,
                title: m.title,
                category: m.category,
                close_date: m.close_date,
                yes_bid: m.yes_bid.map(|p| Decimal::from(p) / Decimal::from(100)),
                yes_ask: m.yes_ask.map(|p| Decimal::from(p) / Decimal::from(100)),
                open_interest: m.open_interest.unwrap_or(0),
                status: m.status,
            })
            .collect();

        Ok(markets)
    }

    pub async fn place_order(&self, req: OrderRequest) -> Result<OrderResponse> {
        #[derive(Serialize)]
        struct PlaceOrderRequest {
            ticker: String,
            side: i32,
            order_type: String,
            price: i64,
            quantity: i64,
            client_order_id: Option<String>,
            expiration_ts: Option<u64>,
        }

        #[derive(Deserialize)]
        struct PlaceOrderResponse {
            order_id: String,
            status: String,
            filled_quantity: Option<i64>,
            remaining_quantity: Option<i64>,
            avg_fill_price: Option<i64>,
        }

        let kalshi_side = req.side.to_kalshi_side();
        let kalshi_price = (req.price * Decimal::from(100)).to_i64().unwrap_or(0);
        let kalshi_quantity = req.size.to_i64().unwrap_or(0);

        let order_type_str = match req.order_type {
            OrderType::Market => "market".to_string(),
            OrderType::Limit | OrderType::Gtc => "gtc".to_string(),
            OrderType::Gtd => "gtd".to_string(),
            OrderType::Fok => "fok".to_string(),
        };

        let expiration_ts = if req.order_type == OrderType::Gtd {
            Some(Self::get_timestamp() + 86400000)
        } else {
            None
        };

        let body = serde_json::to_value(PlaceOrderRequest {
            ticker: req.market_id.clone(),
            side: kalshi_side,
            order_type: order_type_str,
            price: kalshi_price,
            quantity: kalshi_quantity,
            client_order_id: req.client_order_id.clone(),
            expiration_ts,
        })?;

        let response: PlaceOrderResponse = self
            .request(reqwest::Method::POST, "/portfolio/orders", Some(&body))
            .await?;

        info!("Placed Kalshi order: {} for market {}", response.order_id, req.market_id);

        Ok(OrderResponse {
            order_id: response.order_id,
            market_id: req.market_id,
            side: req.side,
            status: response.status,
            filled_size: Decimal::from(response.filled_quantity.unwrap_or(0)),
            remaining_size: Decimal::from(response.remaining_quantity.unwrap_or(kalshi_quantity)),
            avg_fill_price: Decimal::from(response.avg_fill_price.unwrap_or(0)) / Decimal::from(100),
        })
    }

    pub async fn get_orders(&self, market_id: Option<&str>) -> Result<Vec<Order>> {
        #[derive(Deserialize)]
        struct OrdersResponse {
            orders: Vec<KalshiOrder>,
        }

        let path = match market_id {
            Some(m) => format!("/portfolio/orders?ticker={}", m),
            None => "/portfolio/orders".to_string(),
        };

        let response: OrdersResponse = self
            .request(reqwest::Method::GET, &path, None)
            .await?;

        let orders = response
            .orders
            .into_iter()
            .map(|o| Order {
                id: o.id,
                market_id: o.ticker,
                side: if o.side > 0 { Side::Yes } else { Side::No },
                order_type: match o.order_type.as_str() {
                    "market" => OrderType::Market,
                    "gtc" => OrderType::Gtc,
                    "gtd" => OrderType::Gtd,
                    "fok" => OrderType::Fok,
                    _ => OrderType::Limit,
                },
                price: Decimal::from(o.price) / Decimal::from(100),
                original_size: Decimal::from(o.original_quantity),
                remaining_size: Decimal::from(o.remaining_quantity),
                status: o.status,
                created_at: o.creation_time.parse().unwrap_or(0),
            })
            .collect();

        Ok(orders)
    }

    pub async fn get_positions(&self) -> Result<Vec<Position>> {
        #[derive(Deserialize)]
        struct PositionsResponse {
            positions: Vec<KalshiPosition>,
        }

        #[derive(Deserialize)]
        struct KalshiPosition {
            ticker: String,
            position: i64,
            total_cost: i64,
            avg_price: i64,
        }

        let response: PositionsResponse = self
            .request(reqwest::Method::GET, "/portfolio/positions", None)
            .await?;

        let positions = response
            .positions
            .into_iter()
            .map(|p| Position {
                market_id: p.ticker,
                side: if p.position > 0 { Side::Yes } else { Side::No },
                size: Decimal::from(p.position.abs()),
                avg_price: Decimal::from(p.avg_price) / Decimal::from(100),
                total_cost: Decimal::from(p.total_cost) / Decimal::from(100),
            })
            .collect();

        Ok(positions)
    }

    pub async fn cancel_order(&self, order_id: &str) -> Result<bool> {
        #[derive(Deserialize)]
        struct CancelResponse {
            success: bool,
        }

        let path = format!("/portfolio/orders/{}", order_id);
        let response: CancelResponse = self
            .request(reqwest::Method::DELETE, &path, None)
            .await?;

        info!("Cancelled Kalshi order: {} (success: {})", order_id, response.success);
        Ok(response.success)
    }

    pub async fn subscribe_markets(&self, market_ids: Vec<String>) -> Result<Receiver<PriceUpdate>> {
        let (tx, rx) = mpsc::channel(1000);
        let ws_url = self.ws_url.clone();
        let api_key = self.api_key.clone();
        
        tokio::spawn(async move {
            if let Err(e) = Self::run_websocket(ws_url, api_key, market_ids, tx).await {
                error!("WebSocket error: {}", e);
            }
        });

        Ok(rx)
    }

    async fn run_websocket(
        ws_url: String,
        api_key: String,
        market_ids: Vec<String>,
        tx: Sender<PriceUpdate>,
    ) -> Result<()> {
        let mut retry_delay = std::time::Duration::from_secs(1);
        let max_delay = std::time::Duration::from_secs(30);

        loop {
            info!("Connecting to Kalshi WebSocket: {}", ws_url);
            
            match connect_async(&ws_url).await {
                Ok((ws_stream, _)) => {
                    info!("Connected to Kalshi WebSocket");
                    retry_delay = std::time::Duration::from_secs(1);

                    let (mut write, mut read) = ws_stream.split();

                    let subscribe_msg = serde_json::json!({
                        "type": "subscribe",
                        "channels": ["orderbook"],
                        "markets": market_ids,
                        "api_key": api_key,
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
        struct WsOrderbookUpdate {
            #[serde(rename = "type")]
            msg_type: String,
            market: String,
            side: String,
            price: i64,
            size: i64,
            timestamp: Option<u64>,
        }

        let update: WsOrderbookUpdate = serde_json::from_str(text)
            .context("Failed to parse WebSocket message")?;

        if update.msg_type != "orderbook_update" {
            return Ok(());
        }

        let side = match update.side.to_lowercase().as_str() {
            "yes" => Side::Yes,
            "no" => Side::No,
            _ => return Ok(()),
        };

        let price_update = PriceUpdate {
            market_id: update.market,
            side,
            price: Decimal::from(update.price) / Decimal::from(100),
            size: Decimal::from(update.size),
            timestamp: update.timestamp.unwrap_or_else(|| Self::get_timestamp()),
        };

        tx.send(price_update)
            .await
            .context("Failed to send price update")?;

        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KalshiMarket {
    pub ticker: String,
    pub title: String,
    pub category: String,
    pub close_date: Option<String>,
    pub yes_bid: Option<Decimal>,
    pub yes_ask: Option<Decimal>,
    pub open_interest: i64,
    pub status: String,
}
