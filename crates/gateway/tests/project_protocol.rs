use std::net::SocketAddr;
use std::time::Duration;

use futures_util::{SinkExt, StreamExt};
use openwebhmi_auth::{Role, SessionManager, UserStore};
use openwebhmi_gateway::{project::DriverHandles, server};
use openwebhmi_project_store::{ArtifactKind, ProjectStore};
use openwebhmi_protocol::{ClientMessage, ServerMessage};
use openwebhmi_tag_engine::TagStore;
use tempfile::tempdir;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::time::{sleep, timeout};
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::Message;

#[tokio::test]
async fn project_protocol_saves_broadcasts_and_loads() {
    let dir = tempdir().unwrap();
    let project_store = ProjectStore::open(dir.path()).unwrap();
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr: SocketAddr = listener.local_addr().unwrap();
    let server = tokio::spawn(server::serve_with_project_store(
        listener,
        TagStore::new(),
        project_store,
    ));

    let (mut ws, _) = connect_async(format!("ws://{addr}")).await.unwrap();
    send(
        &mut ws,
        ClientMessage::ProjectSubscribe {
            project_id: "demo".to_string(),
        },
    )
    .await;
    sleep(Duration::from_millis(25)).await;
    send(
        &mut ws,
        ClientMessage::ProjectSaveArtifact {
            request_id: Some("save-1".to_string()),
            project_id: "demo".to_string(),
            artifact: ArtifactKind::ProjectMeta,
            body: serde_json::json!({
                "schema_version": 1,
                "name": "Demo",
                "drivers": []
            }),
        },
    )
    .await;

    assert!(matches!(
        next(&mut ws).await,
        ServerMessage::ProjectSaveResult {
            request_id: Some(request_id),
            project_id,
            version: 1,
        } if request_id == "save-1" && project_id == "demo"
    ));
    assert!(matches!(
        next(&mut ws).await,
        ServerMessage::ProjectChanged {
            project_id,
            version: 1,
            artifact: ArtifactKind::ProjectMeta,
            ..
        } if project_id == "demo"
    ));

    send(
        &mut ws,
        ClientMessage::ProjectLoad {
            project_id: "demo".to_string(),
        },
    )
    .await;
    assert!(matches!(
        next(&mut ws).await,
        ServerMessage::ProjectSnapshot { version: 1, .. }
    ));

    server.abort();
}

#[tokio::test]
async fn project_backup_exports_downloads_restores_over_http_side_channel() {
    let source_dir = tempdir().unwrap();
    let target_dir = tempdir().unwrap();
    let source_store = ProjectStore::open(source_dir.path()).unwrap();
    let target_store = ProjectStore::open(target_dir.path()).unwrap();
    seed_project(&source_store);

    let users = UserStore::memory().unwrap();
    users
        .create_user("admin", "admin-pass", vec![Role::Administrator])
        .unwrap();
    let sessions = SessionManager::new(b"backup-test-secret".to_vec(), Duration::from_secs(60));
    let token = sessions
        .issue("admin-id", "admin", &[Role::Administrator])
        .unwrap();
    let auth = server::AuthContext::new(users, sessions);

    let ws_listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let ws_addr = ws_listener.local_addr().unwrap();
    let ws_server = tokio::spawn(server::serve_with_project_store_driver_handles_and_auth(
        ws_listener,
        TagStore::new(),
        Some(source_store.clone()),
        DriverHandles::new(),
        auth.clone(),
    ));

    let download_listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let download_addr = download_listener.local_addr().unwrap();
    let download_server = tokio::spawn(server::serve_backup_http(
        download_listener,
        source_store.clone(),
        auth.clone(),
    ));

    let (mut ws, _) = connect_async(format!("ws://{ws_addr}/?token={token}"))
        .await
        .unwrap();
    send(
        &mut ws,
        ClientMessage::ProjectExport {
            request_id: "backup-r1".into(),
            project_id: "demo".into(),
            include_historian: false,
            include_alarm_journal: false,
        },
    )
    .await;
    let download_url = match next(&mut ws).await {
        ServerMessage::ProjectExportReady {
            request_id,
            download_url,
            size_bytes,
        } => {
            assert_eq!(request_id, "backup-r1");
            assert!(size_bytes > 0);
            download_url
        }
        other => panic!("expected project.export_ready, got {other:?}"),
    };
    let archive = http_get(download_addr, &download_url).await;

    let restore_listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let restore_addr = restore_listener.local_addr().unwrap();
    let restore_server = tokio::spawn(server::serve_backup_http(
        restore_listener,
        target_store.clone(),
        auth,
    ));
    let restore_body = http_post(
        restore_addr,
        "/api/projects/demo/restore?mode=replace",
        &token,
        &archive,
    )
    .await;
    assert!(
        std::str::from_utf8(&restore_body)
            .unwrap()
            .contains("\"ok\":true")
    );

    assert_eq!(
        source_store.load("demo").unwrap(),
        target_store.load("demo").unwrap()
    );

    ws_server.abort();
    download_server.abort();
    restore_server.abort();
}

async fn send(
    ws: &mut tokio_tungstenite::WebSocketStream<
        tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>,
    >,
    message: ClientMessage,
) {
    ws.send(Message::Text(serde_json::to_string(&message).unwrap()))
        .await
        .unwrap();
}

async fn next(
    ws: &mut tokio_tungstenite::WebSocketStream<
        tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>,
    >,
) -> ServerMessage {
    let frame = timeout(Duration::from_secs(2), ws.next())
        .await
        .expect("timed out waiting for message")
        .expect("socket closed")
        .expect("websocket error");
    serde_json::from_str(&frame.into_text().unwrap()).unwrap()
}

fn seed_project(store: &ProjectStore) {
    store
        .save_artifact(
            "demo",
            ArtifactKind::ProjectMeta,
            serde_json::json!({
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
            serde_json::json!([{
                "path":"rockwell-1/value",
                "driver":"rockwell-1",
                "address":"value",
                "data_type":"real"
            }]),
        )
        .unwrap();
    store
        .save_artifact("demo", ArtifactKind::Alarms, serde_json::json!([]))
        .unwrap();
}

async fn http_get(addr: SocketAddr, path: &str) -> Vec<u8> {
    let request = format!("GET {path} HTTP/1.1\r\nHost: {addr}\r\n\r\n");
    http_round_trip(addr, request.as_bytes()).await
}

async fn http_post(addr: SocketAddr, path: &str, token: &str, body: &[u8]) -> Vec<u8> {
    let mut request = format!(
        "POST {path} HTTP/1.1\r\nHost: {addr}\r\nAuthorization: Bearer {token}\r\nContent-Length: {}\r\n\r\n",
        body.len()
    )
    .into_bytes();
    request.extend_from_slice(body);
    http_round_trip(addr, &request).await
}

async fn http_round_trip(addr: SocketAddr, request: &[u8]) -> Vec<u8> {
    let mut stream = tokio::net::TcpStream::connect(addr).await.unwrap();
    stream.write_all(request).await.unwrap();
    stream.shutdown().await.unwrap();
    let mut response = Vec::new();
    stream.read_to_end(&mut response).await.unwrap();
    let split = response
        .windows(4)
        .position(|window| window == b"\r\n\r\n")
        .unwrap();
    let status = std::str::from_utf8(&response[..split]).unwrap();
    assert!(status.starts_with("HTTP/1.1 200"), "{status}");
    response[split + 4..].to_vec()
}
