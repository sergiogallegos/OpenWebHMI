use std::process::Command;

use openwebhmi_audit_log::{AuditEvent, AuditLog};
use rusqlite::Connection;

#[test]
fn audit_verify_cli_reports_good_and_tampered_databases() {
    let tempdir = tempfile::tempdir().unwrap();
    let good_path = tempdir.path().join("good.sqlite");
    let tampered_path = tempdir.path().join("tampered.sqlite");

    write_log(&good_path);
    write_log(&tampered_path);
    Connection::open(&tampered_path)
        .unwrap()
        .execute(
            "UPDATE audit_log SET payload = ?1 WHERE id = 2",
            [r#"{"type":"AuthLogout","username":"mallory"}"#],
        )
        .unwrap();

    let good = Command::new(env!("CARGO_BIN_EXE_audit-verify"))
        .arg(&good_path)
        .output()
        .unwrap();
    assert!(good.status.success());
    assert!(String::from_utf8_lossy(&good.stdout).contains("OK: 3 entries verified"));

    let tampered = Command::new(env!("CARGO_BIN_EXE_audit-verify"))
        .arg(&tampered_path)
        .output()
        .unwrap();
    assert_eq!(tampered.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&tampered.stdout).contains("BROKEN at id 2"));
}

fn write_log(path: &std::path::Path) {
    let log = AuditLog::open(path, 0).unwrap();
    for idx in 0..3 {
        log.append_at(
            1_000 + idx,
            Some(format!("user-{idx}")),
            None,
            None,
            AuditEvent::AuthLogin {
                username: format!("user-{idx}"),
                success: true,
                reason: None,
            },
        )
        .unwrap();
    }
}
