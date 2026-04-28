---
id: CODEX-Q
title: crates/alarm-engine — definitions, state machine, journal
owner: codex
phase: 3
status: merged
created: 2026-04-27
last-update: 2026-04-28 claude
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

### 2026-04-28  claude — review pass 1

Spec-compliant. State machine, conditions, journal, and hot-reload all match the brief.

Strong points:
- ✅ **All 5 condition variants present** in `conditions.rs:8-22`: `HighLimit`, `LowLimit`, `Equals`, `Deviation`, `Digital`. Numeric type-coercion via `f64`; non-numeric input on numeric conditions returns `ConditionError::ExpectedNumeric`. Digital on non-bool returns `ExpectedBool`.
- ✅ **Boundary semantics: strict `>` and `<`**. `HighLimit{threshold:10}` against `10.0` does not fire; against `10.1` fires. Documented by test. The brief didn't pin strict vs. inclusive — strict is the right call (alarms shouldn't fire on the exact setpoint).
- ✅ **State machine path**: `Active → Acked → Cleared` (test `state_machine_active_acked_cleared_and_journaled`) and `Active → Cleared` auto-path when `require_ack=false` (test `state_machine_auto_clears_when_ack_not_required`).
- ✅ **Re-entry test** verifies that a fresh activation after clear produces a new `activated_at_ms`. This is the gotcha I flagged in the brief — Codex caught it.
- ✅ **Journal persistence to SQLite** with `(alarm_id, ts_ms, from_state, to_state, who, note)` rows, retrievable by `read_alarm`.
- ✅ **Hot-reload restart on same-path definition change** with a regression test that lowers a threshold on an existing path and observes the new behavior — this is the issue Codex caught and fixed before submitting. Disciplined.
- ✅ **`AlarmEvent` carries `activated_at_ms` + `transitioned_at_ms`** as separate fields per the brief, so clients can tell "when did this episode start" from "when did this state happen".
- ✅ **Edge-detection** (transitions, not level) — tests verify only one Active event per activation, not one per sample.
- ✅ Phase 1 demo extended with `pressure-high` priority-2 alarm at `rockwell-1/Pressure > 200`.
- ✅ Wire forms (`alarm.subscribe`, `alarm.event`, `alarm.ack`) match the brief; gateway forwarding + ack handling shipped in server.rs (closes the diagnostic from earlier).

Findings:

- 🟡 **No hysteresis** on `HighLimit`/`LowLimit` — a tag oscillating around the threshold flaps the alarm. Brief documented this as v2 (`clear_threshold` field). Track for v1.1.
- 🟡 **`who` defaults to `"anonymous"` in the brief** but with CODEX-S now merged, the gateway has real session identity. CODEX-R (the AlarmTable + UI task) should plumb the verified session's username as the `who` value when ack'ing — make sure that's noted in CODEX-R's review.
- 🟡 **`Equals` against floats has IEEE-754 precision risk**. Comparing `Real(0.1 + 0.2)` to `Real(0.3)` would surprise users. Documented limitation; users who need fuzzy-equality should use `Deviation` with a small tolerance. Consider a `relative_tolerance` field in v1.1 if this surfaces.
- 🟡 **Numeric conversion `i64 → f64`** loses precision at magnitudes > 2^53. Practical issue is rare for industrial tags but worth noting if anyone configures alarms on counter tags.
- 🟢 The `ActiveAlarm` snapshot retained by the engine includes `value` + `quality` of the last sample. Lets a late-subscribing client get the current state without replaying transitions.

Acceptance criteria all four checkboxes verified.

## Verdict

**Merged** at the next commit. Closes the SCADA backend trio: **historian + auth + alarms** are now all in. CODEX-R (AlarmTable + designer alarm config) is unblocked — when it lands, the demo HMI will raise + display + ack the `pressure-high` alarm end-to-end.

Three Phase 3 polish items now tracked across the alarm code (no hysteresis, `who` plumbing in CODEX-R, float-equality fuzziness). Together with CODEX-O's Quality serialization and CODEX-S's secret-length check, that's the v1.1 hardening backlog forming.
