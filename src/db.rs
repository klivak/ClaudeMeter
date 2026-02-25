use chrono::{DateTime, Utc};
use rusqlite::{Connection, Result as SqlResult, params};
use std::path::Path;

pub struct Database {
    conn: Connection,
}

#[derive(Debug, Clone)]
pub struct UsageRecord {
    pub timestamp: DateTime<Utc>,
    pub provider: String,
    pub metric: String,
    pub utilization: f64,
    pub resets_at: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ChartPoint {
    /// Hours ago from now (0 = now, 24 = 24h ago)
    pub bucket_hours_ago: f64,
    pub utilization: f64,
}

impl Database {
    pub fn open(exe_dir: &Path) -> SqlResult<Self> {
        let db_path = exe_dir.join("claudemeter.db");
        let conn = Connection::open(db_path)?;
        let db = Self { conn };
        db.init()?;
        Ok(db)
    }

    fn init(&self) -> SqlResult<()> {
        self.conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS usage_history (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                timestamp TEXT NOT NULL DEFAULT (datetime('now')),
                provider TEXT NOT NULL,
                metric TEXT NOT NULL,
                utilization REAL NOT NULL,
                resets_at TEXT
            );
            CREATE INDEX IF NOT EXISTS idx_history_time
                ON usage_history(timestamp);
            CREATE INDEX IF NOT EXISTS idx_history_provider
                ON usage_history(provider, metric);",
        )?;
        // Clean up old records (> 30 days) on startup
        self.conn.execute(
            "DELETE FROM usage_history WHERE timestamp < datetime('now', '-30 days')",
            [],
        )?;
        Ok(())
    }

    pub fn insert(&self, provider: &str, metric: &str, utilization: f64, resets_at: Option<&str>) -> SqlResult<()> {
        self.conn.execute(
            "INSERT INTO usage_history (provider, metric, utilization, resets_at)
             VALUES (?1, ?2, ?3, ?4)",
            params![provider, metric, utilization, resets_at],
        )?;
        Ok(())
    }

    /// Query last 24 hours of `five_hour` metric, bucketed into 30-minute intervals.
    /// Returns up to 48 points ordered by time (oldest first).
    pub fn query_24h_chart(&self) -> SqlResult<Vec<ChartPoint>> {
        let mut stmt = self.conn.prepare(
            "SELECT
                (julianday('now') - julianday(timestamp)) * 24.0 AS hours_ago,
                AVG(utilization) AS avg_util
             FROM usage_history
             WHERE provider = 'claude'
               AND metric = 'five_hour'
               AND timestamp > datetime('now', '-24 hours')
             GROUP BY CAST((julianday('now') - julianday(timestamp)) * 48 AS INTEGER)
             ORDER BY hours_ago DESC",
        )?;

        let points = stmt.query_map([], |row| {
            Ok(ChartPoint {
                bucket_hours_ago: row.get::<_, f64>(0)?,
                utilization: row.get::<_, f64>(1)?,
            })
        })?
        .filter_map(|r| r.ok())
        .collect();

        Ok(points)
    }
}
