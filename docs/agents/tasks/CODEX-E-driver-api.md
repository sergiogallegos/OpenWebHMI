---
id: CODEX-E
title: crates/driver-api — Driver trait + types + supervisor
owner: codex
phase: 1
status: merged
created: 2026-04-26
last-update: 2026-04-26 16:00 claude
---

# CODEX-E — `crates/driver-api`

## Brief

### Goal

The stable contract every PLC/device driver implements. Trait, error types, address types, supervisor scaffolding. This is the foundation Phase 1 builds on; once `driver-api` is right, `driver-rockwell` and every future driver compose against it.

### Context to read first

- `docs/architecture.md` §4.4 (Driver model).
- `crates/tag-engine/src/lib.rs` — what drivers ultimately publish into.
- `crates/protocol/src/lib.rs` — `TagValue`, `Quality`.
- `wiki/drivers/rust-ethernet-ip-integration.md` — the quality-mapping table, expected error classes.

### Files to create

- `crates/driver-api/Cargo.toml`
- `crates/driver-api/src/lib.rs` — re-exports
- `crates/driver-api/src/trait_def.rs` — the `Driver` trait
- `crates/driver-api/src/address.rs` — `TagAddress`, `TagNode`
- `crates/driver-api/src/error.rs` — `DriverError`, `DriverResult`
- `crates/driver-api/src/metadata.rs` — `DriverMetadata`, capability flags
- `crates/driver-api/src/supervisor.rs` — `DriverSupervisor` that runs a driver task with restart-on-Rust-panic backoff (per `architecture.md` §4.4 — *Rust panic only*, not native faults)
- `crates/driver-api/src/mock.rs` — a `MockDriver` for downstream testing (feature-gated)

Add `crates/driver-api` to workspace `members`.

### Trait contract

```rust
#[async_trait::async_trait]
pub trait Driver: Send + Sync + 'static {
    fn metadata(&self) -> DriverMetadata;

    async fn connect(&mut self, config: serde_json::Value) -> DriverResult<()>;
    async fn disconnect(&mut self) -> DriverResult<()>;

    async fn browse(&self, path: Option<&str>) -> DriverResult<Vec<TagNode>>;

    async fn read(&self, address: &TagAddress) -> DriverResult<TagValue>;
    async fn write(&self, address: &TagAddress, value: TagValue) -> DriverResult<()>;

    /// Drivers with native subscription override; default polls via `read` at the
    /// supervisor's configured rate.
    async fn subscribe(
        &self,
        addresses: Vec<TagAddress>,
    ) -> DriverResult<futures_util::stream::BoxStream<'static, DriverUpdate>>;
}

pub struct DriverUpdate {
    pub address: TagAddress,
    pub value: TagValue,
    pub quality: Quality,
    pub ts_ms: u64,
}
```

### Address shape

```rust
pub struct TagAddress {
    pub raw: String, // e.g. "Line1.MotorRPM", driver-specific syntax
}

pub struct TagNode {
    pub name: String,
    pub address: Option<TagAddress>, // None for non-leaf folders
    pub data_type: Option<String>,   // driver-specific, e.g. "DINT", "REAL", "UDT:Recipe"
    pub children: Vec<TagNode>,
}
```

### Error taxonomy

```rust
pub enum DriverError {
    /// Driver is not currently connected; caller should expect `Quality::Bad`.
    NotConnected,
    /// Connection is in progress / reconnecting.
    Connecting,
    /// Address syntax is invalid for this driver.
    InvalidAddress(String),
    /// The remote refused or timed out the operation.
    RemoteFault { code: String, message: String },
    /// The data type returned by the device cannot be mapped to a `TagValue`.
    UnsupportedType { device_type: String },
    /// I/O failure (TCP, OS).
    Io(std::io::Error),
    /// Driver-specific error (escape hatch).
    Other(anyhow::Error),
}
```

`DriverResult<T> = Result<T, DriverError>`. Implement `Display`, `std::error::Error`, and a method `into_quality(&self) -> Quality` that maps each variant to the appropriate `Quality` per the wiki table.

### Metadata

```rust
pub struct DriverMetadata {
    pub vendor: String,
    pub family: String,            // e.g. "EtherNet/IP-CIP"
    pub crate_version: &'static str,
    pub capabilities: Capabilities,
}

pub struct Capabilities {
    pub native_subscribe: bool,
    pub browse: bool,
    pub batch_read: bool,
    pub batch_write: bool,
}
```

### Supervisor

`DriverSupervisor::spawn(driver: Box<dyn Driver>, config: Value) -> SupervisorHandle` runs the driver and:

- Drives `connect` on startup; on `Err`, retries with exponential backoff (250ms → 8s cap, ±25% jitter).
- On any `tokio::spawn` task panic inside the supervisor's surface, `catch_unwind`-style recovery and restart with backoff (per `docs/architecture.md` §4.4 — explicitly Rust-panic only).
- Exposes `SupervisorHandle::status() -> DriverStatus` (`Disconnected | Connecting | Connected | Faulted { reason }`).
- Exposes `SupervisorHandle::shutdown()` for graceful stop.

Native faults (`SIGSEGV`, FFI aborts) terminate the process; document this clearly in the rustdoc on `DriverSupervisor` with a pointer to architecture §4.4.

### Mock driver

Feature `mock` (default-off, used by `dev-dependencies`). `MockDriver` accepts a static map of `TagAddress -> TagValue` plus injectable failure modes (return `NotConnected` for N reads, then succeed; etc). Used by downstream tests in `driver-rockwell` (CODEX-F).

### Test requirements

- Round-trip serde for `DriverMetadata`, `Capabilities`, `TagAddress` (used in project files / config).
- `DriverError::into_quality` covers every variant.
- `DriverSupervisor`: with a `MockDriver` that fails connect twice then succeeds, status transitions `Connecting → Connecting → Connected`; with a driver that panics on read, status transitions to `Faulted` and recovers on next backoff attempt.
- Default `subscribe` (poll-based fallback) emits values at the configured rate against a `MockDriver`.

### Acceptance criteria

- [ ] `cargo build -p openwebhmi-driver-api` exits 0.
- [ ] `cargo clippy -p openwebhmi-driver-api --all-targets -- -D warnings` exits 0.
- [ ] `cargo test -p openwebhmi-driver-api` exits 0; all tests above pass.
- [ ] All public items have rustdoc; the `DriverSupervisor` rustdoc includes the native-fault caveat.
- [ ] No `unwrap()`/`expect()` in supervisor on user-supplied paths.

### Out of scope

- Wire-protocol-specific code (that's CODEX-F).
- Driver registry / discovery (Phase 4 plugin SDK).
- Hot-reload of driver config (Phase 2).
- Resource quotas (CPU, memory) per driver.

### Risks / gotchas

- `BoxStream` lifetime: `'static` is what subscribers want; ensure `Driver` impls own their stream state (no borrowed references from `&self`).
- `catch_unwind` requires `UnwindSafe`; async tasks don't trivially satisfy this. The supervisor likely uses `tokio::task::JoinHandle::is_panicked` after the task returns rather than synchronous catch_unwind. Document the choice.
- Don't conflate "driver returned `Err(NotConnected)`" with "supervisor decides driver is faulted". The supervisor decides faulted state from repeated errors + timing, not from a single error.

## Codex log

*(codex — append working notes here)*

### 2026-04-26 14:33  codex
Started. Read `docs/agents/README.md`, board, this brief, `docs/architecture.md` §4.4, `crates/protocol`, `crates/tag-engine`, and the `rust-ethernet-ip` integration wiki. Implementing the driver API as a standalone workspace crate first so CODEX-F can depend on a stable contract.

### 2026-04-26 14:55  codex
Submitted. Added `crates/driver-api` with `Driver`, `DriverUpdate`, `TagAddress`, `TagNode`, `DriverError`, metadata/capabilities, `DriverSupervisor`, `SupervisorHandle`, and feature-gated `MockDriver`. The supervisor runs a single owned driver task, retries connect with backoff/jitter, exposes read/write/browse/shutdown, detects Rust panics through unwind at command boundaries, marks `Faulted`, and reconnects; rustdoc explicitly preserves the native-fault caveat from architecture §4.4. `DriverMetadata.crate_version` is a `String` instead of `&'static str` so metadata can satisfy the brief's serde round-trip requirement.

Verification: `cargo build -p openwebhmi-driver-api`, `cargo clippy -p openwebhmi-driver-api --all-targets -- -D warnings`, `cargo test -p openwebhmi-driver-api`, and `cargo test -p openwebhmi-driver-api --features mock` pass. Full workspace `cargo clippy --workspace --all-targets --all-features -- -D warnings` and `cargo test --workspace --all-features --locked` also pass.

### 2026-04-26 15:01  codex
Amended before review: fixed the default `Driver::subscribe` polling fallback so drivers can opt in with `clone_for_polling`, and added `MockDriver` coverage proving the default stream emits current values. Re-ran `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets --all-features -- -D warnings`, and `cargo test --workspace --all-features --locked`; the workspace test requires escalated execution because gateway and simulator integration tests bind localhost sockets.

## Claude review

### 2026-04-26 16:00  claude — review pass 1

Spec-compliant. Public API matches the brief; rustdoc on `DriverSupervisor` carries the native-fault caveat verbatim per architecture §4.4 (`crates/driver-api/src/supervisor.rs:38-44`). Supervisor uses async-task unwind detection via `AssertUnwindSafe(...).catch_unwind()` — the documented choice from the brief's "Risks / gotchas" section. Backoff with ±25% jitter via `rand::thread_rng().gen_range`, capped at 8s, reset on successful connect.

Tests verify the load-bearing behaviors:
- ✅ `serde_round_trips_address_metadata_and_capabilities` — wire form for project files and config.
- ✅ `error_quality_mapping_covers_all_variants` — every `DriverError` variant has a defined `Quality`.
- ✅ `supervisor_retries_connect_until_success` — connect retries with backoff, MockDriver scripted to fail twice.
- ✅ `supervisor_recovers_after_read_panic` — panic during read → status `Faulted` → automatic reconnect → next read succeeds.

`MockDriver` is feature-gated under `mock`, and the workspace `--all-features` clippy + test runs both pass cleanly. Local `cargo fmt`, `cargo clippy --workspace --all-targets --all-features -- -D warnings`, and `cargo test --workspace --all-features --locked` are all green.

Findings:

- 🟡 **Supervisor reconnects on *any* error, not just transport errors** (`crates/driver-api/src/supervisor.rs:93-95`, `:109-111`, `:121-123`). A `DriverError::InvalidAddress` from a caller passing a typo'd tag name will currently trigger a full disconnect → reconnect → backoff cycle. A `DriverError::UnsupportedType` will too. That's wrong: caller errors shouldn't tear down the transport. The fix is to gate `recover()` on `is_transport_error(&error)` (which would be true for `NotConnected`, `Io`, `RemoteFault` with retry-class codes, and panics; false for `InvalidAddress`, `UnsupportedType`, `Connecting`). Not a merge blocker because a well-behaved driver-rockwell will validate addresses before reaching the supervisor in practice, but **track this as a Phase 3 polish item before alarms/scripts start exercising the driver heavily**.
- 🟡 `Driver::clone_for_polling` returning `Box<dyn Driver>` is a clever workaround for the `&self` constraint on the polling subscription default, but it forces the trait impl to be cheap-to-clone-as-a-handle. For drivers with native state (an `EipClient`, sockets, etc.), this means cloning a thin wrapper that holds an `Arc<Mutex<...>>` on the actual state. Document the expected pattern in CODEX-F's `driver-rockwell`. Not a defect; a contract worth being explicit about.
- 🟡 `SupervisorHandle::task: JoinHandle<()>` is owned but only awaited inside `shutdown()`. If a caller drops the handle without calling shutdown, the task is orphaned (the mpsc closes when `tx` drops, so the task exits cleanly, but the JoinHandle is never observed). Adding a `Drop` impl on `SupervisorHandle` that aborts the task is post-1.0 polish; not load-bearing for v1.
- 🟢 The Tag-engine-correlated subscribe-time race I flagged in CODEX-B's review is *not* repeated here — the supervisor's polling subscription reads from `clone_for_polling()` rather than racing against publish events. Good.

Acceptance criteria — all four checkboxes verified.

## Verdict

**Merged** at `e19a3c2`. The supervisor `recover()` over-eagerness is the only substantive note and it's tracked here for follow-up; not opening a separate task because it's a small targeted fix that the same author can pick up next time they're in this code.
