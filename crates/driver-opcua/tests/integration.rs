#![cfg(feature = "sim-tests")]

use std::time::Duration;

use futures_util::StreamExt;
use openwebhmi_driver_api::{Driver, TagAddress};
use openwebhmi_driver_opcua::OpcUaDriver;
use openwebhmi_protocol::{Quality, TagValue};
use tokio::time::timeout;

#[tokio::test]
async fn sim_reads_browses_writes_and_subscribes() {
    let sim = sim_opcua::start_ephemeral().await.unwrap();
    let mut driver = OpcUaDriver::new();
    driver
        .connect(serde_json::json!({
            "endpoint": sim.endpoint(),
            "sampling_interval_ms": 100,
            "pki_dir": "./target/opcua-driver-test-pki"
        }))
        .await
        .unwrap();
    let pressure = format!("ns={};s=Pressure", sim.namespace_index());

    assert_eq!(
        driver.read(&TagAddress::new(&pressure)).await.unwrap(),
        TagValue::Real(12.5)
    );
    let tree = driver.browse(None).await.unwrap();
    assert!(
        tree.iter().any(|node| node.name == "Pressure"),
        "browse should expose Pressure: {tree:?}"
    );

    let mut stream = driver
        .subscribe(vec![TagAddress::new(&pressure)])
        .await
        .unwrap();
    sim.set_pressure(42.25).unwrap();
    let update = timeout(Duration::from_secs(2), stream.next())
        .await
        .unwrap()
        .unwrap();
    assert_eq!(update.quality, Quality::Good);

    driver
        .write(&TagAddress::new(&pressure), TagValue::Real(7.5))
        .await
        .unwrap();
    assert_eq!(
        driver.read(&TagAddress::new(&pressure)).await.unwrap(),
        TagValue::Real(7.5)
    );
    sim.stop();
}
