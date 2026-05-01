use std::net::SocketAddr;

use anyhow::Result;
use clap::Parser;

/// MQTT simulator.
#[derive(Debug, Parser)]
struct Args {
    /// Broker socket address.
    #[arg(long, default_value = "127.0.0.1:1883")]
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
    let sim = sim_mqtt::start(args.bind).await?;
    tracing::info!(addr = %sim.addr(), ws_addr = %sim.ws_addr(), "sim-mqtt listening");
    loop {
        sim.publish_once().await?;
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
    }
}
