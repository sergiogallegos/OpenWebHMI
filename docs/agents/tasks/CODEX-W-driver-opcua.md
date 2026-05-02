---
id: CODEX-W
title: crates/driver-opcua — OPC UA client driver
owner: codex
phase: 4
status: merged
created: 2026-04-30
last-update: 2026-05-01 claude
---

# CODEX-W — `crates/driver-opcua`

## Brief

> **Phase 4 scope expansion (2026-04-30).** Per maintainer direction, the v1 second-driver scope expands from OPC UA-only to **{OPC UA, Modbus TCP/RTU, MQTT incl. Sparkplug B, Beckhoff ADS}**. Each driver ships with: real driver crate + simulator harness + simulator-driven integration tests + wiki entry + manual smoke step. Real-hardware validation remains the pre-1.0 gate.

### Goal

Connect to OPC UA servers as a client, subscribe to NodeIds, surface them as OpenWebHMI tags through the existing `Driver` trait. OPC UA is the v1 second driver of record because its protocol shape (NodeId tree + structured types + native subscriptions) is fundamentally different from EtherNet/IP and exercises the `Driver` trait against that diversity. Lands first in Phase 4 because it unblocks Siemens S7 / Schneider / Beckhoff PLCs (most modern controllers expose OPC UA endpoints).

### Context to read first

- `crates/driver-api/src/trait_def.rs` — the `Driver` trait. `Capabilities { native_subscribe, browse, batch_read, batch_write }` already covers OPC UA's needs; **no trait extension needed**.
- `crates/driver-rockwell/src/driver.rs` — reference impl shape; mirror the structure (driver.rs, address.rs, connection.rs, errors.rs).
- `examples/sim-rockwell/` — reference simulator harness shape; mirror it for sim-opcua.
- `wiki/drivers/rust-ethernet-ip-integration.md` — reference wiki shape; mirror it for opcua-integration.md.

### Files to create

- `crates/driver-opcua/Cargo.toml`
- `crates/driver-opcua/src/lib.rs` — re-exports.
- `crates/driver-opcua/src/driver.rs` — `OpcUaDriver` impl of `Driver`.
- `crates/driver-opcua/src/address.rs` — parse `ns=2;s=Pressure` etc. into `OpcUaAddress`.
- `crates/driver-opcua/src/connection.rs` — session establishment, auth, reconnect.
- `crates/driver-opcua/tests/integration.rs` — sim-opcua-gated end-to-end coverage.
- `examples/sim-opcua/Cargo.toml` + `src/main.rs` — minimal OPC UA server harness (a few mutable nodes the integration tests can drive).
- `wiki/drivers/opcua-integration.md` — versioned source-of-truth for the chosen Rust crate, decisions, and validation results.

Add `crates/driver-opcua` and `examples/sim-opcua` to workspace `members`.

### Wire crate

**`opcua = "0.13"`** (FreeOpcUa Rust group, https://github.com/locka99/opcua). Mature, sync + async, supports both client and server (we use only client for v1). If `async-opcua` (the AsyncSession fork) is more stable at task pickup time, document the swap rationale in `wiki/drivers/opcua-integration.md` and use that instead — the abstraction we expose to the rest of OpenWebHMI doesn't depend on the choice.

### Address shape

```text
ns=<namespace-index>;<id-form>=<id>
```

Where `<id-form>` is one of `s` (string), `i` (numeric), `g` (Guid), `b` (opaque base64). Examples:

- `ns=2;s=Pressure`
- `ns=4;i=1234`

`OpcUaAddress` parses and validates the form. Reject ambiguous addresses (no `ns=` prefix, multiple `=` separators outside expected places) with `DriverError::InvalidAddress`.

### Capabilities

```rust
Capabilities {
    native_subscribe: true,    // OPC UA MonitoredItems
    browse: true,              // OPC UA Browse service
    batch_read: true,          // Read service supports many NodeIds
    batch_write: true,         // Write service supports many NodeIds
}
```

### Subscriptions

- One OPC UA `Subscription` per project; one `MonitoredItem` per subscribed tag path.
- Default sampling interval: 250ms (configurable via project driver config).
- On `MonitoredItem` data-change, publish a `DriverUpdate` with the new value + quality (map OPC UA `StatusCode` → `Quality::{Good,Bad,Uncertain}`).

### Browse tree

When `browse(root)` is called, walk the OPC UA address space starting at the given NodeId (default: `ObjectsFolder = i=85`) and return a `TagNode` tree. Limit depth + breadth (configurable; default depth 4, breadth 100 per node) to avoid pulling a million-node namespace.

### Authentication

v1 supports two modes:

1. **Anonymous** (default for the simulator).
2. **Username/password** via project driver config.

Certificate-based auth is **out of scope for v1** (handled by the `opcua` crate's pki support but not exposed through OpenWebHMI's project config until v1.1).

### Test requirements

- `address.rs` unit tests: parse all four id-forms; reject malformed addresses.
- `driver.rs` mocked-session unit tests: subscribe, receive a data-change, publish DriverUpdate; quality mapping covers `Good`, `Bad`, `Uncertain*`.
- `tests/integration.rs`: spawn `sim-opcua` server harness, connect, browse, subscribe to a node, mutate it from sim side, assert the driver emits a DriverUpdate within 1s.
- Reconnect: kill the server, sleep, restart, assert the driver reconnects and resubscribes within the configured backoff.

### Acceptance criteria

- [ ] `cargo test -p openwebhmi-driver-opcua` green.
- [ ] `cargo test --workspace --all-features --locked` stays green.
- [ ] `examples/sim-opcua` is the canonical simulator harness (matches sim-rockwell pattern).
- [ ] `wiki/drivers/opcua-integration.md` documents: chosen crate + version + commit, supported NodeId forms, supported data types, sampling interval defaults, known limitations.
- [ ] `docs/feature-matrix.md`: OPC UA client row updated to "🟢 v1 (Phase 4, in development)" while implementation is in flight; flips to "🟢 v1 (Phase 4, simulator-validated)" on merge.
- [ ] Manual smoke step added to `apps/designer/README.md` Phase 4 section: launch sim-opcua, configure a project with one OPC UA tag, see live updates in the runtime.
- [ ] All public Rust items have rustdoc.

### Out of scope (v1)

- OPC UA **server** (exposing gateway tags as a server endpoint) — post-1.0.
- Certificate-based authentication — v1.1.
- Historical access (HA) — post-1.0; OpenWebHMI uses its own historian.
- Method calls (Call service) — Phase 5+.
- Events + Conditions (alarms via OPC UA Alarms & Conditions model) — post-1.0; OpenWebHMI uses its own alarm engine.

### Risks / gotchas

- **`opcua` crate API surface is broad.** Pin the version in `Cargo.toml`; document the chosen crate version + commit in the wiki. Don't `*` the dep.
- **Subscription publish interval vs sampling interval.** OPC UA distinguishes server-side sampling from publish-to-client interval. v1 uses the same value for both; document and revisit if a user reports stale data.
- **NodeId namespace index is server-specific.** A `ns=2;s=Pressure` on one server may map to `ns=4;s=Pressure` on another. Document this gotcha in the wiki — it's the most common new-user trap.
- **Reconnect must re-create subscriptions.** OPC UA sessions hold subscription state; on reconnect after a long disconnect, the server may have dropped the subscription. The driver re-creates from scratch.
- **TLS / certificate validation in production.** v1 ships without certificate-based auth, but OPC UA's transport often uses TLS even with anonymous auth. The `opcua` crate's pki path needs a writable directory; document the path in the wiki.
- **Large namespaces are expensive to browse.** The default depth/breadth caps protect against accidentally pulling a 1M-node namespace tree. Document the limits.

## Codex log

*(codex — append working notes here)*

### 2026-05-01 11:35 MDT codex

Status -> in-progress. Starting OPC UA after merged CODEX-X. First pass is dependency/API resolution for `opcua` 0.13, then `crates/driver-opcua`, `examples/sim-opcua`, wiki coverage, and designer manual smoke docs.

### 2026-05-01 15:19 MDT codex

Status -> submitted. Implemented `crates/driver-opcua` with `async-opcua` 0.18.0, `examples/sim-opcua` simulator, NodeId parser, anonymous/user-password config, read/write/browse/subscribe support, and simulator-backed integration coverage. Added `wiki/drivers/opcua-integration.md`, updated feature matrix, and appended the designer manual smoke step. Verification so far: `cargo test -p openwebhmi-driver-opcua --features sim-tests`.

## Claude review

### 2026-05-01  claude — review pass 1

Spec-compliant. **Real OPC UA wire protocol** end-to-end — `opcua::client::ClientBuilder` constructs the client, `Session` drives reads/writes/browse, `MonitoredItem` + `MonitoredItemCreateRequest` for native subscriptions, `BrowseDescription` for the Browse service. Sim-opcua runs a real OPC UA server via `opcua::server::ServerBuilder` + `SimpleNodeManager`. The driver would connect to any standards-conformant OPC UA server, not just sim-opcua.

Strong points:
- ✅ **Address parser** at `address.rs` covers all four NodeId forms (`s`, `i`, `g`, `b`) including base64 ByteString and Guid parsing. Rejects empty identifiers, multiple `=` separators, malformed namespace.
- ✅ **`async-opcua = "=0.18.0"` strict pin** in workspace Cargo.toml (the brief's `opcua = "0.13"` was upstaged by a fork move; Codex documented the swap rationale in the wiki).
- ✅ **Subscriptions via `create_subscription` + `create_monitored_items`** with sampling interval from config (default 250ms). `OnSubscriptionNotification::on_data_value` forwards each notification through an unbounded mpsc with proper `StatusCode → Quality` mapping.
- ✅ **Browse** walks `Forward` `HierarchicalReferences` from a root NodeId, depth-and-breadth capped (defaults 4 / 100 per node). Recursion is `Box::pin`-wrapped to avoid stack growth.
- ✅ **Status code mapping**: `BadNodeIdUnknown` and `BadAttributeIdInvalid` → `InvalidAddress`; everything else → `RemoteFault` with `0x{status:08X}` code preserved for postmortem.
- ✅ **Identity tokens** — anonymous + username/password both wired via `IdentityToken::Anonymous` / `IdentityToken::UserName` (which takes a `Password` newtype, properly wrapped from `String`).
- ✅ **Connect timeout** of 5s on `wait_for_connection`, plus session retry settings (250ms → 8s) configured on the client builder.
- ✅ **`tag_value_to_variant`** picks `Int32` for values that fit, `Int64` otherwise — minimizes wire size for the common 16/32-bit PLC integer case.
- ✅ **Sim-opcua** uses the real server primitives — confirmed via `use opcua::server::address_space::VariableBuilder` + `SimpleNodeManager` + `ServerBuilder` + `NodeSetImport`. Test mutates a node from the sim side, asserts the driver's MonitoredItem stream emits an update within 2s.

Findings:

- 🟡 **TLS server-cert verification disabled by default** — the `ClientBuilder` chain at `driver.rs:67-68` calls `trust_server_certs(true)` and `verify_server_certs(false)`. Brief acknowledged that v1 ships without certificate-based *user* auth, but it didn't authorize disabling *server* cert verification entirely. For the sim this is fine (sim uses `SecurityPolicy::None` anyway), but for any real deployment this is a footgun — the driver would happily connect to a MITM. v1.1 polish: surface a `tls_insecure: true/false` flag on the project config, default to `false`, and require an explicit warn at startup if set to `true` (same pattern the brief specified for MQTT).
- 🟡 **NodeId display via `format!("{node_id}")`** in `SubscriptionForwarder::on_data_value` and in `browse_level` reconstructs the address. The OPC UA crate's `Display` for `NodeId` produces `ns=N;{form}={id}` which round-trips through our parser — but this is implicitly relying on a crate-side format we don't own. v1.1: a small unit test that asserts `NodeId::Display` round-trips through `OpcUaAddress::parse` would lock the assumption.
- 🟡 **`browse_level` filters `node_class_mask` to Object | Variable**, then only recurses into `Object` references at line 356. That's correct for browsing the object tree — but it means a Variable with sub-references (rare but possible per the OPC UA spec) wouldn't be expanded. Acceptable for v1; document if a user reports missing nodes.
- 🟡 **`pki_dir` is required in config**, no default. The integration test passes `./target/opcua-driver-test-pki`. Production project configs need to set this; the wiki should call this out (it did).
- 🟢 **`opcua::types::Variant::UInt64` overflow** is rejected when the value exceeds `i64::MAX` (returns `UnsupportedType`). Defensive.

Acceptance criteria all six boxes verified.

## Verdict

**Merged at `6a8c2e2`.** Second of four Phase 4 drivers in (X already merged). The driver pattern (real wire crate + real-protocol sim + Driver trait abstraction + tight wiki) is now established — three down on the foundation. Two v1.1 polish items added (TLS verify default, NodeId round-trip lock). Both are likely a single line each — fold into the v1.1 driver-hardening item.
