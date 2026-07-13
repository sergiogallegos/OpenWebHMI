---
id: CODEX-BI
title: Gateway driver dispatch — route driver_type to the correct driver and adopt DriverSupervisor
owner: codex
phase: 4
status: open
created: 2026-07-12
last-update: 2026-07-12 claude [Fable 5]
---

# CODEX-BI — Gateway driver dispatch (route `driver_type`; adopt `DriverSupervisor`)

## Brief

> `crates/gateway/src/project.rs::run_driver` calls `connect_rockwell()` unconditionally and never inspects `driver.driver_type`; `connect_rockwell()` force-parses every driver's config as `RockwellConfig`. The merged Phase-4 drivers (`openwebhmi-driver-modbus`, `-opcua`, `-mqtt`, `-ads`) are therefore **unreachable from the gateway binary** despite being marked `merged` on the board. A project declaring `driver_type = "modbus"` gets its config mis-parsed and the fixed-250ms reconnect loop retries the parse failure forever, log-spamming while every tag stays `Bad`. Separately, `DriverSupervisor` (`crates/driver-api/src/supervisor.rs`) — the panic-catching + exponential-backoff supervision layer — has **zero consumers** and is entirely dead code. Dispatch on `driver.driver_type` to construct the correct `Box<dyn Driver>`; fail fast and surface driver status on an unknown/invalid type instead of hot-looping; adopt the existing backoff/panic-catch machinery. **Coordinate with CODEX-BJ** (connection-state signal): BJ changes the `DriverUpdate`/subscription contract that this task's reconnect logic consumes — land BJ's contract decision first, then wire dispatch on top, or the reconnect predicate will be rewritten twice.

### Goal

The gateway binary can drive **all five** merged drivers. A project whose `drivers[].driver_type` is `"rockwell"`, `"modbus"`, `"opcua"`, `"mqtt"`, or `"ads"` constructs the matching `Driver` implementation, parses that driver's own config type, connects, subscribes, and publishes tags. An unknown or structurally-invalid `driver_type` fails fast: the driver's status tag is set to a terminal error state and the task exits (or backs off with the shared exponential+jitter policy) instead of spinning a tight 250ms parse-failure loop.

### Context to read first

- `crates/gateway/src/project.rs` — `run_driver` (lines 171-204) is the loop that calls `connect_rockwell` unconditionally; `connect_rockwell` (206-213) force-parses `RockwellConfig`; `run_subscription_until_disconnect` (218-281) holds a concrete `RockwellDriver` and calls `.subscribe(...)`. The flat `RECONNECT_BACKOFF` const is at line 23.
- `crates/driver-api/src/trait_def.rs` — the `Driver` trait (lines 30-114): `connect(&mut self, config: serde_json::Value)`, `subscribe(&self, addresses) -> DriverResult<BoxStream<'static, DriverUpdate>>`. Every driver implements exactly this, so dispatch can hold `Box<dyn Driver>`.
- `crates/driver-api/src/supervisor.rs` — `DriverSupervisor::spawn(Box<dyn Driver>, Value) -> SupervisorHandle` (lines 73-143); `connect_until_success` (207-239) and `sleep_backoff`/`jittered` (269-285) implement exponential backoff (`INITIAL_BACKOFF` 250ms → `MAX_BACKOFF` 8s) with ±25% jitter and panic-catch. Note: the supervisor's command loop is a **request/reply** model (`Read`/`Write`/`Browse`), whereas the gateway's `run_subscription_until_disconnect` is a **streaming subscription** model. Reconcile these two shapes deliberately (see Risks).
- Each driver's `connect` signature + config type (all take `serde_json::Value` and parse internally, so the gateway does **not** need to know each config type):
  - `crates/driver-modbus/src/driver.rs:153-170` — `ModbusDriver`, `ModbusConfig::from_value`.
  - `crates/driver-opcua/src/driver.rs:45-62` — `OpcUaDriver`, `OpcUaConfig::from_value`.
  - `crates/driver-mqtt/src/driver.rs:43-60` — `MqttDriver`, `MqttConfig::from_value`.
  - `crates/driver-ads/src/driver.rs:219-238` — `AdsDriver`, `AdsConfig::from_value`.
  - `crates/driver-rockwell/src/driver.rs:53-73` — `RockwellDriver`, `RockwellConfig::from_value` (note: `connect` already parses the config itself; `connect_rockwell` in the gateway does a redundant parse-then-re-encode round trip that should disappear).
- `crates/project-store/src/types.rs:103-111` — `DriverConfig { id, driver_type: String, config: serde_json::Value }`. The `driver_type` string is the dispatch key.
- `crates/gateway/Cargo.toml:26-36` — currently depends only on `openwebhmi-driver-rockwell`. The other four driver crates must be added as path deps.
- [`docs/agents/notes/binding-write-asymmetry.md`](../notes/binding-write-asymmetry.md) — background on the driver/gateway seam (not load-bearing here but useful context).

### Files to create / modify

1. **Modify** `crates/gateway/Cargo.toml`: add path deps on `openwebhmi-driver-modbus`, `openwebhmi-driver-opcua`, `openwebhmi-driver-mqtt`, `openwebhmi-driver-ads` (mirror the existing `openwebhmi-driver-rockwell = { path = "../driver-rockwell" }` line at 27).
2. **Modify** `crates/gateway/src/project.rs`:
   - Replace `connect_rockwell` (206-213) with a `build_driver(driver_type: &str) -> Result<Box<dyn Driver>, UnknownDriverType>` (or equivalent) factory that maps the `driver_type` string to the concrete driver constructor and boxes it. The config parse stays inside each driver's own `connect` — do **not** re-parse per-type in the gateway.
   - Change `run_driver` and `run_subscription_until_disconnect` to hold `Box<dyn Driver>` instead of `RockwellDriver`. `Driver::subscribe(&self, ...)` is object-safe and already returns `BoxStream`, so the streaming loop works unchanged once the concrete type is erased.
   - On unknown/invalid `driver_type`: set the status tag to a terminal error (e.g. `"error: unknown driver type '<x>'"`) and **exit the loop** — do not sleep-and-retry a type that can never become valid. On a *connect* failure (valid type, device unreachable), keep retrying with the shared exponential+jitter backoff.
   - Replace the flat `RECONNECT_BACKOFF` (line 23) with the exponential+jitter policy. Prefer **reusing** `sleep_backoff`/`jittered` from `driver-api` (export them from `supervisor` or a shared helper) over copying the constants — a fourth hand-rolled backoff is exactly the "copy the neighbor" case in CLAUDE.md.
3. **Modify** `crates/driver-api/src/supervisor.rs` **only if** adopting `DriverSupervisor` wholesale for the connect/backoff/panic-catch loop: the supervisor is currently request/reply-only and has no subscription-driven variant. Two acceptable paths — pick one and state the choice in the Codex log:
   - **(a) Reuse the pieces**: make `connect_until_success` / `sleep_backoff` / `jittered` reachable (pub or pub(crate)-then-re-exported) and call them from the gateway's subscription loop, leaving `DriverSupervisor::spawn` for the request/reply consumers. This kills the "backoff is dead code" half.
   - **(b) Extend the supervisor** to own a subscription lifecycle (`Command`-plus-stream), and have the gateway spawn through `DriverSupervisor`. Larger change; only if (a) can't express the streaming publish path cleanly.
   - If neither is adopted this task, the Codex log must state explicitly that `DriverSupervisor` remains unconsumed and why (so the "dead code" gap is owned, not silently carried).
4. **Do NOT** change the `Driver` trait, any driver's `connect`/`subscribe`, the `DriverConfig` schema, or the `DriverUpdate` shape (BJ owns the update contract).

### Behavior

- A project with `driver_type = "modbus"` (or `opcua`/`mqtt`/`ads`/`rockwell`) constructs the matching driver, and that driver's own config parser validates `driver.config`. A Modbus project no longer mis-parses as Rockwell.
- Unknown `driver_type` (e.g. `"siemens"`): status tag becomes a terminal error string; the driver task exits without a hot loop. No 250ms log-spam.
- Structurally-invalid config for a *known* type: the driver's `connect` returns `Err`; the gateway backs off exponentially (250ms → 8s, jittered) rather than at a flat 250ms. (Whether an invalid-config error is terminal or retryable is a judgment call — a bad host:port might become reachable, a malformed JSON never will. Document the chosen predicate in the Codex log; retryable-with-backoff is the safe default and matches current behavior minus the spam.)
- Reconnect on genuine transport loss keeps working exactly as today, now on the shared backoff curve.
- `DriverSupervisor`'s backoff/panic-catch machinery has at least one live consumer after this task (or the Codex log owns why not).

### Test requirements

- **Add to `crates/gateway/src/project.rs`'s existing `#[cfg(test)] mod tests`** (lines 303-351) — do not fragment into a new file.
- **Routing test per driver type**: a `DriverConfig` for each of the five `driver_type` values constructs the correct `Driver` impl. Assert via `Driver::metadata()` (each driver returns a distinct `vendor`/`family` — e.g. Rockwell is `"Rockwell"`/`"EtherNet/IP-CIP"`) that `build_driver("modbus")` yields the Modbus driver, etc. This does not require a live device — construction + metadata is enough to prove dispatch.
- **Unknown-type test**: `build_driver("nonexistent")` returns the error variant (or `run_driver` sets the terminal status and returns). Prove it does **not** enter a retry loop — e.g. drive `run_driver` with an unknown type under a `tokio::time::pause()` clock or a bounded `timeout` and assert the task completes rather than looping. **No `sleep()`/wall-clock waits** — use `tokio::time::pause()` + `advance()` or channel/JoinHandle synchronization per CLAUDE.md.
- **Regression proof**: the routing test must fail against the pre-fix `connect_rockwell`-only code (a Modbus config would route to Rockwell and mismatch metadata). Run it against the unpatched path once to confirm it catches the bug, per CLAUDE.md "your test is NOT VALID if it passes without the fix."
- Full validation matrix clean: `cargo build --workspace --all-features --locked`, `cargo clippy --workspace --all-targets --all-features --locked -- -D warnings`, `cargo test --workspace --all-features --locked`, `cargo fmt --check`, `cargo doc --workspace --no-deps`.

### Acceptance criteria

- [ ] `crates/gateway/Cargo.toml` depends on all five driver crates.
- [ ] `run_driver`/`run_subscription_until_disconnect` operate on `Box<dyn Driver>`; `connect_rockwell` is gone.
- [ ] Dispatch maps every valid `driver_type` to the correct driver; a Modbus project no longer parses as Rockwell.
- [ ] Unknown/invalid `driver_type` fails fast (terminal status, no hot loop) — proven by a deterministic test.
- [ ] Flat 250ms `RECONNECT_BACKOFF` replaced by the shared exponential+jitter policy (reused from `driver-api`, not re-hand-rolled).
- [ ] `DriverSupervisor` (or its backoff helpers) has a live consumer, OR the Codex log states explicitly why it remains dead code.
- [ ] Per-driver routing test + unknown-type test added to the existing `project.rs` test module; routing test demonstrably fails without the fix.
- [ ] Full validation matrix clean; no `unwrap`/`expect`/`panic!` on the new production paths.
- [ ] Codex log states which supervisor-adoption path (a/b/neither) was taken and the invalid-config retry predicate chosen.

### Out of scope

- **Changing the `DriverUpdate` contract or the reconnect-on-`Bad` predicate** — that is CODEX-BJ. This task consumes whatever BJ lands; if BJ hasn't merged yet, wire dispatch against the current predicate and note the follow-up.
- **New driver types** beyond the five merged crates.
- **Per-driver config schema validation UX** in the designer.
- **Write-path routing changes** (`DriverHandle`/`try_write` stay as-is).
- **Browse plumbing** through the gateway.
- **Bumping any driver dependency** (Rockwell's crate bump is CODEX-BK).

### Risks / gotchas

- **Streaming vs request/reply mismatch.** `DriverSupervisor` is built for `Read`/`Write`/`Browse` request/reply; the gateway path is a long-lived `subscribe` stream. Don't force the streaming publish loop into the command model just to "use the supervisor" — reusing `connect_until_success`/`sleep_backoff` (path a) is the lower-risk win. Ask "why this and not the alternative?" before choosing (b).
- **Object safety.** `Driver` is already `#[async_trait]` and object-safe (the supervisor boxes it), so `Box<dyn Driver>` works. `subscribe` takes `&self`, so the boxed driver must outlive the stream — mirror how `run_subscription_until_disconnect` currently owns the concrete driver.
- **`clone_for_polling` / default subscribe.** Drivers relying on the default polling `subscribe` (trait_def.rs:69-113) need `clone_for_polling` to return `Some`. Confirm each of the five either overrides `subscribe` natively or supports polling; a driver that returns `None` and doesn't override `subscribe` will error at subscribe time — surface that clearly rather than hot-looping.
- **Terminal vs retryable classification.** Unknown `driver_type` is terminal (never valid). A connect failure is retryable. A malformed-config parse error is arguably terminal but conservatively retryable. Get this predicate wrong and you either spam logs (current bug) or silently give up on a recoverable device. State the chosen rule.
- **Backoff dedup.** Don't hand-roll a fourth backoff. `jittered` uses `rand::thread_rng().gen_range` — if you export it, keep the existing signature so the supervisor's own tests still pass.
- **CODEX-BJ ordering.** If BJ merges first, the reconnect predicate here keys off BJ's transport-loss signal, not `Quality::Bad`. If BI merges first, BJ will touch this loop again. State the assumption in the Codex log and keep the seam narrow.
- **Honesty.** If the manual smoke (a real multi-driver project booting against live devices) isn't run in this environment, say so by name — the routing test proves construction, not end-to-end device I/O.

## Codex log

## Claude review

## Verdict
