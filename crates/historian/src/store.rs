//! SQLite store for historical tag samples.

use std::path::Path;
use std::sync::{Arc, Mutex};

use openwebhmi_protocol::{Quality, TagValue};
use rusqlite::{Connection, DatabaseName, OptionalExtension, params};
use serde::{Deserialize, Serialize};

use crate::aggregations::{Aggregation, aggregate};

/// A historical point returned to clients.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HistoryPoint {
    /// Unix epoch milliseconds.
    pub ts_ms: u64,
    /// Recorded value.
    pub value: TagValue,
    /// Recorded quality.
    pub quality: Quality,
}

/// SQLite-backed historian store.
#[derive(Clone)]
pub struct HistorianStore {
    conn: Arc<Mutex<Connection>>,
}

impl HistorianStore {
    /// Open a historian database and create the schema if needed.
    pub fn open(path: impl AsRef<Path>) -> anyhow::Result<Self> {
        let conn = Connection::open(path)?;
        init_schema(&conn)?;
        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
        })
    }

    /// Open an in-memory historian database.
    pub fn memory() -> anyhow::Result<Self> {
        let conn = Connection::open_in_memory()?;
        init_schema(&conn)?;
        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
        })
    }

    /// Write one sample.
    pub fn write_sample(
        &self,
        tag_path: &str,
        ts_ms: u64,
        value: &TagValue,
        quality: Quality,
    ) -> anyhow::Result<()> {
        let mut conn = self.lock()?;
        let tx = conn.transaction()?;
        let tag_id = intern_tag(&tx, tag_path)?;
        tx.execute(
            "INSERT OR REPLACE INTO tag_history (tag_id, ts_ms, value, quality)
             VALUES (?1, ?2, ?3, ?4)",
            params![
                tag_id,
                millis_to_i64(ts_ms),
                serde_json::to_string(value)?,
                serde_json::to_string(&quality)?.trim_matches('"')
            ],
        )?;
        tx.commit()?;
        Ok(())
    }

    /// Read raw or aggregated samples for a tag.
    pub fn read(
        &self,
        tag_path: &str,
        t_start_ms: u64,
        t_end_ms: u64,
        aggregation: Aggregation,
        max_points: u32,
    ) -> anyhow::Result<Vec<HistoryPoint>> {
        let points = self.read_raw_unbounded(tag_path, t_start_ms, t_end_ms)?;
        Ok(aggregate(
            &points,
            t_start_ms,
            t_end_ms,
            aggregation,
            max_points,
        )?)
    }

    /// Snapshot the live SQLite database to `path` using SQLite's online backup API.
    pub fn backup_to_path(&self, path: impl AsRef<Path>) -> anyhow::Result<()> {
        self.lock()?.backup(DatabaseName::Main, path, None)?;
        Ok(())
    }

    fn read_raw_unbounded(
        &self,
        tag_path: &str,
        t_start_ms: u64,
        t_end_ms: u64,
    ) -> anyhow::Result<Vec<HistoryPoint>> {
        let conn = self.lock()?;
        let Some(tag_id) = tag_id(&conn, tag_path)? else {
            return Ok(Vec::new());
        };
        let mut stmt = conn.prepare(
            "SELECT ts_ms, value, quality
             FROM tag_history
             WHERE tag_id = ?1 AND ts_ms >= ?2 AND ts_ms <= ?3
             ORDER BY ts_ms ASC",
        )?;
        let rows = stmt.query_map(
            params![tag_id, millis_to_i64(t_start_ms), millis_to_i64(t_end_ms)],
            |row| {
                let value: String = row.get(1)?;
                let quality: String = row.get(2)?;
                Ok(HistoryPoint {
                    ts_ms: row.get::<_, i64>(0)? as u64,
                    value: serde_json::from_str(&value).map_err(to_sql_error)?,
                    quality: serde_json::from_str(&format!("\"{quality}\""))
                        .map_err(to_sql_error)?,
                })
            },
        )?;
        rows.collect::<Result<Vec<_>, _>>().map_err(Into::into)
    }

    fn lock(&self) -> anyhow::Result<std::sync::MutexGuard<'_, Connection>> {
        self.conn
            .lock()
            .map_err(|_| anyhow::anyhow!("historian sqlite lock poisoned"))
    }
}

fn millis_to_i64(value: u64) -> i64 {
    value.min(i64::MAX as u64) as i64
}

fn init_schema(conn: &Connection) -> anyhow::Result<()> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS tag_dictionary (
            tag_id INTEGER PRIMARY KEY AUTOINCREMENT,
            tag_path TEXT UNIQUE NOT NULL
        );
        CREATE TABLE IF NOT EXISTS tag_history (
            tag_id INTEGER NOT NULL,
            ts_ms  INTEGER NOT NULL,
            value  TEXT    NOT NULL,
            quality TEXT   NOT NULL,
            PRIMARY KEY (tag_id, ts_ms)
        ) WITHOUT ROWID;",
    )?;
    Ok(())
}

fn intern_tag(conn: &Connection, tag_path: &str) -> anyhow::Result<i64> {
    conn.execute(
        "INSERT OR IGNORE INTO tag_dictionary (tag_path) VALUES (?1)",
        params![tag_path],
    )?;
    tag_id(conn, tag_path)?.ok_or_else(|| anyhow::anyhow!("failed to intern tag path"))
}

fn tag_id(conn: &Connection, tag_path: &str) -> anyhow::Result<Option<i64>> {
    conn.query_row(
        "SELECT tag_id FROM tag_dictionary WHERE tag_path = ?1",
        params![tag_path],
        |row| row.get(0),
    )
    .optional()
    .map_err(Into::into)
}

fn to_sql_error(err: serde_json::Error) -> rusqlite::Error {
    rusqlite::Error::FromSqlConversionFailure(0, rusqlite::types::Type::Text, Box::new(err))
}
