use std::net::SocketAddr;
use std::path::PathBuf;

use anyhow::Context;
use clap::Parser;
use openwebhmi_gateway::{project, server, sim_provider};
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
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    init_tracing(&args.log_level)?;

    let store = TagStore::new();
    tokio::spawn(sim_provider::run(store.clone()));
    if let Some(path) = args.project {
        let project = project::load(&path)?;
        project::spawn_project(project, store.clone())?;
    }

    // TODO Phase 3 auth/TLS: this Phase 0 endpoint is intentionally unauthenticated WS.
    tokio::select! {
        result = server::run(args.bind, store) => result,
        signal = tokio::signal::ctrl_c() => {
            signal.context("failed to listen for ctrl-c")?;
            info!("shutdown signal received");
            Ok(())
        }
    }
}

fn init_tracing(log_level: &str) -> anyhow::Result<()> {
    let filter = EnvFilter::try_from_default_env()
        .or_else(|_| EnvFilter::try_new(log_level))
        .context("invalid log filter")?;

    tracing_subscriber::fmt().with_env_filter(filter).init();
    Ok(())
}
