---
id: CODEX-BO
title: Modbus robustness — hostname/DNS, per-request timeouts, bounds-checked decode
owner: codex
phase: 4
status: open
created: 2026-07-12
last-update: 2026-07-12 claude [Fable 5]
---

# CODEX-BO — Modbus robustness (DNS, timeouts, bounds)

## Brief

> **HIGH — three related robustness gaps in the Modbus TCP path.** (a) Connect parses the endpoint with `format!("{host}:{port}").parse::<SocketAddr>()`, which accepts only IP literals — but the config field is documented "Hostname or IP address", so `plc1.plant.local` fails with "invalid TCP endpoint". Resolve via DNS. (b) Only the initial connect is timeout-wrapped; individual `tokio-modbus` requests have no deadline, so a half-open connection blocks `read_group` forever — the subscription unfold never ticks, no Bad is ever emitted, and the shared-client mutex wedges every other read/write. Wrap every request in a per-request timeout; on timeout emit Bad and trigger a reconnect. (c) Group decode does unchecked slice indexing (`&bits[offset..offset+count]`, `&registers[offset..offset+count]`); `tokio-modbus` 0.16.1 only `debug_assert`s response length, so in release a short device response panics inside the subscription-driving task with no supervisor. Bounds-check and map short responses to a `DriverError`.

### Goal

The Modbus driver connects to hostnames, not just IP literals. Every request has a bounded deadline so a half-open connection surfaces as Bad quality and a reconnect rather than a permanent hang that also wedges the shared client mutex. A short or malformed device response returns a `DriverError` (mapped to Bad) instead of panicking.

### Context to read first

- `crates/driver-modbus/src/driver.rs`:
  - `connect_client`, lines 452-499 — TCP branch (454-467): `format!("{host}:{port}").parse::<SocketAddr>()` at 455-459 (IP-literal-only), and the single connect timeout at 460-466. The RTU branch is unaffected.
  - `poll_groups`, lines 319-352 — the unchecked slices: `item.parsed.decode_bits(&bits[offset..offset + count])` at 327 and `.decode_registers(&registers[offset..offset + count])` at 337. `offset`/`count` come from the request's own address math, but a **short response** (device returns fewer elements than requested) makes `offset+count` exceed the slice length.
  - `read_group`, lines 354-371 — issues the actual `read_coils`/`read_holding_registers`/… calls under `client.lock().await` (355). This is where the per-request timeout must wrap, and where a hang holds the mutex.
  - `read()` / `write()`, lines 183-246 — one-shot paths that take the same `client.lock().await` (186, 225); they wedge if a subscription request hangs holding the lock. Their requests need the same per-request timeout.
  - `subscribe()` / `SubscriptionState`, lines 248-302 — the `stream::unfold` that `state.interval.tick().await` then `poll_groups(...)`. A hung `read_group` inside `poll_groups` means the unfold future never resolves; the stream stalls and the gateway's per-driver task (see below) never sees an update.
- `crates/driver-modbus/src/connection.rs`, lines 40-47 — `Connection::Tcp { host, port }`, `host` documented "Hostname or IP address" (line 42). Confirm whether `ModbusConfig` already carries a `connection_timeout_ms` (used at driver.rs 461) and add a `request_timeout_ms` field (with a sane default) if none exists.
- `crates/gateway/src/project.rs`, lines 243-258 — the per-driver task that drives the subscription stream and calls `publish_bad_for_tags` when a `Quality::Bad` update or stream end arrives. This is the "supervisor" the panic would escape from; note it drives the stream but does not catch a panic in the polled future.
- `tokio-modbus` 0.16.1 client API: confirm `Reader`/`Writer` methods return the `Result<Result<_, ExceptionCode>, Error>` shape already wrapped by `ModbusRequestResult`, and confirm the library only `debug_assert!`s the response element count (so release builds index out of bounds on a short response). Confirm the timeout wrap point — wrapping the `async` request future in `tokio::time::timeout` is sufficient; there is no per-request deadline knob in the client.
- `tokio::net::lookup_host` — the DNS resolution entry point (async; returns an iterator of `SocketAddr`).

### Files to create / modify

1. **Modify** `crates/driver-modbus/src/driver.rs` `connect_client` TCP branch: replace the `SocketAddr::parse` with `tokio::net::lookup_host(format!("{host}:{port}"))`, taking the first resolved address (error if resolution yields nothing), still inside the existing connect timeout. Preserve the current IP-literal behavior (a literal resolves through `lookup_host` too).
2. **Modify** the request path so **every** `tokio-modbus` call (in `read_group`, `read`, `write`) is wrapped in a per-request `tokio::time::timeout` using a configurable `request_timeout_ms`. On timeout: map to a `DriverError` (so `into_quality()` is Bad and the gateway reacts), and trigger a reconnect of the shared client rather than leaving a wedged half-open connection. Ensure a hung request cannot hold `client.lock()` indefinitely (the timeout must bound the locked section).
3. **Modify** `poll_groups` (327, 337) to bounds-check `offset + count` against the response length before slicing; a short response yields a per-item `DriverError` update (Bad) instead of an index panic. Apply the same guard to any other unchecked slice in the decode path.
4. **Modify** `crates/driver-modbus/src/connection.rs` to add `request_timeout_ms` to `ModbusConfig` with a `#[serde(default = …)]` if it's not already present.
5. **Add tests** to the existing `#[cfg(test)] mod tests` in `crates/driver-modbus/src/driver.rs` (do not create a new test file). The `MockClient` seam already exists — extend it to simulate a hang (a request that never resolves) and a short response.

### Behavior

- A hostname config (`plc1.plant.local:502`) resolves and connects; an IP literal still works.
- A request that exceeds `request_timeout_ms` returns Bad and triggers reconnect; it does not hang the unfold or hold the mutex past the deadline. Subsequent reads/writes proceed after reconnect.
- A device response shorter than requested produces a per-item Bad update, not a panic — in both debug and release.
- Happy-path reads/writes and read-group merging are unchanged.

### Test requirements

- **Hostname resolution test**: a `Connection::Tcp` with a resolvable host (e.g. `localhost`) reaches the connect path; assert it does not fail with "invalid TCP endpoint". (Bind a `127.0.0.1:0` listener and resolve/connect to its port; read the assigned port back from the listener — no hardcoded ports.)
- **Request-timeout test (deterministic)**: use `tokio::time::pause()` + `advance()` (no `sleep()`, no wall-clock) with a `MockClient` whose request future never resolves; assert the wrapped request returns a Bad update after the deadline and that a reconnect is triggered. Prove it fails pre-fix (the un-wrapped request hangs the test / never yields Bad).
- **Short-response test**: `MockClient` returns fewer elements than the group requested; assert `poll_groups` yields a Bad update for the affected item(s) and does not panic. Prove the pre-fix code panics/over-indexes on this input (run the pre-fix path in a `catch_unwind` or assert the slice would exceed bounds).
- Full matrix clean: `cargo test -p openwebhmi-driver-modbus`, workspace `clippy -D warnings`, `fmt --check`.

### Acceptance criteria

- [ ] TCP connect resolves hostnames via `tokio::net::lookup_host`; IP literals still work; empty resolution errors cleanly.
- [ ] Every `tokio-modbus` request (subscription `read_group`, one-shot `read`/`write`) is wrapped in a configurable per-request timeout; timeout → Bad + reconnect, and never holds the client mutex past the deadline.
- [ ] `poll_groups` bounds-checks before slicing (327, 337); a short response is a Bad update, not a panic, in release and debug.
- [ ] `request_timeout_ms` exists on `ModbusConfig` with a sane serde default.
- [ ] Timeout test uses `tokio::time::pause()`/`advance()` — no `sleep()`; resolution test uses `127.0.0.1:0` — no hardcoded port.
- [ ] Timeout and short-response tests fail against pre-fix code and pass after.
- [ ] No `unwrap`/`expect`/`panic` on production paths introduced; the `unreachable!` in `address.rs` (line 248) is out of scope and unchanged. New suppressions use `#[expect(...)]`.
- [ ] Codex log states the confirmed `tokio-modbus` 0.16.1 short-response behavior (debug_assert only) and the chosen reconnect mechanism.

### Out of scope

- RTU serial timeouts and hostname handling (RTU has no host); this brief is the TCP path. If the per-request timeout wrap naturally covers RTU requests too, that's fine — note it — but don't build RTU-specific reconnect.
- A full connection-supervisor/backpressure redesign — the reconnect here is the minimal "drop the wedged client, re-run `connect_client`" needed to recover; a richer supervisor is a separate brief.
- Modbus write range-checking (already handled in `address.rs` `encode_registers`).
- Changing read-group merge heuristics (`build_read_groups`, `max_group_count`).

### Risks / gotchas

- **Reconnect must not deadlock the mutex.** The shared client is `Arc<Mutex<Box<dyn ModbusClientLike>>>`; reconnect replaces the inner client. Decide how a subscription task (holding no lock at the moment of timeout, since the timeout bounds the locked section) signals reconnect vs how `connect`/`disconnect` own the client. The simplest correct shape: on timeout, drop/replace the connection under a short-held lock; document the ordering.
- **`lookup_host` yields multiple addresses.** Take the first (or first IPv4, matching prior behavior) and error if the iterator is empty. Don't silently connect to an unexpected family without noting the choice.
- **The panic has no supervisor.** The subscription stream is polled by the gateway's per-driver task (project.rs 243-258), which does not `catch_unwind`. A panic there kills that task and silently ends the subscription — which is exactly why the bounds-check matters. Don't "fix" it by wrapping the gateway task in `catch_unwind`; fix the decode to not panic.
- **Timeout granularity.** A per-request timeout that's shorter than the poll interval is fine; make the default `request_timeout_ms` sensible relative to `connection_timeout_ms` and `poll_rate_ms`, and state the relationship.
- **Don't take the "invalid TCP endpoint" symptom at face value** — the fix is DNS resolution at connect, not massaging the error message. Confirm `lookup_host` accepts the `host:port` string form.
- **Honesty:** if reconnect-after-timeout can only be unit-tested against the mock (no real half-open PLC in CI), say so by name in the Codex log.

## Codex log

## Claude review

## Verdict
