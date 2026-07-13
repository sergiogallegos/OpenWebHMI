---
id: CODEX-BJ
title: DriverUpdate connection-state signal — distinguish one bad tag from a lost connection
owner: codex
phase: 4
status: open
created: 2026-07-12
last-update: 2026-07-12 claude [Fable 5]
---

# CODEX-BJ — DriverUpdate connection-state signal (one bad tag ≠ lost connection)

## Brief

> `DriverUpdate` (`crates/driver-api/src/trait_def.rs:16-26`) carries only per-address `quality`. The gateway (`crates/gateway/src/project.rs:254-258`) treats **any** `Quality::Bad` update as a disconnection: it marks every tag `Bad` and tears down/reconnects the driver. But Rockwell legitimately emits per-tag `Bad` for a single missing/mistyped tag via `PartialError` (`crates/driver-rockwell/src/driver.rs:177-182`, `204-214`) — a healthy connection reporting one bad tag. Result: one mistyped tag address forces the whole connection to cycle connect→subscribe→`Bad`→disconnect every ~250ms forever, flapping every *healthy* tag on the device with it. Add an explicit connection-state signal to the driver API so the gateway reconnects only on **transport loss** and leaves healthy tags live when one tag is `Bad`. This is a **cross-layer contract decision** that should land **before** more drivers wire into the gateway — coordinate with CODEX-BI (which erases the driver type behind `Box<dyn Driver>` and consumes this same reconnect predicate).

### Goal

The driver API can distinguish "this one tag is bad" from "the transport is gone." The gateway's subscription loop reconnects only on genuine transport loss (stream end, connection error) and, when a single tag reports `Bad`, publishes that tag `Bad` while leaving every other tag on the device live and updating. A mistyped tag address no longer collapses the whole connection.

### Context to read first

- `crates/driver-api/src/trait_def.rs:15-26` — `DriverUpdate { address, value, quality, ts_ms }`. This is the only signal a driver sends per update; there is no transport-level channel. `subscribe` returns `BoxStream<'static, DriverUpdate>` (line 69-113) — stream **end** (`None`) is today the only unambiguous "connection gone" marker.
- `crates/gateway/src/project.rs:243-281` — `run_subscription_until_disconnect`. Line 246-249: `stream.next()` returning `None` → publish all `Bad`, return (correct — real disconnect). Line 255-258: **any** `update.quality == Quality::Bad` → publish all `Bad`, return (the bug — conflates one bad tag with disconnect). Line 234-241: subscribe failure → publish all `Bad`, return.
- `crates/driver-rockwell/src/driver.rs:172-233` — `event_to_updates` and `map_value_result`. `PartialError` (177) yields per-tag `Bad` for the failing tags while other tags in the same snapshot stay `Good`. `ReadFailure` (183) yields `Bad` for **all** requested addresses — that one *is* a transport/group-level failure. This asymmetry is the crux: `PartialError` must NOT reconnect; `ReadFailure` / stream-end SHOULD.
- The other four drivers' `Bad`-emitting paths, to confirm the chosen shape fits each:
  - `crates/driver-opcua/src/driver.rs:255-257` (`status_to_quality` → `Quality::Bad` per-node) and its subscribe loop.
  - `crates/driver-mqtt/src/driver.rs:390` (`update.quality = Quality::Bad` on payload decode failure — per-topic, not transport).
  - `crates/driver-modbus/src/driver.rs` subscribe/read path (per-register vs connection failure).
  - `crates/driver-ads/src/driver.rs` connect/subscribe (symbol read failures vs AMS route loss).
- `crates/driver-api/src/error.rs` — `DriverError` variants; `NotConnected`/`Io`/`Connecting` are the transport-loss family, `InvalidAddress`/`UnsupportedType`/`RemoteFault` are per-tag. `into_quality()` (lib.rs:62-88) already maps these — useful precedent for classifying tag-level vs transport-level.
- `crates/driver-api/src/supervisor.rs:22-35` — `DriverStatus` enum already models `Disconnected`/`Connecting`/`Connected`/`Faulted`. The new signal should align with, not duplicate, this vocabulary.

### Files to create / modify

Pick **one** of two contract shapes and state the choice + rationale in the Codex log:

- **(A) `DriverEvent` enum** wrapping the stream item: `subscribe` returns `BoxStream<'static, DriverEvent>` where `DriverEvent::Tag(DriverUpdate)` carries per-tag quality and `DriverEvent::Transport(Connected | Disconnected { reason })` (or similar) signals connection state. Cleanest separation; touches every driver's `subscribe` return type and the gateway's `stream.next()` match arm.
- **(B) Typed field on `DriverUpdate`** — e.g. an `enum UpdateKind { Tag, TransportLost }` or a `transport_state: Option<TransportState>` — so a `Bad` update is disambiguated by an explicit field rather than the gateway inferring transport loss from `quality`. Smaller diff; risks overloading `DriverUpdate` with a concern it doesn't cleanly own.

Recommendation: **(A)** — a `Bad` *tag* and a *transport* event are genuinely different kinds of message, and an enum makes the gateway's match exhaustive (the compiler forces every driver and the gateway to handle both). But (B) is acceptable if it wires all six surfaces with less churn; justify whichever you pick.

1. **Modify** `crates/driver-api/src/trait_def.rs`: introduce the chosen type; update `DriverUpdate` and/or add `DriverEvent`; update the default polling `subscribe` (69-113) to emit the transport signal appropriately (a polling driver whose underlying `read` fails for *all* addresses is transport loss; a single-address failure is a bad tag — the current default silently drops read errors at 102-108, which should be reconsidered so bad tags surface as `Bad` rather than vanishing).
2. **Modify** each driver's `subscribe`/event translation to emit the new signal:
   - `driver-rockwell`: `event_to_updates` maps `PartialError` → per-tag `Bad` (no transport signal), `ReadFailure` → transport-lost signal (not N per-tag `Bad`). This is the headline fix.
   - `driver-opcua`, `driver-mqtt`, `driver-modbus`, `driver-ads`: map their transport-level failures to the transport signal and their per-tag failures to per-tag `Bad`.
3. **Modify** `crates/gateway/src/project.rs::run_subscription_until_disconnect`: reconnect only on the transport signal (or stream end); on a per-tag `Bad`, publish that tag `Bad` and **continue the loop** — do not tear down the connection or flap healthy tags. The line 255-258 blanket-`Bad`-triggers-return logic is removed.
4. **Do NOT** change the `DriverConfig` schema or the write path. Keep `DriverUpdate`'s existing `address`/`value`/`quality`/`ts_ms` fields intact under shape (A).

### Behavior

- Rockwell `PartialError` for one mistyped tag: that tag publishes `Bad`; every other tag on the device keeps updating `Good`; the connection stays up. No connect→disconnect flapping.
- Rockwell `ReadFailure` (group-level) or stream end: transport signal fires; gateway marks all tags `Bad` and reconnects with backoff (the correct existing behavior).
- Same distinction holds for the other four drivers: a single bad topic/node/register does not reconnect; a lost broker/session/socket does.
- The default polling `subscribe` surfaces a per-address read failure as a `Bad` tag update (not a silent drop) and only signals transport loss when the driver can no longer be reached.

### Test requirements

- **Rockwell (headline):** add to `crates/driver-rockwell`'s existing tests (the `event_to_updates` tests near driver.rs, or the closest existing test file — do not fragment). Assert `PartialError` with one failing tag and one good tag yields one `Bad` and one `Good` update and **no** transport signal; assert `ReadFailure` yields the transport-lost signal. This is the regression that must **fail against the current code** (today `PartialError` per-tag `Bad` is indistinguishable to the gateway from a disconnect) — run it against pre-fix once to confirm it catches the bug.
- **Gateway:** add to the existing `#[cfg(test)] mod tests` in `crates/gateway/src/project.rs`. Two cases:
  - One-bad-tag: feed the subscription loop a stream containing a per-tag `Bad` event followed by more `Good` events; assert the loop does **not** return/reconnect and the healthy tags keep publishing. Drive with an in-memory `BoxStream` (no live device); use channels/`JoinHandle` for synchronization — **no `sleep()`/wall-clock waits** (CLAUDE.md).
  - Transport-loss: feed the transport-lost event (or stream end); assert the loop returns so the outer reconnect fires.
- **Other drivers:** at minimum extend each driver's existing subscribe/quality test to assert its transport-vs-tag classification compiles and returns the right variant. If a driver has no such test, add the minimal case to its closest existing test file.
- Full validation matrix clean: `cargo build --workspace --all-features --locked`, `cargo clippy --workspace --all-targets --all-features --locked -- -D warnings`, `cargo test --workspace --all-features --locked`, `cargo fmt --check`, `cargo doc --workspace --no-deps`.

### Acceptance criteria

- [ ] The driver API distinguishes per-tag quality from transport connection state (shape A or B, justified in the Codex log).
- [ ] All five drivers emit the new signal correctly: per-tag failures → `Bad` tag; transport failures → transport signal.
- [ ] Rockwell `PartialError` no longer reads as a disconnect; `ReadFailure`/stream-end still does.
- [ ] Gateway reconnects only on transport loss; a single `Bad` tag leaves healthy tags live and the connection up.
- [ ] Default polling `subscribe` surfaces per-address read failures as `Bad` rather than silently dropping them.
- [ ] Rockwell regression test fails against pre-fix code; gateway one-bad-tag and transport-loss tests pass, with no `sleep()`/wall-clock waits.
- [ ] Full validation matrix clean; no `unwrap`/`expect`/`panic!` on new production paths; `#[expect]` over `#[allow]` for any new lint.
- [ ] Codex log confirms the chosen contract shape fits all six surfaces (gateway + five drivers) and names any driver that needed special handling.

### Out of scope

- **Driver-type dispatch / `Box<dyn Driver>`** — that is CODEX-BI. Land this contract first (or coordinate ordering); BI consumes the reconnect predicate this task defines.
- **Rockwell dependency bump / typed writes** — CODEX-BK.
- **Surfacing per-tag `Bad` reasons to the web runtime UI** beyond what `quality` already carries.
- **Retry/backoff tuning** — BI owns the backoff curve; this task only changes *when* reconnect is triggered, not the backoff shape.
- **A full driver health/diagnostics telemetry surface** — a single transport signal is the v1 scope, not a metrics stream.

### Risks / gotchas

- **This is a contract change touching six crates.** Shape (A) makes the gateway match exhaustive (compiler-enforced) but changes every driver's `subscribe` return type — a wide but mechanical diff. Keep the enum minimal; resist adding fields "while we're here."
- **Rockwell `ReadFailure` vs `PartialError` is the exact discriminator.** `event_to_updates` (driver.rs:172-202) already branches on `TagGroupEventKind`: `PartialError` is per-tag (do not reconnect), `ReadFailure` is group-level (reconnect). The classification already exists in the data — the fix is propagating it through `DriverUpdate`/`DriverEvent` instead of flattening both to `Quality::Bad`.
- **Default polling `subscribe` currently drops read errors** (trait_def.rs:102-108 logs and continues). Under the new contract, a per-address read error should become a `Bad` tag update; only a driver that's wholesale unreachable should signal transport loss. Don't accidentally make every transient single-read hiccup a reconnect.
- **`DriverUpdate` is `PartialEq` and used in tests** across drivers. Under shape (A) the stream item type changes from `DriverUpdate` to `DriverEvent`; every driver test that matches on `stream.next()` items updates. Under shape (B) the field addition may break exhaustive struct literals in tests. Either way, expect test churn — budget for it.
- **MQTT/Sparkplug nuance.** MQTT "connection" is broker-level; a Sparkplug node going offline (NDEATH) is arguably a per-source transport event, not a global one. Confirm the single-signal model is adequate or note the limitation for a follow-up (don't over-engineer a per-source state machine in v1).
- **Ordering with BI.** If BI merges first, its reconnect loop keys off `Quality::Bad` and will be rewritten here — that's expected, own it in the verdict. Landing BJ first is cleaner: BI then wires dispatch against the already-correct predicate.
- **Honesty.** If the fix is validated only against in-memory sim streams (not a live CompactLogix emitting a real `PartialError`), say so — the sim proves the classification logic, not the on-wire event mapping. That's the maintainer's manual-smoke gate.

## Codex log

## Claude review

## Verdict
