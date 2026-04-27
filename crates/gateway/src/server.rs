//! WebSocket server for the Phase 0 gateway.

use std::collections::HashMap;
use std::net::SocketAddr;

use anyhow::Context;
use futures_util::{SinkExt, StreamExt};
use openwebhmi_project_store::{ArtifactKind, ProjectStore};
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

/// Bind `addr` and serve WebSocket clients with project-store protocol support.
pub async fn run_with_project_store(
    addr: SocketAddr,
    store: TagStore,
    project_store: ProjectStore,
) -> anyhow::Result<()> {
    let listener = TcpListener::bind(addr)
        .await
        .with_context(|| format!("failed to bind gateway listener at {addr}"))?;
    serve_with_project_store(listener, store, project_store).await
}

/// Serve WebSocket clients from an already-bound listener.
pub async fn serve(listener: TcpListener, store: TagStore) -> anyhow::Result<()> {
    serve_inner(listener, store, None).await
}

/// Serve WebSocket clients with project-store protocol support.
pub async fn serve_with_project_store(
    listener: TcpListener,
    store: TagStore,
    project_store: ProjectStore,
) -> anyhow::Result<()> {
    serve_inner(listener, store, Some(project_store)).await
}

async fn serve_inner(
    listener: TcpListener,
    store: TagStore,
    project_store: Option<ProjectStore>,
) -> anyhow::Result<()> {
    let local_addr = listener
        .local_addr()
        .context("failed to read listener local addr")?;
    info!(%local_addr, "gateway listening");

    loop {
        let (stream, peer_addr) = listener.accept().await.context("accept failed")?;
        let store = store.clone();
        let project_store = project_store.clone();

        tokio::spawn(async move {
            if let Err(err) = handle_connection(stream, peer_addr, store, project_store).await {
                warn!(%peer_addr, error = %err, "connection handler failed");
            }
        });
    }
}

async fn handle_connection(
    stream: TcpStream,
    peer_addr: SocketAddr,
    store: TagStore,
    project_store: Option<ProjectStore>,
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
    let mut project_subscriptions: HashMap<String, JoinHandle<()>> = HashMap::new();

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
            ClientMessage::ProjectSubscribe { project_id } => {
                let Some(project_store) = project_store.clone() else {
                    send_unavailable(&out_tx);
                    continue;
                };
                if project_subscriptions.contains_key(&project_id) {
                    continue;
                }
                let handle = spawn_project_change_forwarder(
                    project_id.clone(),
                    project_store,
                    out_tx.clone(),
                );
                project_subscriptions.insert(project_id, handle);
            }
            ClientMessage::ProjectUnsubscribe { project_id } => {
                if let Some(handle) = project_subscriptions.remove(&project_id) {
                    handle.abort();
                }
            }
            ClientMessage::ProjectLoad { project_id } => {
                let Some(project_store) = project_store.as_ref() else {
                    send_unavailable(&out_tx);
                    continue;
                };
                match project_store.load(&project_id) {
                    Ok(project) => {
                        let version = project.version;
                        match serde_json::to_value(project) {
                            Ok(project) => try_send_message(
                                &out_tx,
                                ServerMessage::ProjectSnapshot { project, version },
                            ),
                            Err(err) => send_error(&out_tx, "project.serialize", err.to_string()),
                        }
                    }
                    Err(err) => send_error(&out_tx, "project.load", err.to_string()),
                }
            }
            ClientMessage::ProjectSaveArtifact {
                request_id,
                project_id,
                artifact,
                body,
            } => {
                let Some(project_store) = project_store.as_ref() else {
                    send_unavailable(&out_tx);
                    continue;
                };
                match project_store.save_artifact(&project_id, artifact, body) {
                    Ok(version) => try_send_message(
                        &out_tx,
                        ServerMessage::ProjectSaveResult {
                            request_id,
                            project_id,
                            version,
                        },
                    ),
                    Err(err) => send_error(&out_tx, "project.save_artifact", err.to_string()),
                }
            }
            ClientMessage::ViewOpen {
                project_id,
                view_id,
            } => {
                let Some(project_store) = project_store.as_ref() else {
                    send_unavailable(&out_tx);
                    continue;
                };
                match project_store.load(&project_id) {
                    Ok(project) => {
                        match project.views.into_iter().find(|view| view.id == view_id) {
                            Some(view) => try_send_message(
                                &out_tx,
                                ServerMessage::ViewDefinition {
                                    project_id,
                                    view_id,
                                    version: project.version,
                                    view,
                                },
                            ),
                            None => send_error(&out_tx, "view.not_found", "view not found".into()),
                        }
                    }
                    Err(err) => send_error(&out_tx, "view.open", err.to_string()),
                }
            }
            ClientMessage::ViewClose { .. } => {}
        }
    }

    for (_, handle) in subscriptions {
        handle.abort();
    }
    for (_, handle) in project_subscriptions {
        handle.abort();
    }
    drop(out_tx);
    writer.abort();
    info!(%peer_addr, "connection cleanup complete");

    Ok(())
}

fn spawn_project_change_forwarder(
    project_id: String,
    project_store: ProjectStore,
    out_tx: OutboundTx,
) -> JoinHandle<()> {
    tokio::spawn(async move {
        let mut rx = project_store.subscribe_changes(Some(&project_id));
        loop {
            match rx.recv().await {
                Ok(change) if change.project_id == project_id => {
                    let is_view = matches!(change.artifact, ArtifactKind::View { .. });
                    try_send_message(
                        &out_tx,
                        ServerMessage::ProjectChanged {
                            project_id: change.project_id,
                            version: change.version,
                            artifact: change.artifact,
                            action: change.action,
                        },
                    );
                    if is_view {
                        // CODEX-L pushes refreshed view definitions to open views.
                    }
                }
                Ok(_) => {}
                Err(tokio::sync::broadcast::error::RecvError::Lagged(skipped)) => {
                    warn!(%project_id, skipped, "project subscriber lagged");
                }
                Err(tokio::sync::broadcast::error::RecvError::Closed) => break,
            }
        }
    })
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

fn send_unavailable(out_tx: &OutboundTx) {
    send_error(
        out_tx,
        "project_store.unavailable",
        "project store is not enabled".to_string(),
    );
}

fn send_error(out_tx: &OutboundTx, code: &str, message: String) {
    try_send_message(
        out_tx,
        ServerMessage::Error {
            code: code.to_string(),
            message,
        },
    );
}

fn snapshot_to_message(snapshot: TagSnapshot) -> ServerMessage {
    ServerMessage::TagUpdate {
        path: snapshot.path,
        value: snapshot.value,
        quality: snapshot.quality,
        ts: snapshot.ts,
    }
}
