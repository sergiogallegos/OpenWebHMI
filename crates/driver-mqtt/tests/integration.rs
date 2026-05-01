#![cfg(feature = "sim-tests")]

use std::time::Duration;

use futures_util::StreamExt;
use openwebhmi_driver_api::{Driver, TagAddress};
use openwebhmi_driver_mqtt::MqttDriver;
use openwebhmi_protocol::TagValue;
use tokio::time::timeout;

#[tokio::test]
async fn sim_publishes_generic_and_sparkplug_updates() {
    let sim = sim_mqtt::start_ephemeral().await.unwrap();
    let mut driver = MqttDriver::new();
    driver
        .connect(serde_json::json!({
            "host": sim.host(),
            "port": sim.port(),
            "client_id": "openwebhmi-mqtt-test",
            "topics": [
                {
                    "address": "factory/line1/temperature",
                    "payload_type": "raw_float_be"
                },
                {
                    "address": "spB/v1.0/group/DDATA/edge/device/Pressure",
                    "payload_type": "sparkplug_metric"
                }
            ]
        }))
        .await
        .unwrap();
    let mut stream = driver
        .subscribe(vec![
            TagAddress::new("factory/line1/temperature"),
            TagAddress::new("spB/v1.0/group/DDATA/edge/device/Pressure"),
        ])
        .await
        .unwrap();

    sim.publish_once().await.unwrap();
    let first = timeout(Duration::from_secs(3), stream.next())
        .await
        .unwrap()
        .unwrap();
    let second = timeout(Duration::from_secs(3), stream.next())
        .await
        .unwrap()
        .unwrap();
    let mut values = vec![
        (first.address.raw.clone(), first.value.clone()),
        (second.address.raw.clone(), second.value.clone()),
    ];
    values.sort_by(|a, b| a.0.cmp(&b.0));
    assert_eq!(
        values,
        vec![
            (
                "factory/line1/temperature".to_string(),
                TagValue::Real(12.5)
            ),
            (
                "spB/v1.0/group/DDATA/edge/device/Pressure".to_string(),
                TagValue::Real(42.25)
            ),
        ]
    );
}
