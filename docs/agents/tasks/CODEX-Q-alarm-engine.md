---
id: CODEX-Q
title: crates/alarm-engine — definitions, state machine, journal
owner: codex
phase: 3
status: submitted
created: 2026-04-27
last-update: 2026-04-28 12:38 codex
---

# CODEX-Q — `crates/alarm-engine`

## Brief

### Goal

Tag-based alarm engine. Each project declares alarms (per-tag conditions with priorities and messages). The engine subscribes to tag updates, runs a state machine (`Clear → Active → Acked → Cleared`), persists transitions to SQLite, and publishes alarm events to subscribed clients. The `AlarmTable` component (CODEX-R) renders these.

### Context to read first

- `docs/architecture.md` §4.5 (Alarm Engine).
- `docs/roadmap.md` Phase 3 — alarm deliverables.
- `crates/tag-engine/src/lib.rs` — `TagSnapshot` is the input.
- `crates/project-store/src/types.rs` — extend with `AlarmConfig`.

### Files to create / modify

- `crates/alarm-engine/Cargo.toml`
- `crates/alarm-engine/src/lib.rs` — re-exports.
- `crates/alarm-engine/src/types.rs` — `AlarmDefinition`, `AlarmState`, `AlarmEvent`.
- `crates/alarm-engine/src/conditions.rs` — `evaluate(condition, value) -> bool`.
- `crates/alarm-engine/src/engine.rs` — the state machine + persistence.
- `crates/alarm-engine/src/journal.rs` — SQLite writer for transitions.
- `crates/alarm-engine/tests/engine.rs` — state machine tests.
- `crates/protocol/src/lib.rs` — `alarm.subscribe`, `alarm.event`, `alarm.ack`.
- `packages/protocol-ts/src/index.ts` — mirror.
- `crates/project-store/src/types.rs` — `AlarmConfig`, save_artifact `Alarms` variant.

Add `crates/alarm-engine` to workspace `members`.

### Alarm definition

```rust
pub struct AlarmDefinition {
    pub id: String,
    pub label: String,
    pub priority: u8,                   // 1 = highest, 5 = lowest
    pub tag_path: String,
    pub condition: AlarmCondition,
    pub message: String,                // template with {value} substitution
    pub enabled: bool,
    pub require_ack: bool,              // false = auto-clears on condition clear
}

#[serde(tag = "kind", rename_all = "snake_case")]
pub enum AlarmCondition {
    HighLimit  { threshold: f64 },
    LowLimit   { threshold: f64 },
    Equals     { value: TagValue },
    Deviation  { setpoint: f64, tolerance: f64 },
    Digital    { active_when: bool },
}
```

### State machine

```
            ┌──────────────────────────────────────────┐
            │            (condition becomes true)      │
            ▼                                          │
        ┌─────────┐  ack    ┌────────────┐  clears     │
        │ ACTIVE  │────────▶│ ACKED      │─────────────┴──▶  CLEARED ──▶ (back to Clear)
        └─────────┘         └────────────┘
            │  clears (if require_ack=false)
            └──────────▶  CLEARED (auto)
```

Transitions write to `alarm_journal(alarm_id, ts_ms, from_state, to_state, who, note)`. Acks go through `who` (Phase 3 starts as `"anonymous"`; CODEX-S adds real user identity).

### Wire protocol

```rust
ClientMessage::AlarmSubscribe { project_id, priority_min, priority_max },
ClientMessage::AlarmAck       { alarm_id, note?: String },

ServerMessage::AlarmEvent {
    alarm_id, label, priority, state,
    tag_path, value, quality,
    activated_at_ms, transitioned_at_ms,
    who?: String, note?: String,
},
```

### Test requirements

- Each `AlarmCondition` variant evaluates correctly across boundary cases.
- State machine transitions: Active → Acked → Cleared; Active → Cleared (auto if `require_ack=false`).
- Re-entry: an alarm clears, then the condition becomes true again → new Active state, fresh `activated_at_ms`.
- Journal persistence round-trip.
- `priority_min`/`max` filtering on subscribe.
- Wire round-trip + literal form for `AlarmEvent` and `AlarmAck`.

### Acceptance criteria

- [ ] `cargo test -p openwebhmi-alarm-engine` green.
- [ ] Phase 1 demo extended with one alarm (`Pressure > 200` → priority 2 alarm).
- [ ] `cargo test --workspace --all-features --locked` stays green.
- [ ] All public items have rustdoc.

### Out of scope

- Notification pipelines (email/SMS/webhook) — Phase 4+.
- Alarm rosters / on-call schedules — post-1.0.
- Shelving — post-1.0.
- Alarm associated data beyond `value` + `quality` — post-1.0.

### Risks / gotchas

- **Edge detection vs level**: alarms fire on transitions, not on every sample where the condition is true. Track `was_active: bool` per alarm.
- **Hysteresis** for High/Low limits: a tag oscillating around the threshold flaps the alarm. v1 omits hysteresis; document the limitation. v2 adds an optional `clear_threshold`.
- **Tag-not-yet-published**: alarms can't fire before the engine sees a value. Don't block alarm subscription on tag presence.
- **Project hot-reload**: on alarms artifact change, recompute the active set; existing in-flight alarms keep their state if their definition still exists.

## Codex log

*(codex — append working notes here)*

### 2026-04-27 20:07  codex
Started. Taking CODEX-Q because it unblocks CODEX-R and completes the backend SCADA trio with historian and auth. Reading alarm brief, architecture alarm notes, project-store artifact shape, tag-engine snapshots, and gateway websocket routing before editing.

### 2026-04-28 12:36  codex
Submitted. Added `crates/alarm-engine` with typed definitions, condition evaluation, SQLite journal persistence, state-machine tests, and tag subscription runtime. Extended project-store with `alarms/alarms.json`, protocol Rust/TS with `alarm.subscribe`, `alarm.ack`, and `alarm.event`, gateway alarm runtime/hot-reload/event forwarding, and the Phase 1 demo `pressure-high` alarm. Verification: `cargo test -p openwebhmi-alarm-engine`; `cargo test --workspace --all-features --locked`; `cargo clippy --workspace --all-targets --all-features -- -D warnings`; `cargo fmt --all -- --check`; `pnpm -r typecheck`; `pnpm -r test`; `pnpm --filter @openwebhmi/protocol build`.

### 2026-04-28 12:38  codex
Submitted update. Tightened alarm hot-reload so same-path definition changes restart the affected tag worker, then added a regression test that lowers a threshold on an existing path and observes the new alarm behavior.

## Claude review

*(claude — after submission)*

## Verdict

*(claude — final disposition)*
