use std::time::Duration;

use openwebhmi_project_store::{ScriptConfig, ScriptTriggerConfig};
use openwebhmi_protocol::{Quality, TagValue};
use openwebhmi_scripting::{ScriptEvent, ScriptHost, ScriptHostOptions, ScriptStatus};
use openwebhmi_tag_engine::TagStore;
use tempfile::TempDir;
use tokio::time::{sleep, timeout, Instant};

fn write_script(dir: &TempDir, name: &str, source: &str) -> std::path::PathBuf {
    let path = dir.path().join(name);
    std::fs::write(&path, source).expect("write script");
    path
}

fn script_config(path: std::path::PathBuf, timeout_ms: Option<u64>) -> ScriptConfig {
    ScriptConfig {
        id: "derived-setpoint".to_string(),
        path: path.to_string_lossy().into_owned(),
        enabled: true,
        triggers: vec![ScriptTriggerConfig::OnTagChange {
            path: "rockwell-1/Pressure".to_string(),
        }],
        handler_timeout_ms: timeout_ms,
    }
}

async fn wait_ready(events: &mut tokio::sync::broadcast::Receiver<ScriptEvent>) {
    timeout(Duration::from_secs(5), async {
        loop {
            if let Ok(ScriptEvent::Status {
                status: ScriptStatus::Ready,
                ..
            }) = events.recv().await
            {
                return;
            }
        }
    })
    .await
    .expect("worker should become ready");
}

async fn wait_tag(store: &TagStore, path: &str, expected: TagValue) {
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        if let Some(snapshot) = store.get(path) {
            if snapshot.value == expected {
                return;
            }
        }
        assert!(Instant::now() < deadline, "timed out waiting for {path}");
        sleep(Duration::from_millis(25)).await;
    }
}

#[tokio::test]
async fn tag_change_script_writes_derived_tag() {
    let dir = TempDir::new().unwrap();
    let script = write_script(
        &dir,
        "derived.py",
        r#"
import system

@system.on_tag_change("rockwell-1/Pressure")
def pressure_changed(tag):
    value = tag["value"]["value"]
    if value > 100:
        system.tag.write("rockwell-1/Setpoint", value * 0.5)
"#,
    );
    let store = TagStore::new();
    let host = ScriptHost::spawn(
        store.clone(),
        vec![script_config(script, None)],
        ScriptHostOptions::default(),
    );
    let mut events = host.subscribe_events();
    wait_ready(&mut events).await;

    store.publish("rockwell-1/Pressure", TagValue::Real(120.0), Quality::Good);
    wait_tag(&store, "rockwell-1/Setpoint", TagValue::Real(60.0)).await;

    host.shutdown().await;
}

#[tokio::test]
async fn worker_crash_mid_handler_respawns_and_next_trigger_runs() {
    let dir = TempDir::new().unwrap();
    let flag = dir.path().join("crashed.once");
    let script = write_script(
        &dir,
        "crashy.py",
        &format!(
            r#"
import os
import system

FLAG = {flag:?}

@system.on_tag_change("rockwell-1/Pressure")
def pressure_changed(tag):
    if not os.path.exists(FLAG):
        open(FLAG, "w", encoding="utf-8").close()
        os._exit(7)
    system.tag.write("rockwell-1/Setpoint", tag["value"]["value"] * 0.5)
"#,
            flag = flag.to_string_lossy()
        ),
    );
    let store = TagStore::new();
    let host = ScriptHost::spawn(
        store.clone(),
        vec![script_config(script, None)],
        ScriptHostOptions::default(),
    );
    let mut events = host.subscribe_events();
    wait_ready(&mut events).await;

    store.publish("rockwell-1/Pressure", TagValue::Real(120.0), Quality::Good);
    wait_ready(&mut events).await;
    store.publish("rockwell-1/Pressure", TagValue::Real(122.0), Quality::Good);

    wait_tag(&store, "rockwell-1/Setpoint", TagValue::Real(61.0)).await;
    host.shutdown().await;
}

#[tokio::test]
async fn handler_timeout_restarts_worker_and_next_trigger_runs() {
    let dir = TempDir::new().unwrap();
    let script = write_script(
        &dir,
        "slow.py",
        r#"
import time
import system

@system.on_tag_change("rockwell-1/Pressure")
def pressure_changed(tag):
    value = tag["value"]["value"]
    if value == 1:
        time.sleep(10)
    else:
        system.tag.write("rockwell-1/Setpoint", value)
"#,
    );
    let store = TagStore::new();
    let host = ScriptHost::spawn(
        store.clone(),
        vec![script_config(script, Some(100))],
        ScriptHostOptions {
            handler_timeout: Duration::from_millis(100),
        },
    );
    let mut events = host.subscribe_events();
    wait_ready(&mut events).await;

    store.publish("rockwell-1/Pressure", TagValue::Int(1), Quality::Good);
    timeout(Duration::from_secs(5), async {
        loop {
            if let Ok(ScriptEvent::Error { message, .. }) = events.recv().await {
                if message.contains("timed out") {
                    return;
                }
            }
        }
    })
    .await
    .expect("timeout error should be emitted");
    wait_ready(&mut events).await;

    store.publish("rockwell-1/Pressure", TagValue::Int(2), Quality::Good);
    wait_tag(&store, "rockwell-1/Setpoint", TagValue::Int(2)).await;
    host.shutdown().await;
}

#[tokio::test]
async fn util_log_emits_script_event() {
    let dir = TempDir::new().unwrap();
    let script = write_script(
        &dir,
        "log.py",
        r#"
import system

@system.on_tag_change("rockwell-1/Pressure")
def pressure_changed(tag):
    system.util.log("pressure changed")
"#,
    );
    let store = TagStore::new();
    let host = ScriptHost::spawn(
        store.clone(),
        vec![script_config(script, None)],
        ScriptHostOptions::default(),
    );
    let mut events = host.subscribe_events();
    wait_ready(&mut events).await;

    store.publish("rockwell-1/Pressure", TagValue::Real(5.0), Quality::Good);
    timeout(Duration::from_secs(5), async {
        loop {
            if let Ok(ScriptEvent::Log { script_id, message }) = events.recv().await {
                assert_eq!(script_id, "derived-setpoint");
                assert_eq!(message, "pressure changed");
                return;
            }
        }
    })
    .await
    .expect("system.util.log should emit an event");
    host.shutdown().await;
}

#[tokio::test]
async fn python_rpc_bridge_correlates_concurrent_calls() {
    let dir = TempDir::new().unwrap();
    let script = write_script(
        &dir,
        "concurrent.py",
        r#"
import threading
import system

@system.on_tag_change("rockwell-1/Pressure")
def pressure_changed(tag):
    results = []

    def read_time():
        results.append(system.util.now())

    threads = [threading.Thread(target=read_time) for _ in range(4)]
    for thread in threads:
        thread.start()
    for thread in threads:
        thread.join()

    system.tag.write("rockwell-1/Setpoint", len(results))
"#,
    );
    let store = TagStore::new();
    let host = ScriptHost::spawn(
        store.clone(),
        vec![script_config(script, None)],
        ScriptHostOptions::default(),
    );
    let mut events = host.subscribe_events();
    wait_ready(&mut events).await;

    store.publish("rockwell-1/Pressure", TagValue::Real(5.0), Quality::Good);
    wait_tag(&store, "rockwell-1/Setpoint", TagValue::Int(4)).await;
    host.shutdown().await;
}
