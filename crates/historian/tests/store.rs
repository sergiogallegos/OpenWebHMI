use std::time::Duration;

use openwebhmi_historian::{Aggregation, HistorianStore, HistoryTagConfig, spawn_recorder};
use openwebhmi_protocol::{Quality, TagValue};
use openwebhmi_tag_engine::TagStore;
use tokio::time::sleep;

#[test]
fn raw_round_trip_preserves_order_and_quality() {
    let store = HistorianStore::memory().unwrap();
    store
        .write_sample("a", 10, &TagValue::Real(1.0), Quality::Good)
        .unwrap();
    store
        .write_sample("a", 20, &TagValue::Real(2.0), Quality::Stale)
        .unwrap();

    let points = store.read("a", 0, 100, Aggregation::Raw, 10).unwrap();
    assert_eq!(points.len(), 2);
    assert_eq!(points[0].ts_ms, 10);
    assert_eq!(points[1].quality, Quality::Stale);
}

#[test]
fn aggregations_match_known_dataset() {
    let store = HistorianStore::memory().unwrap();
    for (ts, value) in [(0, 1.0), (10, 3.0), (20, 5.0), (30, 7.0)] {
        store
            .write_sample("a", ts, &TagValue::Real(value), Quality::Good)
            .unwrap();
    }

    assert_eq!(
        values(store.read("a", 0, 40, Aggregation::Avg, 2).unwrap()),
        vec![2.0, 6.0]
    );
    assert_eq!(
        values(store.read("a", 0, 40, Aggregation::Min, 2).unwrap()),
        vec![1.0, 5.0]
    );
    assert_eq!(
        values(store.read("a", 0, 40, Aggregation::Max, 2).unwrap()),
        vec![3.0, 7.0]
    );
    assert_eq!(
        values(store.read("a", 0, 40, Aggregation::Sum, 2).unwrap()),
        vec![4.0, 12.0]
    );
    assert_eq!(
        store
            .read("a", 0, 40, Aggregation::Count, 2)
            .unwrap()
            .into_iter()
            .map(|point| point.value)
            .collect::<Vec<_>>(),
        vec![TagValue::Int(2), TagValue::Int(2)]
    );
}

#[tokio::test]
async fn recorder_applies_deadband() {
    let tag_store = TagStore::new();
    let historian = HistorianStore::memory().unwrap();
    let handle = spawn_recorder(
        tag_store.clone(),
        historian.clone(),
        vec![HistoryTagConfig {
            path: "a".into(),
            rate_ms: 0,
            deadband: 0.1,
        }],
    );
    sleep(Duration::from_millis(10)).await;

    tag_store.publish("a", TagValue::Real(1.0), Quality::Good);
    sleep(Duration::from_millis(2)).await;
    tag_store.publish("a", TagValue::Real(1.0001), Quality::Good);
    sleep(Duration::from_millis(2)).await;
    tag_store.publish("a", TagValue::Real(1.5), Quality::Good);
    sleep(Duration::from_millis(50)).await;

    let points = historian
        .read("a", 0, i64::MAX as u64, Aggregation::Raw, 10)
        .unwrap();
    handle.abort();
    assert_eq!(values(points), vec![1.0, 1.5]);
}

#[tokio::test]
async fn recorder_applies_rate_limit() {
    let tag_store = TagStore::new();
    let historian = HistorianStore::memory().unwrap();
    let handle = spawn_recorder(
        tag_store.clone(),
        historian.clone(),
        vec![HistoryTagConfig {
            path: "a".into(),
            rate_ms: 1_000,
            deadband: 0.0,
        }],
    );
    sleep(Duration::from_millis(10)).await;

    for value in 0..10 {
        tag_store.publish("a", TagValue::Int(value), Quality::Good);
        sleep(Duration::from_millis(10)).await;
    }

    let points = historian
        .read("a", 0, i64::MAX as u64, Aggregation::Raw, 10)
        .unwrap();
    handle.abort();
    assert!(points.len() <= 2, "expected rate limit, got {points:?}");
}

#[tokio::test]
async fn recorder_hot_reload_removes_and_adds_tags() {
    let tag_store = TagStore::new();
    let historian = HistorianStore::memory().unwrap();
    let mut handle = spawn_recorder(
        tag_store.clone(),
        historian.clone(),
        vec![HistoryTagConfig {
            path: "a".into(),
            rate_ms: 0,
            deadband: 0.0,
        }],
    );
    sleep(Duration::from_millis(10)).await;

    tag_store.publish("a", TagValue::Real(1.0), Quality::Good);
    sleep(Duration::from_millis(5)).await;
    handle.update_configs(vec![HistoryTagConfig {
        path: "b".into(),
        rate_ms: 0,
        deadband: 0.0,
    }]);
    sleep(Duration::from_millis(10)).await;
    tag_store.publish("a", TagValue::Real(2.0), Quality::Good);
    tag_store.publish("b", TagValue::Real(3.0), Quality::Good);
    sleep(Duration::from_millis(20)).await;

    let a = historian
        .read("a", 0, i64::MAX as u64, Aggregation::Raw, 10)
        .unwrap();
    let b = historian
        .read("b", 0, i64::MAX as u64, Aggregation::Raw, 10)
        .unwrap();
    handle.abort();
    assert_eq!(values(a), vec![1.0]);
    assert_eq!(values(b), vec![3.0]);
}

#[tokio::test]
async fn recorder_hot_reload_restarts_changed_config() {
    let tag_store = TagStore::new();
    let historian = HistorianStore::memory().unwrap();
    let mut handle = spawn_recorder(
        tag_store.clone(),
        historian.clone(),
        vec![HistoryTagConfig {
            path: "a".into(),
            rate_ms: 1_000,
            deadband: 0.0,
        }],
    );
    sleep(Duration::from_millis(10)).await;

    tag_store.publish("a", TagValue::Real(1.0), Quality::Good);
    sleep(Duration::from_millis(5)).await;
    tag_store.publish("a", TagValue::Real(2.0), Quality::Good);
    sleep(Duration::from_millis(5)).await;
    handle.update_configs(vec![HistoryTagConfig {
        path: "a".into(),
        rate_ms: 0,
        deadband: 0.0,
    }]);
    sleep(Duration::from_millis(10)).await;
    tag_store.publish("a", TagValue::Real(3.0), Quality::Good);
    sleep(Duration::from_millis(20)).await;

    let points = historian
        .read("a", 0, i64::MAX as u64, Aggregation::Raw, 10)
        .unwrap();
    handle.abort();
    assert_eq!(values(points), vec![1.0, 2.0, 3.0]);
}

fn values(points: Vec<openwebhmi_historian::HistoryPoint>) -> Vec<f64> {
    points
        .into_iter()
        .map(|point| match point.value {
            TagValue::Real(value) => value,
            other => panic!("expected real value, got {other:?}"),
        })
        .collect()
}
