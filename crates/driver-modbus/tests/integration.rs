#![cfg(feature = "sim-tests")]

use std::time::Duration;

use futures_util::StreamExt;
use openwebhmi_driver_api::{Driver, TagAddress};
use openwebhmi_driver_modbus::ModbusDriver;
use openwebhmi_protocol::{Quality, TagValue};
use tokio::time::timeout;

#[tokio::test]
async fn sim_reads_writes_and_subscribes() {
    let (listener, addr) = sim_modbus::bind_ephemeral().await.unwrap();
    let state = sim_modbus::default_state();
    let server_state = state.clone();
    let server = tokio::spawn(async move {
        let _ = sim_modbus::serve(listener, server_state).await;
    });

    let mut driver = ModbusDriver::new();
    driver
        .connect(serde_json::json!({
            "transport": "tcp",
            "host": addr.ip().to_string(),
            "port": addr.port(),
            "poll_rate_ms": 50,
            "connection_timeout_ms": 1000
        }))
        .await
        .unwrap();

    assert_eq!(
        driver
            .read(&TagAddress::new("1/holding/100:2:f32"))
            .await
            .unwrap(),
        TagValue::Real(12.5)
    );
    assert_eq!(
        driver.read(&TagAddress::new("1/coils/0")).await.unwrap(),
        TagValue::Bool(true)
    );

    driver
        .write(&TagAddress::new("1/coils/1"), TagValue::Bool(true))
        .await
        .unwrap();
    timeout(Duration::from_secs(1), state.lock().await.wait_for_write())
        .await
        .ok();
    assert_eq!(state.lock().await.coil(1).await, Some(true));

    let mut stream = driver
        .subscribe(vec![
            TagAddress::new("1/holding/100:2:f32"),
            TagAddress::new("1/coils/1"),
        ])
        .await
        .unwrap();
    let first = timeout(Duration::from_secs(1), stream.next())
        .await
        .unwrap()
        .unwrap();
    let second = timeout(Duration::from_secs(1), stream.next())
        .await
        .unwrap()
        .unwrap();
    assert_eq!(first.quality, Quality::Good);
    assert_eq!(second.quality, Quality::Good);

    server.abort();
}
