use anyhow::Result;
use chrono::{DateTime, Utc};
use parking_lot::Mutex;
use rusqlite::{params, Connection};
use rust_decimal::prelude::*;
use rust_decimal::Decimal;
use std::path::Path;
use std::str::FromStr;
use tracing::{debug, info, instrument};

pub struct SqliteStorage {
    conn: Mutex<Connection>,
}

pub struct TrackingRecord {
    pub id: i64,
    pub market_key: String,
    pub platform: String,
    pub entry_price: Decimal,
    pub exit_price: Option<Decimal>,
    pub profit: Option<Decimal>,
    pub timestamp: DateTime<Utc>,
}

pub struct AutoTradeRecord {
    pub id: i64,
    pub market_key: String,
    pub amount: Decimal,
    pub profit: Decimal,
    pub timestamp: DateTime<Utc>,
}

pub struct AppSettings {
    pub min_profit_margin: Decimal,
    pub default_bet_amount: Decimal,
    pub auto_trade_enabled: bool,
    pub auto_trade_max_amount: Decimal,
    pub auto_trade_max_count: u32,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            min_profit_margin: Decimal::from_str("0.02").unwrap(),
            default_bet_amount: Decimal::from_str("10").unwrap(),
            auto_trade_enabled: false,
            auto_trade_max_amount: Decimal::from_str("100").unwrap(),
            auto_trade_max_count: 10,
        }
    }
}

impl SqliteStorage {
    #[instrument(skip(path))]
    pub fn new(path: &Path) -> Result<Self> {
        info!("Initializing SQLite storage at {:?}", path);

        let conn = Connection::open(path)?;
        let storage = Self {
            conn: Mutex::new(conn),
        };

        storage.init_tables()?;
        info!("SQLite storage initialized successfully");
        Ok(storage)
    }

    fn init_tables(&self) -> Result<()> {
        debug!("Initializing database tables");
        let conn = self.conn.lock();

        conn.execute_batch(
            r#"
            CREATE TABLE IF NOT EXISTS tracking (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                market_key TEXT NOT NULL,
                platform TEXT NOT NULL,
                entry_price TEXT NOT NULL,
                exit_price TEXT,
                profit TEXT,
                timestamp TEXT NOT NULL
            );

            CREATE INDEX IF NOT EXISTS idx_tracking_timestamp ON tracking(timestamp);
            CREATE INDEX IF NOT EXISTS idx_tracking_market_key ON tracking(market_key);

            CREATE TABLE IF NOT EXISTS auto_trade (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                market_key TEXT NOT NULL,
                amount TEXT NOT NULL,
                profit TEXT NOT NULL,
                timestamp TEXT NOT NULL
            );

            CREATE INDEX IF NOT EXISTS idx_auto_trade_timestamp ON auto_trade(timestamp);
            CREATE INDEX IF NOT EXISTS idx_auto_trade_market_key ON auto_trade(market_key);

            CREATE TABLE IF NOT EXISTS settings (
                id INTEGER PRIMARY KEY CHECK (id = 1),
                min_profit_margin TEXT NOT NULL,
                default_bet_amount TEXT NOT NULL,
                auto_trade_enabled INTEGER NOT NULL,
                auto_trade_max_amount TEXT NOT NULL,
                auto_trade_max_count INTEGER NOT NULL
            );

            CREATE TABLE IF NOT EXISTS excluded_markets (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                market_key TEXT NOT NULL UNIQUE
            );

            CREATE INDEX IF NOT EXISTS idx_excluded_markets_key ON excluded_markets(market_key);
            "#,
        )?;

        let settings_exist: bool = conn.query_row(
            "SELECT EXISTS(SELECT 1 FROM settings WHERE id = 1)",
            [],
            |row| row.get(0),
        )?;

        if !settings_exist {
            let defaults = AppSettings::default();
            conn.execute(
                r#"
                INSERT INTO settings (id, min_profit_margin, default_bet_amount, auto_trade_enabled, auto_trade_max_amount, auto_trade_max_count)
                VALUES (1, ?1, ?2, ?3, ?4, ?5)
                "#,
                params![
                    defaults.min_profit_margin.to_string(),
                    defaults.default_bet_amount.to_string(),
                    defaults.auto_trade_enabled as i32,
                    defaults.auto_trade_max_amount.to_string(),
                    defaults.auto_trade_max_count as i32,
                ],
            )?;
            debug!("Inserted default settings");
        }

        Ok(())
    }

    #[instrument(skip(self, record))]
    pub fn track_opportunity(&self, record: &TrackingRecord) -> Result<i64> {
        debug!("Tracking opportunity for market: {}", record.market_key);
        let conn = self.conn.lock();

        conn.execute(
            r#"
            INSERT INTO tracking (market_key, platform, entry_price, exit_price, profit, timestamp)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6)
            "#,
            params![
                record.market_key,
                record.platform,
                record.entry_price.to_string(),
                record.exit_price.as_ref().map(|d| d.to_string()),
                record.profit.as_ref().map(|d| d.to_string()),
                record.timestamp.to_rfc3339(),
            ],
        )?;

        let id = conn.last_insert_rowid();
        info!("Created tracking record with id: {}", id);
        Ok(id)
    }

    #[instrument(skip(self))]
    pub fn update_tracking(&self, id: i64, exit_price: Decimal) -> Result<()> {
        debug!(
            "Updating tracking record {} with exit price: {}",
            id, exit_price
        );
        let conn = self.conn.lock();

        let rows_affected = conn.execute(
            "UPDATE tracking SET exit_price = ?1 WHERE id = ?2",
            params![exit_price.to_string(), id],
        )?;

        if rows_affected == 0 {
            anyhow::bail!("Tracking record {} not found", id);
        }

        Ok(())
    }

    #[instrument(skip(self))]
    pub fn end_tracking(&self, id: i64, profit: Decimal) -> Result<()> {
        debug!("Ending tracking record {} with profit: {}", id, profit);
        let conn = self.conn.lock();

        let rows_affected = conn.execute(
            "UPDATE tracking SET profit = ?1 WHERE id = ?2",
            params![profit.to_string(), id],
        )?;

        if rows_affected == 0 {
            anyhow::bail!("Tracking record {} not found", id);
        }

        info!("Ended tracking record {} with profit: {}", id, profit);
        Ok(())
    }

    #[instrument(skip(self))]
    pub fn get_history(&self, limit: i64, offset: i64) -> Result<Vec<TrackingRecord>> {
        debug!(
            "Fetching tracking history (limit: {}, offset: {})",
            limit, offset
        );
        let conn = self.conn.lock();

        let mut stmt = conn.prepare(
            r#"
            SELECT id, market_key, platform, entry_price, exit_price, profit, timestamp
            FROM tracking
            ORDER BY timestamp DESC
            LIMIT ?1 OFFSET ?2
            "#,
        )?;

        let records = stmt
            .query_map(params![limit, offset], |row| {
                let entry_price_str: String = row.get(3)?;
                let exit_price_str: Option<String> = row.get(4)?;
                let profit_str: Option<String> = row.get(5)?;
                let timestamp_str: String = row.get(6)?;

                Ok(TrackingRecord {
                    id: row.get(0)?,
                    market_key: row.get(1)?,
                    platform: row.get(2)?,
                    entry_price: Decimal::from_str(&entry_price_str).unwrap_or_default(),
                    exit_price: exit_price_str
                        .as_ref()
                        .and_then(|s| Decimal::from_str(s).ok()),
                    profit: profit_str.as_ref().and_then(|s| Decimal::from_str(s).ok()),
                    timestamp: DateTime::parse_from_rfc3339(&timestamp_str)
                        .map(|dt| dt.with_timezone(&Utc))
                        .unwrap_or_else(|_| Utc::now()),
                })
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;

        Ok(records)
    }

    #[instrument(skip(self, record))]
    pub fn save_auto_trade(&self, record: &AutoTradeRecord) -> Result<i64> {
        debug!("Saving auto trade for market: {}", record.market_key);
        let conn = self.conn.lock();

        conn.execute(
            r#"
            INSERT INTO auto_trade (market_key, amount, profit, timestamp)
            VALUES (?1, ?2, ?3, ?4)
            "#,
            params![
                record.market_key,
                record.amount.to_string(),
                record.profit.to_string(),
                record.timestamp.to_rfc3339(),
            ],
        )?;

        let id = conn.last_insert_rowid();
        info!("Saved auto trade record with id: {}", id);
        Ok(id)
    }

    #[instrument(skip(self))]
    pub fn get_auto_trade_history(&self, limit: i64) -> Result<Vec<AutoTradeRecord>> {
        debug!("Fetching auto trade history (limit: {})", limit);
        let conn = self.conn.lock();

        let mut stmt = conn.prepare(
            r#"
            SELECT id, market_key, amount, profit, timestamp
            FROM auto_trade
            ORDER BY timestamp DESC
            LIMIT ?1
            "#,
        )?;

        let records = stmt
            .query_map(params![limit], |row| {
                let amount_str: String = row.get(2)?;
                let profit_str: String = row.get(3)?;
                let timestamp_str: String = row.get(4)?;

                Ok(AutoTradeRecord {
                    id: row.get(0)?,
                    market_key: row.get(1)?,
                    amount: Decimal::from_str(&amount_str).unwrap_or_default(),
                    profit: Decimal::from_str(&profit_str).unwrap_or_default(),
                    timestamp: DateTime::parse_from_rfc3339(&timestamp_str)
                        .map(|dt| dt.with_timezone(&Utc))
                        .unwrap_or_else(|_| Utc::now()),
                })
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;

        Ok(records)
    }

    #[instrument(skip(self))]
    pub fn get_auto_trade_count_today(&self) -> Result<u32> {
        debug!("Counting auto trades for today");
        let conn = self.conn.lock();

        let today_start = Utc::now()
            .date_naive()
            .and_hms_opt(0, 0, 0)
            .unwrap()
            .and_utc();

        let count: u32 = conn.query_row(
            r#"
            SELECT COUNT(*) FROM auto_trade
            WHERE timestamp >= ?1
            "#,
            params![today_start.to_rfc3339()],
            |row| row.get(0),
        )?;

        debug!("Auto trade count today: {}", count);
        Ok(count)
    }

    #[instrument(skip(self))]
    pub fn get_settings(&self) -> Result<AppSettings> {
        debug!("Fetching app settings");
        let conn = self.conn.lock();

        let settings = conn.query_row(
            r#"
            SELECT min_profit_margin, default_bet_amount, auto_trade_enabled, auto_trade_max_amount, auto_trade_max_count
            FROM settings
            WHERE id = 1
            "#,
            [],
            |row| {
                let min_profit_margin_str: String = row.get(0)?;
                let default_bet_amount_str: String = row.get(1)?;
                let auto_trade_enabled: i32 = row.get(2)?;
                let auto_trade_max_amount_str: String = row.get(3)?;
                let auto_trade_max_count: i32 = row.get(4)?;

                Ok(AppSettings {
                    min_profit_margin: Decimal::from_str(&min_profit_margin_str).unwrap_or_default(),
                    default_bet_amount: Decimal::from_str(&default_bet_amount_str).unwrap_or_default(),
                    auto_trade_enabled: auto_trade_enabled != 0,
                    auto_trade_max_amount: Decimal::from_str(&auto_trade_max_amount_str).unwrap_or_default(),
                    auto_trade_max_count: auto_trade_max_count as u32,
                })
            },
        )?;

        Ok(settings)
    }

    #[instrument(skip(self, settings))]
    pub fn update_settings(&self, settings: &AppSettings) -> Result<()> {
        debug!("Updating app settings");
        let conn = self.conn.lock();

        conn.execute(
            r#"
            UPDATE settings
            SET min_profit_margin = ?1,
                default_bet_amount = ?2,
                auto_trade_enabled = ?3,
                auto_trade_max_amount = ?4,
                auto_trade_max_count = ?5
            WHERE id = 1
            "#,
            params![
                settings.min_profit_margin.to_string(),
                settings.default_bet_amount.to_string(),
                settings.auto_trade_enabled as i32,
                settings.auto_trade_max_amount.to_string(),
                settings.auto_trade_max_count as i32,
            ],
        )?;

        info!("App settings updated");
        Ok(())
    }

    #[instrument(skip(self))]
    pub fn exclude_market(&self, market_key: &str) -> Result<()> {
        debug!("Excluding market: {}", market_key);
        let conn = self.conn.lock();

        conn.execute(
            "INSERT OR IGNORE INTO excluded_markets (market_key) VALUES (?1)",
            params![market_key],
        )?;

        info!("Market excluded: {}", market_key);
        Ok(())
    }

    #[instrument(skip(self))]
    pub fn unexclude_market(&self, market_key: &str) -> Result<()> {
        debug!("Unexcluding market: {}", market_key);
        let conn = self.conn.lock();

        conn.execute(
            "DELETE FROM excluded_markets WHERE market_key = ?1",
            params![market_key],
        )?;

        info!("Market unexcluded: {}", market_key);
        Ok(())
    }

    #[instrument(skip(self))]
    pub fn is_market_excluded(&self, market_key: &str) -> Result<bool> {
        debug!("Checking if market is excluded: {}", market_key);
        let conn = self.conn.lock();

        let excluded: bool = conn.query_row(
            "SELECT EXISTS(SELECT 1 FROM excluded_markets WHERE market_key = ?1)",
            params![market_key],
            |row| row.get(0),
        )?;

        Ok(excluded)
    }

    #[instrument(skip(self))]
    pub fn get_excluded_markets(&self) -> Result<Vec<String>> {
        debug!("Fetching all excluded markets");
        let conn = self.conn.lock();

        let mut stmt =
            conn.prepare("SELECT market_key FROM excluded_markets ORDER BY market_key")?;

        let markets = stmt
            .query_map([], |row| row.get(0))?
            .collect::<std::result::Result<Vec<_>, _>>()?;

        Ok(markets)
    }
}
