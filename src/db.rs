use chrono::{DateTime, Utc};
use rusqlite::{params, Connection, Result as SqlResult};
use std::io::Write;
use std::path::Path;

pub struct Database {
    conn: Connection,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct UsageRecord {
    pub timestamp: DateTime<Utc>,
    pub provider: String,
    pub metric: String,
    pub utilization: f64,
    pub resets_at: Option<String>,
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
        // Remove five_hour records with no active session (resets_at is NULL)
        self.conn.execute(
            "DELETE FROM usage_history WHERE metric = 'five_hour' AND resets_at IS NULL",
            [],
        )?;
        Ok(())
    }

    pub fn insert(
        &self,
        provider: &str,
        metric: &str,
        utilization: f64,
        resets_at: Option<&str>,
    ) -> SqlResult<()> {
        self.conn.execute(
            "INSERT INTO usage_history (provider, metric, utilization, resets_at)
             VALUES (?1, ?2, ?3, ?4)",
            params![provider, metric, utilization, resets_at],
        )?;
        Ok(())
    }

    /// Query last 24 hours of `five_hour` metric, bucketed into 30-minute intervals.
    /// Always returns exactly 48 elements (oldest first: index 0 = 24h ago, index 47 = now).
    /// Missing slots are filled with 0.0.
    pub fn query_24h_chart(&self) -> SqlResult<Vec<f64>> {
        let mut stmt = self.conn.prepare(
            "SELECT
                CAST((julianday('now') - julianday(timestamp)) * 48 AS INTEGER) AS bucket,
                AVG(utilization) AS avg_util
             FROM usage_history
             WHERE provider = 'claude'
               AND metric = 'five_hour'
               AND resets_at IS NOT NULL
               AND timestamp > datetime('now', '-24 hours')
             GROUP BY bucket",
        )?;

        let mut slots = vec![0.0f64; 48];

        let rows = stmt.query_map([], |row| Ok((row.get::<_, i64>(0)?, row.get::<_, f64>(1)?)))?;

        for row in rows.flatten() {
            let (bucket, util) = row;
            // bucket 0 = now, bucket 47 = ~24h ago
            // We want index 0 = oldest, index 47 = newest
            let idx = 47 - bucket.clamp(0, 47) as usize;
            slots[idx] = util;
        }

        Ok(slots)
    }

    /// Query last 7 days of `five_hour` metric, bucketed into 4-hour intervals.
    /// Always returns exactly 42 elements (oldest first: index 0 = 7d ago, index 41 = now).
    /// Missing slots are filled with 0.0.
    pub fn query_7d_chart(&self) -> SqlResult<Vec<f64>> {
        let mut stmt = self.conn.prepare(
            "SELECT
                CAST((julianday('now') - julianday(timestamp)) * 6 AS INTEGER) AS bucket,
                AVG(utilization) AS avg_util
             FROM usage_history
             WHERE provider = 'claude'
               AND metric = 'five_hour'
               AND resets_at IS NOT NULL
               AND timestamp > datetime('now', '-7 days')
             GROUP BY bucket",
        )?;

        let mut slots = vec![0.0f64; 42];

        let rows = stmt.query_map([], |row| Ok((row.get::<_, i64>(0)?, row.get::<_, f64>(1)?)))?;

        for row in rows.flatten() {
            let (bucket, util) = row;
            let idx = 41 - bucket.clamp(0, 41) as usize;
            slots[idx] = util;
        }

        Ok(slots)
    }

    /// Query the most recent utilization value for each metric.
    /// Returns a list of (metric_name, utilization, resets_at) tuples.
    pub fn query_latest(&self) -> SqlResult<Vec<(String, f64, Option<String>)>> {
        let mut stmt = self.conn.prepare(
            "SELECT metric, utilization, resets_at
             FROM usage_history
             WHERE provider = 'claude'
               AND id IN (
                   SELECT MAX(id) FROM usage_history
                   WHERE provider = 'claude'
                   GROUP BY metric
               )",
        )?;

        let rows = stmt.query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, f64>(1)?,
                row.get::<_, Option<String>>(2)?,
            ))
        })?;

        Ok(rows.flatten().collect())
    }

    /// Export all usage history to a CSV file. Returns the number of rows written.
    pub fn export_csv(&self, path: &Path) -> SqlResult<usize> {
        let mut stmt = self.conn.prepare(
            "SELECT timestamp, provider, metric, utilization, resets_at
             FROM usage_history ORDER BY timestamp DESC",
        )?;

        let mut file = std::fs::File::create(path)
            .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?;

        let _ = writeln!(file, "timestamp,provider,metric,utilization,resets_at");
        let mut count = 0usize;

        let rows = stmt.query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, f64>(3)?,
                row.get::<_, Option<String>>(4)?,
            ))
        })?;

        for (ts, provider, metric, util, resets) in rows.flatten() {
            let resets_str = resets.unwrap_or_default();
            let _ = writeln!(
                file,
                "{},{},{},{:.2},{}",
                ts, provider, metric, util, resets_str
            );
            count += 1;
        }

        Ok(count)
    }
}
