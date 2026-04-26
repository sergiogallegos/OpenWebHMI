//! Simulated Phase 0 tag provider.

use std::f64::consts::TAU;
use std::time::Duration;

use openwebhmi_protocol::{Quality, TagValue};
use openwebhmi_tag_engine::TagStore;
use tokio::time;

const SIN_PATH: &str = "system/sim/sin";
const COUNTER_PATH: &str = "system/sim/counter";

/// Publish a 60-second sine wave and an incrementing counter once per second.
pub async fn run(store: TagStore) {
    let mut tick = time::interval(Duration::from_secs(1));
    let mut counter = 0_i64;

    loop {
        tick.tick().await;

        let phase = (counter as f64).rem_euclid(60.0) / 60.0;
        let sin = (TAU * phase).sin();

        store.publish(SIN_PATH, TagValue::Real(sin), Quality::Good);
        store.publish(COUNTER_PATH, TagValue::Int(counter), Quality::Good);

        counter = counter.saturating_add(1);
    }
}
