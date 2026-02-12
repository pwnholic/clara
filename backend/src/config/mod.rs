use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::path::Path;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("Failed to read config file: {0}")]
    Io(#[from] std::io::Error),
    #[error("Failed to parse config file: {0}")]
    Parse(#[from] toml::de::Error),
    #[error("Invalid configuration: {0}")]
    Validation(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub server: ServerConfig,
    pub kalshi: KalshiConfig,
    pub polymarket: PolymarketConfig,
    pub arbitrage: ArbitrageConfig,
    pub database: DatabaseConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KalshiConfig {
    pub api_url: String,
    pub ws_url: String,
    pub api_key: String,
    pub private_key_pem: String,
    pub rate_limit_per_second: u32,
    pub min_profit_threshold: Decimal,
    pub max_position_size: Decimal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolymarketConfig {
    pub api_url: String,
    pub ws_url: String,
    pub private_key: String,
    pub chain_id: u64,
    pub gas_price_multiplier: Decimal,
    pub min_profit_threshold: Decimal,
    pub max_position_size: Decimal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArbitrageConfig {
    pub min_profit_threshold: Decimal,
    pub max_position_size: Decimal,
    pub execution_timeout_ms: u64,
    pub retry_attempts: u32,
    pub retry_delay_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    pub path: String,
    pub pool_size: u32,
}

impl Config {
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, ConfigError> {
        let content = std::fs::read_to_string(path)?;
        let config: Config = toml::from_str(&content)?;
        config.validate()?;
        Ok(config)
    }

    pub fn from_env() -> Result<Self, ConfigError> {
        let config = Config {
            server: ServerConfig {
                host: std::env::var("SERVER_HOST").unwrap_or_else(|_| "0.0.0.0".to_string()),
                port: std::env::var("SERVER_PORT")
                    .ok()
                    .and_then(|p| p.parse().ok())
                    .unwrap_or(8080),
            },
            kalshi: KalshiConfig {
                api_url: std::env::var("KALSHI_API_URL")
                    .unwrap_or_else(|_| "https://api.kalshi.com".to_string()),
                ws_url: std::env::var("KALSHI_WS_URL")
                    .unwrap_or_else(|_| "wss://api.kalshi.com".to_string()),
                api_key: std::env::var("KALSHI_API_KEY")
                    .map_err(|_| ConfigError::Validation("KALSHI_API_KEY not set".into()))?,
                private_key_pem: std::env::var("KALSHI_PRIVATE_KEY_PEM").map_err(|_| {
                    ConfigError::Validation("KALSHI_PRIVATE_KEY_PEM not set".into())
                })?,
                rate_limit_per_second: std::env::var("KALSHI_RATE_LIMIT")
                    .ok()
                    .and_then(|r| r.parse().ok())
                    .unwrap_or(10),
                min_profit_threshold: Decimal::from_str_exact(
                    &std::env::var("KALSHI_MIN_PROFIT").unwrap_or_else(|_| "0.01".to_string()),
                )
                .map_err(|e| {
                    ConfigError::Validation(format!("Invalid KALSHI_MIN_PROFIT: {}", e))
                })?,
                max_position_size: Decimal::from_str_exact(
                    &std::env::var("KALSHI_MAX_POSITION").unwrap_or_else(|_| "10000".to_string()),
                )
                .map_err(|e| {
                    ConfigError::Validation(format!("Invalid KALSHI_MAX_POSITION: {}", e))
                })?,
            },
            polymarket: PolymarketConfig {
                api_url: std::env::var("POLYMARKET_API_URL")
                    .unwrap_or_else(|_| "https://clob.polymarket.com".to_string()),
                ws_url: std::env::var("POLYMARKET_WS_URL")
                    .unwrap_or_else(|_| "wss://ws-subscriptions-clob.polymarket.com".to_string()),
                private_key: std::env::var("POLYMARKET_PRIVATE_KEY").map_err(|_| {
                    ConfigError::Validation("POLYMARKET_PRIVATE_KEY not set".into())
                })?,
                chain_id: std::env::var("POLYMARKET_CHAIN_ID")
                    .ok()
                    .and_then(|c| c.parse().ok())
                    .unwrap_or(137),
                gas_price_multiplier: Decimal::from_str_exact(
                    &std::env::var("GAS_PRICE_MULTIPLIER").unwrap_or_else(|_| "1.1".to_string()),
                )
                .map_err(|e| {
                    ConfigError::Validation(format!("Invalid GAS_PRICE_MULTIPLIER: {}", e))
                })?,
                min_profit_threshold: Decimal::from_str_exact(
                    &std::env::var("POLYMARKET_MIN_PROFIT").unwrap_or_else(|_| "0.01".to_string()),
                )
                .map_err(|e| {
                    ConfigError::Validation(format!("Invalid POLYMARKET_MIN_PROFIT: {}", e))
                })?,
                max_position_size: Decimal::from_str_exact(
                    &std::env::var("POLYMARKET_MAX_POSITION")
                        .unwrap_or_else(|_| "10000".to_string()),
                )
                .map_err(|e| {
                    ConfigError::Validation(format!("Invalid POLYMARKET_MAX_POSITION: {}", e))
                })?,
            },
            arbitrage: ArbitrageConfig {
                min_profit_threshold: Decimal::from_str_exact(
                    &std::env::var("ARBITRAGE_MIN_PROFIT").unwrap_or_else(|_| "0.005".to_string()),
                )
                .map_err(|e| {
                    ConfigError::Validation(format!("Invalid ARBITRAGE_MIN_PROFIT: {}", e))
                })?,
                max_position_size: Decimal::from_str_exact(
                    &std::env::var("ARBITRAGE_MAX_POSITION").unwrap_or_else(|_| "5000".to_string()),
                )
                .map_err(|e| {
                    ConfigError::Validation(format!("Invalid ARBITRAGE_MAX_POSITION: {}", e))
                })?,
                execution_timeout_ms: std::env::var("EXECUTION_TIMEOUT_MS")
                    .ok()
                    .and_then(|t| t.parse().ok())
                    .unwrap_or(5000),
                retry_attempts: std::env::var("RETRY_ATTEMPTS")
                    .ok()
                    .and_then(|r| r.parse().ok())
                    .unwrap_or(3),
                retry_delay_ms: std::env::var("RETRY_DELAY_MS")
                    .ok()
                    .and_then(|d| d.parse().ok())
                    .unwrap_or(100),
            },
            database: DatabaseConfig {
                path: std::env::var("DATABASE_PATH").unwrap_or_else(|_| "polkas.db".to_string()),
                pool_size: std::env::var("DATABASE_POOL_SIZE")
                    .ok()
                    .and_then(|p| p.parse().ok())
                    .unwrap_or(5),
            },
        };
        config.validate()?;
        Ok(config)
    }

    fn validate(&self) -> Result<(), ConfigError> {
        if self.server.port == 0 {
            return Err(ConfigError::Validation("Server port cannot be 0".into()));
        }

        if self.kalshi.min_profit_threshold <= Decimal::ZERO {
            return Err(ConfigError::Validation(
                "Kalshi min_profit_threshold must be positive".into(),
            ));
        }

        if self.polymarket.min_profit_threshold <= Decimal::ZERO {
            return Err(ConfigError::Validation(
                "Polymarket min_profit_threshold must be positive".into(),
            ));
        }

        if self.arbitrage.min_profit_threshold <= Decimal::ZERO {
            return Err(ConfigError::Validation(
                "Arbitrage min_profit_threshold must be positive".into(),
            ));
        }

        if self.arbitrage.max_position_size <= Decimal::ZERO {
            return Err(ConfigError::Validation(
                "Arbitrage max_position_size must be positive".into(),
            ));
        }

        Ok(())
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            server: ServerConfig {
                host: "0.0.0.0".to_string(),
                port: 8080,
            },
            kalshi: KalshiConfig {
                api_url: "https://api.kalshi.com".to_string(),
                ws_url: "wss://api.kalshi.com".to_string(),
                api_key: String::new(),
                private_key_pem: String::new(),
                rate_limit_per_second: 10,
                min_profit_threshold: Decimal::new(1, 2),
                max_position_size: Decimal::from(10000),
            },
            polymarket: PolymarketConfig {
                api_url: "https://clob.polymarket.com".to_string(),
                ws_url: "wss://ws-subscriptions-clob.polymarket.com".to_string(),
                private_key: String::new(),
                chain_id: 137,
                gas_price_multiplier: Decimal::from_str_exact("1.1").unwrap(),
                min_profit_threshold: Decimal::new(1, 2),
                max_position_size: Decimal::from(10000),
            },
            arbitrage: ArbitrageConfig {
                min_profit_threshold: Decimal::from_str_exact("0.005").unwrap(),
                max_position_size: Decimal::from(5000),
                execution_timeout_ms: 5000,
                retry_attempts: 3,
                retry_delay_ms: 100,
            },
            database: DatabaseConfig {
                path: "polkas.db".to_string(),
                pool_size: 5,
            },
        }
    }
}
