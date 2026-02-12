use polkas_backend::{
    api::{create_router, state::AppState},
    clients::{KalshiClient, PolymarketClient},
    config::Config,
    services::{ArbitrageCalculator, ArbitrageService, KalshiClient as ServiceKalshiClient, MarketMatcher, PolymarketClient as ServicePolymarketClient},
    storage::Storage,
};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::broadcast;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "info".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let config = Config::from_file("config.toml").unwrap_or_else(|_| Config::default());
    
    let storage = Arc::new(Storage::new(std::path::Path::new(&config.database.path))?);
    
    let kalshi_client = KalshiClient::new(&config.kalshi)?;
    let polymarket_client = PolymarketClient::new(&config.polymarket).await?;
    
    let (price_tx, _) = broadcast::channel(1024);
    
    let service = ArbitrageService {
        kalshi: Arc::new(ServiceKalshiClient::new(
            &config.kalshi.api_url,
            &config.kalshi.api_key,
            config.kalshi.rate_limit_per_second,
        )),
        polymarket: Arc::new(ServicePolymarketClient::new(
            &config.polymarket.api_url,
            config.polymarket.chain_id,
        )),
        calculator: ArbitrageCalculator::new(
            config.arbitrage.min_profit_threshold,
            rust_decimal::Decimal::new(7, 2),
        ),
        matcher: MarketMatcher::new(24),
        storage: storage.clone(),
        matched_markets: Arc::new(tokio::sync::RwLock::new(Vec::new())),
        opportunities: Arc::new(tokio::sync::RwLock::new(Vec::new())),
        price_tx,
        min_profit_threshold: config.arbitrage.min_profit_threshold,
        max_position_size: config.arbitrage.max_position_size,
        execution_timeout_ms: config.arbitrage.execution_timeout_ms,
        retry_attempts: config.arbitrage.retry_attempts,
        retry_delay_ms: config.arbitrage.retry_delay_ms,
        running: Arc::new(tokio::sync::RwLock::new(false)),
    };
    
    let state = AppState::new(
        service,
        storage,
        kalshi_client,
        polymarket_client,
    );

    let app = create_router(state);

    let addr: SocketAddr = format!("{}:{}", config.server.host, config.server.port).parse()?;
    
    tracing::info!("Starting server on {}", addr);
    
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
