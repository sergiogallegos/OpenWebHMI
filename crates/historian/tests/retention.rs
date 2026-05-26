use openwebhmi_historian::{Aggregation, HistorianStore};
use openwebhmi_protocol::{Quality, TagValue};

#[test]
fn prune_to_max_rows_keeps_newest_samples() {
    let store = HistorianStore::memory().unwrap();
    for idx in 0..1000_u64 {
        store
            .write_sample(
                "driver/Pressure",
                idx,
                &TagValue::Real(idx as f64),
                Quality::Good,
            )
            .unwrap();
    }

    assert_eq!(
        store.prune_to_max_rows("driver/Pressure", 500).unwrap(),
        500
    );
    let points = store
        .read("driver/Pressure", 0, 1_000, Aggregation::Raw, 2_000)
        .unwrap();
    assert_eq!(points.len(), 500);
    assert_eq!(points.first().unwrap().ts_ms, 500);
    assert_eq!(points.last().unwrap().ts_ms, 999);
}

#[test]
fn prune_older_than_removes_samples_before_cutoff() {
    let store = HistorianStore::memory().unwrap();
    let day_ms = 24 * 60 * 60 * 1000;
    for day in 0..10_u64 {
        store
            .write_sample(
                "driver/Pressure",
                day * day_ms,
                &TagValue::Int(day as i64),
                Quality::Good,
            )
            .unwrap();
    }

    assert_eq!(
        store
            .prune_older_than("driver/Pressure", 5 * day_ms)
            .unwrap(),
        5
    );
    let points = store
        .read("driver/Pressure", 0, 10 * day_ms, Aggregation::Raw, 100)
        .unwrap();
    assert_eq!(points.len(), 5);
    assert_eq!(points.first().unwrap().ts_ms, 5 * day_ms);
}
