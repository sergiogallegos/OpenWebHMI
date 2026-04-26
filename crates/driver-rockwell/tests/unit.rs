use std::collections::HashMap;
use std::time::SystemTime;

use futures_util::StreamExt;
use openwebhmi_driver_api::{Driver, DriverError, TagAddress};
use openwebhmi_driver_rockwell::test_support::{data_event, MockEipClient, RecordedCall};
use openwebhmi_driver_rockwell::{RockwellConfig, RockwellDriver};
use openwebhmi_protocol::{Quality, TagValue};
use rust_ethernet_ip::{
    EtherNetIpError, PlcValue, TagGroupEvent, TagGroupEventKind, TagGroupSnapshot,
    TagGroupValueResult,
};
use tokio::time::{timeout, Duration};

#[tokio::test]
async fn read_maps_real_value() {
    let mock = MockEipClient::with_values(HashMap::from([(
        "Pressure".to_string(),
        PlcValue::Real(1.5),
    )]));
    let driver = RockwellDriver::with_client(Box::new(mock), RockwellConfig::default());

    let value = driver.read(&TagAddress::new("Pressure")).await.unwrap();

    assert_eq!(value, TagValue::Real(1.5));
}

#[tokio::test]
async fn write_maps_each_openwebhmi_value_variant() {
    let mock = MockEipClient::default();
    mock.push_read(Ok(PlcValue::String("old".to_string())));
    let driver = RockwellDriver::with_client(Box::new(mock.clone()), RockwellConfig::default());

    driver
        .write(&TagAddress::new("BoolTag"), TagValue::Bool(true))
        .await
        .unwrap();
    driver
        .write(&TagAddress::new("DintTag"), TagValue::Int(42))
        .await
        .unwrap();
    driver
        .write(&TagAddress::new("RealTag"), TagValue::Real(1.25))
        .await
        .unwrap();
    driver
        .write(
            &TagAddress::new("StringTag"),
            TagValue::String("new".to_string()),
        )
        .await
        .unwrap();

    assert_eq!(
        mock.calls(),
        vec![
            RecordedCall::Write("BoolTag".to_string(), PlcValue::Bool(true)),
            RecordedCall::Write("DintTag".to_string(), PlcValue::Dint(42)),
            RecordedCall::Write("RealTag".to_string(), PlcValue::Real(1.25)),
            RecordedCall::Read("StringTag".to_string()),
            RecordedCall::Write("StringTag".to_string(), PlcValue::String("new".to_string())),
        ]
    );
}

#[tokio::test]
async fn string_write_uses_read_before_write_workaround_path() {
    let mock = MockEipClient::default();
    mock.push_read(Ok(PlcValue::String("old".to_string())));
    let driver = RockwellDriver::with_client(Box::new(mock.clone()), RockwellConfig::default());

    driver
        .write(
            &TagAddress::new("StatusText"),
            TagValue::String("ready".to_string()),
        )
        .await
        .unwrap();

    assert_eq!(
        mock.calls(),
        vec![
            RecordedCall::Read("StatusText".to_string()),
            RecordedCall::Write(
                "StatusText".to_string(),
                PlcValue::String("ready".to_string())
            ),
        ]
    );
}

#[tokio::test]
async fn no_such_tag_maps_to_invalid_address() {
    let mock = MockEipClient::default();
    mock.push_read(Err(EtherNetIpError::TagNotFound("Missing".to_string())));
    let driver = RockwellDriver::with_client(Box::new(mock), RockwellConfig::default());

    let error = driver.read(&TagAddress::new("Missing")).await.unwrap_err();

    assert!(matches!(error, DriverError::InvalidAddress(tag) if tag == "Missing"));
}

#[tokio::test]
async fn subscribe_maps_data_partial_error_and_read_failure_quality() {
    let mock = MockEipClient::default();
    mock.push_subscription_events(vec![
        data_event(vec![
            TagGroupValueResult {
                tag_name: "Pressure".to_string(),
                value: Some(PlcValue::Real(2.5)),
                error: None,
            },
            TagGroupValueResult {
                tag_name: "Missing".to_string(),
                value: None,
                error: Some("no such tag".to_string()),
            },
        ]),
        TagGroupEvent {
            kind: TagGroupEventKind::ReadFailure,
            snapshot: TagGroupSnapshot {
                group_name: "test".to_string(),
                sampled_at: SystemTime::now(),
                values: Vec::new(),
            },
            error: Some("connection lost".to_string()),
            failure: None,
        },
    ]);
    let driver = RockwellDriver::with_client(
        Box::new(mock),
        RockwellConfig {
            poll_rate_ms: 25,
            ..RockwellConfig::default()
        },
    );

    let mut stream = driver
        .subscribe(vec![
            TagAddress::new("Pressure"),
            TagAddress::new("Missing"),
        ])
        .await
        .unwrap();

    let first = next_update(&mut stream).await;
    let second = next_update(&mut stream).await;
    assert_eq!(first.quality, Quality::Good);
    assert_eq!(first.value, TagValue::Real(2.5));
    assert_eq!(second.quality, Quality::Bad);
    assert_eq!(second.address, TagAddress::new("Missing"));

    let third = next_update(&mut stream).await;
    let fourth = next_update(&mut stream).await;
    assert_eq!(third.quality, Quality::Bad);
    assert_eq!(fourth.quality, Quality::Bad);
}

async fn next_update(
    stream: &mut futures_util::stream::BoxStream<'static, openwebhmi_driver_api::DriverUpdate>,
) -> openwebhmi_driver_api::DriverUpdate {
    timeout(Duration::from_millis(100), stream.next())
        .await
        .expect("update should arrive")
        .expect("stream should stay open")
}
