//! WebSocket server for the Phase 0 gateway.

use std::collections::HashMap;
use std::net::SocketAddr;
use std::str::FromStr;
use std::sync::{Arc, Mutex, OnceLock};

use anyhow::Context;
use futures_util::{SinkExt, StreamExt};
use openwebhmi_auth::{Permission, Role, SessionManager, UserPatch, UserStore, VerifiedSession};
use openwebhmi_historian::{Aggregation, HistorianStore};
use openwebhmi_project_store::ProjectStore;
use openwebhmi_protocol::{ArtifactKind, AuthUser, ClientMessage, ServerMessage};
use openwebhmi_tag_engine::{TagSnapshot, TagStore};
use tokio::io::{AsyncRead, AsyncWrite};
use tokio::net::TcpListener;
use tokio::sync::mpsc::{self, error::TrySendError};
use tokio::task::JoinHandle;
use tokio_tungstenite::accept_hdr_async;
use tokio_tungstenite::tungstenite::handshake::server::{Request, Response};
use tokio_tungstenite::tungstenite::Message;
use tracing::{debug, info, warn};

use crate::project::{DriverHandle, DriverHandles, WriteCommand, WriteEnqueueError};

type OutboundTx = mpsc::Sender<ServerMessage>;
const OUTBOUND_CAPACITY: usize = 256;
static DEFAULT_HISTORIAN: OnceLock<HistorianStore> = OnceLock::new();

/// Authentication state shared by websocket connections.
#[derive(Clone)]
pub struct AuthContext {
    users: UserStore,
    sessions: SessionManager,
}

impl AuthContext {
    /// Create a websocket auth context.
    pub fn new(users: UserStore, sessions: SessionManager) -> Self {
        Self { users, sessions }
    }
}

/// Set the process-wide historian used by websocket `history.read` handlers.
pub fn set_default_historian(historian: HistorianStore) {
    let _ = DEFAULT_HISTORIAN.set(historian);
}

/// Bind `addr` and serve WebSocket clients forever.
pub async fn run(addr: SocketAddr, store: TagStore) -> anyhow::Result<()> {
    let listener = TcpListener::bind(addr)
        .await
        .with_context(|| format!("failed to bind gateway listener at {addr}"))?;
    serve(listener, store).await
}

/// Bind `addr` and serve WebSocket clients with driver write handles.
pub async fn run_with_driver_handles(
    addr: SocketAddr,
    store: TagStore,
    driver_handles: DriverHandles,
) -> anyhow::Result<()> {
    let listener = TcpListener::bind(addr)
        .await
        .with_context(|| format!("failed to bind gateway listener at {addr}"))?;
    serve_with_driver_handles(listener, store, driver_handles).await
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
    serve_inner(listener, store, None, DriverHandles::new(), None).await
}

/// Serve WebSocket clients with driver write handles.
pub async fn serve_with_driver_handles(
    listener: TcpListener,
    store: TagStore,
    driver_handles: DriverHandles,
) -> anyhow::Result<()> {
    serve_inner(listener, store, None, driver_handles, None).await
}

/// Serve WebSocket clients with project-store protocol support.
pub async fn serve_with_project_store(
    listener: TcpListener,
    store: TagStore,
    project_store: ProjectStore,
) -> anyhow::Result<()> {
    serve_inner(
        listener,
        store,
        Some(project_store),
        DriverHandles::new(),
        None,
    )
    .await
}

/// Serve WebSocket clients with project-store protocol and driver write handles.
pub async fn serve_with_project_store_and_driver_handles(
    listener: TcpListener,
    store: TagStore,
    project_store: ProjectStore,
    driver_handles: DriverHandles,
) -> anyhow::Result<()> {
    serve_inner(listener, store, Some(project_store), driver_handles, None).await
}

/// Serve WebSocket clients with auth enabled.
pub async fn serve_with_auth(
    listener: TcpListener,
    store: TagStore,
    auth: AuthContext,
) -> anyhow::Result<()> {
    serve_inner(listener, store, None, DriverHandles::new(), Some(auth)).await
}

/// Serve WebSocket clients with project-store protocol, driver handles, and auth.
pub async fn serve_with_project_store_driver_handles_and_auth(
    listener: TcpListener,
    store: TagStore,
    project_store: Option<ProjectStore>,
    driver_handles: DriverHandles,
    auth: AuthContext,
) -> anyhow::Result<()> {
    serve_inner(listener, store, project_store, driver_handles, Some(auth)).await
}

async fn serve_inner(
    listener: TcpListener,
    store: TagStore,
    project_store: Option<ProjectStore>,
    driver_handles: DriverHandles,
    auth: Option<AuthContext>,
) -> anyhow::Result<()> {
    let local_addr = listener
        .local_addr()
        .context("failed to read listener local addr")?;
    info!(%local_addr, "gateway listening");

    loop {
        let (stream, peer_addr) = listener.accept().await.context("accept failed")?;
        let store = store.clone();
        let project_store = project_store.clone();
        let driver_handles = driver_handles.clone();
        let auth = auth.clone();

        tokio::spawn(async move {
            if let Err(err) = handle_connection(
                stream,
                peer_addr,
                store,
                project_store,
                driver_handles,
                auth,
            )
            .await
            {
                warn!(%peer_addr, error = %err, "connection handler failed");
            }
        });
    }
}

/// Handle one accepted websocket stream.
#[allow(clippy::result_large_err)]
pub async fn handle_connection<S>(
    stream: S,
    peer_addr: SocketAddr,
    store: TagStore,
    project_store: Option<ProjectStore>,
    driver_handles: DriverHandles,
    auth: Option<AuthContext>,
) -> anyhow::Result<()>
where
    S: AsyncRead + AsyncWrite + Unpin + Send + 'static,
{
    let token = Arc::new(Mutex::new(None::<String>));
    let token_for_callback = token.clone();
    let ws = accept_hdr_async(stream, move |request: &Request, response: Response| {
        if let Some(value) = query_token(request.uri().query()) {
            if let Ok(mut token) = token_for_callback.lock() {
                *token = Some(value.to_string());
            }
        }
        Ok(response)
    })
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

    let token = token.lock().ok().and_then(|token| token.clone());
    let mut session = match (&auth, token.as_deref()) {
        (Some(auth), Some(token)) => match auth.sessions.verify(token) {
            Ok(session) => Some(session),
            Err(err) => {
                debug!(error = %err, "websocket session token rejected");
                try_send_message(
                    &out_tx,
                    ServerMessage::Error {
                        code: "auth.required".to_string(),
                        message: "session token is invalid or expired".to_string(),
                    },
                );
                drop(out_tx);
                return Ok(());
            }
        },
        _ => None,
    };
    let mut current_view_allowed_roles: Option<Vec<String>> = None;
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
            ClientMessage::AuthLogin { username, password } => {
                let Some(auth) = auth.as_ref() else {
                    send_error(&out_tx, "auth.unavailable", "auth is not enabled".into());
                    continue;
                };
                match auth.users.authenticate(&username, &password) {
                    Ok(Some(user)) => {
                        match auth.sessions.issue(&user.id, &user.username, &user.roles) {
                            Ok(token) => {
                                let roles = user
                                    .roles
                                    .iter()
                                    .map(ToString::to_string)
                                    .collect::<Vec<_>>();
                                let user_id = user.id.clone();
                                session = Some(VerifiedSession {
                                    user_id: user_id.clone(),
                                    username: user.username,
                                    roles: user.roles,
                                });
                                try_send_message(
                                    &out_tx,
                                    ServerMessage::AuthResult {
                                        session_token: Some(token),
                                        user_id: Some(user_id),
                                        roles,
                                        error: None,
                                    },
                                );
                            }
                            Err(err) => send_error(&out_tx, "auth.session", err.to_string()),
                        }
                    }
                    Ok(None) => try_send_message(
                        &out_tx,
                        ServerMessage::AuthResult {
                            session_token: None,
                            user_id: None,
                            roles: Vec::new(),
                            error: Some("invalid username or password".into()),
                        },
                    ),
                    Err(err) => send_error(&out_tx, "auth.login", err.to_string()),
                }
            }
            ClientMessage::AuthLogout => {
                session = None;
                try_send_message(
                    &out_tx,
                    ServerMessage::AuthResult {
                        session_token: None,
                        user_id: None,
                        roles: Vec::new(),
                        error: None,
                    },
                );
            }
            ClientMessage::TagSubscribe { paths } => {
                if !authorize(
                    &out_tx,
                    auth.as_ref(),
                    session.as_ref(),
                    Permission::ReadTags,
                ) {
                    continue;
                }
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
                if !authorize(
                    &out_tx,
                    auth.as_ref(),
                    session.as_ref(),
                    Permission::ReadTags,
                ) {
                    continue;
                }
                for path in paths {
                    if let Some(handle) = subscriptions.remove(&path) {
                        handle.abort();
                    }
                }
            }
            ClientMessage::TagWrite { path, value } => {
                if !authorize_write(
                    &out_tx,
                    auth.as_ref(),
                    session.as_ref(),
                    current_view_allowed_roles.as_deref(),
                ) {
                    continue;
                }
                handle_tag_write(&out_tx, &driver_handles, path, value);
            }
            ClientMessage::Ping => {
                try_send_message(&out_tx, ServerMessage::Pong);
            }
            ClientMessage::ProjectSubscribe { project_id } => {
                if !authorize(
                    &out_tx,
                    auth.as_ref(),
                    session.as_ref(),
                    Permission::ReadViews,
                ) {
                    continue;
                }
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
                if !authorize(
                    &out_tx,
                    auth.as_ref(),
                    session.as_ref(),
                    Permission::ReadViews,
                ) {
                    continue;
                }
                if let Some(handle) = project_subscriptions.remove(&project_id) {
                    handle.abort();
                }
            }
            ClientMessage::ProjectLoad { project_id } => {
                if !authorize(
                    &out_tx,
                    auth.as_ref(),
                    session.as_ref(),
                    Permission::ReadViews,
                ) {
                    continue;
                }
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
            ClientMessage::HistoryRead {
                request_id,
                tag_path,
                t_start_ms,
                t_end_ms,
                aggregation,
                max_points,
            } => {
                if !authorize(
                    &out_tx,
                    auth.as_ref(),
                    session.as_ref(),
                    Permission::ReadTags,
                ) {
                    continue;
                }
                let Some(historian) = DEFAULT_HISTORIAN.get() else {
                    send_error(
                        &out_tx,
                        "history.unavailable",
                        "historian is not enabled".into(),
                    );
                    continue;
                };
                let aggregation = match Aggregation::from_str(&aggregation) {
                    Ok(aggregation) => aggregation,
                    Err(err) => {
                        send_error(&out_tx, "history.aggregation", err.to_string());
                        continue;
                    }
                };
                match historian.read(&tag_path, t_start_ms, t_end_ms, aggregation, max_points) {
                    Ok(points) => try_send_message(
                        &out_tx,
                        ServerMessage::HistoryResult {
                            request_id,
                            tag_path,
                            points: points
                                .into_iter()
                                .map(|point| openwebhmi_protocol::HistoryPoint {
                                    ts_ms: point.ts_ms,
                                    value: point.value,
                                    quality: point.quality,
                                })
                                .collect(),
                        },
                    ),
                    Err(err) => send_error(&out_tx, "history.read", err.to_string()),
                }
            }
            ClientMessage::ProjectSaveArtifact {
                request_id,
                project_id,
                artifact,
                body,
            } => {
                if !authorize(
                    &out_tx,
                    auth.as_ref(),
                    session.as_ref(),
                    Permission::AuthorProject,
                ) {
                    continue;
                }
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
                if !authorize(
                    &out_tx,
                    auth.as_ref(),
                    session.as_ref(),
                    Permission::ReadViews,
                ) {
                    continue;
                }
                let Some(project_store) = project_store.as_ref() else {
                    send_unavailable(&out_tx);
                    continue;
                };
                match project_store.load(&project_id) {
                    Ok(project) => {
                        match project.views.into_iter().find(|view| view.id == view_id) {
                            Some(view) => {
                                current_view_allowed_roles = view.allowed_roles.clone();
                                try_send_message(
                                    &out_tx,
                                    ServerMessage::ViewDefinition {
                                        project_id,
                                        view_id,
                                        version: project.version,
                                        view,
                                    },
                                )
                            }
                            None => send_error(&out_tx, "view.not_found", "view not found".into()),
                        }
                    }
                    Err(err) => send_error(&out_tx, "view.open", err.to_string()),
                }
            }
            ClientMessage::ViewClose { .. } => {}
            ClientMessage::UserList => {
                let Some(auth) = auth.as_ref() else {
                    send_error(&out_tx, "auth.unavailable", "auth is not enabled".into());
                    continue;
                };
                if !authorize(
                    &out_tx,
                    Some(auth),
                    session.as_ref(),
                    Permission::ManageUsers,
                ) {
                    continue;
                }
                send_user_list(&out_tx, auth);
            }
            ClientMessage::UserUpsert {
                username,
                password,
                roles,
            } => {
                let Some(auth) = auth.as_ref() else {
                    send_error(&out_tx, "auth.unavailable", "auth is not enabled".into());
                    continue;
                };
                if !authorize(
                    &out_tx,
                    Some(auth),
                    session.as_ref(),
                    Permission::ManageUsers,
                ) {
                    continue;
                }
                let roles = match parse_roles(&roles) {
                    Ok(roles) => roles,
                    Err(err) => {
                        send_error(&out_tx, "auth.roles", err);
                        continue;
                    }
                };
                match auth.users.upsert_user(UserPatch {
                    username,
                    password,
                    roles,
                }) {
                    Ok(_) => send_user_list(&out_tx, auth),
                    Err(err) => send_error(&out_tx, "user.upsert", err.to_string()),
                }
            }
            ClientMessage::UserDelete { user_id } => {
                let Some(auth) = auth.as_ref() else {
                    send_error(&out_tx, "auth.unavailable", "auth is not enabled".into());
                    continue;
                };
                if !authorize(
                    &out_tx,
                    Some(auth),
                    session.as_ref(),
                    Permission::ManageUsers,
                ) {
                    continue;
                }
                match auth.users.delete_user(&user_id) {
                    Ok(()) => send_user_list(&out_tx, auth),
                    Err(err) => send_error(&out_tx, "user.delete", err.to_string()),
                }
            }
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

fn handle_tag_write(
    out_tx: &OutboundTx,
    driver_handles: &DriverHandles,
    path: String,
    value: openwebhmi_protocol::TagValue,
) {
    let Some((driver_id, address)) = split_tag_path(&path) else {
        send_error(
            out_tx,
            "tag.write.failed",
            format!("tag path '{path}' must be '<driver_id>/<address>'"),
        );
        return;
    };

    let Some(driver) = driver_handles.get(driver_id) else {
        send_error(
            out_tx,
            "tag.write.unknown_driver",
            format!("unknown driver '{driver_id}'"),
        );
        return;
    };

    match try_write(driver, address, value) {
        Ok(()) => {}
        Err(WriteEnqueueError::Busy) => send_error(
            out_tx,
            "tag.write.busy",
            format!("driver '{driver_id}' write queue is full"),
        ),
        Err(WriteEnqueueError::Closed) => send_error(
            out_tx,
            "tag.write.failed",
            format!("driver '{driver_id}' write channel is closed"),
        ),
    }
}

fn authorize(
    out_tx: &OutboundTx,
    auth: Option<&AuthContext>,
    session: Option<&VerifiedSession>,
    permission: Permission,
) -> bool {
    if auth.is_none() {
        return true;
    }
    let Some(session) = session else {
        send_error(out_tx, "auth.required", "authentication required".into());
        return false;
    };
    if session
        .roles
        .iter()
        .copied()
        .any(|role| role.allows(permission))
    {
        true
    } else {
        send_error(out_tx, "auth.forbidden", "permission denied".into());
        false
    }
}

fn authorize_write(
    out_tx: &OutboundTx,
    auth: Option<&AuthContext>,
    session: Option<&VerifiedSession>,
    allowed_roles: Option<&[String]>,
) -> bool {
    if !authorize(out_tx, auth, session, Permission::WriteTags) {
        return false;
    }
    let Some(session) = session else {
        return true;
    };
    let Some(allowed_roles) = allowed_roles else {
        return true;
    };
    if session
        .roles
        .iter()
        .any(|role| allowed_roles.iter().any(|allowed| allowed == role.as_str()))
    {
        true
    } else {
        send_error(
            out_tx,
            "auth.forbidden",
            "view ACL denies tag writes".into(),
        );
        false
    }
}

fn parse_roles(values: &[String]) -> Result<Vec<Role>, String> {
    values
        .iter()
        .map(|value| Role::from_str(value).map_err(|err| err.to_string()))
        .collect()
}

fn send_user_list(out_tx: &OutboundTx, auth: &AuthContext) {
    match auth.users.list_users() {
        Ok(users) => try_send_message(
            out_tx,
            ServerMessage::UserList {
                users: users
                    .into_iter()
                    .map(|user| AuthUser {
                        id: user.id,
                        username: user.username,
                        roles: user
                            .roles
                            .into_iter()
                            .map(|role| role.to_string())
                            .collect(),
                    })
                    .collect(),
            },
        ),
        Err(err) => send_error(out_tx, "user.list", err.to_string()),
    }
}

fn query_token(query: Option<&str>) -> Option<&str> {
    query?.split('&').find_map(|pair| {
        let (key, value) = pair.split_once('=')?;
        (key == "token" && !value.is_empty()).then_some(value)
    })
}

fn try_write(
    driver: &DriverHandle,
    address: &str,
    value: openwebhmi_protocol::TagValue,
) -> Result<(), WriteEnqueueError> {
    driver.try_write(WriteCommand {
        address: openwebhmi_driver_api::TagAddress::new(address.to_string()),
        value,
    })
}

fn split_tag_path(path: &str) -> Option<(&str, &str)> {
    let (driver_id, address) = path.split_once('/')?;
    (!driver_id.is_empty() && !address.is_empty()).then_some((driver_id, address))
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
