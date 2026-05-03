//! SQLite audit-log store.

use std::path::Path;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

use rusqlite::{Connection, params};
use tokio::sync::broadcast;

use crate::{AuditEntry, AuditEvent, AuditQuery};

const SCHEMA_VERSION: u32 = 1;
const MAX_LIMIT: usize = 1000;

/// SQLite-backed append-only audit log.
#[derive(Clone)]
pub struct AuditLog {
    conn: Arc<Mutex<Connection>>,
    events: broadcast::Sender<AuditEntry>,
}

impl AuditLog {
    /// Open an audit database and apply default retention.
    pub fn open(path: impl AsRef<Path>, retention_days: u64) -> anyhow::Result<Self> {
        let conn = Connection::open(path)?;
        init_schema(&conn)?;
        prune_retention(&conn, retention_days)?;
        let (events, _) = broadcast::channel(1024);
        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
            events,
        })
    }

    /// Open an in-memory audit log.
    pub fn memory() -> anyhow::Result<Self> {
        let conn = Connection::open_in_memory()?;
        init_schema(&conn)?;
        let (events, _) = broadcast::channel(1024);
        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
            events,
        })
    }

    /// Subscribe to newly appended entries.
    pub fn subscribe(&self) -> broadcast::Receiver<AuditEntry> {
        self.events.subscribe()
    }

    /// Append a new audit event with current timestamp.
    pub fn append(
        &self,
        user: Option<String>,
        session_id: Option<String>,
        source_ip: Option<String>,
        event: AuditEvent,
    ) -> anyhow::Result<AuditEntry> {
        self.append_at(now_ms(), user, session_id, source_ip, event)
    }

    /// Append a new audit event with an explicit timestamp.
    pub fn append_at(
        &self,
        ts_ms: u64,
        user: Option<String>,
        session_id: Option<String>,
        source_ip: Option<String>,
        event: AuditEvent,
    ) -> anyhow::Result<AuditEntry> {
        let payload = serde_json::to_string(&event)?;
        let kind = event.kind_name();
        let conn = self.lock()?;
        conn.execute(
            "INSERT INTO audit_log (ts_ms, user, session_id, source_ip, kind, payload)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                millis_to_i64(ts_ms),
                user,
                session_id,
                source_ip,
                kind,
                payload
            ],
        )?;
        let id = conn.last_insert_rowid() as u64;
        let entry = AuditEntry {
            id,
            ts_ms,
            user,
            session_id,
            source_ip,
            kind: event,
        };
        let _ = self.events.send(entry.clone());
        Ok(entry)
    }

    /// Query entries and total matching count.
    pub fn query(&self, query: &AuditQuery) -> anyhow::Result<(Vec<AuditEntry>, u64)> {
        let conn = self.lock()?;
        let entries = read_all(&conn)?;
        let mut filtered = filter_entries(entries, query);
        let total = filtered.len() as u64;
        let offset = query.offset.min(filtered.len());
        let limit = query.limit.min(MAX_LIMIT);
        filtered = filtered.into_iter().skip(offset).take(limit).collect();
        Ok((filtered, total))
    }

    /// Drop entries older than the retention cutoff.
    pub fn prune_retention(&self, retention_days: u64) -> anyhow::Result<()> {
        let conn = self.lock()?;
        prune_retention(&conn, retention_days)
    }

    fn lock(&self) -> anyhow::Result<std::sync::MutexGuard<'_, Connection>> {
        self.conn
            .lock()
            .map_err(|_| anyhow::anyhow!("audit log sqlite lock poisoned"))
    }
}

fn init_schema(conn: &Connection) -> anyhow::Result<()> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS meta (
            key TEXT PRIMARY KEY,
            value TEXT NOT NULL
        );
        INSERT OR IGNORE INTO meta (key, value) VALUES ('schema_version', '1');
        CREATE TABLE IF NOT EXISTS audit_log (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            ts_ms INTEGER NOT NULL,
            user TEXT,
            session_id TEXT,
            source_ip TEXT,
            kind TEXT NOT NULL,
            payload TEXT NOT NULL
        );
        CREATE INDEX IF NOT EXISTS idx_audit_ts_user ON audit_log(ts_ms, user);
        CREATE INDEX IF NOT EXISTS idx_audit_kind ON audit_log(kind, ts_ms);",
    )?;
    let version: String = conn.query_row(
        "SELECT value FROM meta WHERE key = 'schema_version'",
        [],
        |row| row.get(0),
    )?;
    if version != SCHEMA_VERSION.to_string() {
        anyhow::bail!("unsupported audit schema version {version}");
    }
    Ok(())
}

fn read_all(conn: &Connection) -> anyhow::Result<Vec<AuditEntry>> {
    let mut stmt = conn.prepare(
        "SELECT id, ts_ms, user, session_id, source_ip, payload
         FROM audit_log
         ORDER BY ts_ms DESC, id DESC",
    )?;
    let rows = stmt.query_map([], |row| {
        let payload: String = row.get(5)?;
        let event = serde_json::from_str(&payload).map_err(|err| {
            rusqlite::Error::FromSqlConversionFailure(5, rusqlite::types::Type::Text, Box::new(err))
        })?;
        Ok(AuditEntry {
            id: row.get::<_, i64>(0)? as u64,
            ts_ms: row.get::<_, i64>(1)? as u64,
            user: row.get(2)?,
            session_id: row.get(3)?,
            source_ip: row.get(4)?,
            kind: event,
        })
    })?;
    rows.collect::<Result<Vec<_>, _>>().map_err(Into::into)
}

fn filter_entries(entries: Vec<AuditEntry>, query: &AuditQuery) -> Vec<AuditEntry> {
    entries
        .into_iter()
        .filter(|entry| {
            query.from_ts_ms.is_none_or(|from| entry.ts_ms >= from)
                && query.to_ts_ms.is_none_or(|to| entry.ts_ms <= to)
                && query.user.as_ref().is_none_or(|user| {
                    entry
                        .user
                        .as_ref()
                        .is_some_and(|entry_user| entry_user == user)
                })
                && (query.kinds.is_empty()
                    || query
                        .kinds
                        .iter()
                        .any(|kind| kind == entry.kind.kind_name()))
        })
        .collect()
}

fn prune_retention(conn: &Connection, retention_days: u64) -> anyhow::Result<()> {
    if retention_days == 0 {
        return Ok(());
    }
    let retention_ms = retention_days.saturating_mul(24 * 60 * 60 * 1000);
    let cutoff = now_ms().saturating_sub(retention_ms);
    conn.execute(
        "DELETE FROM audit_log WHERE ts_ms < ?1",
        params![millis_to_i64(cutoff)],
    )?;
    Ok(())
}

fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis() as u64)
        .unwrap_or_default()
}

fn millis_to_i64(value: u64) -> i64 {
    value.min(i64::MAX as u64) as i64
}
