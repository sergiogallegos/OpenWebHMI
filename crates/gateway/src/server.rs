//! WebSocket server for the Phase 0 gateway.

use std::collections::HashMap;
use std::net::IpAddr;
use std::net::SocketAddr;
use std::str::FromStr;
use std::sync::{Arc, Mutex, OnceLock};
use std::time::{Duration, Instant};

use anyhow::Context;
use futures_util::{SinkExt, StreamExt};
use openwebhmi_alarm_engine::{AlarmEngineHandle, AlarmEvent, AlarmJournal};
use openwebhmi_audit_log::{
    AuditEntry as StoredAuditEntry, AuditEvent, AuditLog, AuditQuery as StoredAuditQuery,
    UserAdminAction, WriteSource,
};
use openwebhmi_auth::{Permission, Role, SessionManager, UserPatch, UserStore, VerifiedSession};
use openwebhmi_backup::{BackupOptions, ImportMode, RestoreOptions};
use openwebhmi_historian::{Aggregation, HistorianStore};
use openwebhmi_project_store::ProjectStore;
use openwebhmi_protocol::{AlarmState, ArtifactKind, AuthUser, ClientMessage, ServerMessage};
use openwebhmi_scripting::{ScriptEvent, ScriptHost, ScriptStatus};
use openwebhmi_tag_engine::{TagSnapshot, TagStore};
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};
use tokio::net::TcpListener;
use tokio::sync::mpsc::{self, error::TrySendError};
use tokio::task::JoinHandle;
use tokio_tungstenite::accept_hdr_async;
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::tungstenite::handshake::server::{Request, Response};
use tracing::{debug, info, warn};

use crate::project::{DriverHandle, DriverHandles, WriteCommand, WriteEnqueueError};

type OutboundTx = mpsc::Sender<ServerMessage>;
const OUTBOUND_CAPACITY: usize = 256;
static DEFAULT_HISTORIAN: OnceLock<HistorianStore> = OnceLock::new();
static DEFAULT_ALARM_JOURNAL: OnceLock<AlarmJournal> = OnceLock::new();
static DEFAULT_ALARM_ENGINE: OnceLock<Arc<Mutex<AlarmEngineHandle>>> = OnceLock::new();
static DEFAULT_SCRIPT_HOST: OnceLock<Arc<ScriptHost>> = OnceLock::new();
static DEFAULT_AUDIT_LOG: OnceLock<AuditLog> = OnceLock::new();
static BACKUP_DOWNLOADS: OnceLock<Mutex<HashMap<String, BackupDownload>>> = OnceLock::new();

struct BackupDownload {
    project_id: String,
    archive: Vec<u8>,
    expires_at: Instant,
    peer_ip: IpAddr,
}

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

/// Set the process-wide alarm journal used by backup export handlers.
pub fn set_default_alarm_journal(journal: AlarmJournal) {
    let _ = DEFAULT_ALARM_JOURNAL.set(journal);
}

/// Set the process-wide alarm engine used by websocket alarm handlers.
pub fn set_default_alarm_engine(engine: Arc<Mutex<AlarmEngineHandle>>) {
    let _ = DEFAULT_ALARM_ENGINE.set(engine);
}

/// Set the process-wide script host used by websocket script-event handlers.
pub fn set_default_script_host(host: Arc<ScriptHost>) {
    let _ = DEFAULT_SCRIPT_HOST.set(host);
}

/// Set the process-wide audit log used by websocket audit handlers.
pub fn set_default_audit_log(audit_log: AuditLog) {
    let _ = DEFAULT_AUDIT_LOG.set(audit_log);
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

/// Serve backup/restore HTTP side-channel requests from an already-bound listener.
pub async fn serve_backup_http(
    listener: TcpListener,
    project_store: ProjectStore,
    auth: AuthContext,
) -> anyhow::Result<()> {
    let local_addr = listener
        .local_addr()
        .context("failed to read backup listener local addr")?;
    info!(%local_addr, "backup HTTP side-channel listening");

    loop {
        let (mut stream, peer_addr) = listener.accept().await.context("accept failed")?;
        let project_store = project_store.clone();
        let auth = auth.clone();
        tokio::spawn(async move {
            if let Err(err) = handle_backup_http(&mut stream, peer_addr, project_store, auth).await
            {
                warn!(%peer_addr, error = %err, "backup HTTP request failed");
                let _ = write_http_response(
                    &mut stream,
                    500,
                    "Internal Server Error",
                    "text/plain",
                    b"backup request failed",
                )
                .await;
            }
        });
    }
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
    let mut alarm_subscriptions: HashMap<String, JoinHandle<()>> = HashMap::new();
    let mut script_subscriptions: HashMap<String, JoinHandle<()>> = HashMap::new();
    let mut audit_subscriptions: HashMap<String, JoinHandle<()>> = HashMap::new();

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
                        append_audit(
                            None,
                            peer_addr,
                            AuditEvent::AuthLogin {
                                username: username.clone(),
                                success: true,
                                reason: None,
                            },
                        );
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
                    Ok(None) => {
                        append_audit(
                            None,
                            peer_addr,
                            AuditEvent::AuthLogin {
                                username,
                                success: false,
                                reason: Some("invalid username or password".into()),
                            },
                        );
                        try_send_message(
                            &out_tx,
                            ServerMessage::AuthResult {
                                session_token: None,
                                user_id: None,
                                roles: Vec::new(),
                                error: Some("invalid username or password".into()),
                            },
                        )
                    }
                    Err(err) => {
                        append_audit(
                            None,
                            peer_addr,
                            AuditEvent::AuthLogin {
                                username,
                                success: false,
                                reason: Some(err.to_string()),
                            },
                        );
                        send_error(&out_tx, "auth.login", err.to_string())
                    }
                }
            }
            ClientMessage::AuthLogout => {
                if let Some(session) = session.as_ref() {
                    append_audit(
                        Some(session),
                        peer_addr,
                        AuditEvent::AuthLogout {
                            username: session.username.clone(),
                        },
                    );
                }
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
            ClientMessage::AlarmSubscribe {
                project_id,
                priority_min,
                priority_max,
            } => {
                if !authorize(
                    &out_tx,
                    auth.as_ref(),
                    session.as_ref(),
                    Permission::ReadTags,
                ) {
                    continue;
                }
                let Some(engine) = DEFAULT_ALARM_ENGINE.get() else {
                    send_error(
                        &out_tx,
                        "alarm.unavailable",
                        "alarm engine is not enabled".into(),
                    );
                    continue;
                };
                if alarm_subscriptions.contains_key(&project_id) {
                    continue;
                }
                let rx = match engine.lock() {
                    Ok(engine) => engine.subscribe_events(),
                    Err(_) => {
                        send_error(
                            &out_tx,
                            "alarm.unavailable",
                            "alarm engine lock poisoned".into(),
                        );
                        continue;
                    }
                };
                let handle = spawn_alarm_forwarder(
                    project_id.clone(),
                    rx,
                    priority_min.unwrap_or(1),
                    priority_max.unwrap_or(5),
                    out_tx.clone(),
                );
                alarm_subscriptions.insert(project_id, handle);
            }
            ClientMessage::AlarmAck { alarm_id, note } => {
                if !authorize(
                    &out_tx,
                    auth.as_ref(),
                    session.as_ref(),
                    Permission::WriteTags,
                ) {
                    continue;
                }
                let Some(engine) = DEFAULT_ALARM_ENGINE.get() else {
                    send_error(
                        &out_tx,
                        "alarm.unavailable",
                        "alarm engine is not enabled".into(),
                    );
                    continue;
                };
                match engine.lock() {
                    Ok(engine) => {
                        let who = session
                            .as_ref()
                            .map(|session| session.username.as_str())
                            .unwrap_or("anonymous");
                        engine.ack(&alarm_id, who, note);
                    }
                    Err(_) => send_error(&out_tx, "alarm.ack", "alarm engine lock poisoned".into()),
                }
            }
            ClientMessage::ScriptSubscribe { project_id } => {
                if !authorize(
                    &out_tx,
                    auth.as_ref(),
                    session.as_ref(),
                    Permission::AuthorProject,
                ) {
                    continue;
                }
                let Some(host) = DEFAULT_SCRIPT_HOST.get() else {
                    send_error(
                        &out_tx,
                        "script.unavailable",
                        "script host is not enabled".into(),
                    );
                    continue;
                };
                if script_subscriptions.contains_key(&project_id) {
                    continue;
                }
                let handle = spawn_script_event_forwarder(
                    project_id.clone(),
                    host.subscribe_events(),
                    out_tx.clone(),
                );
                script_subscriptions.insert(project_id, handle);
            }
            ClientMessage::ScriptUnsubscribe { project_id } => {
                if !authorize(
                    &out_tx,
                    auth.as_ref(),
                    session.as_ref(),
                    Permission::AuthorProject,
                ) {
                    continue;
                }
                if let Some(handle) = script_subscriptions.remove(&project_id) {
                    handle.abort();
                }
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
                handle_tag_write(
                    &out_tx,
                    &driver_handles,
                    session.as_ref(),
                    peer_addr,
                    path,
                    value,
                );
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
                let artifact_kind = format!("{artifact:?}");
                match project_store.save_artifact(&project_id, artifact, body) {
                    Ok(version) => {
                        append_audit(
                            session.as_ref(),
                            peer_addr,
                            AuditEvent::ProjectSave {
                                project_id: project_id.clone(),
                                artifact_kind,
                            },
                        );
                        try_send_message(
                            &out_tx,
                            ServerMessage::ProjectSaveResult {
                                request_id,
                                project_id,
                                version,
                            },
                        )
                    }
                    Err(err) => send_error(&out_tx, "project.save_artifact", err.to_string()),
                }
            }
            ClientMessage::ProjectReadArtifact {
                request_id,
                project_id,
                artifact,
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
                match project_store.read_artifact(&project_id, artifact.clone()) {
                    Ok(body) => try_send_message(
                        &out_tx,
                        ServerMessage::ProjectArtifact {
                            request_id,
                            project_id,
                            artifact,
                            body: body.unwrap_or(serde_json::Value::Null),
                        },
                    ),
                    Err(err) => send_error(&out_tx, "project.read_artifact", err.to_string()),
                }
            }
            ClientMessage::ProjectDeleteArtifact {
                request_id,
                project_id,
                artifact,
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
                match project_store.delete_artifact(&project_id, artifact.clone()) {
                    Ok(()) => try_send_message(
                        &out_tx,
                        ServerMessage::ProjectDeleteResult {
                            request_id,
                            project_id,
                            artifact,
                        },
                    ),
                    Err(err) => send_error(&out_tx, "project.delete_artifact", err.to_string()),
                }
            }
            ClientMessage::ProjectExport {
                request_id,
                project_id,
                include_historian,
                include_alarm_journal,
            } => {
                if !authorize_admin(&out_tx, auth.as_ref(), session.as_ref()) {
                    continue;
                }
                let Some(project_store) = project_store.as_ref() else {
                    send_unavailable(&out_tx);
                    continue;
                };
                let options = BackupOptions {
                    historian_sqlite: None,
                    alarm_journal_sqlite: None,
                    historian_store: include_historian
                        .then(|| DEFAULT_HISTORIAN.get().cloned())
                        .flatten(),
                    alarm_journal: include_alarm_journal
                        .then(|| DEFAULT_ALARM_JOURNAL.get().cloned())
                        .flatten(),
                    audit_log: DEFAULT_AUDIT_LOG.get().cloned(),
                    user: session.as_ref().map(|session| session.username.clone()),
                };
                match openwebhmi_backup::export_project(project_store, &project_id, options) {
                    Ok(archive) => {
                        let token = uuid::Uuid::new_v4().to_string();
                        let size_bytes = archive.len() as u64;
                        let downloads = BACKUP_DOWNLOADS.get_or_init(|| Mutex::new(HashMap::new()));
                        match downloads.lock() {
                            Ok(mut downloads) => {
                                downloads.insert(
                                    token.clone(),
                                    BackupDownload {
                                        project_id: project_id.clone(),
                                        archive,
                                        expires_at: Instant::now() + Duration::from_secs(5 * 60),
                                        peer_ip: peer_addr.ip(),
                                    },
                                );
                                try_send_message(
                                    &out_tx,
                                    ServerMessage::ProjectExportReady {
                                        request_id,
                                        download_url: format!(
                                            "/api/projects/{project_id}/backup/{token}"
                                        ),
                                        size_bytes,
                                    },
                                );
                            }
                            Err(_) => send_error(
                                &out_tx,
                                "backup.registry",
                                "backup token registry lock poisoned".into(),
                            ),
                        }
                    }
                    Err(err) => send_error(&out_tx, "project.export", err.to_string()),
                }
            }
            ClientMessage::ProjectImport {
                request_id,
                project_id: _,
                mode: _,
            } => {
                if !authorize_admin(&out_tx, auth.as_ref(), session.as_ref()) {
                    continue;
                }
                try_send_message(
                    &out_tx,
                    ServerMessage::ProjectImportProgress {
                        request_id,
                        phase: "upload-ready".into(),
                        percent: 0,
                    },
                );
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
                let target_user = username.clone();
                match auth.users.upsert_user(UserPatch {
                    username,
                    password,
                    roles,
                }) {
                    Ok(_) => {
                        let actor = session
                            .as_ref()
                            .map(|session| session.username.clone())
                            .unwrap_or_else(|| "unknown".into());
                        append_audit(
                            session.as_ref(),
                            peer_addr,
                            AuditEvent::UserAdmin {
                                actor,
                                action: UserAdminAction::Created,
                                target_user,
                            },
                        );
                        send_user_list(&out_tx, auth)
                    }
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
                    Ok(()) => {
                        let actor = session
                            .as_ref()
                            .map(|session| session.username.clone())
                            .unwrap_or_else(|| "unknown".into());
                        append_audit(
                            session.as_ref(),
                            peer_addr,
                            AuditEvent::UserAdmin {
                                actor,
                                action: UserAdminAction::Deleted,
                                target_user: user_id,
                            },
                        );
                        send_user_list(&out_tx, auth)
                    }
                    Err(err) => send_error(&out_tx, "user.delete", err.to_string()),
                }
            }
            ClientMessage::AuditSubscribe { request_id } => {
                if !authorize_admin(&out_tx, auth.as_ref(), session.as_ref()) {
                    continue;
                }
                let Some(audit_log) = DEFAULT_AUDIT_LOG.get() else {
                    send_error(
                        &out_tx,
                        "audit.unavailable",
                        "audit log is not enabled".into(),
                    );
                    continue;
                };
                if audit_subscriptions.contains_key(&request_id) {
                    continue;
                }
                let mut rx = audit_log.subscribe();
                let out = out_tx.clone();
                let handle = tokio::spawn(async move {
                    while let Ok(entry) = rx.recv().await {
                        try_send_message(
                            &out,
                            ServerMessage::AuditEvent {
                                entry: audit_entry_to_wire(entry),
                            },
                        );
                    }
                });
                audit_subscriptions.insert(request_id, handle);
            }
            ClientMessage::AuditUnsubscribe { request_id } => {
                if let Some(handle) = audit_subscriptions.remove(&request_id) {
                    handle.abort();
                }
            }
            ClientMessage::AuditQuery { request_id, query } => {
                if !authorize_admin(&out_tx, auth.as_ref(), session.as_ref()) {
                    continue;
                }
                let Some(audit_log) = DEFAULT_AUDIT_LOG.get() else {
                    send_error(
                        &out_tx,
                        "audit.unavailable",
                        "audit log is not enabled".into(),
                    );
                    continue;
                };
                match audit_log.query(&StoredAuditQuery {
                    from_ts_ms: query.from_ts_ms,
                    to_ts_ms: query.to_ts_ms,
                    user: query.user,
                    kinds: query.kinds,
                    limit: query.limit,
                    offset: query.offset,
                }) {
                    Ok((entries, total)) => try_send_message(
                        &out_tx,
                        ServerMessage::AuditQueryResult {
                            request_id,
                            entries: entries.into_iter().map(audit_entry_to_wire).collect(),
                            total,
                        },
                    ),
                    Err(err) => send_error(&out_tx, "audit.query", err.to_string()),
                }
            }
            ClientMessage::ScriptRun {
                request_id,
                script_id,
                ..
            } => {
                if !authorize(
                    &out_tx,
                    auth.as_ref(),
                    session.as_ref(),
                    Permission::AuthorProject,
                ) {
                    continue;
                }
                try_send_message(
                    &out_tx,
                    ServerMessage::ScriptError {
                        request_id,
                        script_id,
                        message: "script.run is reserved for CODEX-U designer integration".into(),
                    },
                );
            }
        }
    }

    for (_, handle) in subscriptions {
        handle.abort();
    }
    for (_, handle) in project_subscriptions {
        handle.abort();
    }
    for (_, handle) in alarm_subscriptions {
        handle.abort();
    }
    for (_, handle) in script_subscriptions {
        handle.abort();
    }
    for (_, handle) in audit_subscriptions {
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
    session: Option<&VerifiedSession>,
    peer_addr: SocketAddr,
    path: String,
    value: openwebhmi_protocol::TagValue,
) {
    let Some((driver_id, address)) = split_tag_path(&path) else {
        append_audit(
            session,
            peer_addr,
            AuditEvent::TagWrite {
                path: path.clone(),
                value,
                success: false,
                source: WriteSource::WebSocket,
                error: Some(format!("tag path '{path}' must be '<driver_id>/<address>'")),
            },
        );
        send_error(
            out_tx,
            "tag.write.failed",
            format!("tag path '{path}' must be '<driver_id>/<address>'"),
        );
        return;
    };

    let Some(driver) = driver_handles.get(driver_id) else {
        append_audit(
            session,
            peer_addr,
            AuditEvent::TagWrite {
                path: path.clone(),
                value,
                success: false,
                source: WriteSource::WebSocket,
                error: Some(format!("unknown driver '{driver_id}'")),
            },
        );
        send_error(
            out_tx,
            "tag.write.unknown_driver",
            format!("unknown driver '{driver_id}'"),
        );
        return;
    };

    match try_write(driver, address, value.clone()) {
        Ok(()) => append_audit(
            session,
            peer_addr,
            AuditEvent::TagWrite {
                path,
                value,
                success: true,
                source: WriteSource::WebSocket,
                error: None,
            },
        ),
        Err(WriteEnqueueError::Busy) => {
            let error = format!("driver '{driver_id}' write queue is full");
            append_audit(
                session,
                peer_addr,
                AuditEvent::TagWrite {
                    path,
                    value,
                    success: false,
                    source: WriteSource::WebSocket,
                    error: Some(error.clone()),
                },
            );
            send_error(out_tx, "tag.write.busy", error)
        }
        Err(WriteEnqueueError::Closed) => {
            let error = format!("driver '{driver_id}' write channel is closed");
            append_audit(
                session,
                peer_addr,
                AuditEvent::TagWrite {
                    path,
                    value,
                    success: false,
                    source: WriteSource::WebSocket,
                    error: Some(error.clone()),
                },
            );
            send_error(out_tx, "tag.write.failed", error)
        }
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

fn authorize_admin(
    out_tx: &OutboundTx,
    auth: Option<&AuthContext>,
    session: Option<&VerifiedSession>,
) -> bool {
    if auth.is_none() {
        return true;
    }
    let Some(session) = session else {
        send_error(out_tx, "auth.required", "authentication required".into());
        return false;
    };
    if session.roles.contains(&Role::Administrator) {
        true
    } else {
        send_error(out_tx, "forbidden", "permission denied".into());
        false
    }
}

fn append_audit(session: Option<&VerifiedSession>, peer_addr: SocketAddr, event: AuditEvent) {
    let Some(audit_log) = DEFAULT_AUDIT_LOG.get() else {
        return;
    };
    let user = session.map(|session| session.username.clone());
    let session_id = session.map(|session| session.user_id.clone());
    if let Err(err) = audit_log.append(user, session_id, Some(peer_addr.ip().to_string()), event) {
        warn!(error = %err, "failed to append audit event");
    }
}

fn audit_entry_to_wire(entry: StoredAuditEntry) -> openwebhmi_protocol::AuditEntry {
    openwebhmi_protocol::AuditEntry {
        id: entry.id,
        ts_ms: entry.ts_ms,
        user: entry.user,
        session_id: entry.session_id,
        source_ip: entry.source_ip,
        kind: entry.kind.kind_name().to_string(),
        payload: serde_json::to_value(entry.kind).unwrap_or(serde_json::Value::Null),
    }
}

async fn handle_backup_http<S>(
    stream: &mut S,
    peer_addr: SocketAddr,
    project_store: ProjectStore,
    auth: AuthContext,
) -> anyhow::Result<()>
where
    S: AsyncRead + AsyncWrite + Unpin,
{
    let request = read_http_request(stream).await?;
    match request.method.as_str() {
        "GET" => handle_backup_download(stream, peer_addr, &request).await,
        "POST" => handle_backup_restore(stream, project_store, auth, request).await,
        _ => {
            write_http_response(
                stream,
                405,
                "Method Not Allowed",
                "text/plain",
                b"method not allowed",
            )
            .await;
            Ok(())
        }
    }
}

async fn handle_backup_download<S>(
    stream: &mut S,
    peer_addr: SocketAddr,
    request: &HttpRequest,
) -> anyhow::Result<()>
where
    S: AsyncRead + AsyncWrite + Unpin,
{
    let Some((project_id, token)) = parse_backup_download_path(&request.path) else {
        write_http_response(stream, 404, "Not Found", "text/plain", b"not found").await;
        return Ok(());
    };
    let download = {
        let downloads = BACKUP_DOWNLOADS.get_or_init(|| Mutex::new(HashMap::new()));
        let mut downloads = downloads
            .lock()
            .map_err(|_| anyhow::anyhow!("backup download registry lock poisoned"))?;
        downloads.remove(token)
    };
    let Some(download) = download else {
        write_http_response(
            stream,
            404,
            "Not Found",
            "text/plain",
            b"backup token not found",
        )
        .await;
        return Ok(());
    };
    if download.project_id != project_id || download.expires_at <= Instant::now() {
        write_http_response(stream, 410, "Gone", "text/plain", b"backup token expired").await;
        return Ok(());
    }
    if download.peer_ip != peer_addr.ip() {
        write_http_response(
            stream,
            403,
            "Forbidden",
            "text/plain",
            b"backup token peer mismatch",
        )
        .await;
        return Ok(());
    }
    write_http_response(
        stream,
        200,
        "OK",
        "application/octet-stream",
        &download.archive,
    )
    .await;
    Ok(())
}

async fn handle_backup_restore<S>(
    stream: &mut S,
    project_store: ProjectStore,
    auth: AuthContext,
    request: HttpRequest,
) -> anyhow::Result<()>
where
    S: AsyncRead + AsyncWrite + Unpin,
{
    let Some(project_id) = parse_backup_restore_path(&request.path) else {
        write_http_response(stream, 404, "Not Found", "text/plain", b"not found").await;
        return Ok(());
    };
    let Some(session) = bearer_session(&auth, &request) else {
        write_http_response(
            stream,
            401,
            "Unauthorized",
            "text/plain",
            b"authentication required",
        )
        .await;
        return Ok(());
    };
    if !session.roles.contains(&Role::Administrator) {
        write_http_response(stream, 403, "Forbidden", "text/plain", b"permission denied").await;
        return Ok(());
    }
    let mode = if request
        .query
        .get("mode")
        .is_some_and(|mode| mode == "merge")
    {
        ImportMode::Merge
    } else {
        ImportMode::Replace
    };
    let manifest = openwebhmi_backup::import_project(
        &project_store,
        &request.body,
        RestoreOptions {
            mode,
            historian_store: DEFAULT_HISTORIAN.get().cloned(),
            alarm_journal: DEFAULT_ALARM_JOURNAL.get().cloned(),
            audit_log: DEFAULT_AUDIT_LOG.get().cloned(),
            user: Some(session.username),
        },
    )?;
    if manifest.project_id != project_id {
        write_http_response(
            stream,
            400,
            "Bad Request",
            "text/plain",
            b"project id mismatch",
        )
        .await;
        return Ok(());
    }
    let body = serde_json::to_vec(&serde_json::json!({
        "ok": true,
        "project_id": manifest.project_id,
        "schema_version": manifest.schema_version
    }))?;
    write_http_response(stream, 200, "OK", "application/json", &body).await;
    Ok(())
}

struct HttpRequest {
    method: String,
    path: String,
    query: HashMap<String, String>,
    headers: HashMap<String, String>,
    body: Vec<u8>,
}

async fn read_http_request<S>(stream: &mut S) -> anyhow::Result<HttpRequest>
where
    S: AsyncRead + Unpin,
{
    let mut bytes = Vec::new();
    let header_end = loop {
        let mut chunk = [0u8; 1024];
        let read = stream.read(&mut chunk).await?;
        if read == 0 {
            anyhow::bail!("connection closed before HTTP headers");
        }
        bytes.extend_from_slice(&chunk[..read]);
        if let Some(pos) = find_header_end(&bytes) {
            break pos;
        }
        if bytes.len() > 32 * 1024 {
            anyhow::bail!("HTTP headers too large");
        }
    };
    let header_text = std::str::from_utf8(&bytes[..header_end])?;
    let mut lines = header_text.split("\r\n");
    let request_line = lines
        .next()
        .ok_or_else(|| anyhow::anyhow!("missing HTTP request line"))?;
    let mut request_parts = request_line.split_whitespace();
    let method = request_parts
        .next()
        .ok_or_else(|| anyhow::anyhow!("missing HTTP method"))?
        .to_string();
    let target = request_parts
        .next()
        .ok_or_else(|| anyhow::anyhow!("missing HTTP target"))?;
    let (path, query) = split_http_target(target);
    let mut headers = HashMap::new();
    for line in lines {
        if let Some((key, value)) = line.split_once(':') {
            headers.insert(key.trim().to_ascii_lowercase(), value.trim().to_string());
        }
    }
    let content_length = headers
        .get("content-length")
        .and_then(|value| value.parse::<usize>().ok())
        .unwrap_or(0);
    let body_start = header_end + 4;
    let mut body = bytes.get(body_start..).unwrap_or_default().to_vec();
    while body.len() < content_length {
        let mut chunk = vec![0u8; content_length - body.len()];
        let read = stream.read(&mut chunk).await?;
        if read == 0 {
            anyhow::bail!("connection closed before HTTP body completed");
        }
        body.extend_from_slice(&chunk[..read]);
    }
    body.truncate(content_length);
    Ok(HttpRequest {
        method,
        path,
        query,
        headers,
        body,
    })
}

async fn write_http_response<S>(
    stream: &mut S,
    status: u16,
    reason: &str,
    content_type: &str,
    body: &[u8],
) where
    S: AsyncWrite + Unpin,
{
    let headers = format!(
        "HTTP/1.1 {status} {reason}\r\nContent-Type: {content_type}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
        body.len()
    );
    let _ = stream.write_all(headers.as_bytes()).await;
    let _ = stream.write_all(body).await;
    let _ = stream.flush().await;
}

fn find_header_end(bytes: &[u8]) -> Option<usize> {
    bytes.windows(4).position(|window| window == b"\r\n\r\n")
}

fn split_http_target(target: &str) -> (String, HashMap<String, String>) {
    let (path, raw_query) = target.split_once('?').unwrap_or((target, ""));
    let query = raw_query
        .split('&')
        .filter_map(|part| part.split_once('='))
        .map(|(key, value)| (key.to_string(), value.to_string()))
        .collect();
    (path.to_string(), query)
}

fn parse_backup_download_path(path: &str) -> Option<(&str, &str)> {
    let rest = path.strip_prefix("/api/projects/")?;
    let (project_id, rest) = rest.split_once("/backup/")?;
    (!project_id.is_empty() && !rest.is_empty()).then_some((project_id, rest))
}

fn parse_backup_restore_path(path: &str) -> Option<&str> {
    let rest = path.strip_prefix("/api/projects/")?;
    let project_id = rest.strip_suffix("/restore")?;
    (!project_id.is_empty()).then_some(project_id)
}

fn bearer_session(auth: &AuthContext, request: &HttpRequest) -> Option<VerifiedSession> {
    let header = request.headers.get("authorization")?;
    let token = header.strip_prefix("Bearer ")?;
    auth.sessions.verify(token).ok()
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

fn spawn_alarm_forwarder(
    project_id: String,
    mut rx: tokio::sync::broadcast::Receiver<AlarmEvent>,
    priority_min: u8,
    priority_max: u8,
    out_tx: OutboundTx,
) -> JoinHandle<()> {
    tokio::spawn(async move {
        loop {
            match rx.recv().await {
                Ok(event) if event.priority >= priority_min && event.priority <= priority_max => {
                    try_send_message(&out_tx, alarm_event_to_message(event));
                }
                Ok(_) => {}
                Err(tokio::sync::broadcast::error::RecvError::Lagged(skipped)) => {
                    warn!(%project_id, skipped, "alarm subscriber lagged");
                }
                Err(tokio::sync::broadcast::error::RecvError::Closed) => break,
            }
        }
    })
}

fn spawn_script_event_forwarder(
    project_id: String,
    mut rx: tokio::sync::broadcast::Receiver<ScriptEvent>,
    out_tx: OutboundTx,
) -> JoinHandle<()> {
    tokio::spawn(async move {
        loop {
            match rx.recv().await {
                Ok(event) if script_event_project_id(&event) == project_id => {
                    try_send_message(&out_tx, script_event_to_message(event));
                }
                Ok(_) => {}
                Err(tokio::sync::broadcast::error::RecvError::Lagged(skipped)) => {
                    warn!(%project_id, skipped, "script subscriber lagged");
                }
                Err(tokio::sync::broadcast::error::RecvError::Closed) => break,
            }
        }
    })
}

fn script_event_project_id(event: &ScriptEvent) -> &str {
    match event {
        ScriptEvent::Status { project_id, .. }
        | ScriptEvent::Log { project_id, .. }
        | ScriptEvent::Error { project_id, .. } => project_id,
    }
}

fn script_event_to_message(event: ScriptEvent) -> ServerMessage {
    match event {
        ScriptEvent::Status {
            project_id,
            script_id,
            status,
        } => ServerMessage::ScriptEvent {
            project_id,
            script_id,
            event_kind: "status".to_string(),
            status: Some(script_status_wire(status).to_string()),
            message: None,
        },
        ScriptEvent::Log {
            project_id,
            script_id,
            message,
        } => ServerMessage::ScriptEvent {
            project_id,
            script_id,
            event_kind: "log".to_string(),
            status: None,
            message: Some(message),
        },
        ScriptEvent::Error {
            project_id,
            script_id,
            message,
        } => ServerMessage::ScriptEvent {
            project_id,
            script_id,
            event_kind: "error".to_string(),
            status: None,
            message: Some(message),
        },
    }
}

fn script_status_wire(status: ScriptStatus) -> &'static str {
    match status {
        ScriptStatus::Starting => "starting",
        ScriptStatus::Ready => "ready",
        ScriptStatus::Restarting => "restarting",
        ScriptStatus::Stopped => "stopped",
    }
}

fn alarm_event_to_message(event: AlarmEvent) -> ServerMessage {
    ServerMessage::AlarmEvent {
        alarm_id: event.alarm_id,
        label: event.label,
        priority: event.priority,
        state: match event.state {
            openwebhmi_alarm_engine::AlarmState::Clear => AlarmState::Clear,
            openwebhmi_alarm_engine::AlarmState::Active => AlarmState::Active,
            openwebhmi_alarm_engine::AlarmState::Acked => AlarmState::Acked,
            openwebhmi_alarm_engine::AlarmState::Cleared => AlarmState::Cleared,
        },
        tag_path: event.tag_path,
        value: event.value,
        quality: event.quality,
        activated_at_ms: event.activated_at_ms,
        transitioned_at_ms: event.transitioned_at_ms,
        who: event.who,
        note: event.note,
        message: event.message,
    }
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
