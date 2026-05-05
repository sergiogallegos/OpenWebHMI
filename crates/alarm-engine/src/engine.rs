//! Alarm state machine and tag subscription runtime.

use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};

use openwebhmi_protocol::{Quality, TagPath, TagValue};
use openwebhmi_tag_engine::{TagSnapshot, TagStore};
use tokio::sync::broadcast;
use tokio::task::AbortHandle;
use tracing::warn;

use crate::conditions::evaluate;
use crate::journal::AlarmJournal;
use crate::types::{ActiveAlarm, AlarmDefinition, AlarmEvent, AlarmState, AlarmTransition};

/// Alarm engine handle.
pub struct AlarmEngineHandle {
    tag_store: TagStore,
    journal: AlarmJournal,
    events: broadcast::Sender<AlarmEvent>,
    handles: HashMap<TagPath, AbortHandle>,
    definitions: HashMap<String, AlarmDefinition>,
    acks: Arc<Mutex<HashMap<String, AckRequest>>>,
}

impl AlarmEngineHandle {
    /// Subscribe to alarm events.
    pub fn subscribe_events(&self) -> broadcast::Receiver<AlarmEvent> {
        self.events.subscribe()
    }

    /// Replace configured alarms.
    pub fn update_definitions(&mut self, definitions: Vec<AlarmDefinition>) {
        let next = definitions_by_id(definitions);
        let existing_by_path = definitions_for_path(&self.definitions);
        let next_by_path = definitions_for_path(&next);
        let paths = existing_by_path
            .keys()
            .chain(next_by_path.keys())
            .cloned()
            .collect::<HashSet<_>>();
        self.definitions = next;
        for path in paths {
            if existing_by_path.get(&path) == next_by_path.get(&path) {
                continue;
            }
            if let Some(handle) = self.handles.remove(&path) {
                handle.abort();
            }
            if next_by_path.contains_key(&path) {
                self.handles.insert(
                    path.clone(),
                    spawn_path(
                        path,
                        self.tag_store.clone(),
                        self.journal.clone(),
                        self.events.clone(),
                        definitions_for_path(&self.definitions),
                        self.acks.clone(),
                    ),
                );
            }
        }
    }

    /// Acknowledge an alarm by id.
    ///
    /// This emits an ack event for subscribers. Path tasks keep the condition
    /// edge state and write clear transitions when the tag later clears.
    pub fn ack(&self, alarm_id: &str, who: &str, note: Option<String>) {
        if let Some(definition) = self.definitions.get(alarm_id) {
            let ts_ms = now_ms();
            if let Ok(mut acks) = self.acks.lock() {
                acks.insert(alarm_id.to_string(), AckRequest { ts_ms });
            }
            let transition = AlarmTransition {
                alarm_id: definition.id.clone(),
                ts_ms,
                from_state: AlarmState::Active,
                to_state: AlarmState::Acked,
                who: Some(who.to_string()),
                note: note.clone(),
            };
            if let Err(err) = self.journal.write_transition(&transition) {
                warn!(%alarm_id, error = %err, "failed to write alarm ack transition");
            }
            let event = AlarmEvent {
                alarm_id: definition.id.clone(),
                label: definition.label.clone(),
                priority: definition.priority,
                state: AlarmState::Acked,
                tag_path: definition.tag_path.clone(),
                value: TagValue::String("ack".into()),
                quality: Quality::Good,
                activated_at_ms: None,
                transitioned_at_ms: ts_ms,
                who: Some(who.to_string()),
                note,
                message: definition.message.clone(),
            };
            let _ = self.events.send(event);
        }
    }

    /// Stop all subscription tasks.
    pub fn abort(&mut self) {
        for (_, handle) in self.handles.drain() {
            handle.abort();
        }
    }
}

/// Spawn an alarm engine.
pub fn spawn_alarm_engine(
    tag_store: TagStore,
    journal: AlarmJournal,
    definitions: Vec<AlarmDefinition>,
) -> AlarmEngineHandle {
    let definitions = definitions_by_id(definitions);
    let (events, _) = broadcast::channel(1024);
    let acks = Arc::new(Mutex::new(HashMap::new()));
    let handles = tag_paths(&definitions)
        .into_iter()
        .map(|path| {
            let handle = spawn_path(
                path.clone(),
                tag_store.clone(),
                journal.clone(),
                events.clone(),
                definitions_for_path(&definitions),
                acks.clone(),
            );
            (path, handle)
        })
        .collect();
    AlarmEngineHandle {
        tag_store,
        journal,
        events,
        handles,
        definitions,
        acks,
    }
}

fn spawn_path(
    path: TagPath,
    tag_store: TagStore,
    journal: AlarmJournal,
    events: broadcast::Sender<AlarmEvent>,
    definitions: HashMap<TagPath, Vec<AlarmDefinition>>,
    acks: Arc<Mutex<HashMap<String, AckRequest>>>,
) -> AbortHandle {
    let handle = tokio::spawn(async move {
        let mut rx = tag_store.subscribe(&path);
        let mut states = AlarmRuntime::default();
        if let Some(snapshot) = tag_store.get(&path) {
            states
                .evaluate_snapshot(&snapshot, &definitions, &journal, &events, &acks)
                .await;
        }
        loop {
            match rx.recv().await {
                Ok(snapshot) => {
                    states
                        .evaluate_snapshot(&snapshot, &definitions, &journal, &events, &acks)
                        .await
                }
                Err(tokio::sync::broadcast::error::RecvError::Lagged(skipped)) => {
                    warn!(%path, skipped, "alarm engine lagged");
                }
                Err(tokio::sync::broadcast::error::RecvError::Closed) => break,
            }
        }
    });
    // Dropping a JoinHandle does not cancel its task; AbortHandle is the cancel-only handle.
    handle.abort_handle()
}

#[derive(Default)]
struct AlarmRuntime {
    active: HashMap<String, ActiveAlarm>,
}

#[derive(Debug, Clone)]
struct AckRequest {
    ts_ms: u64,
}

impl AlarmRuntime {
    async fn evaluate_snapshot(
        &mut self,
        snapshot: &TagSnapshot,
        definitions: &HashMap<TagPath, Vec<AlarmDefinition>>,
        journal: &AlarmJournal,
        events: &broadcast::Sender<AlarmEvent>,
        acks: &Arc<Mutex<HashMap<String, AckRequest>>>,
    ) {
        let Some(definitions) = definitions.get(&snapshot.path) else {
            return;
        };
        for definition in definitions.iter().filter(|definition| definition.enabled) {
            let condition_active =
                evaluate(&definition.condition, &snapshot.value).unwrap_or(false);
            if let Some(ack) = take_ack(acks, &definition.id) {
                self.mark_acked(definition, snapshot, ack);
            }
            self.apply(definition, snapshot, condition_active, journal, events)
                .await;
        }
    }

    fn mark_acked(
        &mut self,
        definition: &AlarmDefinition,
        snapshot: &TagSnapshot,
        ack: AckRequest,
    ) {
        if let Some(active) = self.active.get_mut(&definition.id) {
            if active.state == AlarmState::Active {
                active.state = AlarmState::Acked;
                active.transitioned_at_ms = ack.ts_ms;
                active.value = snapshot.value.clone();
                active.quality = snapshot.quality;
            }
        }
    }

    async fn apply(
        &mut self,
        definition: &AlarmDefinition,
        snapshot: &TagSnapshot,
        condition_active: bool,
        journal: &AlarmJournal,
        events: &broadcast::Sender<AlarmEvent>,
    ) {
        let current = self
            .active
            .get(&definition.id)
            .map(|active| active.state)
            .unwrap_or(AlarmState::Clear);
        match (current, condition_active, definition.require_ack) {
            (AlarmState::Clear | AlarmState::Cleared, true, _) => {
                self.transition(definition, snapshot, AlarmState::Active, journal, events)
                    .await;
            }
            (AlarmState::Active, false, false) | (AlarmState::Acked, false, _) => {
                self.transition(definition, snapshot, AlarmState::Cleared, journal, events)
                    .await;
                self.active.remove(&definition.id);
            }
            (AlarmState::Active, false, true) => {
                self.update_value(definition, snapshot);
            }
            _ => self.update_value(definition, snapshot),
        }
    }

    async fn transition(
        &mut self,
        definition: &AlarmDefinition,
        snapshot: &TagSnapshot,
        to_state: AlarmState,
        journal: &AlarmJournal,
        events: &broadcast::Sender<AlarmEvent>,
    ) {
        let from_state = self
            .active
            .get(&definition.id)
            .map(|active| active.state)
            .unwrap_or(AlarmState::Clear);
        let activated_at_ms = if to_state == AlarmState::Active {
            Some(snapshot.ts)
        } else {
            self.active
                .get(&definition.id)
                .and_then(|active| active.activated_at_ms)
        };
        let transition = AlarmTransition {
            alarm_id: definition.id.clone(),
            ts_ms: snapshot.ts,
            from_state,
            to_state,
            who: None,
            note: None,
        };
        let journal = journal.clone();
        let write_transition = transition.clone();
        let result =
            tokio::task::spawn_blocking(move || journal.write_transition(&write_transition)).await;
        let result = match result {
            Ok(result) => result,
            Err(err) => {
                warn!(alarm_id = %definition.id, error = %err, "alarm journal write task failed");
                Ok(())
            }
        };
        if let Err(err) = result {
            warn!(alarm_id = %definition.id, error = %err, "failed to write alarm journal transition");
        }
        let event = AlarmEvent {
            alarm_id: definition.id.clone(),
            label: definition.label.clone(),
            priority: definition.priority,
            state: to_state,
            tag_path: definition.tag_path.clone(),
            value: snapshot.value.clone(),
            quality: snapshot.quality,
            activated_at_ms,
            transitioned_at_ms: snapshot.ts,
            who: None,
            note: None,
            message: render_message(&definition.message, &snapshot.value),
        };
        if to_state != AlarmState::Cleared {
            self.active.insert(
                definition.id.clone(),
                ActiveAlarm {
                    alarm_id: definition.id.clone(),
                    state: to_state,
                    activated_at_ms,
                    transitioned_at_ms: snapshot.ts,
                    value: snapshot.value.clone(),
                    quality: snapshot.quality,
                },
            );
        }
        let _ = events.send(event);
    }

    fn update_value(&mut self, definition: &AlarmDefinition, snapshot: &TagSnapshot) {
        if let Some(active) = self.active.get_mut(&definition.id) {
            active.value = snapshot.value.clone();
            active.quality = snapshot.quality;
        }
    }
}

fn render_message(template: &str, value: &TagValue) -> String {
    template.replace("{value}", &format_tag_value(value))
}

fn format_tag_value(value: &TagValue) -> String {
    match value {
        TagValue::Bool(value) => value.to_string(),
        TagValue::Int(value) => value.to_string(),
        TagValue::Real(value) => value.to_string(),
        TagValue::String(value) => value.clone(),
    }
}

fn definitions_by_id(definitions: Vec<AlarmDefinition>) -> HashMap<String, AlarmDefinition> {
    definitions
        .into_iter()
        .map(|definition| (definition.id.clone(), definition))
        .collect()
}

fn definitions_for_path(
    definitions: &HashMap<String, AlarmDefinition>,
) -> HashMap<TagPath, Vec<AlarmDefinition>> {
    let mut by_path = HashMap::<TagPath, Vec<AlarmDefinition>>::new();
    for definition in definitions.values() {
        by_path
            .entry(definition.tag_path.clone())
            .or_default()
            .push(definition.clone());
    }
    by_path
}

fn tag_paths(definitions: &HashMap<String, AlarmDefinition>) -> HashSet<TagPath> {
    definitions
        .values()
        .map(|definition| definition.tag_path.clone())
        .collect()
}

fn take_ack(acks: &Arc<Mutex<HashMap<String, AckRequest>>>, alarm_id: &str) -> Option<AckRequest> {
    acks.lock().ok()?.remove(alarm_id)
}

fn now_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_millis() as u64)
        .unwrap_or(0)
}
