use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use anyhow::Context;
use clap::Parser;
use openwebhmi_auth::{SessionManager, UserStore};
use openwebhmi_gateway::{project, server, sim_provider};
use openwebhmi_historian::{spawn_recorder, HistorianStore};
use openwebhmi_project_store::ProjectStore;
use openwebhmi_protocol::ArtifactKind;
use openwebhmi_tag_engine::TagStore;
use tokio_rustls::rustls::ServerConfig;
use tokio_rustls::TlsAcceptor;
use tracing::{info, warn};
use tracing_subscriber::EnvFilter;

#[derive(Debug, Parser)]
#[command(author, version, about)]
struct Args {
    /// Address the WebSocket gateway binds.
    #[arg(long, default_value = "127.0.0.1:8080")]
    bind: SocketAddr,
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
    init_tracing(&args.log_level)?;

    let store = TagStore::new();
    tokio::spawn(sim_provider::run(store.clone()));
    let auth = init_auth(&args)?;
    let project_store_root = args.project_store.clone().or_else(|| {
        args.project
            .as_ref()
            .and_then(|path| infer_project_store_root(path))
    });

    let project_store = match project_store_root {
        Some(root) => Some(ProjectStore::open(root)?),
        None => None,
    };

    let driver_handles = if let Some(path) = args.project.as_ref() {
        let project = project::load(path)?;
        spawn_history_recorder(store.clone(), project_store.clone(), &project)?;
        project::spawn_project(project, store.clone())?
    } else {
        project::DriverHandles::new()
    };

    // TODO Phase 3 auth/TLS: this Phase 0 endpoint is intentionally unauthenticated WS.
    tokio::select! {
        result = run_server(args.bind, store, project_store, driver_handles, auth, tls_config(&args)?) => result,
        signal = tokio::signal::ctrl_c() => {
            signal.context("failed to listen for ctrl-c")?;
            info!("shutdown signal received");
            Ok(())
        }
    }
}

fn spawn_history_recorder(
    store: TagStore,
    project_store: Option<ProjectStore>,
    project: &openwebhmi_project_store::Project,
) -> anyhow::Result<()> {
    let historian = HistorianStore::open("openwebhmi-history.sqlite")?;
    server::set_default_historian(historian.clone());

    let mut recorder = spawn_recorder(store, historian, project::history_configs(project));
    let Some(project_store) = project_store else {
        tokio::spawn(async move {
            std::future::pending::<()>().await;
            recorder.abort();
        });
        return Ok(());
    };

    let project_id = project.id.clone();
    tokio::spawn(async move {
        let mut changes = project_store.subscribe_changes(Some(&project_id));
        loop {
            match changes.recv().await {
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
    });
    Ok(())
}

async fn run_server(
    bind: SocketAddr,
    store: TagStore,
    project_store: Option<ProjectStore>,
    driver_handles: project::DriverHandles,
    auth: server::AuthContext,
    tls: Option<Arc<ServerConfig>>,
) -> anyhow::Result<()> {
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
            tokio::spawn(async move {
                match acceptor.accept(stream).await {
                    Ok(stream) => {
                        if let Err(err) = server::handle_connection(
                            stream,
                            peer_addr,
                            store,
                            project_store,
                            driver_handles,
                            Some(auth),
                        )
                        .await
                        {
                            warn!(%peer_addr, error = %err, "TLS websocket handler failed");
                        }
                    }
                    Err(err) => warn!(%peer_addr, error = %err, "TLS accept failed"),
                }
            });
        }
    }

    match project_store {
        Some(project_store) => {
            let listener = tokio::net::TcpListener::bind(bind)
                .await
                .with_context(|| format!("failed to bind gateway listener at {bind}"))?;
            server::serve_with_project_store_driver_handles_and_auth(
                listener,
                store,
                Some(project_store),
                driver_handles,
                auth,
            )
            .await
        }
        None => {
            let listener = tokio::net::TcpListener::bind(bind)
                .await
                .with_context(|| format!("failed to bind gateway listener at {bind}"))?;
            server::serve_with_project_store_driver_handles_and_auth(
                listener,
                store,
                None,
                driver_handles,
                auth,
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

    tracing_subscriber::fmt().with_env_filter(filter).init();
    Ok(())
}

fn infer_project_store_root(project_path: &std::path::Path) -> Option<PathBuf> {
    project_path.parent()?.parent().map(PathBuf::from)
}
