---
id: CODEX-CF
title: Alarm ack state-machine fix — ack re-evaluates, no stuck-active on a stalled tag, no spurious transitions
owner: codex
phase: 4
status: open
created: 2026-07-12
last-update: 2026-07-12 claude [Fable 5]
---

# CODEX-CF — Alarm ack state-machine fix

## Brief

> `AlarmEngineHandle::ack` journals an `Acked` transition and populates the `acks` map, but the in-memory `AlarmRuntime.active` state only advances on the *next* tag snapshot (`take_ack` → `apply`). `ack()` itself does not drive the state machine. Consequence: a `require_ack` alarm goes Active, the tag returns to normal (latched Active), the operator acks, then the tag stops updating — no snapshot arrives, `apply` never runs, and the alarm stays Active in engine memory forever, diverging from the `Acked` event already sent to clients. Ignition and FactoryTalk both clear a normalized-and-acked alarm immediately on ack without waiting for another tag scan; OpenWebHMI must too. Additionally, `ack()` blindly journals `from_state: Active` even for a Clear/Cleared alarm (writing a spurious transition), and the emitted event hardcodes `activated_at_ms: None`, losing the original activation time.

### Goal

Acking a `require_ack` alarm that has already normalized (latched Active with the condition no longer true) clears it immediately — the `Cleared` transition is journaled and broadcast from the ack path, with no dependency on a future tag snapshot. Acking a still-active alarm (condition true) advances it to `Acked` immediately. Acking an alarm that is Clear/Cleared writes **no** journal row and emits no spurious transition. The ack event carries the real `activated_at_ms` from the alarm's active episode, not `None`.

### Context to read first

- `crates/alarm-engine/src/engine.rs:66-103` — `AlarmEngineHandle::ack`. Today it only knows `self.definitions` (static config), has no visibility into per-path `AlarmRuntime`, so it can't tell whether the alarm is Active, Clear, or already normalized. It journals `from_state: Active` unconditionally and emits `activated_at_ms: None`.
- `crates/alarm-engine/src/engine.rs:146-178` — `spawn_path`: each tag path owns an `AlarmRuntime` inside a spawned task. The runtime state is task-local today; the ack path needs shared visibility into it (e.g. lift `AlarmRuntime.active` behind the same `Arc<Mutex<..>>` sharing pattern already used for `acks`, keyed by tag path or alarm id).
- `crates/alarm-engine/src/engine.rs:180-227` — `AlarmRuntime`, `AckRequest`, `evaluate_snapshot`, `mark_acked`. `mark_acked` only flips Active → Acked; it never clears a normalized-and-acked alarm (that path lives in `apply`).
- `crates/alarm-engine/src/engine.rs:229-329` — `apply` and `transition`. `apply`'s `(AlarmState::Acked, false, _)` arm is what clears a normalized acked alarm today — but only when a snapshot arrives. `transition` computes `activated_at_ms` from the retained `ActiveAlarm` (line 272-278); the ack path must reuse the same source.
- `crates/alarm-engine/src/journal.rs:36-50` — `write_transition`. The ack path already calls this directly (synchronously, not `spawn_blocking`) at `engine.rs:84`. Keep transitions honest: only journal a state change that actually happened.
- `crates/alarm-engine/src/types.rs:60-101, 134-149` — `AlarmState`, `ActiveAlarm` (carries `activated_at_ms`), `AlarmTransition`.
- [`docs/agents/notes/python-tag-write-routing.md`](../notes/python-tag-write-routing.md) — unrelated to ack routing, but confirms the engine handle is the single ack entry point (`crates/gateway/src/server.rs:642` wraps `engine.lock().ack(..)` in `spawn_blocking`).

### Files to create / modify

1. **Modify** `crates/alarm-engine/src/engine.rs`:
   - Give the ack path visibility into per-path `AlarmRuntime` state. The cleanest match to the existing pattern is to share the active-alarm map the same way `acks` is shared: an `Arc<Mutex<HashMap<..>>>` the spawned task writes and `ack()` reads/updates. Key it so `ack(alarm_id)` can locate the alarm's current `ActiveAlarm`. Follow the neighbor: `acks` is already `Arc<Mutex<HashMap<String, AckRequest>>>` threaded into `spawn_path`; mirror that shape for the shared active state rather than inventing a new synchronization primitive.
   - In `ack()`: look up the alarm's current `ActiveAlarm`. Branch on its state:
     - Not present / Clear / Cleared → the alarm is not active. Do **not** journal a transition, do **not** emit a spurious `Active→` event. (Decide and document whether ack is a no-op or emits an informational no-state-change signal; a no-op is the safe default.)
     - Active with condition still latched → transition to `Acked`, journal `Active → Acked`, update shared state, emit the event with the real `activated_at_ms`.
     - Active-but-normalized (`require_ack` latch, condition already false) → clear immediately: journal `Active → Cleared` (or `Acked → Cleared` per the state machine), remove from the active map, emit the `Cleared` event carrying the real `activated_at_ms`.
   - Emit `activated_at_ms` from the retained `ActiveAlarm`, never hardcoded `None`.
   - Keep the `acks` map insert so a concurrently-arriving snapshot doesn't double-process, but ensure `take_ack` + `apply` on a later snapshot is now idempotent with the immediate ack (no double transition).
2. **Do not** change the `alarm.event` / `alarm.ack` wire form (`crates/protocol/src/lib.rs`); this is an internal state-machine correctness fix.

### Behavior

- Ack of a latched-normalized `require_ack` alarm → immediate `Cleared` transition (journaled + broadcast) without any further tag event. Engine memory no longer diverges from what clients saw.
- Ack of a still-active alarm → immediate `Acked`. A later normalizing snapshot then clears it (existing `apply` arm), producing exactly one `Cleared` — not two.
- Ack of a Clear/Cleared/absent alarm → no journal row, no spurious `Active→` transition.
- Every ack event carries the correct `activated_at_ms` (the active episode's start), matching what the original Active event reported.
- Whether ack drives the machine immediately or a snapshot arrives first, the resulting journal sequence is identical (idempotent) — no duplicate `Acked` or `Cleared` rows.

### Test requirements

- Add to the existing alarm-engine test file(s); do not fragment.
- Deterministic — no sleeps. Drive snapshots explicitly and call `ack()` at controlled points.
- Stuck-active regression: configure a `require_ack` alarm; feed a snapshot that activates it; feed a snapshot that normalizes it (still Active due to latch); call `ack()`; assert **without any further snapshot** that a `Cleared` transition is journaled and broadcast and the alarm is gone from engine state. This test must fail against the current code (where the alarm stays Active forever).
- Spurious-transition regression: call `ack()` on an alarm that is Clear (never activated); assert the journal has **no** new row and no `Active→` event was emitted. Must fail against current code (which journals `from_state: Active`).
- Activation-time regression: activate an alarm at a known `ts`, ack it, assert the ack/clear event's `activated_at_ms == Some(that ts)`. Must fail against current code (hardcoded `None`).
- Idempotency: ack immediately, then feed a normalizing snapshot; assert only one `Cleared` row exists (no double transition).
- Each regression run once against pre-fix code to confirm it catches the bug; state which in the Codex log.
- Full matrix: `cargo test -p openwebhmi-alarm-engine`, workspace clippy `-D warnings`, `cargo fmt --check`.

### Acceptance criteria

- [ ] `ack()` drives the state machine immediately using current per-path runtime state; a latched-normalized `require_ack` alarm clears on ack with no further tag event.
- [ ] `ack()` on a Clear/Cleared/absent alarm writes no journal row and emits no spurious `Active→` transition.
- [ ] The ack/clear event carries the real `activated_at_ms` from the active episode, never hardcoded `None`.
- [ ] Immediate-ack and snapshot-driven-ack paths are idempotent — never two `Acked` or two `Cleared` rows for one episode.
- [ ] Shared active-state visibility follows the existing `acks` `Arc<Mutex<..>>` pattern, not a new primitive.
- [ ] Three regression tests (stuck-active, spurious-transition, activation-time) present, deterministic, and demonstrably failing without the fix.
- [ ] No `alarm.event` / `alarm.ack` wire change.
- [ ] Journal write from the ack path stays free of `unwrap`/`expect`/`panic` (mirror the existing `warn!`-on-error handling at `engine.rs:84-86`).
- [ ] Full matrix clean; Codex log records the pre-fix confirmation for each regression test.

### Out of scope

- Deadband / on-off delay (that is CODEX-CE — coordinate if both land close together, since both touch `apply` and `AlarmRuntime`).
- Alarm shelving, suppression, or bulk-ack of multiple alarms.
- Persisting ack state across gateway restarts (the journal already records it; rebuilding live state from the journal on startup is a separate brief).
- Any change to `alarm.ack` request routing or the `spawn_blocking` wrap at `server.rs:642`.
- Reworking how `who` / `note` are threaded (only `activated_at_ms` and the from-state correctness are in scope).

### Risks / gotchas

- **Lock ordering / shared-state deadlock.** If the active-alarm map becomes an `Arc<Mutex<..>>` also held inside the spawned task's evaluation loop, `ack()` locking it while a snapshot is mid-`apply` must not deadlock. Keep lock scopes narrow (lock, read/mutate, drop before journaling or broadcasting). The existing `acks` map is locked only briefly in `take_ack`/insert — match that discipline.
- **Double-transition on race.** Ack immediately clears, and a normalizing snapshot already queued may also try to clear. The `acks` map + a state check must make the second path a no-op. Guard on current state, not just the `acks` entry.
- **`from_state` honesty.** The whole point is to stop journaling transitions that didn't happen. Read the actual current state before choosing `from_state`; never assume `Active`.
- **`activated_at_ms` source.** Reuse the `ActiveAlarm.activated_at_ms` that `transition` already reads (`engine.rs:272-278`), so the ack event and the original Active event agree.
- **No `expect`/`unwrap` in the ack path.** The lock is fallible (`acks.lock()` already uses `if let Ok`); the shared active map must handle a poisoned lock the same way — log and skip, don't panic. This is a non-startup, non-test production path per CLAUDE.md "no panic in production code."
- **Coordinate with CODEX-CE.** Both briefs restructure `AlarmRuntime` / `apply`. If CE lands first, CF rebases onto the pending-edge state it added; if CF lands first, CE composes off-delay with the immediate-ack clear. Whoever is second reads the other's merged diff before starting.

## Codex log

## Claude review

## Verdict
