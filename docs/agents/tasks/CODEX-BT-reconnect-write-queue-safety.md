---
id: CODEX-BT
title: Reconnect write-queue safety — don't replay stale operator setpoints after reconnect
owner: codex
phase: 4
status: open
created: 2026-07-12
last-update: 2026-07-12 claude [Fable 5]
---

# CODEX-BT — Reconnect write-queue safety

## Brief

> While a driver is disconnected, the per-driver write receiver is not drained. In `crates/gateway/src/project.rs`, `run_driver` (lines 180-204) only touches `write_rx` inside `run_subscription_until_disconnect` (lines 218-281), and that function returns on disconnect — so up to `WRITE_QUEUE_CAPACITY` (64, line 24) operator commands pile up in the channel. On reconnect the `select!` loop (lines 260-278) flushes every queued write to the PLC. Replaying stale operator setpoints minutes after they were issued is a classic SCADA safety hazard: an operator commands a valve closed, the link drops, they walk away, the link recovers, and the gateway re-sends the now-inappropriate command. Drop-and-audit-as-failed queued writes on disconnect, or time-bound each queued write and discard expired ones. Separately, on write failure and on disconnect the code overwrites numeric tag paths with `TagValue::String(err.to_string())` (lines 271-276) and `publish_bad_for_tags` (lines 289-293) — morphing a numeric tag's value type to a string. Preserve the last-good value and signal fault through `Quality` (`Bad`/`Stale`) instead of corrupting the value type.

### Goal

Operator commands issued while a driver is disconnected are **not** silently replayed to the PLC on reconnect — they are dropped (and audited as failed) or expire by a max-age bound, so the PLC never receives a stale setpoint. A tag's value **type** is stable across a fault: a numeric tag stays numeric, with health communicated via `Quality`, never rewritten to a `String` error message.

### Context to read first

- `crates/gateway/src/project.rs:24` — `WRITE_QUEUE_CAPACITY: usize = 64`; line 23 `RECONNECT_BACKOFF`.
- `crates/gateway/src/project.rs:90` — the `mpsc::channel(WRITE_QUEUE_CAPACITY)` create; `rx` moves into `run_driver`.
- `crates/gateway/src/project.rs:171-204` — `run_driver`: the reconnect loop. `write_rx` is only serviced inside `run_subscription_until_disconnect`; between disconnect (return) and the next successful connect, nothing drains the queue.
- `crates/gateway/src/project.rs:218-281` — `run_subscription_until_disconnect`: the `select!` with the `command = write_rx.recv()` arm (260-278). On reconnect this drains whatever accumulated.
- `crates/gateway/src/project.rs:271-276` — the write-failure publish: `store.publish(path, TagValue::String(err.to_string()), err.into_quality())` onto what may be a numeric tag path.
- `crates/gateway/src/project.rs:289-293` — `publish_bad_for_tags`: same string-morph problem on disconnect for every tag.
- `crates/gateway/src/script_writes.rs` and `crates/gateway/src/server.rs:1407-1418` — the two producers into the write queue (script + WS). Their audit/error surfaces are where a dropped-on-disconnect write should be reflected as failed. Coordinate with CODEX-BS (audit honesty) but do not overlap its unknown-driver scope.
- `crates/protocol` (`Quality`, `TagValue`) — confirm `Quality::Stale`/`Bad` exist and that `TagStore::publish` accepts a value + quality without forcing a value-type change.

### Files to create / modify

1. **Modify** `crates/gateway/src/project.rs` — reconnect write-queue safety:
   - On disconnect (when `run_subscription_until_disconnect` returns), **drain and discard** any queued writes rather than leaving them for the next connection to flush. Discarded writes should be surfaced as failed to their originators/audit where a channel exists (a write command may need to carry enough context — e.g. its tag path — to be auditable; if the current `WriteCommand` lacks it, extend the struct minimally).
   - **Or** attach a monotonic issue-time to each `WriteCommand` and, on reconnect, discard any command older than a `MAX_WRITE_AGE` bound (add a named `const`, e.g. a few seconds). Choose one approach; document the choice and its rationale ("why this and not the alternative") in the Codex log. The drain-on-disconnect approach is simpler and strictly safer for setpoints; the age-bound approach preserves very-recent writes across a brief blip. Either satisfies the safety contract.
   - Preserve queue ordering and backpressure semantics for the accepted (non-stale) writes.

2. **Modify** `crates/gateway/src/project.rs` — value-type preservation on fault:
   - Replace the `store.publish(path, TagValue::String(err.to_string()), …)` on write failure (271-276): keep the tag's last-good value, publish with `Quality::Bad` (or `Stale`), and carry the error text via a channel that does **not** overwrite the value — e.g. log it, and/or a separate status/diagnostic tag, not the value tag itself.
   - Replace `publish_bad_for_tags` (289-293): publish each tag's last-good value with `Quality::Bad`/`Stale`, not `TagValue::String(message)`. If the last-good value is not readily available at that call site, fetch it from the `TagStore` (read-modify-quality) or publish a quality-only update if the store supports it.
   - No `unwrap`/`expect`/`panic!` on production paths.

3. **Modify** `WriteCommand` (in `crates/gateway/src/project.rs`, and its construction sites in `script_writes.rs` and `server.rs`) only as far as needed to carry issue-time and/or tag-path for auditing dropped writes. Keep the change minimal and update all construction sites.

4. **Modify** the `crates/gateway` tests (extend the closest existing test module/file for `project.rs`; do not fragment into a new file):
   - A test proving writes enqueued during a disconnected window are **not** delivered to the driver on reconnect (drain approach), or are delivered only if within `MAX_WRITE_AGE` (age approach).
   - A test proving a numeric tag's `TagValue` variant is unchanged across a write failure and across a disconnect — only its `Quality` changes to `Bad`/`Stale`.

### Behavior

- Disconnect → queued operator writes are discarded (or expire by max-age); the PLC never receives them on reconnect.
- Discarded writes are auditable/observable as failed by their originator, not silently swallowed.
- A numeric tag reads as its last-good numeric value with `Quality::Bad`/`Stale` during a fault — never a `String` error message.
- Recent, in-bound writes (age approach) or all post-reconnect writes (drain approach) continue to flow normally once connected.

### Test requirements

- Use a fake/stub driver whose connect/disconnect and write-observation are controllable **deterministically** — no `sleep()`, no wall-clock waits. Use `tokio::time::pause()` + `advance()` for the age-bound test, channels/oneshots for connect/disconnect sequencing, and observe the driver's received-writes via a channel.
- The stale-replay regression test must **fail against the pre-fix code** (which flushes the queue on reconnect). Confirm fail-before / pass-after; note in the Codex log.
- The value-type-preservation test must fail pre-fix (value morphs to `String`). Confirm fail-before / pass-after.
- No hardcoded ports (bind `127.0.0.1:0` if any listener is involved).
- Full validation matrix clean.

### Acceptance criteria

- [ ] Writes enqueued during a disconnected window are not replayed to the driver on reconnect (dropped, or discarded if older than a named max-age const).
- [ ] Dropped/expired writes are surfaced as failed to their originator/audit, not silently discarded.
- [ ] Chosen approach (drain-on-disconnect vs age-bound) and its rationale documented in the Codex log.
- [ ] Numeric tag `TagValue` variant is preserved across write-failure and disconnect; fault is signaled via `Quality::Bad`/`Stale` only.
- [ ] `publish_bad_for_tags` no longer overwrites tag values with `TagValue::String`.
- [ ] `WriteCommand` changes (if any) are minimal and all construction sites updated.
- [ ] Both regression tests fail pre-fix, pass post-fix; deterministic (no sleeps).
- [ ] No `unwrap`/`expect`/`panic!` on production paths; full matrix clean.

### Out of scope

- The unknown-driver / phantom-memory-tag write path — that is CODEX-BS. This brief assumes writes reach a *known* driver's queue; the hazard is replay timing, not routing.
- Persisting the write queue across a gateway restart (queue is in-memory by design; a restart drops it, which is safe).
- Reworking `RECONNECT_BACKOFF` or the reconnect loop's backoff strategy.
- A full operator-command confirmation/ack protocol (overlaps CODEX-BS's audit-honesty boundary; keep them separate).

### Risks / gotchas

- **Draining vs. age-bounding.** Drain-on-disconnect is the safest default for setpoints — a discarded operator command forces the operator to re-issue with current knowledge, which is what SCADA safety wants. Age-bound is a refinement that keeps a sub-second blip transparent. Don't do both in a way that double-counts; pick one and state why.
- **Read-modify-quality races.** Preserving last-good value may require reading the current `TagStore` entry before republishing with `Bad` quality. Confirm the store API exposes a value-preserving quality update, or fetch-then-publish carefully — but a lost race here only mislabels quality briefly, it doesn't corrupt the value type, which is the point.
- **`WriteCommand` context.** Auditing a dropped write needs the tag path; the current `WriteCommand` may only carry `address` + `value`. Extending it touches `script_writes.rs` and `server.rs` construction sites — keep it minimal and update every site (per CLAUDE.md "match the neighbor" discipline).
- **Determinism.** The reconnect/replay behavior is timing-sensitive; the temptation to `sleep` in the test is exactly the flaky-test trap CLAUDE.md forbids. Drive connect/disconnect via explicit channel signals and time via `tokio::time` pause/advance.
- **`if let` chains, top-level imports, full variable names** per CLAUDE.md Rust discipline.

## Codex log

## Claude review

## Verdict
