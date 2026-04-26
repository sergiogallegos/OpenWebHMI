//! In-memory tag store with publish/subscribe.
//!
//! Single source of truth for live tag values inside the gateway. See
//! `docs/architecture.md` §4.2.
//!
//! ## Subscription semantics
//!
//! Updates are delivered to subscribers via [`tokio::sync::broadcast`].
//! Lagging subscribers drop intermediate messages and receive the latest —
//! for HMI clients this is the desired report-by-exception behavior: the
//! current value matters, the history of changes does not.
//!
//! ## Subscribe-then-read pattern
//!
//! [`TagStore::subscribe`] returns a stream of *future* updates only. To
//! prime a UI with the current value, callers should call [`TagStore::get`]
//! immediately after subscribing. This deliberately keeps the subscription
//! channel as a pure event stream and avoids spurious replays to existing
//! subscribers when a new subscriber joins.

#![deny(missing_docs)]

use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::{SystemTime, UNIX_EPOCH};

use openwebhmi_protocol::{Quality, TagValue};
use tokio::sync::broadcast;

/// A snapshot of a tag's last-known state.
#[derive(Debug, Clone, PartialEq)]
pub struct TagSnapshot {
    /// Full tag path.
    pub path: String,
    /// Current value.
    pub value: TagValue,
    /// Current quality.
    pub quality: Quality,
    /// Unix epoch milliseconds.
    pub ts: u64,
}

/// Per-slot broadcast capacity. Lagging subscribers drop messages.
const BROADCAST_CAPACITY: usize = 1024;

struct TagSlot {
    last: Option<TagSnapshot>,
    tx: broadcast::Sender<TagSnapshot>,
}

/// In-memory store of tag snapshots with publish/subscribe.
///
/// Cheap to clone — the inner state is shared via `Arc`.
#[derive(Clone)]
pub struct TagStore {
    inner: Arc<RwLock<HashMap<String, TagSlot>>>,
}

impl TagStore {
    /// Create an empty store.
    pub fn new() -> Self {
        Self {
            inner: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Publish a new value for `path`. Existing subscribers receive a snapshot.
    ///
    /// If no slot exists yet for `path`, one is created. Sends to a slot with
    /// no subscribers are silently dropped — that's the broadcast contract.
    pub fn publish(&self, path: &str, value: TagValue, quality: Quality) {
        let snap = TagSnapshot {
            path: path.to_string(),
            value,
            quality,
            ts: now_ms(),
        };
        let mut guard = self.inner.write().expect("tag store rwlock poisoned");
        let slot = guard
            .entry(path.to_string())
            .or_insert_with(make_empty_slot);
        slot.last = Some(snap.clone());
        let _ = slot.tx.send(snap);
    }

    /// Subscribe to *future* updates for `path`.
    ///
    /// Does not replay the last-known value. Use [`get`](Self::get) to read
    /// the current snapshot after subscribing, if one is needed.
    pub fn subscribe(&self, path: &str) -> broadcast::Receiver<TagSnapshot> {
        let mut guard = self.inner.write().expect("tag store rwlock poisoned");
        let slot = guard
            .entry(path.to_string())
            .or_insert_with(make_empty_slot);
        slot.tx.subscribe()
    }

    /// Read the current snapshot for `path`, if any has been published.
    pub fn get(&self, path: &str) -> Option<TagSnapshot> {
        let guard = self.inner.read().expect("tag store rwlock poisoned");
        guard.get(path).and_then(|s| s.last.clone())
    }

    /// Number of paths with at least one published value or one subscriber.
    pub fn len(&self) -> usize {
        self.inner.read().expect("tag store rwlock poisoned").len()
    }

    /// Whether the store has no slots.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl Default for TagStore {
    fn default() -> Self {
        Self::new()
    }
}

fn make_empty_slot() -> TagSlot {
    let (tx, _) = broadcast::channel(BROADCAST_CAPACITY);
    TagSlot { last: None, tx }
}

fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::{timeout, Duration};

    #[test]
    fn new_store_is_empty() {
        let store = TagStore::new();
        assert!(store.is_empty());
        assert_eq!(store.len(), 0);
    }

    #[test]
    fn publish_then_get_returns_snapshot() {
        let store = TagStore::new();
        store.publish("a/b", TagValue::Real(1.0), Quality::Good);

        let snap = store.get("a/b").expect("snapshot should exist");
        assert_eq!(snap.path, "a/b");
        assert_eq!(snap.value, TagValue::Real(1.0));
        assert_eq!(snap.quality, Quality::Good);
        assert!(snap.ts > 0, "ts should be populated");
    }

    #[test]
    fn get_unknown_path_returns_none() {
        let store = TagStore::new();
        assert!(store.get("nope").is_none());
    }

    #[test]
    fn publish_overwrites_previous_value() {
        let store = TagStore::new();
        store.publish("a/b", TagValue::Int(1), Quality::Good);
        store.publish("a/b", TagValue::Int(2), Quality::Good);
        assert_eq!(store.get("a/b").unwrap().value, TagValue::Int(2));
    }

    #[tokio::test]
    async fn subscribe_then_publish_delivers_update() {
        let store = TagStore::new();
        let mut rx = store.subscribe("a/b");

        store.publish("a/b", TagValue::Int(7), Quality::Good);

        let snap = timeout(Duration::from_millis(100), rx.recv())
            .await
            .expect("recv should not time out")
            .expect("recv should not error");
        assert_eq!(snap.value, TagValue::Int(7));
    }

    #[tokio::test]
    async fn subscribe_after_publish_does_not_replay() {
        let store = TagStore::new();
        store.publish("a/b", TagValue::Bool(true), Quality::Good);

        let mut rx = store.subscribe("a/b");

        // Subscriptions are forward-only. The current value comes from get().
        assert!(
            rx.try_recv().is_err(),
            "no replay on subscribe — see module docs"
        );
        assert_eq!(store.get("a/b").unwrap().value, TagValue::Bool(true));
    }

    #[tokio::test]
    async fn multiple_subscribers_each_receive_updates() {
        let store = TagStore::new();
        let mut rx1 = store.subscribe("a/b");
        let mut rx2 = store.subscribe("a/b");

        store.publish("a/b", TagValue::Real(2.5), Quality::Good);

        let s1 = timeout(Duration::from_millis(100), rx1.recv())
            .await
            .unwrap()
            .unwrap();
        let s2 = timeout(Duration::from_millis(100), rx2.recv())
            .await
            .unwrap()
            .unwrap();
        assert_eq!(s1.value, TagValue::Real(2.5));
        assert_eq!(s2.value, TagValue::Real(2.5));
    }

    #[tokio::test]
    async fn paths_are_isolated() {
        let store = TagStore::new();
        let mut rx_a = store.subscribe("a");

        store.publish("b", TagValue::Int(1), Quality::Good);

        // Publishing to "b" must not wake up "a"'s subscriber.
        assert!(rx_a.try_recv().is_err());
    }

    #[tokio::test]
    async fn store_is_clone_and_shares_state() {
        let s1 = TagStore::new();
        let s2 = s1.clone();

        let mut rx = s2.subscribe("shared");
        s1.publish("shared", TagValue::String("hi".into()), Quality::Good);

        let snap = timeout(Duration::from_millis(100), rx.recv())
            .await
            .unwrap()
            .unwrap();
        assert_eq!(snap.value, TagValue::String("hi".into()));
        assert_eq!(
            s2.get("shared").unwrap().value,
            TagValue::String("hi".into())
        );
    }
}
