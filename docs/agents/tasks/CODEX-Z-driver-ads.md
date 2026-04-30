---
id: CODEX-Z
title: crates/driver-ads — Beckhoff TwinCAT (ADS) client driver
owner: codex
phase: 4
status: open
created: 2026-04-30
last-update: 2026-04-30 claude
---

# CODEX-Z — `crates/driver-ads`

## Brief

> **Phase 4 scope expansion (2026-04-30).** v1 driver set expanded; Beckhoff ADS bumped from "🔵 maybe" to v1.

### Goal

Connect to **Beckhoff TwinCAT** runtimes via ADS (Automation Device Specification — Beckhoff's proprietary-but-documented protocol). Surfaces TwinCAT symbols (PLC + NC + I/O variables) as OpenWebHMI tags. Beckhoff is the dominant non-Rockwell PLC platform in motion control and high-precision automation; ADS unlocks that segment.

### Context to read first

- `crates/driver-api/src/trait_def.rs` — the `Driver` trait. ADS supports notifications (the moral equivalent of native subscriptions), batch reads, and symbol browsing — fits the trait cleanly.
- `crates/driver-rockwell/src/driver.rs` — closest analog (tag/symbol-based, not register-based).
- ADS protocol reference: Beckhoff Information System (https://infosys.beckhoff.com/), specifically the "TC3 ADS Communication" section. Wiki should cite specific page IDs.

### Files to create

- `crates/driver-ads/Cargo.toml`
- `crates/driver-ads/src/lib.rs` — re-exports.
- `crates/driver-ads/src/driver.rs` — `AdsDriver` impl of `Driver`.
- `crates/driver-ads/src/address.rs` — parse `<port>:<symbol>` → `AdsAddress`.
- `crates/driver-ads/src/connection.rs` — AMS net id + port resolution; route table awareness.
- `crates/driver-ads/src/symbols.rs` — symbol upload + type resolution + struct decoding.
- `crates/driver-ads/tests/integration.rs` — sim-ads-gated end-to-end coverage.
- `examples/sim-ads/Cargo.toml` + `src/main.rs` — minimal ADS responder harness (using the `ads` crate's server feature, or a stubbed responder).
- `wiki/drivers/ads-integration.md`.

Add to workspace `members`.

### Wire crate

**`ads = "0.7"`** (https://crates.io/crates/ads, https://github.com/birkenfeld/ads-rs). Mature, sync API with tokio adapter. Includes both client and server primitives; we use only client for v1. Document the chosen version + commit in wiki.

If `ads-async` (any actively-maintained async fork) is more stable at task pickup time, document the swap rationale and use it. The address shape and trait surface don't depend on the choice.

### Address shape

```text
<port>:<symbol-name>
```

Where `<port>` is the ADS port (851 = TwinCAT 3 PLC runtime; 350 = NC; 27905 = I/O; etc.) and `<symbol-name>` is the dotted symbol path (`MAIN.fbMotor.fActualSpeed`). Examples:

- `851:MAIN.fbMotor.fActualSpeed`
- `851:GVL.bRunning`
- `350:NC.Axes.Axis_1.NcToPlc.ActPos`

Validation: port must be a u16; symbol must be non-empty and contain only `[A-Za-z0-9_.[]]`.

### Capabilities

```rust
Capabilities {
    native_subscribe: true,    // ADS device notifications
    browse: true,              // ADS upload symbol info (UploadInfo + Upload)
    batch_read: true,          // ADS sumup read
    batch_write: true,         // ADS sumup write
}
```

### Subscriptions

ADS device notifications support cyclic + on-change modes. v1 uses **on-change** (the runtime's tag-update model) with a fallback to cyclic at the configured `poll_rate_ms` (default 250ms) for symbols that don't support on-change.

Notification handle is allocated per subscribed symbol; on disconnect, all handles are invalidated and re-allocated on reconnect.

### Symbol resolution

On connect, the driver issues `UploadInfo` + `Upload` to fetch the full symbol table. This populates the browse tree. For each subscribed tag, the driver looks up `(handle, type, size)` from the symbol table; uses `IndexGroup = 0xF003 (ReadByName)` if the symbol isn't pre-resolved.

### AMS net id

ADS connections require an **AMS net id** (e.g. `192.168.1.10.1.1`) and an IP address. v1 supports both:
- Static `(ams_net_id, ip)` pair in project config.
- Local "discovery via route table" — read from `/etc/TwinCAT3/StaticRoutes.xml` or the Windows registry path. **Discovery is best-effort**; the explicit pair always works.

### Test requirements

- `address.rs` unit tests: parse `<port>:<symbol>`; reject malformed.
- `symbols.rs` unit tests: parse `UploadInfo` response; build symbol table; type-resolve a primitive and a struct.
- `driver.rs` mocked-`ads`-client unit tests: subscribe, fire a notification, publish DriverUpdate; reconnect re-establishes notification handles.
- `tests/integration.rs`: spawn `sim-ads` responder, connect via AMS net id, subscribe to a symbol, mutate from sim side, assert driver emits a DriverUpdate.

### Acceptance criteria

- [ ] `cargo test -p openwebhmi-driver-ads` green.
- [ ] `cargo test --workspace --all-features --locked` stays green.
- [ ] `examples/sim-ads` runs; integration tests connect against it.
- [ ] `wiki/drivers/ads-integration.md` covers: AMS routing, symbol upload, type system mapping (PLC types → TagValue), ports table.
- [ ] `docs/feature-matrix.md`: Beckhoff TwinCAT (ADS) row flipped to "🟢 v1 (Phase 4, in development)".
- [ ] Manual smoke step added: launch sim-ads, configure an ADS driver, see symbol values in the runtime.
- [ ] All public Rust items have rustdoc.

### Out of scope (v1)

- **ADS server mode** (the gateway acts as an ADS device): post-1.0.
- **TwinCAT XAR vs XAE distinction in browse**: the driver reads symbols from whatever runtime is on the configured port; doesn't distinguish development vs runtime hosts.
- **ADS-Sumup writes with mixed types**: v1 supports same-type batch writes only. Mixed-type Sumup writes (which TwinCAT does support) — post-1.0.
- **Persistent variables** (PERSISTENT keyword): readable as ordinary symbols, not specially handled — fine.
- **TwinCAT route management** (adding/removing routes from OpenWebHMI): post-1.0; user manages via Beckhoff tools.

### Risks / gotchas

- **AMS net id formatting** is the dominant new-user trap (it's a 6-octet identifier, NOT a 4-octet IP, and they often look similar). Document with two examples in the wiki.
- **Symbol name case sensitivity**: TwinCAT 3 is case-insensitive in source; ADS protocol is case-sensitive in queries. Address parser preserves case as written by the user.
- **Notification handle leak**: each subscription allocates a handle on the runtime. The driver MUST release handles on unsubscribe + on disconnect (the runtime has a finite handle pool, and a leak crashes it after a few hours).
- **Struct decoding** depends on the symbol-table type info. v1 supports primitives (BOOL, INT, DINT, REAL, LREAL, STRING) and arrays of primitives. Struct types are flattened to dotted child symbols (`fbMotor.fActualSpeed` is its own symbol; the driver doesn't materialize the struct value as a single tag). Document.
- **Beckhoff licensing**: ADS itself is open and documented; the TwinCAT runtime is licensed Beckhoff software. The simulator harness uses the `ads` crate's server primitives (no Beckhoff license needed) but documenting that distinction in the wiki helps users.
- **Cross-platform serial / IP routing**: ADS over TCP works on macOS/Linux/Windows; the AMS routing layer historically prefers Windows. Document any platform quirks discovered during validation.

## Codex log

*(codex — append working notes here)*

## Claude review

*(claude — after submission)*

## Verdict

*(claude — final disposition)*
