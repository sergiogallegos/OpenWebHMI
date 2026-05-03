use std::time::Duration;

use openwebhmi_alarm_engine::{
    AlarmCondition, AlarmDefinition, AlarmJournal, AlarmState, AlarmTransition, evaluate,
    spawn_alarm_engine,
};
use openwebhmi_protocol::{Quality, TagValue};
use openwebhmi_tag_engine::TagStore;
use tokio::time::timeout;

#[test]
fn conditions_evaluate_boundaries_and_types() {
    assert!(
        !evaluate(
            &AlarmCondition::HighLimit { threshold: 10.0 },
            &TagValue::Real(10.0),
        )
        .unwrap()
    );
    assert!(
        evaluate(
            &AlarmCondition::HighLimit { threshold: 10.0 },
            &TagValue::Real(10.1),
        )
        .unwrap()
    );
    assert!(
        !evaluate(
            &AlarmCondition::LowLimit { threshold: 10.0 },
            &TagValue::Int(10),
        )
        .unwrap()
    );
    assert!(
        evaluate(
            &AlarmCondition::LowLimit { threshold: 10.0 },
            &TagValue::Int(9),
        )
        .unwrap()
    );
    assert!(
        evaluate(
            &AlarmCondition::Equals {
                value: TagValue::String("fault".into()),
            },
            &TagValue::String("fault".into()),
        )
        .unwrap()
    );
    assert!(
        evaluate(
            &AlarmCondition::Deviation {
                setpoint: 100.0,
                tolerance: 5.0,
            },
            &TagValue::Real(106.0),
        )
        .unwrap()
    );
    assert!(
        !evaluate(
            &AlarmCondition::Deviation {
                setpoint: 100.0,
                tolerance: 5.0,
            },
            &TagValue::Real(105.0),
        )
        .unwrap()
    );
    assert!(
        evaluate(
            &AlarmCondition::Digital { active_when: true },
            &TagValue::Bool(true),
        )
        .unwrap()
    );
    assert!(
        evaluate(
            &AlarmCondition::Digital { active_when: true },
            &TagValue::Real(1.0),
        )
        .is_err()
    );
}

#[tokio::test]
async fn state_machine_active_acked_cleared_and_journaled() {
    let tag_store = TagStore::new();
    let journal = AlarmJournal::memory().unwrap();
    let engine = spawn_alarm_engine(tag_store.clone(), journal.clone(), vec![definition(true)]);
    let mut rx = engine.subscribe_events();

    tag_store.publish("rockwell-1/Pressure", TagValue::Real(250.0), Quality::Good);
    let active = next_event(&mut rx).await;
    assert_eq!(active.state, AlarmState::Active);
    let activated_at = active.activated_at_ms.unwrap();

    engine.ack("pressure-high", "operator", Some("seen".into()));
    let acked = next_event(&mut rx).await;
    assert_eq!(acked.state, AlarmState::Acked);
    assert_eq!(acked.who.as_deref(), Some("operator"));

    tag_store.publish("rockwell-1/Pressure", TagValue::Real(150.0), Quality::Good);
    let cleared = next_event(&mut rx).await;
    assert_eq!(cleared.state, AlarmState::Cleared);
    assert_eq!(cleared.activated_at_ms, Some(activated_at));

    let entries = journal.read_alarm("pressure-high").unwrap();
    assert_eq!(entries.len(), 3);
    assert_eq!(entries[0].to_state, AlarmState::Active);
    assert_eq!(entries[1].to_state, AlarmState::Acked);
    assert_eq!(entries[2].from_state, AlarmState::Acked);
    assert_eq!(entries[2].to_state, AlarmState::Cleared);
}

#[tokio::test]
async fn state_machine_auto_clears_when_ack_not_required() {
    let tag_store = TagStore::new();
    let journal = AlarmJournal::memory().unwrap();
    let engine = spawn_alarm_engine(tag_store.clone(), journal.clone(), vec![definition(false)]);
    let mut rx = engine.subscribe_events();

    tag_store.publish("rockwell-1/Pressure", TagValue::Real(250.0), Quality::Good);
    assert_eq!(next_event(&mut rx).await.state, AlarmState::Active);
    tag_store.publish("rockwell-1/Pressure", TagValue::Real(150.0), Quality::Good);
    assert_eq!(next_event(&mut rx).await.state, AlarmState::Cleared);
}

#[tokio::test]
async fn reentry_after_clear_uses_fresh_activation_timestamp() {
    let tag_store = TagStore::new();
    let journal = AlarmJournal::memory().unwrap();
    let engine = spawn_alarm_engine(tag_store.clone(), journal, vec![definition(false)]);
    let mut rx = engine.subscribe_events();

    tag_store.publish("rockwell-1/Pressure", TagValue::Real(250.0), Quality::Good);
    let first = next_event(&mut rx).await.activated_at_ms.unwrap();
    tag_store.publish("rockwell-1/Pressure", TagValue::Real(150.0), Quality::Good);
    assert_eq!(next_event(&mut rx).await.state, AlarmState::Cleared);
    tokio::time::sleep(Duration::from_millis(2)).await;
    tag_store.publish("rockwell-1/Pressure", TagValue::Real(251.0), Quality::Good);
    let second = next_event(&mut rx).await.activated_at_ms.unwrap();

    assert!(second > first);
}

#[tokio::test]
async fn hot_reload_replaces_same_path_definition() {
    let tag_store = TagStore::new();
    let journal = AlarmJournal::memory().unwrap();
    let mut engine = spawn_alarm_engine(tag_store.clone(), journal, vec![definition(false)]);
    let mut rx = engine.subscribe_events();

    tag_store.publish("rockwell-1/Pressure", TagValue::Real(150.0), Quality::Good);
    assert!(
        timeout(Duration::from_millis(100), rx.recv())
            .await
            .is_err()
    );

    let mut updated = definition(false);
    updated.condition = AlarmCondition::HighLimit { threshold: 100.0 };
    engine.update_definitions(vec![updated]);
    tag_store.publish("rockwell-1/Pressure", TagValue::Real(150.0), Quality::Good);

    let active = next_event(&mut rx).await;
    assert_eq!(active.state, AlarmState::Active);
}

#[test]
fn journal_persists_transition_round_trip() {
    let journal = AlarmJournal::memory().unwrap();
    journal
        .write_transition(&AlarmTransition {
            alarm_id: "a".into(),
            ts_ms: 10,
            from_state: AlarmState::Active,
            to_state: AlarmState::Acked,
            who: Some("operator".into()),
            note: Some("checked".into()),
        })
        .unwrap();

    let entries = journal.read_alarm("a").unwrap();
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].who.as_deref(), Some("operator"));
    assert_eq!(entries[0].note.as_deref(), Some("checked"));
}

#[test]
fn journal_restore_from_path_replaces_existing_transitions() {
    let source = AlarmJournal::memory().unwrap();
    let target = AlarmJournal::memory().unwrap();
    source.write_transition(&transition("a", 10)).unwrap();
    target
        .write_transition(&transition("target-only", 5))
        .unwrap();
    let snapshot = tempfile::NamedTempFile::new().unwrap();
    source.backup_to_path(snapshot.path()).unwrap();

    target.restore_from_path(snapshot.path()).unwrap();

    assert_eq!(target.read_alarm("a").unwrap().len(), 1);
    assert!(target.read_alarm("target-only").unwrap().is_empty());
}

#[test]
fn journal_merge_from_path_appends_transitions() {
    let source = AlarmJournal::memory().unwrap();
    let target = AlarmJournal::memory().unwrap();
    source.write_transition(&transition("a", 10)).unwrap();
    target.write_transition(&transition("b", 5)).unwrap();
    let snapshot = tempfile::NamedTempFile::new().unwrap();
    source.backup_to_path(snapshot.path()).unwrap();

    target.merge_from_path(snapshot.path()).unwrap();

    assert_eq!(target.read_alarm("a").unwrap().len(), 1);
    assert_eq!(target.read_alarm("b").unwrap().len(), 1);
}

fn definition(require_ack: bool) -> AlarmDefinition {
    AlarmDefinition {
        id: "pressure-high".into(),
        label: "High pressure".into(),
        priority: 2,
        tag_path: "rockwell-1/Pressure".into(),
        condition: AlarmCondition::HighLimit { threshold: 200.0 },
        message: "Pressure high: {value}".into(),
        enabled: true,
        require_ack,
    }
}

fn transition(alarm_id: &str, ts_ms: u64) -> AlarmTransition {
    AlarmTransition {
        alarm_id: alarm_id.into(),
        ts_ms,
        from_state: AlarmState::Clear,
        to_state: AlarmState::Active,
        who: Some("operator".into()),
        note: None,
    }
}

async fn next_event(
    rx: &mut tokio::sync::broadcast::Receiver<openwebhmi_alarm_engine::AlarmEvent>,
) -> openwebhmi_alarm_engine::AlarmEvent {
    timeout(Duration::from_millis(500), rx.recv())
        .await
        .expect("timed out waiting for alarm event")
        .expect("alarm event channel closed")
}
