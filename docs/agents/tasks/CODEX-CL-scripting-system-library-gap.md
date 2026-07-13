---
id: CODEX-CL
title: Scripting system.* library gap — implement or formally defer the documented RPC surface + triggers
owner: codex
phase: 4
status: open
created: 2026-07-12
last-update: 2026-07-12 claude [Fable 5]
---

# CODEX-CL — Scripting system.* library gap

## Brief

> The scripting host ships exactly 4 RPC methods (`tag.read`, `tag.write`, `util.now`, `util.log`) and one working trigger (`on_tag_change`); `on_timer`/`on_alarm`/`on_button_click` are stubs. The docs advertise `system.tag.subscribe`, `system.alarm.ack`, `system.db.query`, `system.http.get/post`, `system.util.send_message`, and all four triggers. Close the documented-vs-implemented gap with an **explicit implement-or-defer decision per item**: implement the highest-value v1 gaps end-to-end with tests, formally defer the rest, and leave the docs honest. Recommended: implement `system.alarm.ack`, the `on_timer` trigger, and `system.tag.subscribe`; defer `system.db.query` and `system.http.*` to a later phase. This pairs with CODEX-CI (docs reconciliation) — every defer decision here must be reflected there.

### Goal

Each documented `system.*` method and trigger is either (a) wired end-to-end — Rust `RpcMethod`/trigger + dispatch in the host + Python bridge in `system/` + designer completion stub — with an end-to-end test, or (b) formally deferred with a recorded decision and a docs reframe. No method or trigger remains in the "advertised but silently stubbed" limbo it's in today. The recommended v1 set (`system.alarm.ack`, `on_timer`, `system.tag.subscribe`) ships; `system.db.query` and `system.http.*` are deferred unless the per-item review below says otherwise.

### Context to read first

- `crates/scripting/src/rpc.rs:99-114` — `RpcMethod` enum (the 4 shipped methods) + the arg structs (`TagReadArgs`, `TagWriteArgs`, ...). New methods add variants here with their `#[serde(rename = "...")]` wire names and arg structs.
- `crates/scripting/src/worker.rs:234-278` — `dispatch_rpc` (the `match method { ... }` that services each `RpcMethod` against `TagStore` / `TagWriteSink` / the `ScriptEvent` broadcast). New methods add arms here.
- `crates/scripting/src/triggers.rs:6-45` — `TriggerRegistration` enum; `OnTimer`/`OnAlarm`/`OnButtonClick` are `/// Stub` variants and `tag_change_paths` `filter_map`s them out. `on_timer` implementation extends this to actually schedule.
- `crates/scripting/src/host.rs:22-33` — host orchestration, `handler_timeout`, backoff constants, the `StreamMap` of subscriptions that drives `on_tag_change`. `on_timer` hooks in near where tag-change subscriptions are multiplexed.
- `crates/scripting/python/system/__init__.py` — the `system` shim: `on_tag_change` registration + `_get_tag_change_handler`. New triggers register the same way.
- `crates/scripting/python/system/_bridge.py` — `call(method, args)` JSON-RPC bridge. New `system.*` functions call through this.
- `crates/scripting/python/system/tag.py`, `util.py` — the per-family Python modules. A new `system.alarm` family adds `system/alarm.py`; `tag.subscribe` extends `tag.py`.
- `crates/scripting/python/_runner.py` — the worker runner loop that dispatches host `trigger` frames to registered handlers. `on_timer`/`on_alarm` trigger delivery flows through here.
- `apps/designer/src/lib/systemStubs.ts` — Monaco completion stubs. Today it lists **only the 4 shipped methods + `on_tag_change`** (it is honest). Each newly-implemented method/trigger adds a stub entry here; `apps/designer/src/__tests__/systemStubs.test.ts` covers it.
- **Existing Rust APIs to wire against (these already exist — use them, don't reinvent):**
  - `crates/alarm-engine/src/engine.rs:66-70` — `AlarmEngine::ack(alarm_id, who, note)`. `system.alarm.ack` routes here (via a sink analogous to `TagWriteSink`). Also `subscribe_events` (~28) → `broadcast::Receiver<AlarmEvent>` for a future `on_alarm`.
  - `crates/tag-engine/src/lib.rs:89` — `TagStore::subscribe(path) -> broadcast::Receiver<TagSnapshot>`. `system.tag.subscribe` builds on the same mechanism `on_tag_change` already uses.
- [`VISION.md`](../../../VISION.md) §"What OpenWebHMI is not" and §"What we won't merge" — check before implementing anything. Nothing in VISION forbids `system.db`/`system.http` outright, but they're feature-scope calls: `system.http.*` is script-directed egress (distinct from the gateway's "no third-party calls" runtime invariant, which is about the *gateway*, not user scripts) and `system.db.query` needs a managed connection pool. Assess both against v1 scope; the recommendation is to defer.
- [`docs/roadmap.md`](../../roadmap.md):98-103 — the Phase 3 scripting promises. `system.db.*` appears at line ~101.
- CODEX-CI (`docs/agents/tasks/CODEX-CI-docs-code-reconciliation.md`) — the paired docs task. Every defer decision here must land in CI's reframe.
- [`CLAUDE.md`](../../../CLAUDE.md) §"Why this and not the alternative?" — if any item is ambiguous vs VISION.md scope, **stop and confirm the per-item implement/defer decision in the Codex log before building it**.

### Files to create / modify

Per-item; only for the items decided "implement". For each implemented method/trigger, all four layers move together (Rust variant + dispatch + Python bridge + designer stub) plus a test.

1. **`system.alarm.ack` (recommended: implement)**
   - `rpc.rs`: add `RpcMethod::AlarmAck` (`#[serde(rename = "alarm.ack")]`) + an `AlarmAckArgs { alarm_id, note }` struct.
   - `worker.rs` `dispatch_rpc`: add an arm routing to the alarm engine. This needs an alarm-ack sink threaded into the host analogous to `TagWriteSink` — add an `AlarmAckSink` trait (or reuse an existing alarm handle if one is already plumbed to `crates/scripting`; check first) and wire it from the gateway. The script's identity is the "who".
   - `python/system/alarm.py`: `def ack(alarm_id, note=None): return _bridge.call("alarm.ack", {...})`; export `alarm` from `system/__init__.py`.
   - `systemStubs.ts` + test entry.
   - **Why the sink, not a direct call:** scripting must not depend on `alarm-engine` in a way that inverts the layering — follow the `TagWriteSink` pattern (`crates/scripting/src/sink.rs`) where the gateway injects the concrete implementation. Confirm the existing sink shape before adding a parallel one.

2. **`on_timer` trigger (recommended: implement)**
   - `triggers.rs`: promote `OnTimer { every_ms }` from stub to a real registration; add an extractor analogous to `tag_change_paths` (e.g. `timer_intervals`).
   - `host.rs`: schedule timer firing deterministically and deliver a `trigger` frame to the worker. **Use `tokio::time::interval` / a timer stream merged into the existing `StreamMap`, not `sleep` loops** — and tests must drive it with `tokio::time::pause()` + `advance()` (no wall-clock waits).
   - `python/system/__init__.py` + `_runner.py`: `on_timer(every_ms)` registration + handler dispatch for the timer trigger frame. Decide the handler signature (likely no-arg or a small `{ts_ms}` payload) and document it.
   - `systemStubs.ts` + test entry.

3. **`system.tag.subscribe` (recommended: implement)**
   - Decision needed and stated in the log: is `subscribe` a *callback registration inside a script* (script stays resident, handler fires on change) or *sugar over `on_tag_change`*? Given the worker model already multiplexes tag-change subscriptions in `host.rs`, the cleanest v1 form is a script-registered callback delivered as a trigger frame (reuse the `on_tag_change` machinery). Confirm this in the log before building; avoid a second, divergent subscription path.
   - `rpc.rs`/`host.rs`/`__init__.py`/`_runner.py` as needed; `systemStubs.ts` + test.

4. **`system.util.send_message` (decide: implement thin or defer)**
   - This is inter-script/gateway messaging. If there's no message bus in v1, **defer** and reframe docs. If a lightweight "log an event other scripts/clients can observe" already exists via the `ScriptEvent` broadcast, a thin version may be cheap — assess and state the decision.

5. **`system.db.query` (recommended: DEFER)** — needs a managed connection pool and a query-permission model. Record as formally deferred; ensure CODEX-CI marks it roadmap. Do not implement unless the log-recorded per-item review overrides.

6. **`system.http.get/post` (recommended: DEFER)** — script-directed network egress; needs a policy decision (allowlist, timeouts, the eventual network resource-limit story that architecture.md over-promised). Record as formally deferred; CODEX-CI marks it roadmap.

7. **`on_alarm` / `on_button_click` triggers (decide)** — `on_alarm` could ride `AlarmEngine::subscribe_events` (feasible, medium value); `on_button_click` needs a runtime→gateway component-event path that may not exist in v1. Recommend: defer both unless `on_alarm` is cheap given the alarm-ack plumbing lands in item 1. State per-item decision.

8. **`docs/agents/tasks/CODEX-CL-...` Codex log + CODEX-CI coordination** — record the final implement/defer table so CI's docs reframe matches exactly.

### Behavior

- Every implemented method behaves end-to-end: a Python script calls `system.<x>(...)`, the bridge sends an `rpc` frame, `dispatch_rpc` services it against the real engine (alarm ack actually acknowledges; timer actually fires; subscribe actually delivers change frames), and the result returns to the script.
- Deferred items are removed from the "shipped" framing in the docs (via CODEX-CI) and are not left as silently-advertised stubs. A script calling a deferred `system.*` function should get a clear "not implemented in v1" error, not a confusing hang or a silent no-op — add an explicit unimplemented-method error path in `dispatch_rpc`/the bridge for anything advertised-but-deferred.
- No regression to `on_tag_change`, `tag.read/write`, `util.now/log`.
- Trigger delivery stays deterministic and crash-isolated (worker subprocess model unchanged).

### Test requirements

- **Each newly-implemented method/trigger gets an end-to-end test** in the closest existing scripting test module (`crates/scripting/tests/host.rs` is the integration home — extend it, don't fragment). The test drives a real worker subprocess (as the existing tests do) and asserts the observable effect:
  - `alarm.ack`: a script calling `system.alarm.ack(id)` results in the alarm engine (or its injected sink) recording the ack with the script as "who". The test must fail if the ack never reaches the engine.
  - `on_timer`: with `tokio::time::pause()` + `advance()`, the handler fires the expected number of times over a controlled interval. **No wall-clock waits.**
  - `tag.subscribe`: publishing a tag change delivers the change to the subscribed handler.
- **Regression-validity:** each new test must fail against the pre-implementation code (run it before wiring the feature; if it passes, it isn't testing the feature).
- **Deferred items:** a test (or the docs check in CODEX-CI) asserting the deferred method returns a clear "unimplemented in v1" error rather than hanging or silently succeeding.
- Designer stub tests (`apps/designer/src/__tests__/systemStubs.test.ts`) updated for each new stub.
- Full matrix clean: `cargo build`, `cargo clippy --workspace --all-targets --all-features --locked -- -D warnings`, `cargo test --workspace --all-features --locked`, `cargo doc --workspace --no-deps`, `pnpm -r typecheck`, `pnpm -r test`. No `sleep`/`setTimeout` in tests; run new async tests three consecutive times.
- `#![deny(missing_docs)]` — new public Rust items documented; new Python functions have docstrings matching the `system/` module style.

### Acceptance criteria

- [ ] A final implement/defer decision is recorded (Codex log table) for every documented item: `tag.subscribe`, `alarm.ack`, `db.query`, `http.get/post`, `util.send_message`, and triggers `on_timer`, `on_alarm`, `on_button_click`.
- [ ] The recommended v1 set — `system.alarm.ack`, `on_timer`, `system.tag.subscribe` — is implemented end-to-end (Rust variant/trigger + `dispatch_rpc`/host wiring + Python `system/` bridge + designer stub) with end-to-end tests that fail without the implementation. (Any deviation from this set is justified in the log against VISION.md scope.)
- [ ] `system.alarm.ack` routes through an injected sink (mirroring `TagWriteSink`), not a layering-inverting direct dependency; it actually acknowledges via `AlarmEngine::ack`.
- [ ] `on_timer` fires deterministically (`tokio::time::pause()`+`advance()` in tests); no wall-clock waits anywhere.
- [ ] Deferred items (`system.db.query`, `system.http.*`, and any others decided-defer) return a clear "unimplemented in v1" error path, are not advertised as shipped, and are marked roadmap — coordinated with CODEX-CI.
- [ ] `on_tag_change`, `tag.read/write`, `util.now/log` unregressed.
- [ ] Designer completion stubs + their tests updated for each new method/trigger.
- [ ] Full validation matrix clean (Rust + Node); new async tests deterministic across three runs.
- [ ] Codex log confirms any VISION.md-ambiguous per-item decision *before* the corresponding code was written.

### Out of scope

- **Implementing `system.db.query` or `system.http.*`** unless the per-item review explicitly overrides the defer recommendation — and if it does, that's arguably its own brief given the connection-pool / egress-policy surface. Default: defer + document.
- **The CPU/memory/network resource sandbox** architecture.md over-promised — `system.http.*` egress limits are entangled with that, which is a reason to defer http. The sandbox itself is a separate, larger task.
- **`on_button_click`** end-to-end unless a runtime→gateway component-event path already exists (it likely doesn't in v1) — defer + document.
- **Editing the docs' capability claims** — that's CODEX-CI's job. CL *supplies the implement/defer decisions*; CI *reframes the prose*. Don't duplicate the doc edits here beyond the module docstrings/rustdoc for what CL ships.
- **A general inter-script message bus** for `system.util.send_message` beyond a thin `ScriptEvent`-based version if one is cheap; a real pub/sub bus is out of scope.
- **Changing the worker IPC transport** (JSON over stdio) or the subprocess model.

### Risks / gotchas

- **Layering.** `crates/scripting` must not grow a hard dependency on `alarm-engine` internals — inject an `AlarmAckSink` the way the gateway injects `TagWriteSink`. Check `crates/scripting/src/sink.rs` for the exact shape and mirror it; the gateway wires the concrete impl. Deviating inverts the dependency graph.
- **Timer determinism.** The single hardest correctness risk. Use a timer stream merged into the existing `StreamMap`, and test exclusively with `tokio::time::pause()`/`advance()`. A `sleep`-based timer or a wall-clock test is an automatic reject per VISION.md "No flaky tests."
- **`tag.subscribe` vs `on_tag_change` divergence.** There is already one subscription path. `system.tag.subscribe` should reuse it, not fork a second lifecycle with different backpressure/lag semantics (the tag-engine broadcast already documents "lagging subscribers drop messages" — inherit that, don't reinvent). Confirm the chosen shape in the log before building.
- **Deferred ≠ silently broken.** An advertised-but-deferred method must error clearly ("unimplemented in v1"), not hang the worker waiting on an RPC the host never answers, and not silently return null. Add the explicit unimplemented arm.
- **Honesty coupling with CODEX-CI.** If CL defers an item, CI must mark it roadmap; if CL implements one CI expected to be roadmap, CI marks it shipped. Land the decision table in the CL log so CI has an unambiguous source. Sequence the merges so the docs never claim something the code doesn't do at any commit.
- **Ask before building when ambiguous.** Per CLAUDE.md, if `system.util.send_message`, `on_alarm`, or the `db`/`http` defer is genuinely unclear against VISION.md scope, record the decision (and rationale) in the Codex log *before* writing the code — a mis-scoped implement is more expensive than a one-line log entry.
- **Test-validity discipline.** Run each new test against the unmodified code first and confirm it fails. An `alarm.ack` test that passes before the ack is wired is testing nothing.

## Codex log

## Claude review

## Verdict
