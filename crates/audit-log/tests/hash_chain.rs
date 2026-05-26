use openwebhmi_audit_log::{AuditEvent, AuditLog, AuditQuery, WriteSource};
use openwebhmi_protocol::{TagPath, TagValue};
use rusqlite::{Connection, params};

#[test]
fn verifies_hundred_entry_hash_chain() {
    let log = AuditLog::memory().unwrap();
    append_events(&log, 100);

    assert_eq!(log.verify_chain().unwrap(), 100);
}

#[test]
fn detects_payload_tampering() {
    let tempdir = tempfile::tempdir().unwrap();
    let path = tempdir.path().join("audit.sqlite");
    let log = AuditLog::open(&path, 0).unwrap();
    append_events(&log, 100);

    let conn = Connection::open(&path).unwrap();
    conn.execute(
        "UPDATE audit_log SET payload = ?1 WHERE id = 50",
        [r#"{"type":"AuthLogout","username":"mallory"}"#],
    )
    .unwrap();

    let broken = log.verify_chain().unwrap_err();
    assert_eq!(broken.first_bad_id, 50);
}

#[test]
fn detects_deleted_row() {
    let tempdir = tempfile::tempdir().unwrap();
    let path = tempdir.path().join("audit.sqlite");
    let log = AuditLog::open(&path, 0).unwrap();
    append_events(&log, 100);

    let conn = Connection::open(&path).unwrap();
    conn.execute("DELETE FROM audit_log WHERE id = 30", [])
        .unwrap();

    let broken = log.verify_chain().unwrap_err();
    assert_eq!(broken.first_bad_id, 31);
}

#[test]
fn detects_reordered_ids() {
    let tempdir = tempfile::tempdir().unwrap();
    let path = tempdir.path().join("audit.sqlite");
    let log = AuditLog::open(&path, 0).unwrap();
    append_events(&log, 100);

    let mut conn = Connection::open(&path).unwrap();
    let tx = conn.transaction().unwrap();
    tx.execute("UPDATE audit_log SET id = -5 WHERE id = 5", [])
        .unwrap();
    tx.execute("UPDATE audit_log SET id = 5 WHERE id = 6", [])
        .unwrap();
    tx.execute("UPDATE audit_log SET id = 6 WHERE id = -5", [])
        .unwrap();
    tx.commit().unwrap();

    let broken = log.verify_chain().unwrap_err();
    assert_eq!(broken.first_bad_id, 5);
}

#[test]
fn migrates_pre_hash_schema_and_records_migration_event() {
    let tempdir = tempfile::tempdir().unwrap();
    let path = tempdir.path().join("audit.sqlite");
    create_pre_hash_schema(&path);

    let log = AuditLog::open(&path, 0).unwrap();
    assert_eq!(log.verify_chain().unwrap(), 51);

    let mut query = AuditQuery::default();
    query.kinds = vec!["MigrationCompleted".into()];
    query.limit = 10;
    let (entries, total) = log.query(&query).unwrap();
    assert_eq!(total, 1);
    match &entries[0].kind {
        AuditEvent::MigrationCompleted { rows } => assert_eq!(*rows, 50),
        other => panic!("expected migration event, got {other:?}"),
    }

    query.kinds.clear();
    assert_eq!(log.query(&query).unwrap().1, 51);
}

fn append_events(log: &AuditLog, count: u64) {
    for idx in 0..count {
        let event = if idx % 2 == 0 {
            AuditEvent::AuthLogin {
                username: format!("user-{idx}"),
                success: true,
                reason: None,
            }
        } else {
            AuditEvent::TagWrite {
                path: TagPath::new(format!("driver/tag-{idx}")),
                value: TagValue::Int(idx as i64),
                success: true,
                source: WriteSource::WebSocket,
                error: None,
            }
        };
        log.append_at(1_000 + idx, Some(format!("user-{idx}")), None, None, event)
            .unwrap();
    }
}

fn create_pre_hash_schema(path: &std::path::Path) {
    let conn = Connection::open(path).unwrap();
    conn.execute_batch(
        "CREATE TABLE meta (
            key TEXT PRIMARY KEY,
            value TEXT NOT NULL
        );
        INSERT INTO meta (key, value) VALUES ('schema_version', '1');
        CREATE TABLE audit_log (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            ts_ms INTEGER NOT NULL,
            user TEXT,
            session_id TEXT,
            source_ip TEXT,
            kind TEXT NOT NULL,
            payload TEXT NOT NULL
        );",
    )
    .unwrap();
    for idx in 0..50_u64 {
        let payload = serde_json::to_string(&AuditEvent::AuthLogin {
            username: format!("user-{idx}"),
            success: true,
            reason: None,
        })
        .unwrap();
        conn.execute(
            "INSERT INTO audit_log (ts_ms, user, session_id, source_ip, kind, payload)
             VALUES (?1, ?2, NULL, NULL, ?3, ?4)",
            params![
                1_000_i64 + idx as i64,
                format!("user-{idx}"),
                "AuthLogin",
                payload
            ],
        )
        .unwrap();
    }
}
