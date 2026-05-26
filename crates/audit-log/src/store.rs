//! SQLite audit-log store.

use std::path::Path;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

use rusqlite::{Connection, params};
use serde::Serialize;
use sha2::{Digest, Sha256};
use thiserror::Error;
use tokio::sync::broadcast;

use crate::event::{decode_hash, encode_hash};
use crate::{AuditEntry, AuditEvent, AuditQuery};

const SCHEMA_VERSION: u32 = 1;
const MAX_LIMIT: usize = 1000;
const ZERO_HASH: [u8; 32] = [0; 32];

/// First hash-chain divergence found by audit verification.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
#[error(
    "audit hash chain broken at id {first_bad_id}: expected {expected_hash}, found {found_hash}"
)]
pub struct ChainBroken {
    /// First row whose stored hash or previous hash did not match.
    pub first_bad_id: u64,
    /// Expected hash for the row, hex encoded.
    pub expected_hash: String,
    /// Stored hash found in the row, hex encoded.
    pub found_hash: String,
}

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
        let conn = self.lock()?;
        let entry = append_entry(&conn, ts_ms, user, session_id, source_ip, event)?;
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

    /// Verify every audit row in id order against the tamper-evident hash chain.
    pub fn verify_chain(&self) -> Result<usize, ChainBroken> {
        let conn = self.lock().map_err(|err| ChainBroken {
            first_bad_id: 0,
            expected_hash: "lock".to_string(),
            found_hash: err.to_string(),
        })?;
        verify_chain(&conn).map_err(|err| match err {
            VerifyError::Broken(broken) => broken,
            VerifyError::Sql(err) | VerifyError::Json(err) => ChainBroken {
                first_bad_id: 0,
                expected_hash: "valid audit database".to_string(),
                found_hash: err,
            },
        })
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
            payload TEXT NOT NULL,
            prev_hash TEXT,
            hash TEXT NOT NULL DEFAULT '0000000000000000000000000000000000000000000000000000000000000000'
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
    ensure_hash_columns(conn)?;
    migrate_hash_chain(conn)?;
    Ok(())
}

fn ensure_hash_columns(conn: &Connection) -> anyhow::Result<()> {
    let mut stmt = conn.prepare("PRAGMA table_info(audit_log)")?;
    let columns = stmt
        .query_map([], |row| row.get::<_, String>(1))?
        .collect::<Result<Vec<_>, _>>()?;
    if !columns.iter().any(|column| column == "prev_hash") {
        conn.execute("ALTER TABLE audit_log ADD COLUMN prev_hash TEXT", [])?;
    }
    if !columns.iter().any(|column| column == "hash") {
        conn.execute(
            "ALTER TABLE audit_log ADD COLUMN hash TEXT NOT NULL DEFAULT '0000000000000000000000000000000000000000000000000000000000000000'",
            [],
        )?;
    }
    Ok(())
}

fn migrate_hash_chain(conn: &Connection) -> anyhow::Result<()> {
    let already_completed = conn.query_row(
        "SELECT value FROM meta WHERE key = 'hash_chain_migrated'",
        [],
        |row| row.get::<_, String>(0),
    );
    if matches!(already_completed.as_deref(), Ok("1")) {
        return Ok(());
    }

    let rows = read_rows_by_id(conn)?;
    let needs_migration = rows
        .iter()
        .any(|row| row.hash == ZERO_HASH || row.hash_text == encode_hash(&ZERO_HASH));
    if !needs_migration {
        conn.execute(
            "INSERT OR REPLACE INTO meta (key, value) VALUES ('hash_chain_migrated', '1')",
            [],
        )?;
        return Ok(());
    }

    let mut previous_hash = None;
    let mut migrated = 0_u64;
    for row in rows {
        let hash = compute_hash(previous_hash.as_ref(), &row)?;
        conn.execute(
            "UPDATE audit_log SET prev_hash = ?1, hash = ?2 WHERE id = ?3",
            params![
                previous_hash.as_ref().map(encode_hash),
                encode_hash(&hash),
                row.id as i64
            ],
        )?;
        previous_hash = Some(hash);
        migrated += 1;
    }
    conn.execute(
        "INSERT OR REPLACE INTO meta (key, value) VALUES ('hash_chain_migrated', '1')",
        [],
    )?;
    if migrated > 0 {
        let _ = append_entry(
            conn,
            now_ms(),
            None,
            None,
            None,
            AuditEvent::MigrationCompleted { rows: migrated },
        )?;
    }
    Ok(())
}

fn read_all(conn: &Connection) -> anyhow::Result<Vec<AuditEntry>> {
    let mut stmt = conn.prepare(
        "SELECT id, ts_ms, user, session_id, source_ip, payload, prev_hash, hash
         FROM audit_log
         ORDER BY ts_ms DESC, id DESC",
    )?;
    let rows = stmt.query_map([], |row| {
        let payload: String = row.get(5)?;
        let prev_hash_text: Option<String> = row.get(6)?;
        let hash_text: String = row.get(7)?;
        let event = serde_json::from_str(&payload).map_err(|err| {
            rusqlite::Error::FromSqlConversionFailure(5, rusqlite::types::Type::Text, Box::new(err))
        })?;
        let prev_hash = prev_hash_text
            .as_deref()
            .map(decode_hash)
            .transpose()
            .map_err(|err| {
                rusqlite::Error::FromSqlConversionFailure(
                    6,
                    rusqlite::types::Type::Text,
                    err.into(),
                )
            })?;
        let hash = decode_hash(&hash_text).map_err(|err| {
            rusqlite::Error::FromSqlConversionFailure(7, rusqlite::types::Type::Text, err.into())
        })?;
        Ok(AuditEntry {
            id: row.get::<_, i64>(0)? as u64,
            ts_ms: row.get::<_, i64>(1)? as u64,
            user: row.get(2)?,
            session_id: row.get(3)?,
            source_ip: row.get(4)?,
            prev_hash,
            hash,
            kind: event,
        })
    })?;
    rows.collect::<Result<Vec<_>, _>>().map_err(Into::into)
}

fn append_entry(
    conn: &Connection,
    ts_ms: u64,
    user: Option<String>,
    session_id: Option<String>,
    source_ip: Option<String>,
    event: AuditEvent,
) -> anyhow::Result<AuditEntry> {
    let payload = serde_json::to_string(&event)?;
    let kind = event.kind_name();
    let previous_hash = latest_hash(conn)?;
    let previous_hash_text = previous_hash.as_ref().map(encode_hash);
    conn.execute(
        "INSERT INTO audit_log (ts_ms, user, session_id, source_ip, kind, payload, prev_hash, hash)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        params![
            millis_to_i64(ts_ms),
            user,
            session_id,
            source_ip,
            kind,
            payload,
            previous_hash_text,
            encode_hash(&ZERO_HASH),
        ],
    )?;
    let id = conn.last_insert_rowid() as u64;
    let row = RawAuditRow {
        id,
        ts_ms,
        user: user.clone(),
        session_id: session_id.clone(),
        source_ip: source_ip.clone(),
        kind: kind.to_string(),
        payload,
        prev_hash: previous_hash,
        hash: ZERO_HASH,
        hash_text: encode_hash(&ZERO_HASH),
    };
    let hash = compute_hash(row.prev_hash.as_ref(), &row)?;
    conn.execute(
        "UPDATE audit_log SET hash = ?1 WHERE id = ?2",
        params![encode_hash(&hash), id as i64],
    )?;
    Ok(AuditEntry {
        id,
        ts_ms,
        user,
        session_id,
        source_ip,
        prev_hash: row.prev_hash,
        hash,
        kind: event,
    })
}

fn latest_hash(conn: &Connection) -> anyhow::Result<Option<[u8; 32]>> {
    let result = conn.query_row(
        "SELECT hash FROM audit_log ORDER BY id DESC LIMIT 1",
        [],
        |row| row.get::<_, String>(0),
    );
    match result {
        Ok(hash) => decode_hash(&hash).map(Some).map_err(anyhow::Error::msg),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(err) => Err(err.into()),
    }
}

#[derive(Debug, Clone)]
struct RawAuditRow {
    id: u64,
    ts_ms: u64,
    user: Option<String>,
    session_id: Option<String>,
    source_ip: Option<String>,
    kind: String,
    payload: String,
    prev_hash: Option<[u8; 32]>,
    hash: [u8; 32],
    hash_text: String,
}

#[derive(Serialize)]
struct CanonicalAuditRow<'a> {
    id: u64,
    ts_ms: u64,
    user: &'a Option<String>,
    session_id: &'a Option<String>,
    source_ip: &'a Option<String>,
    kind: &'a str,
    payload: &'a str,
    prev_hash: Option<String>,
}

fn compute_hash(
    previous_hash: Option<&[u8; 32]>,
    row: &RawAuditRow,
) -> Result<[u8; 32], serde_json::Error> {
    let canonical = CanonicalAuditRow {
        id: row.id,
        ts_ms: row.ts_ms,
        user: &row.user,
        session_id: &row.session_id,
        source_ip: &row.source_ip,
        kind: &row.kind,
        payload: &row.payload,
        prev_hash: previous_hash.map(encode_hash),
    };
    let mut hasher = Sha256::new();
    hasher.update(previous_hash.unwrap_or(&ZERO_HASH));
    hasher.update(serde_json::to_vec(&canonical)?);
    Ok(hasher.finalize().into())
}

fn read_rows_by_id(conn: &Connection) -> anyhow::Result<Vec<RawAuditRow>> {
    let mut stmt = conn.prepare(
        "SELECT id, ts_ms, user, session_id, source_ip, kind, payload, prev_hash, hash
         FROM audit_log
         ORDER BY id ASC",
    )?;
    let rows = stmt.query_map([], raw_row_from_sql)?;
    rows.collect::<Result<Vec<_>, _>>().map_err(Into::into)
}

fn raw_row_from_sql(row: &rusqlite::Row<'_>) -> rusqlite::Result<RawAuditRow> {
    let prev_hash_text: Option<String> = row.get(7)?;
    let hash_text: String = row.get(8)?;
    let prev_hash = prev_hash_text
        .as_deref()
        .map(decode_hash)
        .transpose()
        .map_err(|err| {
            rusqlite::Error::FromSqlConversionFailure(7, rusqlite::types::Type::Text, err.into())
        })?;
    let hash = decode_hash(&hash_text).map_err(|err| {
        rusqlite::Error::FromSqlConversionFailure(8, rusqlite::types::Type::Text, err.into())
    })?;
    Ok(RawAuditRow {
        id: row.get::<_, i64>(0)? as u64,
        ts_ms: row.get::<_, i64>(1)? as u64,
        user: row.get(2)?,
        session_id: row.get(3)?,
        source_ip: row.get(4)?,
        kind: row.get(5)?,
        payload: row.get(6)?,
        prev_hash,
        hash,
        hash_text,
    })
}

enum VerifyError {
    Broken(ChainBroken),
    Sql(String),
    Json(String),
}

fn verify_chain(conn: &Connection) -> Result<usize, VerifyError> {
    let rows = read_rows_by_id(conn).map_err(|err| VerifyError::Sql(err.to_string()))?;
    let mut previous_hash = None;
    for row in &rows {
        if row.prev_hash != previous_hash {
            return Err(VerifyError::Broken(ChainBroken {
                first_bad_id: row.id,
                expected_hash: previous_hash
                    .as_ref()
                    .map(encode_hash)
                    .unwrap_or_else(|| "NULL".to_string()),
                found_hash: row
                    .prev_hash
                    .as_ref()
                    .map(encode_hash)
                    .unwrap_or_else(|| "NULL".to_string()),
            }));
        }
        let expected = compute_hash(previous_hash.as_ref(), row)
            .map_err(|err| VerifyError::Json(err.to_string()))?;
        if row.hash != expected {
            return Err(VerifyError::Broken(ChainBroken {
                first_bad_id: row.id,
                expected_hash: encode_hash(&expected),
                found_hash: row.hash_text.clone(),
            }));
        }
        previous_hash = Some(row.hash);
    }
    Ok(rows.len())
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
