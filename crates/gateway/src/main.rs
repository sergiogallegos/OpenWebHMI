use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use anyhow::Context;
use clap::Parser;
use openwebhmi_alarm_engine::{AlarmJournal, spawn_alarm_engine};
use openwebhmi_audit_log::AuditLog;
use openwebhmi_auth::{SessionManager, UserStore};
use openwebhmi_gateway::script_writes::GatewayTagWriteSink;
use openwebhmi_gateway::{project, server, sim_provider};
use openwebhmi_historian::{HistorianStore, spawn_recorder};
use openwebhmi_project_store::ProjectStore;
use openwebhmi_protocol::ArtifactKind;
use openwebhmi_scripting::{ScriptHost, ScriptHostOptions};
use openwebhmi_tag_engine::TagStore;
use tokio::task::JoinSet;
use tokio_rustls::TlsAcceptor;
use tokio_rustls::rustls::ServerConfig;
use tokio_util::sync::CancellationToken;
use tracing::{info, warn};
use tracing_subscriber::EnvFilter;

const SHUTDOWN_GRACE: Duration = Duration::from_secs(5);
const BUILD_COMMIT: &str = env!("OPENWEBHMI_GIT_COMMIT");

#[derive(Debug, Parser)]
#[command(author, version, about)]
struct Args {
    /// Print license and corresponding-source information, then exit.
    #[arg(long)]
    legal: bool,
    /// Address the WebSocket gateway binds.
    #[arg(long, default_value = "127.0.0.1:8080")]
    bind: SocketAddr,
    /// Optional HTTP address for backup/restore archive transfer.
    #[arg(long)]
    backup_bind: Option<SocketAddr>,
    /// Log level used when RUST_LOG is not set.
    #[arg(long, default_value = "info")]
    log_level: String,
    /// Optional Phase 1 project file.
    #[arg(long)]
    project: Option<PathBuf>,
    /// Optional project-store root for designer/runtime project protocol.
    #[arg(long)]
    project_store: Option<PathBuf>,
    /// SQLite auth database path.
    #[arg(long, default_value = "openwebhmi-auth.sqlite")]
    auth_db: PathBuf,
    /// SQLite audit journal path.
    #[arg(long, default_value = "openwebhmi-audit.sqlite")]
    audit_db: PathBuf,
    /// Audit event retention window in days.
    #[arg(long, default_value_t = 90)]
    audit_retention_days: u64,
    /// JWT signing secret. Defaults to OPENWEBHMI_JWT_SECRET or a dev-only fallback.
    #[arg(long, env = "OPENWEBHMI_JWT_SECRET")]
    jwt_secret: Option<String>,
    /// Initial admin password. If omitted on first run, a generated one is logged once.
    #[arg(long)]
    admin_password: Option<String>,
    /// TLS certificate PEM path. Must be supplied with --tls-key.
    #[arg(long)]
    tls_cert: Option<PathBuf>,
    /// TLS private key PEM path. Must be supplied with --tls-cert.
    #[arg(long)]
    tls_key: Option<PathBuf>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    if args.legal {
        print_legal_source();
        return Ok(());
    }
    run_gateway(args, CancellationToken::new(), tokio::signal::ctrl_c()).await
}

fn print_legal_source() {
    println!("OpenWebHMI Gateway {}", env!("CARGO_PKG_VERSION"));
    println!("Product core license: AGPL-3.0-only");
    println!("Wire protocol packages: MPL-2.0");
    println!("Build commit: {BUILD_COMMIT}");
    println!(
        "Corresponding source: https://github.com/sergiogallegos/OpenWebHMI/tree/{BUILD_COMMIT}"
    );
    println!(
        "License map: https://github.com/sergiogallegos/OpenWebHMI/blob/{BUILD_COMMIT}/LICENSE-POLICY.md"
    );
    println!("Official builds include LICENSE, LICENSE-POLICY.md, and THIRD_PARTY_NOTICES.md.");
}

/// Cancel-safe: `run_server`, Ctrl-C, and `CancellationToken::cancelled` are
/// selected so dropping this future just stops waiting; owned service tasks
/// remain in the JoinSet passed to the drain path.
async fn run_gateway(
    args: Args,
    shutdown: CancellationToken,
    signal: impl std::future::Future<Output = std::io::Result<()>>,
) -> anyhow::Result<()> {
    init_tracing(&args.log_level)?;

    let mut tasks = JoinSet::new();
    let store = TagStore::new();
    spawn_cancellable(
        &mut tasks,
        shutdown.clone(),
        sim_provider::run(store.clone()),
    );
    let auth = init_auth(&args)?;
    let audit_log = AuditLog::open(&args.audit_db, args.audit_retention_days)
        .with_context(|| format!("failed to open audit db {:?}", args.audit_db))?;
    server::set_default_audit_log(audit_log.clone());
    let project_store_root = args.project_store.clone().or_else(|| {
        args.project
            .as_ref()
            .and_then(|path| infer_project_store_root(path))
    });

    let project_store = match project_store_root {
        Some(root) => Some(ProjectStore::open(root)?),
        None => None,
    };

    let mut _script_host = None;
    let driver_handles = if let Some(path) = args.project.as_ref() {
        let project = project::load(path)?;
        spawn_history_recorder(
            &mut tasks,
            shutdown.clone(),
            store.clone(),
            project_store.clone(),
            &project,
        )?;
        spawn_alarm_runtime(
            &mut tasks,
            shutdown.clone(),
            store.clone(),
            project_store.clone(),
            &project,
        )?;
        let driver_handles = project::spawn_project(project.clone(), store.clone())?;
        _script_host = spawn_script_runtime(
            &mut tasks,
            shutdown.clone(),
            store.clone(),
            project_store.clone(),
            &project,
            driver_handles.clone(),
            Some(audit_log.clone()),
        );
        driver_handles
    } else {
        project::DriverHandles::new()
    };

    // TODO Phase 3 auth/TLS: this Phase 0 endpoint is intentionally unauthenticated WS.
    let backup_project_store = project_store.clone();
    let backup_auth = auth.clone();
    if let (Some(bind), Some(project_store)) = (args.backup_bind, backup_project_store) {
        spawn_cancellable(&mut tasks, shutdown.clone(), async move {
            match tokio::net::TcpListener::bind(bind).await {
                Ok(listener) => {
                    if let Err(err) =
                        server::serve_backup_http(listener, project_store, backup_auth).await
                    {
                        warn!(error = %err, "backup HTTP side-channel stopped");
                    }
                }
                Err(err) => warn!(%bind, error = %err, "failed to bind backup HTTP side-channel"),
            }
        });
    }

    let result = tokio::select! {
        result = run_server(
            ServerRunConfig {
                bind: args.bind,
                store,
                project_store,
                driver_handles,
                auth,
                tls: tls_config(&args)?,
            },
            shutdown.clone(),
            &mut tasks,
        ) => {
            shutdown.cancel();
            result
        }
        signal = signal => {
            signal.context("failed to listen for ctrl-c")?;
            info!("shutdown signal received");
            shutdown.cancel();
            Ok(())
        }
        _ = shutdown.cancelled() => {
            info!("shutdown token cancelled");
            Ok(())
        }
    };
    drain_tasks(&mut tasks).await;
    result
}

struct ServerRunConfig {
    bind: SocketAddr,
    store: TagStore,
    project_store: Option<ProjectStore>,
    driver_handles: project::DriverHandles,
    auth: server::AuthContext,
    tls: Option<Arc<ServerConfig>>,
}

/// Cancel-safe: dropping this helper future cancels the select, and callers
/// must pass work that is safe to drop at any yield point.
fn spawn_cancellable<F>(tasks: &mut JoinSet<()>, shutdown: CancellationToken, work: F)
where
    F: std::future::Future<Output = ()> + Send + 'static,
{
    tasks.spawn(async move {
        tokio::select! {
            _ = shutdown.cancelled() => {}
            _ = work => {}
        }
    });
}

/// Cancel-safe: dropping the drain future only stops awaiting task completion;
/// tasks remain owned by the provided JoinSet.
async fn drain_tasks(tasks: &mut JoinSet<()>) {
    let drained = tokio::time::timeout(SHUTDOWN_GRACE, async {
        while tasks.join_next().await.is_some() {}
    })
    .await;
    if drained.is_err() {
        warn!(
            ?SHUTDOWN_GRACE,
            remaining = tasks.len(),
            "shutdown drain timed out"
        );
        tasks.abort_all();
        while tasks.join_next().await.is_some() {}
    }
}

/// Cancel-safe: the project-change receiver uses Tokio broadcast `recv`, and
/// shutdown cancellation only drops pending restart requests.
fn spawn_script_runtime(
    tasks: &mut JoinSet<()>,
    shutdown: CancellationToken,
    store: TagStore,
    project_store: Option<ProjectStore>,
    project: &openwebhmi_project_store::Project,
    driver_handles: project::DriverHandles,
    audit_log: Option<AuditLog>,
) -> Option<ScriptHost> {
    let scripts = project
        .scripts
        .iter()
        .filter(|script| script.enabled)
        .cloned()
        .collect::<Vec<_>>();
    if scripts.is_empty() {
        return None;
    }

    info!(
        project_id = %project.id,
        count = scripts.len(),
        "starting project scripts"
    );
    let write_sink = std::sync::Arc::new(GatewayTagWriteSink::with_audit_log(
        driver_handles,
        store.clone(),
        audit_log,
    ));
    let host = ScriptHost::spawn(
        project.id.clone(),
        store,
        write_sink,
        scripts,
        ScriptHostOptions::default(),
    );
    server::set_default_script_host(host.clone());
    let shutdown_handle = host.handle();
    if let Some(project_store) = project_store {
        let project_id = project.id.clone();
        let handle = host.handle();
        tasks.spawn(async move {
            let mut changes = project_store.subscribe_changes(Some(&project_id));
            loop {
                tokio::select! {
                    _ = shutdown.cancelled() => {
                        shutdown_handle.shutdown().await;
                        return;
                    }
                    change = changes.recv() => match change {
                    Ok(change)
                        if change.project_id == project_id
                            && matches!(
                                change.artifact,
                                ArtifactKind::ScriptSource { .. } | ArtifactKind::Script { .. }
                            ) =>
                    {
                        let script_id = match change.artifact {
                            ArtifactKind::ScriptSource { id } | ArtifactKind::Script { id } => id,
                            _ => continue,
                        };
                        if let Err(err) = handle.kill_worker(script_id.clone()).await {
                            warn!(
                                project_id = %project_id,
                                script_id = %script_id,
                                error = %err,
                                "failed to restart script after project change"
                            );
                        }
                    }
                    Ok(_) => {}
                    Err(tokio::sync::broadcast::error::RecvError::Lagged(skipped)) => {
                        warn!(project_id = %project_id, skipped, "script project change subscriber lagged");
                    }
                    Err(tokio::sync::broadcast::error::RecvError::Closed) => return,
                }
                }
            }
        });
    } else {
        tasks.spawn(async move {
            shutdown.cancelled().await;
            shutdown_handle.shutdown().await;
        });
    }
    Some(host)
}

/// Cancel-safe: the project-change receiver uses Tokio broadcast `recv`, and
/// shutdown cancellation aborts owned alarm subscription tasks.
fn spawn_alarm_runtime(
    tasks: &mut JoinSet<()>,
    shutdown: CancellationToken,
    store: TagStore,
    project_store: Option<ProjectStore>,
    project: &openwebhmi_project_store::Project,
) -> anyhow::Result<()> {
    let journal = AlarmJournal::open("openwebhmi-alarms.sqlite")?;
    server::set_default_alarm_journal(journal.clone());
    let engine = spawn_alarm_engine(store, journal, project::alarm_definitions(project)?);
    let Some(project_store) = project_store else {
        let engine = Arc::new(std::sync::Mutex::new(engine));
        server::set_default_alarm_engine(engine.clone());
        tasks.spawn(async move {
            shutdown.cancelled().await;
            if let Ok(mut engine) = engine.lock() {
                engine.abort();
            }
        });
        return Ok(());
    };

    let project_id = project.id.clone();
    let engine = Arc::new(std::sync::Mutex::new(engine));
    server::set_default_alarm_engine(engine.clone());
    tasks.spawn(async move {
        let mut changes = project_store.subscribe_changes(Some(&project_id));
        loop {
            tokio::select! {
                _ = shutdown.cancelled() => {
                    if let Ok(mut engine) = engine.lock() {
                        engine.abort();
                    }
                    return;
                }
                change = changes.recv() => match change {
                Ok(change)
                    if change.project_id == project_id
                        && matches!(change.artifact, ArtifactKind::Alarms) =>
                {
                    match project_store.load(&project_id) {
                        Ok(project) => match project::alarm_definitions(&project) {
                            Ok(definitions) => {
                                if let Ok(mut engine) = engine.lock() {
                                    engine.update_definitions(definitions);
                                }
                            }
                            Err(err) => warn!(
                                project_id = %project_id,
                                error = %err,
                                "failed to reload alarm definitions"
                            ),
                        },
                        Err(err) => warn!(
                            project_id = %project_id,
                            error = %err,
                            "failed to reload project for alarm definitions"
                        ),
                    }
                }
                Ok(_) => {}
                Err(tokio::sync::broadcast::error::RecvError::Lagged(skipped)) => {
                    warn!(project_id = %project_id, skipped, "alarm project change subscriber lagged");
                }
                Err(tokio::sync::broadcast::error::RecvError::Closed) => return,
            }
            }
        }
    });
    Ok(())
}

/// Cancel-safe: the project-change receiver uses Tokio broadcast `recv`, and
/// shutdown cancellation aborts owned historian recorder tasks.
fn spawn_history_recorder(
    tasks: &mut JoinSet<()>,
    shutdown: CancellationToken,
    store: TagStore,
    project_store: Option<ProjectStore>,
    project: &openwebhmi_project_store::Project,
) -> anyhow::Result<()> {
    let historian = HistorianStore::open("openwebhmi-history.sqlite")?;
    server::set_default_historian(historian.clone());

    let mut recorder = spawn_recorder(store, historian, project::history_configs(project));
    let Some(project_store) = project_store else {
        tasks.spawn(async move {
            shutdown.cancelled().await;
            recorder.abort();
        });
        return Ok(());
    };

    let project_id = project.id.clone();
    tasks.spawn(async move {
        let mut changes = project_store.subscribe_changes(Some(&project_id));
        loop {
            tokio::select! {
                _ = shutdown.cancelled() => {
                    recorder.abort();
                    return;
                }
                change = changes.recv() => match change {
                Ok(change)
                    if change.project_id == project_id
                        && matches!(change.artifact, ArtifactKind::Tags) =>
                {
                    match project_store.load(&project_id) {
                        Ok(project) => recorder.update_configs(project::history_configs(&project)),
                        Err(err) => {
                            warn!(
                                project_id = %project_id,
                                error = %err,
                                "failed to reload historian tag configuration"
                            );
                        }
                    }
                }
                Ok(_) => {}
                Err(tokio::sync::broadcast::error::RecvError::Lagged(skipped)) => {
                    warn!(
                        project_id = %project_id,
                        skipped,
                        "historian project change subscriber lagged"
                    );
                }
                Err(tokio::sync::broadcast::error::RecvError::Closed) => return,
            }
            }
        }
    });
    Ok(())
}

/// Cancel-safe: dropping the server future stops accepting new connections;
/// per-connection handlers already registered in the service JoinSet receive
/// the shared shutdown token and drain through `drain_tasks`.
async fn run_server(
    config: ServerRunConfig,
    shutdown: CancellationToken,
    tasks: &mut JoinSet<()>,
) -> anyhow::Result<()> {
    let ServerRunConfig {
        bind,
        store,
        project_store,
        driver_handles,
        auth,
        tls,
    } = config;
    if let Some(tls) = tls {
        let listener = tokio::net::TcpListener::bind(bind)
            .await
            .with_context(|| format!("failed to bind gateway listener at {bind}"))?;
        let local_addr = listener.local_addr().context("failed to read local addr")?;
        info!(%local_addr, "gateway listening with TLS");
        let acceptor = TlsAcceptor::from(tls);
        loop {
            let (stream, peer_addr) = listener.accept().await.context("accept failed")?;
            let acceptor = acceptor.clone();
            let store = store.clone();
            let project_store = project_store.clone();
            let driver_handles = driver_handles.clone();
            let auth = auth.clone();
            let shutdown = shutdown.clone();
            tasks.spawn(async move {
                match acceptor.accept(stream).await {
                    Ok(stream) => {
                        if let Err(err) = server::handle_connection(
                            stream,
                            peer_addr,
                            store,
                            project_store,
                            driver_handles,
                            Some(auth),
                            shutdown,
                        )
                        .await
                        {
                            warn!(%peer_addr, error = %err, "TLS websocket handler failed");
                        }
                    }
                    Err(err) => warn!(%peer_addr, error = %err, "TLS accept failed"),
                }
            });
            while let Some(result) = tasks.try_join_next() {
                if let Err(err) = result {
                    warn!(error = %err, "TLS websocket task failed");
                }
            }
        }
    }

    match project_store {
        Some(project_store) => {
            let listener = tokio::net::TcpListener::bind(bind)
                .await
                .with_context(|| format!("failed to bind gateway listener at {bind}"))?;
            server::serve_with_project_store_driver_handles_auth_and_shutdown(
                listener,
                store,
                Some(project_store),
                driver_handles,
                auth,
                shutdown,
                tasks,
            )
            .await
        }
        None => {
            let listener = tokio::net::TcpListener::bind(bind)
                .await
                .with_context(|| format!("failed to bind gateway listener at {bind}"))?;
            server::serve_with_project_store_driver_handles_auth_and_shutdown(
                listener,
                store,
                None,
                driver_handles,
                auth,
                shutdown,
                tasks,
            )
            .await
        }
    }
}

fn init_auth(args: &Args) -> anyhow::Result<server::AuthContext> {
    let users = UserStore::open(&args.auth_db)
        .with_context(|| format!("failed to open auth db {:?}", args.auth_db))?;
    let bootstrap = users.bootstrap_admin(args.admin_password.as_deref())?;
    if let Some(password) = bootstrap.generated_password {
        info!(
            username = %bootstrap.user.username,
            password = %password,
            "generated first-run admin password; ROTATE THIS IMMEDIATELY"
        );
    }

    let secret = args.jwt_secret.clone().unwrap_or_else(|| {
        warn!("OPENWEBHMI_JWT_SECRET/--jwt-secret not set; using dev-only signing secret");
        "openwebhmi-dev-secret-rotate-immediately".to_string()
    });
    Ok(server::AuthContext::new(
        users,
        SessionManager::new(secret.into_bytes(), Duration::from_secs(8 * 60 * 60)),
    ))
}

fn tls_config(args: &Args) -> anyhow::Result<Option<Arc<ServerConfig>>> {
    let (Some(cert_path), Some(key_path)) = (&args.tls_cert, &args.tls_key) else {
        if args.tls_cert.is_some() || args.tls_key.is_some() {
            anyhow::bail!("--tls-cert and --tls-key must be supplied together");
        }
        return Ok(None);
    };

    let cert_file = std::fs::File::open(cert_path)
        .with_context(|| format!("failed to open TLS certificate {:?}", cert_path))?;
    let mut cert_reader = std::io::BufReader::new(cert_file);
    let certs = rustls_pemfile::certs(&mut cert_reader)
        .collect::<Result<Vec<_>, _>>()
        .context("failed to parse TLS certificate")?;

    let key_file = std::fs::File::open(key_path)
        .with_context(|| format!("failed to open TLS private key {:?}", key_path))?;
    let mut key_reader = std::io::BufReader::new(key_file);
    let key = rustls_pemfile::private_key(&mut key_reader)
        .context("failed to parse TLS private key")?
        .context("TLS private key file contained no key")?;

    let config = ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(certs, key)
        .context("invalid TLS certificate/key pair")?;
    Ok(Some(Arc::new(config)))
}

fn init_tracing(log_level: &str) -> anyhow::Result<()> {
    let filter = EnvFilter::try_from_default_env()
        .or_else(|_| EnvFilter::try_new(log_level))
        .context("invalid log filter")?;

    let _ = tracing_subscriber::fmt().with_env_filter(filter).try_init();
    Ok(())
}

fn infer_project_store_root(project_path: &std::path::Path) -> Option<PathBuf> {
    project_path.parent()?.parent().map(PathBuf::from)
}

#[cfg(test)]
mod tests {
    use std::future;
    use std::net::{IpAddr, Ipv4Addr};

    use futures_util::{SinkExt, StreamExt};
    use tokio_tungstenite::connect_async;
    use tokio_tungstenite::tungstenite::Message;
    use tokio_tungstenite::tungstenite::protocol::frame::coding::CloseCode;

    use super::*;

    #[tokio::test]
    async fn shutdown_token_drains_gateway_tasks() {
        let tempdir = tempfile::tempdir().unwrap();
        let shutdown = CancellationToken::new();
        let cancel = shutdown.clone();
        let args = Args {
            legal: false,
            bind: SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 0),
            backup_bind: None,
            log_level: "off".to_string(),
            project: None,
            project_store: None,
            auth_db: tempdir.path().join("auth.sqlite"),
            audit_db: tempdir.path().join("audit.sqlite"),
            audit_retention_days: 90,
            jwt_secret: Some("test-secret".to_string()),
            admin_password: Some("admin-password".to_string()),
            tls_cert: None,
            tls_key: None,
        };

        let gateway = tokio::spawn(run_gateway(
            args,
            shutdown,
            future::pending::<std::io::Result<()>>(),
        ));
        cancel.cancel();

        tokio::time::timeout(SHUTDOWN_GRACE + Duration::from_secs(1), gateway)
            .await
            .expect("gateway should drain before the shutdown deadline")
            .expect("gateway task should not panic")
            .expect("gateway should shut down cleanly");
    }

    #[tokio::test]
    async fn shutdown_token_drains_project_gateway_tasks() {
        let tempdir = tempfile::tempdir().unwrap();
        let project_path = write_shutdown_project_fixture(tempdir.path());
        let shutdown = CancellationToken::new();
        let cancel = shutdown.clone();
        let args = test_args(
            tempdir.path(),
            SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 0),
        );
        let args = Args {
            project: Some(project_path),
            project_store: Some(tempdir.path().to_path_buf()),
            backup_bind: Some(SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 0)),
            ..args
        };

        let gateway = tokio::spawn(run_gateway(
            args,
            shutdown,
            future::pending::<std::io::Result<()>>(),
        ));
        cancel.cancel();

        tokio::time::timeout(SHUTDOWN_GRACE + Duration::from_secs(1), gateway)
            .await
            .expect("gateway should drain project tasks before the shutdown deadline")
            .expect("gateway task should not panic")
            .expect("gateway should shut down cleanly");
    }

    #[tokio::test]
    async fn shutdown_token_sends_websocket_close_frame() {
        let tempdir = tempfile::tempdir().unwrap();
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let bind = listener.local_addr().unwrap();
        let shutdown = CancellationToken::new();
        let cancel = shutdown.clone();
        let auth = init_auth(&test_args(tempdir.path(), bind)).unwrap();
        let mut tasks = JoinSet::new();
        let server_shutdown = shutdown.clone();
        let gateway = tokio::spawn(async move {
            let result = server::serve_with_project_store_driver_handles_auth_and_shutdown(
                listener,
                TagStore::new(),
                None,
                project::DriverHandles::new(),
                auth,
                server_shutdown,
                &mut tasks,
            )
            .await;
            drain_tasks(&mut tasks).await;
            result
        });

        let url = format!("ws://{bind}");
        let (mut ws, _) = connect_async(&url)
            .await
            .expect("websocket should connect to already-bound test listener");
        ws.send(Message::Text(
            serde_json::to_string(&openwebhmi_protocol::ClientMessage::Ping).unwrap(),
        ))
        .await
        .unwrap();
        assert!(matches!(
            ws.next().await,
            Some(Ok(Message::Text(text)))
                if matches!(
                    serde_json::from_str::<openwebhmi_protocol::ServerMessage>(&text),
                    Ok(openwebhmi_protocol::ServerMessage::Pong)
                )
        ));

        cancel.cancel();
        let close = tokio::time::timeout(Duration::from_secs(1), ws.next())
            .await
            .expect("websocket should receive shutdown close within 1s");
        assert!(matches!(
            close,
            Some(Ok(Message::Close(Some(frame))))
                if frame.code == CloseCode::Away
                    && frame.reason.as_ref() == "gateway shutting down"
        ));

        gateway.abort();
        tokio::time::timeout(SHUTDOWN_GRACE + Duration::from_secs(1), gateway)
            .await
            .expect("gateway task should stop after abort")
            .expect_err("gateway accept loop should be aborted after websocket close assertion");
    }

    fn test_args(root: &std::path::Path, bind: SocketAddr) -> Args {
        Args {
            legal: false,
            bind,
            backup_bind: None,
            log_level: "off".to_string(),
            project: None,
            project_store: None,
            auth_db: root.join("auth.sqlite"),
            audit_db: root.join("audit.sqlite"),
            audit_retention_days: 90,
            jwt_secret: Some("test-secret".to_string()),
            admin_password: Some("admin-password".to_string()),
            tls_cert: None,
            tls_key: None,
        }
    }

    fn write_shutdown_project_fixture(root: &std::path::Path) -> PathBuf {
        let project_dir = root.join("shutdown-fixture");
        std::fs::create_dir_all(project_dir.join("alarms")).unwrap();
        std::fs::create_dir_all(project_dir.join("scripts")).unwrap();
        std::fs::write(
            project_dir.join("project.toml"),
            r#"
schema_version = 1
name = "shutdown-fixture"

[[drivers]]
id = "rockwell-1"
type = "rockwell"

[drivers.config]
host = "127.0.0.1"
slot = 0
connection_timeout_ms = 50
poll_rate_ms = 50

[[tags]]
path = "rockwell-1/Pressure"
driver = "rockwell-1"
address = "Pressure"

[tags.history]
rate_ms = 10
"#,
        )
        .unwrap();
        std::fs::write(
            project_dir.join("alarms/alarms.json"),
            r#"[{
  "id": "pressure-high",
  "label": "High pressure",
  "priority": 2,
  "tag_path": "rockwell-1/Pressure",
  "condition": { "kind": "high_limit", "threshold": 200.0 },
  "message": "Pressure high",
  "enabled": true,
  "require_ack": true
}]"#,
        )
        .unwrap();
        std::fs::write(
            project_dir.join("scripts/scripts.json"),
            r#"[{
  "id": "noop",
  "path": "scripts/noop.py",
  "enabled": true,
  "triggers": []
}]"#,
        )
        .unwrap();
        std::fs::write(project_dir.join("scripts/noop.py"), "# no-op\n").unwrap();
        project_dir.join("project.toml")
    }
}
