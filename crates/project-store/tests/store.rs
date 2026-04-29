use std::sync::Arc;

use openwebhmi_project_store::{
    ArtifactKind, Binding, BindingSource, ChangeAction, Component, ProjectStore, View,
};
use serde_json::json;
use tempfile::tempdir;
use tokio::task;
use tokio::time::{timeout, Duration};

#[test]
fn save_list_load_and_read_artifacts() {
    let dir = tempdir().unwrap();
    let store = ProjectStore::open(dir.path()).unwrap();

    store
        .save_artifact(
            "demo",
            ArtifactKind::ProjectMeta,
            json!({
                "schema_version": 1,
                "name": "Demo",
                "drivers": [{
                    "id": "rockwell-1",
                    "type": "rockwell",
                    "config": { "host": "127.0.0.1", "slot": 0 }
                }]
            }),
        )
        .unwrap();
    store
        .save_artifact(
            "demo",
            ArtifactKind::Tags,
            json!([
                {
                    "path": "rockwell-1/Pressure",
                    "driver": "rockwell-1",
                    "address": "Pressure",
                    "history": { "rate_ms": 500, "deadband": 0.1 }
                }
            ]),
        )
        .unwrap();
    store
        .save_artifact(
            "demo",
            ArtifactKind::View { id: "home".into() },
            serde_json::to_value(sample_view()).unwrap(),
        )
        .unwrap();
    store
        .save_artifact(
            "demo",
            ArtifactKind::Alarms,
            json!([
                {
                    "id": "pressure-high",
                    "label": "High pressure",
                    "priority": 2,
                    "tag_path": "rockwell-1/Pressure",
                    "condition": { "kind": "high_limit", "threshold": 200.0 },
                    "message": "Pressure high: {value}",
                    "enabled": true,
                    "require_ack": true
                }
            ]),
        )
        .unwrap();

    assert_eq!(store.list().unwrap()[0].id, "demo");
    let project = store.load("demo").unwrap();
    assert_eq!(project.version, 4);
    assert_eq!(project.tags.len(), 1);
    assert_eq!(project.tags[0].history.as_ref().unwrap().rate_ms, Some(500));
    assert_eq!(project.alarms[0].id, "pressure-high");
    assert_eq!(project.views[0].id, "home");
    assert!(store
        .read_artifact("demo", ArtifactKind::View { id: "home".into() })
        .unwrap()
        .is_some());
}

#[test]
fn script_source_artifact_round_trips_as_raw_python() {
    let dir = tempdir().unwrap();
    let store = ProjectStore::open(dir.path()).unwrap();
    store
        .save_artifact("demo", ArtifactKind::ProjectMeta, meta_body())
        .unwrap();
    store
        .save_artifact(
            "demo",
            ArtifactKind::Tags,
            json!([{ "path": "rockwell-1/Pressure", "driver": "rockwell-1", "address": "Pressure" }]),
        )
        .unwrap();
    store
        .save_artifact(
            "demo",
            ArtifactKind::Script {
                id: "derived-setpoint".into(),
            },
            json!({
                "id": "derived-setpoint",
                "path": "derived_setpoint.py",
                "enabled": true,
                "triggers": [{ "kind": "on_tag_change", "path": "rockwell-1/Pressure" }]
            }),
        )
        .unwrap();

    let source = "import system\n\nsystem.util.log('saved')\n";
    store
        .save_artifact(
            "demo",
            ArtifactKind::ScriptSource {
                id: "derived-setpoint".into(),
            },
            json!({ "source": source }),
        )
        .unwrap();

    let path = dir.path().join("demo/scripts/derived_setpoint.py");
    assert_eq!(std::fs::read_to_string(path).unwrap(), source);
    assert_eq!(
        store
            .read_artifact(
                "demo",
                ArtifactKind::ScriptSource {
                    id: "derived-setpoint".into(),
                },
            )
            .unwrap()
            .unwrap(),
        json!({ "source": source })
    );
}

#[test]
fn versions_increment_on_each_save() {
    let dir = tempdir().unwrap();
    let store = ProjectStore::open(dir.path()).unwrap();

    let v1 = store
        .save_artifact("p", ArtifactKind::ProjectMeta, meta_body())
        .unwrap();
    let v2 = store
        .save_artifact("p", ArtifactKind::Tags, json!([]))
        .unwrap();

    assert_eq!((v1, v2), (1, 2));
}

#[tokio::test]
async fn broadcasts_project_changes() {
    let dir = tempdir().unwrap();
    let store = ProjectStore::open(dir.path()).unwrap();
    let mut rx = store.subscribe_changes(Some("p"));

    let version = store
        .save_artifact("p", ArtifactKind::ProjectMeta, meta_body())
        .unwrap();
    let change = timeout(Duration::from_millis(100), rx.recv())
        .await
        .unwrap()
        .unwrap();

    assert_eq!(change.project_id, "p");
    assert_eq!(change.version, version);
    assert_eq!(change.action, ChangeAction::Created);
}

#[tokio::test]
async fn concurrent_saves_do_not_lose_version_updates() {
    let dir = tempdir().unwrap();
    let store = Arc::new(ProjectStore::open(dir.path()).unwrap());

    let a = {
        let store = Arc::clone(&store);
        task::spawn_blocking(move || {
            store.save_artifact("p", ArtifactKind::ProjectMeta, meta_body())
        })
    };
    let b = {
        let store = Arc::clone(&store);
        task::spawn_blocking(move || store.save_artifact("p", ArtifactKind::Tags, json!([])))
    };

    a.await.unwrap().unwrap();
    b.await.unwrap().unwrap();

    assert_eq!(store.list().unwrap()[0].version, 2);
}

#[test]
fn stray_tmp_file_does_not_replace_committed_artifact() {
    let dir = tempdir().unwrap();
    let store = ProjectStore::open(dir.path()).unwrap();
    store
        .save_artifact("p", ArtifactKind::ProjectMeta, meta_body())
        .unwrap();
    let project_path = dir.path().join("p/project.toml");
    std::fs::write(project_path.with_extension("toml.tmp"), "not valid").unwrap();

    assert_eq!(store.load("p").unwrap().name, "Demo");
}

#[test]
fn validation_rejects_schema_and_tag_prefix_errors() {
    let dir = tempdir().unwrap();
    let store = ProjectStore::open(dir.path()).unwrap();
    store
        .save_artifact(
            "p",
            ArtifactKind::ProjectMeta,
            json!({
                "schema_version": 2,
                "name": "Bad",
                "drivers": []
            }),
        )
        .unwrap();
    assert!(store.load("p").is_err());

    store
        .save_artifact("p", ArtifactKind::ProjectMeta, meta_body())
        .unwrap();
    store
        .save_artifact(
            "p",
            ArtifactKind::Tags,
            json!([{ "path": "wrong/Tag", "driver": "rockwell-1", "address": "Tag" }]),
        )
        .unwrap();
    assert!(store.load("p").is_err());
}

fn meta_body() -> serde_json::Value {
    json!({
        "schema_version": 1,
        "name": "Demo",
        "drivers": [{
            "id": "rockwell-1",
            "type": "rockwell",
            "config": { "host": "127.0.0.1", "slot": 0 }
        }]
    })
}

fn sample_view() -> View {
    View {
        id: "home".to_string(),
        title: "Home".to_string(),
        allowed_roles: None,
        schema_version: 1,
        root: Component {
            id: "root".to_string(),
            kind: "Container".to_string(),
            props: json!({ "direction": "column" }),
            bindings: Vec::new(),
            children: vec![Component {
                id: "value".to_string(),
                kind: "ValueDisplay".to_string(),
                props: json!({ "format": "number" }),
                bindings: vec![Binding {
                    prop: "value".to_string(),
                    source: BindingSource::Tag {
                        path: "rockwell-1/Pressure".to_string(),
                    },
                }],
                children: Vec::new(),
            }],
        },
    }
}
