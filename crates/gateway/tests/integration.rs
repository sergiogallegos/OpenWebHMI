use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use futures_util::{SinkExt, StreamExt};
use openwebhmi_alarm_engine::{AlarmCondition, AlarmDefinition, AlarmJournal, spawn_alarm_engine};
use openwebhmi_auth::{Role, SessionManager, UserStore};
use openwebhmi_gateway::{server, sim_provider};
use openwebhmi_project_store::{ScriptConfig, ScriptTriggerConfig};
use openwebhmi_protocol::{ClientMessage, Quality, ServerMessage, TagPath, TagValue};
use openwebhmi_scripting::{MemorySink, ScriptEvent, ScriptHost, ScriptHostOptions, ScriptStatus};
use openwebhmi_tag_engine::TagStore;
use tempfile::TempDir;
use tokio::net::TcpListener;
use tokio::time::{sleep, timeout};
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::Message;

type TestWs =
    tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>;

#[tokio::test]
async fn websocket_gateway_handles_subscription_ping_parse_errors_and_unsubscribe() {
    let store = TagStore::new();
    tokio::spawn(sim_provider::run(store.clone()));

    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr: SocketAddr = listener.local_addr().unwrap();
    let server = tokio::spawn(server::serve(listener, store));

    let (mut ws, _) = connect_async(format!("ws://{addr}")).await.unwrap();

    send_client(
        &mut ws,
        ClientMessage::TagSubscribe {
            paths: vec![TagPath::new("system/sim/sin")],
        },
    )
    .await;

    assert_valid_sin_update(next_message(&mut ws, Duration::from_millis(2_500)).await);
    assert_valid_sin_update(next_message(&mut ws, Duration::from_millis(2_500)).await);

    send_client(&mut ws, ClientMessage::Ping).await;
    assert_eq!(
        next_message(&mut ws, Duration::from_millis(100)).await,
        ServerMessage::Pong
    );

    ws.send(Message::Text("not json".to_string()))
        .await
        .unwrap();
    assert!(matches!(
        next_message(&mut ws, Duration::from_millis(100)).await,
        ServerMessage::Error { code, .. } if code == "protocol.parse"
    ));

    send_client(&mut ws, ClientMessage::Ping).await;
    assert_eq!(
        next_message(&mut ws, Duration::from_millis(100)).await,
        ServerMessage::Pong
    );

    ws.send(Message::Binary(vec![1, 2, 3])).await.unwrap();
    assert!(matches!(
        next_message(&mut ws, Duration::from_millis(100)).await,
        ServerMessage::Error { code, .. } if code == "protocol.binary_unsupported"
    ));

    send_client(
        &mut ws,
        ClientMessage::TagUnsubscribe {
            paths: vec![TagPath::new("system/sim/sin")],
        },
    )
    .await;

    sleep(Duration::from_millis(1_500)).await;
    assert!(
        timeout(Duration::from_millis(50), ws.next()).await.is_err(),
        "no further tag.update should arrive after unsubscribe"
    );

    server.abort();
}

#[tokio::test]
async fn websocket_gateway_requires_auth_and_accepts_valid_session() {
    let store = TagStore::new();
    tokio::spawn(sim_provider::run(store.clone()));
    let users = UserStore::memory().unwrap();
    users
        .create_user("admin", "admin-pass", vec![Role::Administrator])
        .unwrap();
    let sessions = SessionManager::new(b"test-secret".to_vec(), Duration::from_secs(60));
    let token = sessions
        .issue("test-admin", "admin", &[Role::Administrator])
        .unwrap();
    let auth = server::AuthContext::new(users, sessions);

    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr: SocketAddr = listener.local_addr().unwrap();
    let server = tokio::spawn(server::serve_with_auth(listener, store, auth));

    let (mut anonymous, _) = connect_async(format!("ws://{addr}")).await.unwrap();
    send_client(
        &mut anonymous,
        ClientMessage::TagSubscribe {
            paths: vec![TagPath::new("system/sim/sin")],
        },
    )
    .await;
    assert!(matches!(
        next_message(&mut anonymous, Duration::from_millis(100)).await,
        ServerMessage::Error { code, .. } if code == "auth.required"
    ));

    send_client(
        &mut anonymous,
        ClientMessage::AuthLogin {
            username: "admin".into(),
            password: "admin-pass".into(),
        },
    )
    .await;
    match next_message(&mut anonymous, Duration::from_millis(2_500)).await {
        ServerMessage::AuthResult {
            session_token: Some(token),
            roles,
            ..
        } => {
            assert_eq!(roles, vec!["Administrator"]);
            assert!(!token.is_empty());
        }
        other => panic!("expected auth.result, got {other:?}"),
    };

    let (mut authorized, _) = connect_async(format!("ws://{addr}/?token={token}"))
        .await
        .unwrap();
    send_client(&mut authorized, ClientMessage::Ping).await;
    assert_eq!(
        next_message(&mut authorized, Duration::from_millis(1_000)).await,
        ServerMessage::Pong
    );
    send_client(
        &mut authorized,
        ClientMessage::TagSubscribe {
            paths: vec![TagPath::new("system/sim/sin")],
        },
    )
    .await;
    assert_valid_sin_update(next_message(&mut authorized, Duration::from_millis(2_500)).await);

    server.abort();
}

#[tokio::test]
async fn websocket_gateway_forwards_alarm_events_with_priority_filter() {
    let store = TagStore::new();
    let journal = AlarmJournal::memory().unwrap();
    let engine = spawn_alarm_engine(
        store.clone(),
        journal,
        vec![AlarmDefinition {
            id: "pressure-high".into(),
            label: "High pressure".into(),
            priority: 2,
            tag_path: "rockwell-1/Pressure".into(),
            condition: AlarmCondition::HighLimit { threshold: 200.0 },
            message: "Pressure high: {value}".into(),
            enabled: true,
            require_ack: true,
        }],
    );
    server::set_default_alarm_engine(Arc::new(Mutex::new(engine)));

    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr: SocketAddr = listener.local_addr().unwrap();
    let server = tokio::spawn(server::serve(listener, store.clone()));

    let (mut ws, _) = connect_async(format!("ws://{addr}")).await.unwrap();
    send_client(
        &mut ws,
        ClientMessage::AlarmSubscribe {
            project_id: "phase1-demo".into(),
            priority_min: Some(1),
            priority_max: Some(2),
        },
    )
    .await;
    sleep(Duration::from_millis(10)).await;
    store.publish("rockwell-1/Pressure", TagValue::Real(250.0), Quality::Good);

    match next_message(&mut ws, Duration::from_millis(500)).await {
        ServerMessage::AlarmEvent {
            alarm_id,
            priority,
            state,
            ..
        } => {
            assert_eq!(alarm_id, "pressure-high");
            assert_eq!(priority, 2);
            assert_eq!(state, openwebhmi_protocol::AlarmState::Active);
        }
        other => panic!("expected alarm.event, got {other:?}"),
    }

    server.abort();
}

#[tokio::test]
async fn websocket_gateway_forwards_script_events_by_project() {
    let dir = TempDir::new().unwrap();
    let script_path = dir.path().join("logger.py");
    std::fs::write(
        &script_path,
        r#"
import system

@system.on_tag_change("rockwell-1/Pressure")
def pressure_changed(tag):
    system.util.log("pressure changed")
"#,
    )
    .unwrap();
    let store = TagStore::new();
    let host = ScriptHost::spawn(
        "phase1-demo",
        store.clone(),
        Arc::new(MemorySink::new(store.clone())),
        vec![ScriptConfig {
            id: "logger".into(),
            path: script_path.to_string_lossy().into_owned(),
            enabled: true,
            triggers: vec![ScriptTriggerConfig::OnTagChange {
                path: "rockwell-1/Pressure".into(),
            }],
            handler_timeout_ms: None,
        }],
        ScriptHostOptions::default(),
    );
    let mut host_events = host.subscribe_events();
    server::set_default_script_host(host);
    timeout(Duration::from_secs(5), async {
        loop {
            if let Ok(ScriptEvent::Status {
                status: ScriptStatus::Ready,
                ..
            }) = host_events.recv().await
            {
                return;
            }
        }
    })
    .await
    .expect("script worker should become ready");

    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr: SocketAddr = listener.local_addr().unwrap();
    let server = tokio::spawn(server::serve(listener, store.clone()));

    let (mut ws, _) = connect_async(format!("ws://{addr}")).await.unwrap();
    send_client(
        &mut ws,
        ClientMessage::ScriptSubscribe {
            project_id: "other-project".into(),
        },
    )
    .await;
    send_client(
        &mut ws,
        ClientMessage::ScriptSubscribe {
            project_id: "phase1-demo".into(),
        },
    )
    .await;
    store.publish("rockwell-1/Pressure", TagValue::Real(250.0), Quality::Good);

    let event = timeout(Duration::from_millis(1_000), async {
        loop {
            if let ServerMessage::ScriptEvent {
                project_id,
                script_id,
                event_kind,
                message,
                ..
            } = next_message(&mut ws, Duration::from_millis(1_000)).await
            {
                if event_kind == "log" {
                    return (project_id, script_id, message);
                }
            }
        }
    })
    .await
    .expect("script log event should be forwarded");
    assert_eq!(event.0, "phase1-demo");
    assert_eq!(event.1, "logger");
    assert_eq!(event.2.as_deref(), Some("pressure changed"));

    server.abort();
}

async fn send_client(ws: &mut TestWs, message: ClientMessage) {
    let json = serde_json::to_string(&message).unwrap();
    ws.send(Message::Text(json)).await.unwrap();
}

async fn next_message(ws: &mut TestWs, wait: Duration) -> ServerMessage {
    let frame = timeout(wait, ws.next())
        .await
        .expect("timed out waiting for message")
        .expect("socket closed")
        .expect("websocket error");
    let text = frame.into_text().expect("expected text frame");
    serde_json::from_str::<ServerMessage>(&text).expect("valid server message")
}

fn assert_valid_sin_update(message: ServerMessage) {
    match message {
        ServerMessage::TagUpdate {
            path,
            value,
            quality,
            ..
        } => {
            assert_eq!(path, TagPath::new("system/sim/sin"));
            assert_eq!(quality, Quality::Good);
            match value {
                TagValue::Real(value) => assert!((-1.0..=1.0).contains(&value)),
                other => panic!("expected real tag value, got {other:?}"),
            }
        }
        other => panic!("expected tag.update, got {other:?}"),
    }
}
