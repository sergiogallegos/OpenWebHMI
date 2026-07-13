---
id: CODEX-BL
title: MQTT driver event-loop resilience — survive broker hiccups, propagate Bad on loss
owner: codex
phase: 4
status: open
created: 2026-07-12
last-update: 2026-07-12 claude [Fable 5]
---

# CODEX-BL — MQTT driver event-loop resilience

## Brief

> **CRITICAL.** The MQTT driver's background worker `break`s out of its poll loop on the first `event_loop.poll()` error, permanently ending every subscription — but `read()` keeps serving the last cached value with implicit-Good semantics, so the gateway never observes the loss. `rumqttc` only reconnects while the caller keeps polling after an error; breaking defeats its built-in reconnect. Keep polling across poll errors (log + backoff, don't `break`), mark cache entries Bad/Stale on connection loss so `read()` and subscribers actually see the failure, and stop fabricating placeholder values that masquerade as data. This is the MQTT slice of the cross-driver resilience sweep — the six drivers should converge on one shape for "connection dropped → tags go Bad, driver reconnects".

### Goal

A broker hiccup (TCP drop, broker restart, keep-alive timeout) no longer kills the MQTT driver. The worker keeps polling the event loop so `rumqttc` reconnects and re-subscribes; while disconnected, cached tags report `Quality::Bad` (or `Stale`) through both `read()` and the subscription stream instead of a frozen-Good last value. No cache entry is ever seeded with a fabricated `TagValue::String("no value seen")` carrying implicit-Good semantics.

### Context to read first

- `crates/driver-mqtt/src/driver.rs`:
  - `connect()` worker loop, lines 75-91 — the `match event_loop.poll().await { … Err(err) => { warn!; break; } }` that ends the loop, and the `mark_bad(&cache, &worker_tx).await` after the loop.
  - `read()`, lines 126-132 — serves from cache, returns `Ok(update.value)` regardless of quality; only `None` yields an error.
  - `subscribe()` BroadcastStream, lines 162-171 — the `filter_map` whose `_ => None` arm silently swallows `BroadcastStreamRecvError::Lagged` (and every other error) with no log.
  - `seed_uncertain_cache()`, lines 373-385 — inserts `TagValue::String("no value seen")` at `Quality::Uncertain` for every mapping before any real data arrives.
  - `mark_bad()`, lines 387-393 — broadcasts each cached update at `Quality::Bad` but never writes the Bad quality back into the cache, so `read()` still returns the stale Good value.
- `rumqttc` 0.25 `EventLoop::poll` semantics — confirm that a returned `ConnectionError` does **not** consume the event loop: the documented reconnect model is "keep calling `poll()` and it re-establishes the connection". Verify the exact error type and which variants are transient vs fatal (e.g. auth rejection) before deciding what to retry.
- `openwebhmi_protocol::Quality` — the available variants (`Good`, `Uncertain`, `Bad`, and whether a `Stale` variant exists). Pick the variant the other drivers use for "last-known value, connection lost".
- [`docs/agents/notes/mqtt-tls-in-ci.md`](../notes/mqtt-tls-in-ci.md) — CI runs `rumqttd 0.20.0` plaintext-only; any test broker must be plaintext.
- Sibling drivers for the target shape: `crates/driver-modbus/src/driver.rs` `poll_groups`/`update_from_result` (per-tag Bad on read error) and the gateway's `publish_bad_for_tags` path in `crates/gateway/src/project.rs` (~247-258) — the consumer that already knows how to react to a `Quality::Bad` update.

### Files to create / modify

1. **Modify** `crates/driver-mqtt/src/driver.rs`:
   - Rework the worker loop so a `poll()` error logs and continues (with a bounded backoff to avoid a hot spin while the broker is down) instead of `break`ing. Keep polling so `rumqttc` reconnects and the subscriptions it holds are re-established.
   - On transition into a disconnected state, mark cache entries Bad/Stale **in the cache itself** and broadcast the Bad update, so a subsequent `read()` returns the degraded quality rather than a frozen-Good value. On reconnect + fresh data, quality returns to Good naturally through `publish_update`.
   - Remove the fabricated-value seeding: either drop `seed_uncertain_cache` or change it so a never-seen tag reports an honest "no data yet" state (Uncertain/Bad with no invented `TagValue`), and make `read()` distinguish "connected but no sample yet" from "have a real sample". Do not return `Ok(TagValue::String("no value seen"))` as if it were device data.
   - Handle the `BroadcastStream` `Lagged` case in `subscribe()` with a `tracing::warn!` (dropped-update count) instead of the current silent `_ => None`.
2. **Modify** `read()` so its return reflects cache quality — a Bad/Stale cache entry must not surface as an unqualified `Ok(value)` that the gateway reads as healthy. (Confirm how the gateway consumes `Driver::read` for one-shot reads before changing the signature contract; if `read` can only return a value, encode loss as a `DriverError` so `into_quality()` carries Bad.)
3. **Add tests** to the existing `#[cfg(test)] mod tests` in `crates/driver-mqtt/src/driver.rs` (do not create a new test file).

### Behavior

- Worker survives a `poll()` error: it logs, backs off, and keeps polling so `rumqttc` reconnects and re-subscribes. It only exits when the driver is dropped/disconnected (worker `abort()` in `disconnect()`).
- While disconnected, the cache reflects Bad/Stale and both `read()` and live subscribers observe it. On reconnect, real samples restore Good.
- No cache entry ever carries a fabricated placeholder `TagValue` with Good/Uncertain-implying-data semantics.
- `BroadcastStream` lag is logged, not silently dropped.
- No behavior change to the happy path (publish decode, Sparkplug handling, JSONPath) beyond quality plumbing.

### Test requirements

- **Broker-drop resilience test**: drive the worker (or an extracted poll-handling helper) with a simulated `poll()` error sequence — error, then success — and assert the worker does **not** terminate and resumes handling publishes after the error. Prove it fails against the current `break`-on-error code (the pre-fix worker exits; the test must catch that).
- **Loss-marks-Bad test**: after a simulated disconnect, assert `read()` for a previously-Good tag no longer returns an unqualified Good value (returns Bad/Stale or a `DriverError` that maps to Bad), and that a Bad update is broadcast to subscribers.
- **No-fabrication test**: a tag that has never received a publish does not `read()` back as `Ok(TagValue::String("no value seen"))`.
- Structure the worker so the poll-result handling is unit-testable without a live TCP broker (extract a helper that takes a `Result<Event, _>` and mutates cache/broadcast; the `tokio::spawn` loop becomes a thin driver over it). No real sockets, no `sleep()` — use deterministic inputs. If a broker is unavoidable, bind `127.0.0.1:0` plaintext and read the port back.
- Full matrix clean: `cargo test -p openwebhmi-driver-mqtt`, workspace `clippy -D warnings`, `fmt --check`.

### Acceptance criteria

- [ ] `event_loop.poll()` errors no longer `break` the worker; the loop keeps polling (with bounded backoff) so `rumqttc` reconnects.
- [ ] Connection loss writes Bad/Stale into the cache; `read()` and subscribers observe the degraded quality; reconnect restores Good on fresh data.
- [ ] `seed_uncertain_cache`'s fabricated `TagValue::String("no value seen")` is gone; never-seen tags report an honest no-data state, not invented device data.
- [ ] `BroadcastStream` `Lagged` is logged, not silently dropped.
- [ ] Broker-drop resilience test fails against the pre-fix `break` code and passes after.
- [ ] No `sleep()`/wall-clock waits in tests; no hardcoded ports; tests added to the existing test module.
- [ ] No `unwrap`/`expect`/`panic` on production paths introduced; new suppressions use `#[expect(...)]` with a reason, not `#[allow]`.
- [ ] Codex log states the exact `rumqttc` 0.25 `poll()` error type and which variants are treated transient vs fatal, and why.

### Out of scope

- TLS/mTLS validation (maintainer manual-smoke gate — see the TLS note).
- Sparkplug datatype/DEATH/command correctness — that is CODEX-BQ; don't overlap.
- Reconnect backoff tuning knobs in `MqttConfig` beyond what's needed to avoid a hot spin (a sane fixed/bounded backoff is fine; a configurable policy is a separate brief if wanted).
- Changing the broadcast channel capacity or the cache data structure wholesale.

### Risks / gotchas

- **Don't defeat rumqttc's reconnect by re-creating the client.** The fix is to keep polling the existing `EventLoop`; confirm from the 0.25 docs that `poll()` after `Err` re-drives the connection. If a specific error variant is genuinely fatal (bad credentials), exiting on *that* is correct — distinguish, and log which.
- **Backoff must not hot-spin.** A `poll()` that returns `Err` immediately in a tight loop will peg a core. Use a small bounded delay between failed polls; keep it deterministic-testable (inject the delay or gate it so tests don't wait on wall-clock).
- **`read()` contract.** If `Driver::read` returns `DriverResult<TagValue>` with no quality channel, the only honest way to signal loss is a `DriverError` whose `into_quality()` is Bad. Check how sibling drivers and the gateway one-shot read path treat this before changing behavior — match the neighbor.
- **Quality variant choice.** Use whatever variant the rest of the codebase uses for "last value, connection lost" (Bad vs a dedicated Stale). Don't invent a new one.
- **Honesty:** if a live-broker integration test can't run in the merge environment, say so by name in the Codex log; don't claim reconnect was validated against a real broker if it was only unit-tested against injected errors.

## Codex log

## Claude review

## Verdict
