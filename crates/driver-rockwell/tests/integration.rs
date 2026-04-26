#![cfg(feature = "sim-tests")]

use std::net::TcpListener;
use std::path::PathBuf;
use std::process::Stdio;
use std::time::Duration;

use futures_util::StreamExt;
use openwebhmi_driver_api::{Driver, TagAddress};
use openwebhmi_driver_rockwell::RockwellDriver;
use openwebhmi_protocol::{Quality, TagValue};
use tokio::process::{Child, Command};
use tokio::time::{sleep, timeout};

#[tokio::test]
async fn sim_reads_writes_and_subscribes() {
    if std::env::var("OPENWEBHMI_SIM_RUNNING").ok().as_deref() != Some("1") {
        eprintln!("skipping sim test; set OPENWEBHMI_SIM_RUNNING=1");
        return;
    }

    let port = unused_port();
    let mut sim = spawn_sim(port).await;

    let result = async {
        let mut driver = connect_driver(port).await;

        let first = read_counter(&driver).await;
        sleep(Duration::from_millis(200)).await;
        let second = read_counter(&driver).await;
        assert!(second > first, "counter should increment across reads");

        driver
            .write(&TagAddress::new("Setpoint"), TagValue::Real(42.5))
            .await
            .unwrap();
        assert_eq!(
            driver.read(&TagAddress::new("Setpoint")).await.unwrap(),
            TagValue::Real(42.5)
        );

        let mut stream = driver
            .subscribe(vec![
                TagAddress::new("Pressure"),
                TagAddress::new("Heartbeat"),
            ])
            .await
            .unwrap();
        let mut good_updates = 0;
        let deadline = tokio::time::Instant::now() + Duration::from_secs(2);
        while good_updates < 3 && tokio::time::Instant::now() < deadline {
            if timeout(Duration::from_millis(750), stream.next())
                .await
                .unwrap()
                .unwrap()
                .quality
                == Quality::Good
            {
                good_updates += 1;
            }
        }
        assert!(
            good_updates >= 3,
            "expected at least 3 good subscription updates"
        );

        let _ = sim.kill().await;
        let deadline = tokio::time::Instant::now() + Duration::from_secs(3);
        let mut saw_bad = false;
        while tokio::time::Instant::now() < deadline {
            let update = timeout(Duration::from_millis(750), stream.next())
                .await
                .expect("read failure should produce updates")
                .expect("stream should stay open");
            if update.quality == Quality::Bad {
                saw_bad = true;
                break;
            }
        }
        assert!(saw_bad, "expected a Bad update after simulator shutdown");

        sim = spawn_sim(port).await;
        driver.disconnect().await.unwrap();
        driver
            .connect(config(port))
            .await
            .expect("driver should reconnect after simulator restart");
        let mut restored = driver
            .subscribe(vec![TagAddress::new("Pressure")])
            .await
            .expect("subscription should restart after reconnect");
        let restored_update = timeout(Duration::from_secs(5), restored.next())
            .await
            .expect("good update should return after restart")
            .expect("restored stream should stay open");
        assert_eq!(restored_update.quality, Quality::Good);

        Ok::<(), Box<dyn std::error::Error>>(())
    }
    .await;

    let _ = sim.kill().await;
    result.unwrap();
}

fn unused_port() -> u16 {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind ephemeral port");
    listener.local_addr().expect("local addr").port()
}

async fn connect_driver(port: u16) -> RockwellDriver {
    let deadline = tokio::time::Instant::now() + Duration::from_secs(3);
    loop {
        let mut driver = RockwellDriver::new();
        match driver.connect(config(port)).await {
            Ok(()) => return driver,
            Err(err) if tokio::time::Instant::now() < deadline => {
                let _ = err;
                sleep(Duration::from_millis(50)).await;
            }
            Err(err) => panic!("driver should connect to simulator: {err}"),
        }
    }
}

fn config(port: u16) -> serde_json::Value {
    serde_json::json!({
        "host": format!("127.0.0.1:{port}"),
        "slot": 0,
        "poll_rate_ms": 100,
        "connection_timeout_ms": 1000
    })
}

async fn read_counter(driver: &RockwellDriver) -> i64 {
    match driver.read(&TagAddress::new("Counter")).await.unwrap() {
        TagValue::Int(value) => value,
        other => panic!("expected Counter int, got {other:?}"),
    }
}

async fn spawn_sim(port: u16) -> Child {
    let mut command = if let Some(exe) = sim_binary_path() {
        Command::new(exe)
    } else {
        let mut command = Command::new(std::env::var("CARGO").unwrap_or_else(|_| "cargo".into()));
        command.args(["run", "-p", "sim-rockwell", "--quiet", "--"]);
        command
    };

    command
        .arg("--bind")
        .arg("127.0.0.1")
        .arg("--port")
        .arg(port.to_string())
        .arg("--config")
        .arg("does-not-exist.toml")
        .stdout(Stdio::null())
        .stderr(Stdio::null());

    let child = command.spawn().expect("spawn simulator");
    wait_for_port(port).await;
    child
}

fn sim_binary_path() -> Option<PathBuf> {
    let mut path = std::env::current_exe().ok()?;
    path.pop();
    if path.file_name().and_then(|name| name.to_str()) == Some("deps") {
        path.pop();
    }
    path.push("sim-rockwell");
    path.exists().then_some(path)
}

async fn wait_for_port(port: u16) {
    let deadline = tokio::time::Instant::now() + Duration::from_secs(5);
    while tokio::time::Instant::now() < deadline {
        if tokio::net::TcpStream::connect(("127.0.0.1", port))
            .await
            .is_ok()
        {
            return;
        }
        sleep(Duration::from_millis(50)).await;
    }
    panic!("simulator did not listen on port {port}");
}
