//! SQLite alarm transition journal.

use std::path::Path;
use std::sync::{Arc, Mutex};

use rusqlite::{params, Connection};

use crate::types::{AlarmState, AlarmTransition};

/// SQLite-backed alarm journal.
#[derive(Clone)]
pub struct AlarmJournal {
    conn: Arc<Mutex<Connection>>,
}

impl AlarmJournal {
    /// Open a journal database and create schema if needed.
    pub fn open(path: impl AsRef<Path>) -> anyhow::Result<Self> {
        let conn = Connection::open(path)?;
        init_schema(&conn)?;
        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
        })
    }

    /// Open an in-memory journal.
    pub fn memory() -> anyhow::Result<Self> {
        let conn = Connection::open_in_memory()?;
        init_schema(&conn)?;
        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
        })
    }

    /// Write one transition.
    pub fn write_transition(&self, transition: &AlarmTransition) -> anyhow::Result<()> {
        self.lock()?.execute(
            "INSERT INTO alarm_journal (alarm_id, ts_ms, from_state, to_state, who, note)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                transition.alarm_id,
                millis_to_i64(transition.ts_ms),
                transition.from_state.as_str(),
                transition.to_state.as_str(),
                transition.who,
                transition.note,
            ],
        )?;
        Ok(())
    }

    /// Read transitions for one alarm.
    pub fn read_alarm(&self, alarm_id: &str) -> anyhow::Result<Vec<JournalEntry>> {
        let conn = self.lock()?;
        let mut stmt = conn.prepare(
            "SELECT alarm_id, ts_ms, from_state, to_state, who, note
             FROM alarm_journal
             WHERE alarm_id = ?1
             ORDER BY ts_ms ASC, rowid ASC",
        )?;
        let rows = stmt.query_map(params![alarm_id], |row| {
            Ok(JournalEntry {
                alarm_id: row.get(0)?,
                ts_ms: row.get::<_, i64>(1)? as u64,
                from_state: parse_state(row.get::<_, String>(2)?.as_str())?,
                to_state: parse_state(row.get::<_, String>(3)?.as_str())?,
                who: row.get(4)?,
                note: row.get(5)?,
            })
        })?;
        rows.collect::<Result<Vec<_>, _>>().map_err(Into::into)
    }

    fn lock(&self) -> anyhow::Result<std::sync::MutexGuard<'_, Connection>> {
        self.conn
            .lock()
            .map_err(|_| anyhow::anyhow!("alarm journal sqlite lock poisoned"))
    }
}

/// Persisted alarm transition.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JournalEntry {
    /// Alarm id.
    pub alarm_id: String,
    /// Transition timestamp.
    pub ts_ms: u64,
    /// Previous state.
    pub from_state: AlarmState,
    /// New state.
    pub to_state: AlarmState,
    /// Actor.
    pub who: Option<String>,
    /// Note.
    pub note: Option<String>,
}

fn init_schema(conn: &Connection) -> anyhow::Result<()> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS alarm_journal (
            alarm_id TEXT NOT NULL,
            ts_ms INTEGER NOT NULL,
            from_state TEXT NOT NULL,
            to_state TEXT NOT NULL,
            who TEXT,
            note TEXT
        );
        CREATE INDEX IF NOT EXISTS idx_alarm_journal_alarm_ts
            ON alarm_journal(alarm_id, ts_ms);",
    )?;
    Ok(())
}

fn millis_to_i64(value: u64) -> i64 {
    value.min(i64::MAX as u64) as i64
}

fn parse_state(value: &str) -> Result<AlarmState, rusqlite::Error> {
    match value {
        "clear" => Ok(AlarmState::Clear),
        "active" => Ok(AlarmState::Active),
        "acked" => Ok(AlarmState::Acked),
        "cleared" => Ok(AlarmState::Cleared),
        _ => Err(rusqlite::Error::InvalidColumnType(
            0,
            "state".to_string(),
            rusqlite::types::Type::Text,
        )),
    }
}
