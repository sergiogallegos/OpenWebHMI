use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, Mutex as StdMutex};

use async_trait::async_trait;
use rust_ethernet_ip::{EtherNetIpError, PlcValue, TagGroupEvent, TagGroupValueResult};

use crate::eip_client::{EipClientLike, EipResult, TagGroupSubscriptionLike};

#[derive(Clone, Debug, PartialEq)]
/// Mock upstream client call recorded by tests.
pub enum RecordedCall {
    /// Read call with tag name.
    Read(String),
    /// Write call with tag name and value.
    Write(String, PlcValue),
    /// Tag-group upsert call.
    UpsertGroup {
        /// Group name.
        name: String,
        /// Tags in the group.
        tags: Vec<String>,
        /// Update rate in milliseconds.
        update_rate_ms: u32,
    },
    /// Tag-group subscription call.
    SubscribeGroup(String),
}

#[derive(Clone, Default)]
/// Mock `EipClientLike` for wrapper tests.
pub struct MockEipClient {
    reads: Arc<StdMutex<VecDeque<EipResult<PlcValue>>>>,
    writes: Arc<StdMutex<VecDeque<EipResult<()>>>>,
    subscriptions: Arc<StdMutex<VecDeque<Vec<TagGroupEvent>>>>,
    values: Arc<StdMutex<HashMap<String, PlcValue>>>,
    calls: Arc<StdMutex<Vec<RecordedCall>>>,
}

impl MockEipClient {
    /// Build a mock with initial tag values.
    pub fn with_values(values: HashMap<String, PlcValue>) -> Self {
        Self {
            values: Arc::new(StdMutex::new(values)),
            ..Self::default()
        }
    }

    /// Queue a scripted read result.
    pub fn push_read(&self, result: EipResult<PlcValue>) {
        self.reads.lock().unwrap().push_back(result);
    }

    /// Queue a scripted write result.
    pub fn push_write(&self, result: EipResult<()>) {
        self.writes.lock().unwrap().push_back(result);
    }

    /// Queue events returned by the next subscription.
    pub fn push_subscription_events(&self, events: Vec<TagGroupEvent>) {
        self.subscriptions.lock().unwrap().push_back(events);
    }

    /// Return all recorded upstream calls.
    pub fn calls(&self) -> Vec<RecordedCall> {
        self.calls.lock().unwrap().clone()
    }
}

#[async_trait]
impl EipClientLike for MockEipClient {
    async fn read_tag(&mut self, tag_name: &str) -> EipResult<PlcValue> {
        self.calls
            .lock()
            .unwrap()
            .push(RecordedCall::Read(tag_name.to_string()));
        if let Some(result) = self.reads.lock().unwrap().pop_front() {
            return result;
        }

        self.values
            .lock()
            .unwrap()
            .get(tag_name)
            .cloned()
            .ok_or_else(|| EtherNetIpError::TagNotFound(tag_name.to_string()))
    }

    async fn write_tag(&mut self, tag_name: &str, value: PlcValue) -> EipResult<()> {
        self.calls
            .lock()
            .unwrap()
            .push(RecordedCall::Write(tag_name.to_string(), value.clone()));
        if let Some(result) = self.writes.lock().unwrap().pop_front() {
            return result;
        }

        self.values
            .lock()
            .unwrap()
            .insert(tag_name.to_string(), value);
        Ok(())
    }

    async fn upsert_tag_group(
        &mut self,
        group_name: &str,
        tags: &[&str],
        update_rate_ms: u32,
    ) -> EipResult<()> {
        self.calls.lock().unwrap().push(RecordedCall::UpsertGroup {
            name: group_name.to_string(),
            tags: tags.iter().map(|tag| (*tag).to_string()).collect(),
            update_rate_ms,
        });
        Ok(())
    }

    async fn subscribe_tag_group(
        &mut self,
        group_name: &str,
    ) -> EipResult<Box<dyn TagGroupSubscriptionLike>> {
        self.calls
            .lock()
            .unwrap()
            .push(RecordedCall::SubscribeGroup(group_name.to_string()));
        let events = self
            .subscriptions
            .lock()
            .unwrap()
            .pop_front()
            .unwrap_or_default();
        Ok(Box::new(MockSubscription {
            events: StdMutex::new(VecDeque::from(events)),
            stopped: StdMutex::new(false),
        }))
    }
}

struct MockSubscription {
    events: StdMutex<VecDeque<TagGroupEvent>>,
    stopped: StdMutex<bool>,
}

#[async_trait]
impl TagGroupSubscriptionLike for MockSubscription {
    async fn wait_for_update(&self) -> Option<TagGroupEvent> {
        self.events.lock().unwrap().pop_front()
    }

    fn stop(&self) {
        *self.stopped.lock().unwrap() = true;
    }
}

/// Build a data or partial-error event from value results.
pub fn data_event(values: Vec<TagGroupValueResult>) -> TagGroupEvent {
    TagGroupEvent {
        kind: if values.iter().any(|value| value.error.is_some()) {
            rust_ethernet_ip::TagGroupEventKind::PartialError
        } else {
            rust_ethernet_ip::TagGroupEventKind::Data
        },
        snapshot: rust_ethernet_ip::TagGroupSnapshot {
            group_name: "test".to_string(),
            sampled_at: std::time::SystemTime::now(),
            values,
        },
        error: None,
        failure: None,
    }
}
