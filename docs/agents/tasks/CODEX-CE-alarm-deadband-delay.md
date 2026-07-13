---
id: CODEX-CE
title: Alarm deadband/hysteresis + on/off delay — stop analog-alarm chattering
owner: codex
phase: 4
status: open
created: 2026-07-12
last-update: 2026-07-12 claude [Fable 5]
---

# CODEX-CE — Alarm deadband/hysteresis + on/off delay

## Brief

> A noisy analog tag oscillating around a `HighLimit` threshold produces a rapid Active/Cleared storm today — one SQLite journal write per edge plus one broadcast event, burying operators. Add per-alarm **deadband** (a return band the value must re-cross to clear, distinct from the trip point) and configurable **on-delay / off-delay** (the condition must hold for T milliseconds before the state transitions) to the alarm definition and its evaluation. Every real SCADA alarm subsystem (Ignition alarm pipeline, FactoryTalk Alarms & Events) ships deadband + activation/deactivation delay; OpenWebHMI's absence of them is a v1 integrator blocker for analog alarms. Preserve the existing state machine, journal shape, and event wire form.

### Goal

`crates/alarm-engine` supports per-alarm deadband and on/off delay. An analog tag oscillating inside the deadband around a `HighLimit`/`LowLimit`/`Deviation` trip point transitions Active exactly once and stays Active until the value returns past the deadband boundary — no chatter, no journal storm. A transient spike shorter than the configured on-delay never trips; a brief dip shorter than the off-delay never clears. Alarms with deadband/delay left unset behave exactly as they do today (zero deadband, zero delay).

### Context to read first

- `crates/alarm-engine/src/conditions.rs:8-22` — `evaluate` returns a bare comparison per condition kind. This is where the deadband band logic attaches (the clear threshold differs from the trip threshold once an alarm is active).
- `crates/alarm-engine/src/engine.rs:190-257` — `AlarmRuntime::evaluate_snapshot` → `apply`. `apply` maps `(current_state, condition_active, require_ack)` to a transition. On/off delay gates the transition here: a candidate edge must persist across snapshots for `>= delay_ms` before `apply` commits it.
- `crates/alarm-engine/src/engine.rs:259-329` — `transition` writes the journal row (via `spawn_blocking`, keep that) and broadcasts the event. Do not change its shape.
- `crates/alarm-engine/src/types.rs:7-58` — `AlarmDefinition` and `AlarmCondition`. New fields land here. Note: `AlarmDefinition` is **not** currently `#[non_exhaustive]`; if this brief adds `#[non_exhaustive]`, the struct-literal construction in `crates/gateway/src/project.rs:122-131` and every test constructing `AlarmDefinition` must switch to a builder or `..Default::default()` — prefer instead to add fields with sensible defaults and update the two construction sites directly (simpler, smaller diff).
- `crates/gateway/src/project.rs:116-158` — `alarm_definitions` maps the project-store config into `AlarmDefinition`; the deadband/delay fields must be threaded through here from the project config.
- `crates/project-store/src/types.rs:139-191` — `AlarmConfig` / `AlarmConditionConfig`, the on-disk project shape. New config fields land here with `#[serde(default)]` so existing project files keep loading.
- `crates/protocol/src/lib.rs:120-132, 492-510` — the wire `AlarmState` and `alarm.event`. Deadband/delay are **evaluation** parameters, not part of the emitted event — confirm no wire change is needed (the event still carries `state` / `activated_at_ms` / `transitioned_at_ms`). If a wire field is genuinely required, keep Rust↔TS (`packages/protocol-ts/src/index.ts`) in sync and document why.
- [`docs/agents/notes/binding-write-asymmetry.md`](../notes/binding-write-asymmetry.md) — not directly relevant, but the config-field-with-default pattern mirrors the `tagPath?` additive approach.

### Files to create / modify

1. **Modify** `crates/alarm-engine/src/types.rs` — add to `AlarmDefinition`:
   - `deadband: f64` (absolute return band in engineering units; `0.0` = no hysteresis, current behavior). Document that for `HighLimit` the alarm clears only when `value <= threshold - deadband`, for `LowLimit` only when `value >= threshold + deadband`, for `Deviation` only when `abs(value - setpoint) <= tolerance - deadband`. `Equals` / `Digital` ignore deadband (discrete conditions).
   - `on_delay_ms: u64` and `off_delay_ms: u64` (default `0` = immediate, current behavior).
2. **Modify** `crates/alarm-engine/src/conditions.rs` — the deadband changes the *clear* comparison, so evaluation needs to know whether the alarm is currently active. Either extend `evaluate` to take a `currently_active: bool` and apply the deadband-shifted threshold when active, or add a sibling `evaluate_with_deadband`. Keep `ConditionError` semantics. Do not apply deadband to `Equals`/`Digital`.
3. **Modify** `crates/alarm-engine/src/engine.rs` — thread deadband into the `evaluate` call (`AlarmRuntime` knows the current state per alarm id at `engine.rs:237-241`). Implement on/off delay: track a pending-edge timestamp per alarm id (candidate transition first observed at snapshot ts); commit the transition in `apply` only once `snapshot.ts - pending_since >= delay_ms`. A candidate edge that reverses before the delay elapses is discarded (no transition, no journal write). Use `snapshot.ts` (event-time), not wall-clock, so replay and tests are deterministic.
4. **Modify** `crates/gateway/src/project.rs:122-131` — thread the new fields from `AlarmConfig` into `AlarmDefinition`.
5. **Modify** `crates/project-store/src/types.rs` `AlarmConfig` — add `#[serde(default)] deadband`, `#[serde(default)] on_delay_ms`, `#[serde(default)] off_delay_ms` so existing project JSON without these keys still loads.

### Behavior

- Deadband: once Active on a `HighLimit { threshold }`, the alarm stays Active while `value > threshold - deadband` and clears only when `value <= threshold - deadband`. Symmetric for `LowLimit`. `Deviation` shrinks the clear tolerance by `deadband`.
- On-delay: `condition_active` must hold continuously for `on_delay_ms` (measured in `snapshot.ts` deltas) before Clear/Cleared → Active is journaled and broadcast. A spike that clears before the delay elapses produces no transition.
- Off-delay: the clear condition must hold for `off_delay_ms` before Active/Acked → Cleared. A brief dip below the clear boundary that recovers before off-delay elapses keeps the alarm Active.
- Delay applies to the condition edge, independent of `require_ack` latching — an acked-latched alarm still respects off-delay before it clears.
- Zero deadband + zero delay is byte-for-byte the current behavior (existing tests must pass unchanged).

### Test requirements

- Add to the existing alarm-engine test file(s); do not fragment (per CLAUDE.md "add to existing test files").
- Deterministic only — **no `sleep`, no wall-clock**. Drive the state machine by feeding `TagSnapshot`s with explicit `ts` values (event-time) and, where a runtime timer is unavoidable, `tokio::time::pause()` + `advance()`. The engine already keys delay off `snapshot.ts`, so a sequence of snapshots with advancing `ts` is the primary vehicle.
- Chatter test: oscillate a value inside the deadband around a `HighLimit` threshold across many snapshots; assert exactly one Active transition is journaled and one event broadcast (assert on journal row count and received event count).
- On-delay test: a single spike snapshot above threshold followed by a return-to-normal snapshot before `on_delay_ms` elapses produces zero transitions; a spike that persists past `on_delay_ms` produces exactly one Active.
- Off-delay test: an Active alarm dips below the clear boundary for less than `off_delay_ms` then recovers — assert it stays Active with no Cleared transition.
- Each new regression test must fail against the pre-change engine (run once without the deadband/delay logic to confirm it catches the chatter). State this in the Codex log.
- Full matrix: `cargo test -p openwebhmi-alarm-engine`, workspace clippy `-D warnings`, `cargo fmt --check`, plus `pnpm -r typecheck` / `pnpm -r test` if any wire type changed.

### Acceptance criteria

- [ ] `AlarmDefinition` carries `deadband`, `on_delay_ms`, `off_delay_ms`; `AlarmConfig` carries the same with `#[serde(default)]`.
- [ ] `alarm_definitions` in `crates/gateway/src/project.rs` threads all three fields through.
- [ ] Deadband applied to `HighLimit` / `LowLimit` / `Deviation` clear comparisons only; `Equals` / `Digital` unaffected.
- [ ] On-delay and off-delay gate transitions using `snapshot.ts` event-time; reversing edges within the window produce no transition and no journal write.
- [ ] Chatter, on-delay, and off-delay regression tests present, deterministic (no sleeps), and demonstrably fail without the fix.
- [ ] Zero-deadband / zero-delay alarms behave identically to today; all pre-existing alarm-engine tests pass unchanged.
- [ ] Existing `alarm.event` wire form unchanged (or, if a field was genuinely required, Rust↔TS kept in sync and justified).
- [ ] Journal writes stay wrapped in `spawn_blocking` (`engine.rs` transition path unchanged in that respect).
- [ ] Full matrix clean; Codex log states which pre-change run confirmed each regression test catches the bug.

### Out of scope

- Rate-of-change / bad-quality alarm conditions, alarm shelving, and alarm grouping — separate briefs.
- Designer UI for editing deadband/delay — this brief is the gateway/engine/config plumbing only; the config accepts the fields, the designer form is a follow-up.
- Per-condition deadband on `Equals` / `Digital` (discrete conditions have no meaningful hysteresis).
- Changing the `spawn_blocking` journal-write discipline (CODEX-AJ) or the broadcast channel sizing.
- Reworking `require_ack` latch semantics beyond making off-delay compose with them.

### Risks / gotchas

- **Event-time vs wall-clock.** Delay must be measured in `snapshot.ts` deltas, not `SystemTime::now()`, or tests can't be deterministic and replayed history would mis-fire. The engine already has `snapshot.ts` at every evaluation site — use it.
- **Deadband direction sign.** `HighLimit` clears *below* `threshold - deadband`; `LowLimit` clears *above* `threshold + deadband`. Getting the sign wrong inverts the hysteresis and makes chatter worse. The chatter test guards this.
- **`AlarmDefinition` construction sites.** Adding non-defaulted fields breaks every struct literal. `crates/gateway/src/project.rs:122-131` plus every test constructor must be updated. Prefer plain fields updated at each site over `#[non_exhaustive]` + builder churn — smaller diff, and the maintainer can audit each construction.
- **Pending-edge state must reset on the opposing edge.** If the condition flips back before the delay elapses, the pending timestamp must clear so a later genuine edge starts a fresh timer. A stale pending timestamp would fire a delayed transition spuriously.
- **`serde(default)` on `AlarmConfig` is load-bearing** for existing on-disk projects. Without it, every project file that predates this change fails to deserialize. Confirm a round-trip test with a project JSON that omits the new keys.
- **Don't widen the event storm to the delay timer.** The delay suppresses transitions; it must not itself schedule per-snapshot journal writes or broadcasts. Only the committed transition writes.

## Codex log

## Claude review

## Verdict
