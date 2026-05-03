use openwebhmi_audit_log::{AuditEvent, AuditLog, AuditQuery, WriteSource};
use openwebhmi_protocol::TagValue;

#[test]
fn audit_events_round_trip_and_query_filters() {
    let log = AuditLog::memory().unwrap();
    for idx in 0..50_u64 {
        let user = format!("user-{}", idx % 5);
        let event = match idx % 4 {
            0 => AuditEvent::AuthLogin {
                username: user.clone(),
                success: true,
                reason: None,
            },
            1 => AuditEvent::TagWrite {
                path: format!("driver/tag-{idx}"),
                value: TagValue::Int(idx as i64),
                success: true,
                source: WriteSource::WebSocket,
                error: None,
            },
            2 => AuditEvent::ProjectSave {
                project_id: "demo".into(),
                artifact_kind: "view".into(),
            },
            _ => AuditEvent::AuthLogout {
                username: user.clone(),
            },
        };
        log.append_at(1_000 + idx, Some(user), None, None, event)
            .unwrap();
    }

    let (entries, total) = log
        .query(&AuditQuery {
            user: Some("user-1".into()),
            limit: 100,
            ..AuditQuery::default()
        })
        .unwrap();
    assert_eq!(total, 10);
    assert!(
        entries
            .iter()
            .all(|entry| entry.user.as_deref() == Some("user-1"))
    );

    let (entries, total) = log
        .query(&AuditQuery {
            kinds: vec!["TagWrite".into()],
            limit: 5,
            offset: 2,
            ..AuditQuery::default()
        })
        .unwrap();
    assert_eq!(total, 13);
    assert_eq!(entries.len(), 5);
    assert!(
        entries
            .iter()
            .all(|entry| entry.kind.kind_name() == "TagWrite")
    );

    let (entries, total) = log
        .query(&AuditQuery {
            from_ts_ms: Some(1_010),
            to_ts_ms: Some(1_019),
            limit: 100,
            ..AuditQuery::default()
        })
        .unwrap();
    assert_eq!(total, 10);
    assert_eq!(entries.len(), 10);
}
