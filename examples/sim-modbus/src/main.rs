use std::net::SocketAddr;

use anyhow::Result;
use clap::Parser;

/// Minimal Modbus TCP simulator.
#[derive(Debug, Parser)]
struct Args {
    /// Socket address to bind.
    #[arg(long, default_value = "127.0.0.1:5502")]
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
    sim_modbus::bind_and_serve(args.bind, sim_modbus::default_state()).await
}
