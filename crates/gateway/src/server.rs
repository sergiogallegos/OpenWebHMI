//! WebSocket server for the Phase 0 gateway.

use std::collections::HashMap;
use std::net::SocketAddr;

use anyhow::Context;
use futures_util::{SinkExt, StreamExt};
use openwebhmi_protocol::{ClientMessage, ServerMessage};
use openwebhmi_tag_engine::{TagSnapshot, TagStore};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::mpsc::{self, error::TrySendError};
use tokio::task::JoinHandle;
use tokio_tungstenite::tungstenite::Message;
use tracing::{debug, info, warn};

type OutboundTx = mpsc::Sender<ServerMessage>;
const OUTBOUND_CAPACITY: usize = 256;

/// Bind `addr` and serve WebSocket clients forever.
pub async fn run(addr: SocketAddr, store: TagStore) -> anyhow::Result<()> {
    let listener = TcpListener::bind(addr)
        .await
        .with_context(|| format!("failed to bind gateway listener at {addr}"))?;
    serve(listener, store).await
}

/// Serve WebSocket clients from an already-bound listener.
pub async fn serve(listener: TcpListener, store: TagStore) -> anyhow::Result<()> {
    let local_addr = listener
        .local_addr()
        .context("failed to read listener local addr")?;
    info!(%local_addr, "gateway listening");

    loop {
        let (stream, peer_addr) = listener.accept().await.context("accept failed")?;
        let store = store.clone();

        tokio::spawn(async move {
            if let Err(err) = handle_connection(stream, peer_addr, store).await {
                warn!(%peer_addr, error = %err, "connection handler failed");
            }
        });
    }
}

async fn handle_connection(
    stream: TcpStream,
    peer_addr: SocketAddr,
    store: TagStore,
) -> anyhow::Result<()> {
    let ws = tokio_tungstenite::accept_async(stream)
        .await
        .context("websocket handshake failed")?;
    info!(%peer_addr, "websocket connected");

    let (mut sink, mut incoming) = ws.split();
    let (out_tx, mut out_rx) = mpsc::channel::<ServerMessage>(OUTBOUND_CAPACITY);

    let writer = tokio::spawn(async move {
        while let Some(message) = out_rx.recv().await {
            let json = match serde_json::to_string(&message) {
                Ok(json) => json,
                Err(err) => {
                    warn!(error = %err, "failed to serialize server message");
                    continue;
                }
            };

            if let Err(err) = sink.send(Message::Text(json)).await {
                debug!(error = %err, "failed to send websocket message");
                break;
            }
        }
    });

    let mut subscriptions: HashMap<String, JoinHandle<()>> = HashMap::new();

    while let Some(item) = incoming.next().await {
        let message = match item {
            Ok(message) => message,
            Err(err) => {
                info!(%peer_addr, error = %err, "websocket receive error");
                break;
            }
        };

        if message.is_close() {
            info!(%peer_addr, "websocket closed");
            break;
        }

        if message.is_binary() {
            try_send_message(
                &out_tx,
                ServerMessage::Error {
                    code: "protocol.binary_unsupported".to_string(),
                    message: "v1 uses JSON only".to_string(),
                },
            );
            continue;
        }

        if !message.is_text() {
            continue;
        }

        let text = message.into_text().context("text frame was not utf-8")?;
        let client_message = match serde_json::from_str::<ClientMessage>(&text) {
            Ok(message) => message,
            Err(err) => {
                try_send_message(
                    &out_tx,
                    ServerMessage::Error {
                        code: "protocol.parse".to_string(),
                        message: err.to_string(),
                    },
                );
                continue;
            }
        };

        match client_message {
            ClientMessage::TagSubscribe { paths } => {
                for path in paths {
                    if subscriptions.contains_key(&path) {
                        continue;
                    }

                    if let Some(snapshot) = store.get(&path) {
                        try_send_message(&out_tx, snapshot_to_message(snapshot));
                    }

                    let handle = spawn_forwarder(path.clone(), store.clone(), out_tx.clone());
                    subscriptions.insert(path, handle);
                }
            }
            ClientMessage::TagUnsubscribe { paths } => {
                for path in paths {
                    if let Some(handle) = subscriptions.remove(&path) {
                        handle.abort();
                    }
                }
            }
            ClientMessage::Ping => {
                try_send_message(&out_tx, ServerMessage::Pong);
            }
        }
    }

    for (_, handle) in subscriptions {
        handle.abort();
    }
    drop(out_tx);
    writer.abort();
    info!(%peer_addr, "connection cleanup complete");

    Ok(())
}

fn spawn_forwarder(path: String, store: TagStore, out_tx: OutboundTx) -> JoinHandle<()> {
    tokio::spawn(async move {
        let mut rx = store.subscribe(&path);
        loop {
            match rx.recv().await {
                Ok(snapshot) => {
                    try_send_message(&out_tx, snapshot_to_message(snapshot));
                }
                Err(tokio::sync::broadcast::error::RecvError::Lagged(skipped)) => {
                    warn!(%path, skipped, "tag subscriber lagged");
                }
                Err(tokio::sync::broadcast::error::RecvError::Closed) => break,
            }
        }
    })
}

fn try_send_message(out_tx: &OutboundTx, message: ServerMessage) {
    match out_tx.try_send(message) {
        Ok(()) => {}
        Err(TrySendError::Full(_)) => {
            warn!("client outbound queue full; dropping server message");
        }
        Err(TrySendError::Closed(_)) => {}
    }
}

fn snapshot_to_message(snapshot: TagSnapshot) -> ServerMessage {
    ServerMessage::TagUpdate {
        path: snapshot.path,
        value: snapshot.value,
        quality: snapshot.quality,
        ts: snapshot.ts,
    }
}
