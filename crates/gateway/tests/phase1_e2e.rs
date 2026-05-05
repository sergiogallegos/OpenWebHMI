#![cfg(feature = "sim-tests")]

use std::net::{SocketAddr, TcpListener};
use std::path::PathBuf;
use std::process::Stdio;
use std::time::Duration;

use futures_util::{SinkExt, StreamExt};
use openwebhmi_gateway::{project, server, sim_provider};
use openwebhmi_protocol::{ClientMessage, Quality, ServerMessage, TagPath, TagValue};
use openwebhmi_tag_engine::TagStore;
use tokio::process::{Child, Command};
use tokio::time::{sleep, timeout};
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::Message;

#[tokio::test]
async fn phase1_project_publishes_rockwell_tags_and_recovers() {
    if std::env::var("OPENWEBHMI_SIM_RUNNING").ok().as_deref() != Some("1") {
        eprintln!("skipping phase1 e2e; set OPENWEBHMI_SIM_RUNNING=1");
        return;
    }

    let sim_port = unused_port();
    let project_path = write_project_file(sim_port);

    let store = TagStore::new();
    tokio::spawn(sim_provider::run(store.clone()));
    let project = project::load(&project_path).unwrap();
    let driver_handles = project::spawn_project(project, store.clone()).unwrap();

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let gateway_addr: SocketAddr = listener.local_addr().unwrap();
    let gateway = tokio::spawn(server::serve_with_driver_handles(
        listener,
        store,
        driver_handles,
    ));

    let (mut ws, _) = connect_async(format!("ws://{gateway_addr}")).await.unwrap();
    subscribe(
        &mut ws,
        &[
            "system/drivers/rockwell-1/status",
            "rockwell-1/Pressure",
            "rockwell-1/Setpoint",
        ],
    )
    .await;

    assert_status(&mut ws, "connecting", Duration::from_secs(3)).await;

    let mut sim = spawn_sim(sim_port).await;

    assert_status(&mut ws, "connected", Duration::from_secs(8)).await;
    assert_good_pressure_updates(&mut ws, 2, Duration::from_secs(3)).await;
    write_tag(&mut ws, "rockwell-1/Setpoint", TagValue::Real(42.5)).await;
    assert_setpoint_value(&mut ws, 42.5, Duration::from_secs(3)).await;

    let _ = sim.kill().await;
    assert_status_and_pressure(
        &mut ws,
        "disconnected",
        Quality::Bad,
        Duration::from_secs(5),
    )
    .await;

    sim = spawn_sim(sim_port).await;
    assert_status_and_pressure(&mut ws, "connected", Quality::Good, Duration::from_secs(8)).await;

    let _ = sim.kill().await;
    gateway.abort();
    if let Some(dir) = project_path.parent() {
        let _ = std::fs::remove_dir_all(dir);
    }
}

async fn write_tag(
    ws: &mut tokio_tungstenite::WebSocketStream<
        tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>,
    >,
    path: &str,
    value: TagValue,
) {
    let message = ClientMessage::TagWrite {
        path: TagPath::new(path),
        value,
    };
    ws.send(Message::Text(serde_json::to_string(&message).unwrap()))
        .await
        .unwrap();
}

async fn subscribe(
    ws: &mut tokio_tungstenite::WebSocketStream<
        tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>,
    >,
    paths: &[&str],
) {
    let message = ClientMessage::TagSubscribe {
        paths: paths.iter().map(|path| TagPath::new(*path)).collect(),
    };
    ws.send(Message::Text(serde_json::to_string(&message).unwrap()))
        .await
        .unwrap();
}

async fn assert_status(
    ws: &mut tokio_tungstenite::WebSocketStream<
        tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>,
    >,
    expected: &str,
    wait: Duration,
) {
    let deadline = tokio::time::Instant::now() + wait;
    while tokio::time::Instant::now() < deadline {
        if let ServerMessage::TagUpdate { path, value, .. } =
            next_message(ws, Duration::from_millis(750)).await
        {
            if path.as_str() == "system/drivers/rockwell-1/status"
                && matches!(value, openwebhmi_protocol::TagValue::String(value) if value == expected)
            {
                return;
            }
        }
    }
    panic!("status did not reach {expected}");
}

async fn assert_good_pressure_updates(
    ws: &mut tokio_tungstenite::WebSocketStream<
        tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>,
    >,
    count: usize,
    wait: Duration,
) {
    let deadline = tokio::time::Instant::now() + wait;
    let mut seen = 0;
    while seen < count && tokio::time::Instant::now() < deadline {
        if is_pressure_quality(
            next_message(ws, Duration::from_millis(750)).await,
            Quality::Good,
        ) {
            seen += 1;
        }
    }
    assert!(seen >= count, "expected {count} good pressure updates");
}

async fn assert_setpoint_value(
    ws: &mut tokio_tungstenite::WebSocketStream<
        tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>,
    >,
    expected: f64,
    wait: Duration,
) {
    let deadline = tokio::time::Instant::now() + wait;
    while tokio::time::Instant::now() < deadline {
        match next_message(ws, Duration::from_millis(750)).await {
            ServerMessage::Error { code, message } if code == "protocol.parse" => {
                panic!("tag.write parsed as invalid protocol: {message}");
            }
            ServerMessage::TagUpdate {
                path,
                value: TagValue::Real(value),
                quality: Quality::Good,
                ..
            } if path.as_str() == "rockwell-1/Setpoint" && (value - expected).abs() < 0.001 => {
                return;
            }
            _ => {}
        }
    }
    panic!("setpoint did not update to {expected}");
}

async fn assert_status_and_pressure(
    ws: &mut tokio_tungstenite::WebSocketStream<
        tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>,
    >,
    expected_status: &str,
    expected_quality: Quality,
    wait: Duration,
) {
    let deadline = tokio::time::Instant::now() + wait;
    let mut saw_status = false;
    let mut saw_pressure = false;
    while tokio::time::Instant::now() < deadline {
        let message = next_message(ws, Duration::from_millis(750)).await;
        match &message {
            ServerMessage::TagUpdate { path, value, .. }
                if path.as_str() == "system/drivers/rockwell-1/status"
                    && matches!(value, openwebhmi_protocol::TagValue::String(value) if value == expected_status) =>
            {
                saw_status = true;
            }
            _ => {}
        }
        if is_pressure_quality(message, expected_quality) {
            saw_pressure = true;
        }

        if saw_status && saw_pressure {
            return;
        }
    }

    panic!("did not see status {expected_status} and pressure quality {expected_quality:?}");
}

fn is_pressure_quality(message: ServerMessage, expected: Quality) -> bool {
    matches!(
        message,
        ServerMessage::TagUpdate { path, quality, .. }
            if path.as_str() == "rockwell-1/Pressure" && quality == expected
    )
}

async fn next_message(
    ws: &mut tokio_tungstenite::WebSocketStream<
        tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>,
    >,
    wait: Duration,
) -> ServerMessage {
    let frame = timeout(wait, ws.next())
        .await
        .expect("timed out waiting for message")
        .expect("socket closed")
        .expect("websocket error");
    let text = frame.into_text().expect("expected text frame");
    serde_json::from_str::<ServerMessage>(&text).expect("valid server message")
}

fn unused_port() -> u16 {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind ephemeral port");
    listener.local_addr().expect("local addr").port()
}

fn write_project_file(port: u16) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("openwebhmi-phase1-{}-{port}", std::process::id()));
    std::fs::create_dir_all(&dir).expect("create project dir");
    let path = dir.join("project.toml");
    let body = format!(
        r#"
schema_version = 1
name = "phase1-e2e"

[[drivers]]
id = "rockwell-1"
type = "rockwell"

[drivers.config]
host = "127.0.0.1:{port}"
slot = 0
poll_rate_ms = 100
connection_timeout_ms = 500

[[tags]]
path = "rockwell-1/Pressure"
driver = "rockwell-1"
address = "Pressure"

[[tags]]
path = "rockwell-1/Counter"
driver = "rockwell-1"
address = "Counter"

[[tags]]
path = "rockwell-1/Setpoint"
driver = "rockwell-1"
address = "Setpoint"
"#
    );
    std::fs::write(&path, body).expect("write project file");
    path
}

async fn spawn_sim(port: u16) -> Child {
    let mut command = if let Some(exe) = sim_binary_path() {
        Command::new(exe)
    } else {
        let mut command = Command::new(std::env::var("CARGO").unwrap_or_else(|_| "cargo".into()));
        command.args(["run", "-p", "sim-rockwell", "--quiet", "--"]);
        command
    };

    command
        .arg("--bind")
        .arg("127.0.0.1")
        .arg("--port")
        .arg(port.to_string())
        .arg("--config")
        .arg("does-not-exist.toml")
        .stdout(Stdio::null())
        .stderr(Stdio::null());

    let child = command.spawn().expect("spawn simulator");
    wait_for_port(port).await;
    child
}

fn sim_binary_path() -> Option<PathBuf> {
    let mut path = std::env::current_exe().ok()?;
    path.pop();
    if path.file_name().and_then(|name| name.to_str()) == Some("deps") {
        path.pop();
    }
    path.push("sim-rockwell");
    path.exists().then_some(path)
}

async fn wait_for_port(port: u16) {
    let deadline = tokio::time::Instant::now() + Duration::from_secs(5);
    while tokio::time::Instant::now() < deadline {
        if tokio::net::TcpStream::connect(("127.0.0.1", port))
            .await
            .is_ok()
        {
            return;
        }
        sleep(Duration::from_millis(50)).await;
    }
    panic!("simulator did not listen on port {port}");
}
