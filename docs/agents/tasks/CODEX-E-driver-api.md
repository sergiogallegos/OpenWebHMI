---
id: CODEX-E
title: crates/driver-api — Driver trait + types + supervisor
owner: codex
phase: 1
status: open
created: 2026-04-26
last-update: 2026-04-26 claude
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

## Claude review

*(claude — after submission)*

## Verdict

*(claude — final disposition)*
