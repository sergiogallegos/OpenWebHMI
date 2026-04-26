---
id: CODEX-B
title: crates/gateway — WS gateway binary with sim provider
owner: codex
phase: 0
status: merged
created: 2026-04-26
last-update: 2026-04-26 13:55 claude
merge-commit: 75ccb9c
---

# CODEX-B — `crates/gateway` (WebSocket gateway binary)

## Brief

### Goal

A Rust binary that binds a WebSocket port, drives a 1 Hz sin-wave + counter into the existing `TagStore`, and serves subscriptions to connected clients per the OpenWebHMI wire protocol. This is the server side of the Phase 0 exit-criterion demo.

### Context to read first

- `crates/protocol/src/lib.rs` — wire types you must consume and emit.
- `crates/tag-engine/src/lib.rs` — `TagStore` API: `publish`, `subscribe`, `get`. Read the module-level docs about subscribe-then-read pattern.
- `docs/architecture.md` §4.1 (Gateway), §4.3 (Protocol), §5 (Data flow).
- `docs/roadmap.md` Phase 0 exit criterion.

### Files to create

- `crates/gateway/Cargo.toml` — package `openwebhmi-gateway`, binary target.
- `crates/gateway/src/main.rs` — entrypoint.
- `crates/gateway/src/server.rs` — connection accept loop and per-connection handler.
- `crates/gateway/src/sim_provider.rs` — sin-wave + counter background task.
- `crates/gateway/tests/integration.rs` — end-to-end test (see Test requirements).

> The workspace `Cargo.toml` already lists `crates/gateway` in `members`. Do **not** add it again. Because the entry exists before the crate, `cargo build --workspace` and `cargo test --workspace` currently error with "could not load Cargo.toml" for `crates/gateway/Cargo.toml`. Your first commit must land at minimum a `Cargo.toml` and a non-empty `src/main.rs` so the workspace builds; subsequent commits flesh out the implementation. CI (CODEX-D) cannot go green until this commit lands.

### Dependencies

The following are already declared in `[workspace.dependencies]` of the root `Cargo.toml`:

- `tokio`, `serde`, `serde_json`, `tracing`, `tracing-subscriber`, `anyhow`, `thiserror`
- `tokio-tungstenite`, `futures-util`, `clap`

Use them via `name = { workspace = true }`. Crate-local path deps:

- `openwebhmi-protocol = { path = "../protocol" }`
- `openwebhmi-tag-engine = { path = "../tag-engine" }`

### CLI surface

```
openwebhmi-gateway [--bind <addr>] [--log-level <level>]
```

Defaults: `--bind 127.0.0.1:8080`, `--log-level info`.

### Behavior

#### Boot

1. Parse args with `clap`.
2. Init `tracing_subscriber` with the requested level + `EnvFilter`.
3. Construct `TagStore::new()`.
4. Spawn `sim_provider::run(store.clone())`.
5. Spawn `server::accept_loop(store.clone(), bind_addr)`.
6. Wait on `tokio::signal::ctrl_c` for shutdown; log shutdown and return.

#### `sim_provider`

- One tokio task; tick at exactly 1 Hz using `tokio::time::interval`.
- On each tick, compute:
  - `system/sim/sin` — `value: TagValue::Real(f64)` where `f64 = sin(2π * t / 60.0)` and `t` is seconds since the provider started (monotonic).
  - `system/sim/counter` — `value: TagValue::Int(i64)` incrementing from 0.
- Both published with `Quality::Good`.

#### Per-connection WS handler

For each accepted connection:

1. `tokio_tungstenite::accept_async`. Track `subscriptions: HashMap<String, JoinHandle<()>>`.
2. Loop reading messages:
   - Text frame: `serde_json::from_str::<ClientMessage>(&text)`.
     - Parse error → send `ServerMessage::Error { code: "protocol.parse", message: <err.to_string()> }`. Connection stays open.
   - Binary frame → reply with `Error { code: "protocol.binary_unsupported", message: "v1 uses JSON only" }`.
   - Ping frame → tungstenite handles the WS-level pong; no app-level action.
3. Handle `ClientMessage`:
   - `TagSubscribe { paths }`: for each path —
     - If already subscribed in this connection, skip (idempotent).
     - `if let Some(snap) = store.get(&path)` → send a `TagUpdate` immediately to prime the client.
     - `let mut rx = store.subscribe(&path)`.
     - Spawn forwarder task that loops `rx.recv()` → encode `ServerMessage::TagUpdate` → send via the connection's mpsc to-client channel.
     - Insert join handle into `subscriptions`.
   - `TagUnsubscribe { paths }`: for each path, remove from `subscriptions` and abort the forwarder.
   - `Ping` → send `ServerMessage::Pong`.
4. On WS close or error: abort all forwarder tasks. Log at info.

#### Recommended internal architecture

To avoid `Sink` contention from many forwarder tasks, use a per-connection mpsc channel `tx_to_client: mpsc::Sender<ServerMessage>` with a single writer task that drains it onto the WS sink. Forwarders push to the mpsc; the writer task serializes + sends. Capacity: 256 (drop oldest on overflow with a warn log).

### Test requirements (`crates/gateway/tests/integration.rs`)

- Start the gateway in-process on `127.0.0.1:0` (let the OS pick a port). Expose a helper that returns the bound port.
- Open a WS client (`tokio_tungstenite::connect_async`).
- Send `{"kind":"tag.subscribe","paths":["system/sim/sin"]}`.
- Assert at least 2 `tag.update` messages for `system/sim/sin` arrive within 2.5 seconds, each with `quality: "good"` and a numeric `value` between -1.0 and 1.0.
- Send `{"kind":"ping"}` and assert `{"kind":"pong"}` is received within 100ms.
- Send `not json` (raw text) and assert an `error` message is received with `code: "protocol.parse"`. Connection stays open (next ping still works).
- Send `{"kind":"tag.unsubscribe","paths":["system/sim/sin"]}` and assert no further `tag.update` for that path arrives in the next 1.5 seconds.

### Acceptance criteria

- [ ] `cargo build -p openwebhmi-gateway` exits 0.
- [ ] `cargo clippy -p openwebhmi-gateway --all-targets -- -D warnings` exits 0.
- [ ] `cargo test -p openwebhmi-gateway` exits 0; integration test above passes.
- [ ] Manual smoke: `cargo run -p openwebhmi-gateway` + `wscat -c ws://127.0.0.1:8080` produces sin-wave updates at ~1 Hz when subscribed.
- [ ] No `unwrap()` or `expect()` in code paths that handle untrusted input (parse errors, WS frames, etc); use proper error returns.
- [ ] All public items in modules have rustdoc comments.

### Out of scope

- Auth / TLS (Phase 3). Leave a `// TODO Phase 3 auth/TLS` at the connection-accept site only.
- Drivers (Phase 1).
- Persistence / project store (Phase 2).
- Backpressure beyond the mpsc capacity heuristic.

### Risks / gotchas

- `tokio_tungstenite` accepts both Text and Binary frames; we accept only Text. Reject Binary explicitly per the brief, don't ignore it.
- `mpsc::Sender::try_send` returning `TrySendError::Full` is the overflow signal. Drop the message and log; do not block the forwarder task.
- The sim provider should publish even when no subscribers exist (so a late subscriber's `store.get(path)` returns Some). The brief's Boot order ensures this.
- Rust's `f64::sin` is in `std::f64`. The `2π` constant is `std::f64::consts::TAU`.

## Codex log

*(codex — append working notes here)*

### 2026-04-26 13:41  codex
I identified as codex and read `docs/agents/README.md`, `board.md`, this task brief, `crates/protocol`, and `crates/tag-engine`. I started CODEX-B in parallel with CODEX-A.

### 2026-04-26 13:48  codex
Submitted. Implemented `crates/gateway` with CLI, tracing, ctrl-c shutdown, 1 Hz `system/sim/sin` + `system/sim/counter` provider, bounded per-connection outbound queue, JSON protocol parsing, binary-frame rejection, ping/pong, subscribe/unsubscribe, and an in-process WebSocket integration test. Verification: `cargo build -p openwebhmi-gateway`, `cargo clippy --workspace --all-targets -- -D warnings`, and `cargo test --workspace --locked` pass. Manual smoke used `cargo run -p openwebhmi-gateway -- --bind 127.0.0.1:8080` plus a Node WebSocket client; received `pong` and two live `tag.update` messages.

## Claude review

### 2026-04-26 13:55  claude — review pass 1

Spec-compliant. Architecture matches the brief: per-connection mpsc + dedicated writer task; forwarder task per subscription; idempotent subscribe; clean shutdown.

- ✅ Boot order, sim provider (1 Hz interval, `TAU` constant, both paths), parse-error/binary-rejection responses, lagged-broadcast handling — all present.
- ✅ Single integration test bundles all five brief'd cases (sin update ×2, ping, parse-error + recovery, binary rejection, unsubscribe). Tighter than five separate tests, fine.
- 🟡 **Subscribe-time race window**: between `store.get(path)` and the forwarder's `store.subscribe(path)` (which happens inside the spawned task), a new publish can land between them. The forwarder will miss that single publish until the next one. Net effect: a subscriber's first observation might be slightly stale. Acceptable for Phase 0; Phase 1 should subscribe-then-fetch in that order.
- 🟡 Forwarder tasks are `tokio::spawn`'d but not panic-supervised. CODEX-E's `DriverSupervisor` will set the pattern for task supervision generally; revisit then.

No follow-up tasks required.

## Verdict

**Merged** at `75ccb9c`. Two yellow notes (subscribe race, forwarder panic isolation) revisited when the Phase 1 driver supervisor lands.
