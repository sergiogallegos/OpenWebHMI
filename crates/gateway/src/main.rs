use std::net::SocketAddr;
use std::path::PathBuf;

use anyhow::Context;
use clap::Parser;
use openwebhmi_gateway::{project, server, sim_provider};
use openwebhmi_project_store::ProjectStore;
use openwebhmi_tag_engine::TagStore;
use tracing::info;
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
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    init_tracing(&args.log_level)?;

    let store = TagStore::new();
    tokio::spawn(sim_provider::run(store.clone()));
    let project_store_root = args.project_store.clone().or_else(|| {
        args.project
            .as_ref()
            .and_then(|path| infer_project_store_root(path))
    });

    let driver_handles = if let Some(path) = args.project {
        let project = project::load(&path)?;
        project::spawn_project(project, store.clone())?
    } else {
        project::DriverHandles::new()
    };
    let project_store = match project_store_root {
        Some(root) => Some(ProjectStore::open(root)?),
        None => None,
    };

    // TODO Phase 3 auth/TLS: this Phase 0 endpoint is intentionally unauthenticated WS.
    tokio::select! {
        result = run_server(args.bind, store, project_store, driver_handles) => result,
        signal = tokio::signal::ctrl_c() => {
            signal.context("failed to listen for ctrl-c")?;
            info!("shutdown signal received");
            Ok(())
        }
    }
}

async fn run_server(
    bind: SocketAddr,
    store: TagStore,
    project_store: Option<ProjectStore>,
    driver_handles: project::DriverHandles,
) -> anyhow::Result<()> {
    match project_store {
        Some(project_store) => {
            let listener = tokio::net::TcpListener::bind(bind)
                .await
                .with_context(|| format!("failed to bind gateway listener at {bind}"))?;
            server::serve_with_project_store_and_driver_handles(
                listener,
                store,
                project_store,
                driver_handles,
            )
            .await
        }
        None => server::run_with_driver_handles(bind, store, driver_handles).await,
    }
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
