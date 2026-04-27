use std::net::SocketAddr;
use std::time::Duration;

use futures_util::{SinkExt, StreamExt};
use openwebhmi_gateway::server;
use openwebhmi_project_store::{ArtifactKind, ProjectStore};
use openwebhmi_protocol::{ClientMessage, ServerMessage};
use openwebhmi_tag_engine::TagStore;
use tempfile::tempdir;
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
