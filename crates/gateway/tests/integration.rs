use std::net::SocketAddr;
use std::time::Duration;

use futures_util::{SinkExt, StreamExt};
use openwebhmi_gateway::{server, sim_provider};
use openwebhmi_protocol::{ClientMessage, Quality, ServerMessage, TagValue};
use openwebhmi_tag_engine::TagStore;
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
            paths: vec!["system/sim/sin".to_string()],
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
            paths: vec!["system/sim/sin".to_string()],
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
            assert_eq!(path, "system/sim/sin");
            assert_eq!(quality, Quality::Good);
            match value {
                TagValue::Real(value) => assert!((-1.0..=1.0).contains(&value)),
                other => panic!("expected real tag value, got {other:?}"),
            }
        }
        other => panic!("expected tag.update, got {other:?}"),
    }
}
