use std::net::TcpListener;
use std::process::Stdio;
use std::time::Duration;

use rust_ethernet_ip::{EipClient, PlcValue, TagGroupEventKind};
use tokio::process::{Child, Command};
use tokio::time::{sleep, timeout};

#[tokio::test]
async fn rust_ethernet_ip_client_reads_writes_and_subscribes() {
    let port = unused_port();
    let mut child = spawn_sim(port).await;

    let result = async {
        let mut client = connect_with_retry(port).await?;

        let counter = client.read_tag("Counter").await?;
        assert!(matches!(counter, PlcValue::Dint(_)));

        client.write_tag("Setpoint", PlcValue::Real(42.5)).await?;
        assert_eq!(client.read_tag("Setpoint").await?, PlcValue::Real(42.5));

        client
            .upsert_tag_group("phase1", &["Pressure", "Heartbeat"], 100)
            .await?;
        let sub = client.subscribe_tag_group("phase1").await?;
        let event = timeout(Duration::from_secs(2), sub.wait_for_update())
            .await
            .expect("tag group update should arrive")
            .expect("subscription should remain active");
        assert_eq!(event.kind, TagGroupEventKind::Data, "{event:#?}");

        Ok::<(), Box<dyn std::error::Error>>(())
    }
    .await;

    let _ = child.kill().await;
    result.unwrap();
}

fn unused_port() -> u16 {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind ephemeral port");
    listener.local_addr().expect("local addr").port()
}

async fn spawn_sim(port: u16) -> Child {
    let exe = env!("CARGO_BIN_EXE_sim-rockwell");
    Command::new(exe)
        .arg("--bind")
        .arg("127.0.0.1")
        .arg("--port")
        .arg(port.to_string())
        .arg("--config")
        .arg("does-not-exist.toml")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("spawn simulator")
}

async fn connect_with_retry(port: u16) -> Result<EipClient, Box<dyn std::error::Error>> {
    let address = format!("127.0.0.1:{port}");
    let deadline = tokio::time::Instant::now() + Duration::from_secs(3);
    loop {
        match EipClient::connect(&address).await {
            Ok(client) => return Ok(client),
            Err(err) if tokio::time::Instant::now() < deadline => {
                let _ = err;
                sleep(Duration::from_millis(50)).await;
            }
            Err(err) => return Err(Box::new(err)),
        }
    }
}
