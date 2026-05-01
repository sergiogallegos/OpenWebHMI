use std::net::SocketAddr;

use anyhow::{Context, Result};
use clap::Parser;
use tokio::net::TcpListener;

/// Minimal OPC UA simulator.
#[derive(Debug, Parser)]
struct Args {
    /// Endpoint socket address for the simulator server.
    #[arg(long, default_value = "127.0.0.1:4855")]
    bind: SocketAddr,
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()),
        )
        .init();

    let args = Args::parse();
    let listener = TcpListener::bind(args.bind)
        .await
        .context("bind sim-opcua")?;
    let sim = sim_opcua::start_with_listener(listener).await?;
    tracing::info!(endpoint = %sim.endpoint(), "sim-opcua listening");
    tokio::signal::ctrl_c().await?;
    sim.stop();
    Ok(())
}
