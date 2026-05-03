//! Hardware smoke test for `openwebhmi-driver-ads`.
//!
//! Connects to a real Beckhoff TwinCAT 3 runtime, browses symbols, reads four
//! test variables (BOOL/INT/REAL/STRING), subscribes for native ADS notifications,
//! writes to a REAL, and prints what it sees. Exercises the same `Driver` trait
//! path the gateway uses.
//!
//! Expected PLC program (TwinCAT 3 Standard PLC Project, MAIN program):
//! ```text
//! VAR
//!     bRunning   : BOOL    := TRUE;
//!     nCounter   : INT     := 0;
//!     fSetpoint  : REAL    := 50.0;
//!     sStatus    : STRING(80) := 'Idle';
//!     fbCycle    : TON;
//! END_VAR
//! ```
//!
//! Run from the workspace root:
//! ```bash
//! cargo run --example hardware-smoke -p openwebhmi-driver-ads -- \
//!     --host 127.0.0.1 --net-id 192.168.10.100.1.1 \
//!     --source explicit --source-net-id 192.168.10.98.1.1
//! ```
//!
//! Set `RUST_LOG=ads=debug,openwebhmi_driver_ads=trace` for notification logs.

use std::time::Duration;

use anyhow::Context;
use futures_util::StreamExt;
use openwebhmi_driver_ads::AdsDriver;
use openwebhmi_driver_api::{Driver, TagAddress};
use openwebhmi_protocol::TagValue;

#[derive(Debug)]
struct Args {
    host: String,
    net_id: String,
    port: u16,
    source: String,
    source_net_id: Option<String>,
    source_port: Option<u16>,
    subscribe_seconds: u64,
}

impl Args {
    fn parse() -> anyhow::Result<Self> {
        let mut host = None;
        let mut net_id = None;
        let mut port: u16 = 851;
        let mut source = "request".to_string();
        let mut source_net_id = None;
        let mut source_port = None;
        let mut subscribe_seconds: u64 = 10;

        let mut iter = std::env::args().skip(1);
        while let Some(arg) = iter.next() {
            match arg.as_str() {
                "--host" => host = iter.next(),
                "--net-id" => net_id = iter.next(),
                "--port" => {
                    port = iter
                        .next()
                        .context("--port needs a value")?
                        .parse()
                        .context("--port must be a u16")?
                }
                "--source" => {
                    source = iter
                        .next()
                        .context("--source needs a value (request | auto | explicit)")?
                }
                "--source-net-id" => {
                    source_net_id = iter.next();
                }
                "--source-port" => {
                    source_port = Some(
                        iter.next()
                            .context("--source-port needs a value")?
                            .parse()
                            .context("--source-port must be a u16")?,
                    );
                }
                "--subscribe-seconds" => {
                    subscribe_seconds = iter
                        .next()
                        .context("--subscribe-seconds needs a value")?
                        .parse()
                        .context("--subscribe-seconds must be a u64")?
                }
                "-h" | "--help" => {
                    print_help();
                    std::process::exit(0);
                }
                other => anyhow::bail!("unknown argument: {other}"),
            }
        }

        Ok(Self {
            host: host.context("--host is required (e.g. 192.168.10.100)")?,
            net_id: net_id.context("--net-id is required (e.g. 192.168.10.100.1.1)")?,
            port,
            source,
            source_net_id,
            source_port,
            subscribe_seconds,
        })
    }
}

fn print_help() {
    println!(
        "hardware-smoke — Beckhoff TwinCAT ADS driver smoke test

USAGE:
    cargo run --example hardware-smoke -p openwebhmi-driver-ads -- [OPTIONS]

REQUIRED:
    --host <ip>             ADS router host. Use 127.0.0.1 when TwinCAT is installed
                            locally and has the route to the target PLC.
    --net-id <id>           Target AMS NetId (e.g. 192.168.10.100.1.1)

OPTIONAL:
    --port <u16>            ADS port to browse (default: 851 — TwinCAT 3 PLC runtime 1)
    --source <mode>         Source AMS mode: request | auto | explicit (default: request)
                            request: TwinCAT installed locally, router assigns port
                            auto:    derive source NetId from local IPv4 (no TwinCAT)
                            explicit: use --source-net-id and optional --source-port
    --source-net-id <id>    Source AMS NetId for --source explicit
    --source-port <u16>     Source AMS port for --source explicit
    --subscribe-seconds <u64>  How long to watch notifications (default: 10)
"
    );
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .with_target(true)
        .init();

    let args = Args::parse()?;
    println!(
        "config: host={} net_id={} port={} source={} source_net_id={:?} source_port={:?}\n",
        args.host, args.net_id, args.port, args.source, args.source_net_id, args.source_port
    );

    let mut driver = AdsDriver::new();

    let source = match args.source.as_str() {
        "explicit" => serde_json::json!({
            "explicit": {
                "net_id": args.source_net_id.context("--source explicit requires --source-net-id")?,
                "port": args.source_port,
            }
        }),
        "request" | "auto" => serde_json::json!(args.source),
        other => anyhow::bail!("unsupported --source {other}; use request, auto, or explicit"),
    };

    let config = serde_json::json!({
        "host": args.host,
        "ams_net_id": args.net_id,
        "source": source,
        "ports": [args.port],
        "poll_rate_ms": 250,
        "timeout_ms": 5_000,
    });

    println!("=== Connect ===");
    driver
        .connect(config)
        .await
        .context("ADS connect failed — check AMS route, NetId, and that the runtime is running")?;
    println!("connected.\n");

    // Browse first 25 symbols
    println!("=== Browse (first 25 symbols) ===");
    let nodes = driver.browse(None).await.context("browse failed")?;
    println!("total symbols: {}", nodes.len());
    for node in nodes.iter().take(25) {
        let addr = node
            .address
            .as_ref()
            .map(|a| a.raw.as_str())
            .unwrap_or("(no address)");
        let typ = node.data_type.clone().unwrap_or_else(|| "?".to_string());
        println!("  {:50} {} -> {}", node.name, typ, addr);
    }
    println!();

    // Try to read each test variable. Symbols are local to MAIN, so the path is "MAIN.<name>".
    let test_paths = [
        (format!("{}:MAIN.bRunning", args.port), "BOOL"),
        (format!("{}:MAIN.nCounter", args.port), "INT"),
        (format!("{}:MAIN.fSetPoint", args.port), "REAL"),
        (format!("{}:MAIN.sStatus", args.port), "STRING"),
    ];

    println!("=== Read each test variable ===");
    for (path, typ) in &test_paths {
        match driver.read(&TagAddress::new(path.as_str())).await {
            Ok(value) => println!("  {:30} ({}) = {:?}", path, typ, value),
            Err(err) => println!("  {:30} ({}) ERROR: {}", path, typ, err),
        }
    }
    println!();

    // Write fSetPoint
    let write_path = format!("{}:MAIN.fSetPoint", args.port);
    println!("=== Write {write_path} = 75.0 ===");
    let write_target = TagAddress::new(write_path);
    match driver.write(&write_target, TagValue::Real(75.0)).await {
        Ok(()) => {
            println!("write returned Ok.");
            // Read back to confirm
            tokio::time::sleep(Duration::from_millis(100)).await;
            match driver.read(&write_target).await {
                Ok(value) => println!("read-back: {:?}", value),
                Err(err) => println!("read-back ERROR: {}", err),
            }
        }
        Err(err) => println!("write ERROR: {}", err),
    }
    println!();

    // Subscribe to bRunning + nCounter for N seconds via native ADS notifications.
    println!(
        "=== Subscribe to bRunning + nCounter for {}s via native ADS notifications ===",
        args.subscribe_seconds
    );
    let addrs = vec![
        TagAddress::new(format!("{}:MAIN.bRunning", args.port)),
        TagAddress::new(format!("{}:MAIN.nCounter", args.port)),
    ];
    let mut stream = driver.subscribe(addrs).await.context("subscribe failed")?;

    let timeout = tokio::time::sleep(Duration::from_secs(args.subscribe_seconds));
    tokio::pin!(timeout);

    let mut count = 0u64;
    loop {
        tokio::select! {
            _ = &mut timeout => break,
            update = stream.next() => match update {
                Some(update) => {
                    println!(
                        "  [{}] {} = {:?} (q={:?})",
                        update.ts_ms, update.address.raw, update.value, update.quality,
                    );
                    count += 1;
                }
                None => {
                    println!("subscription stream ended early.");
                    break;
                }
            }
        }
    }
    println!(
        "received {count} notifications in {}s.\n",
        args.subscribe_seconds
    );

    drop(stream);

    println!("=== Disconnect ===");
    driver.disconnect().await?;
    println!("done.");

    Ok(())
}
