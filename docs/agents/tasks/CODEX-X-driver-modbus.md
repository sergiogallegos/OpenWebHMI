---
id: CODEX-X
title: crates/driver-modbus — Modbus TCP + RTU client driver
owner: codex
phase: 4
status: open
created: 2026-04-30
last-update: 2026-04-30 claude
---

# CODEX-X — `crates/driver-modbus`

## Brief

> **Phase 4 scope expansion (2026-04-30).** v1 driver set expanded; Modbus TCP/RTU bumped from post-1.0 to v1.

### Goal

Modbus TCP **and** Modbus RTU (serial), in one crate, two transports. Modbus is the most common legacy industrial protocol — every Schneider, Eaton, AB Micro, GE Versamax, and a thousand instruments speak it. v1 ships master/client only (the gateway polls slaves). Slave mode (the gateway looks like a Modbus device) is post-1.0.

### Context to read first

- `crates/driver-api/src/trait_def.rs` — the `Driver` trait.
- `crates/driver-rockwell/src/driver.rs` — reference impl shape.
- Modbus protocol reference: https://modbus.org/specs.php (the public spec; cite specific page numbers in the wiki where relevant).

### Files to create

- `crates/driver-modbus/Cargo.toml`
- `crates/driver-modbus/src/lib.rs` — re-exports.
- `crates/driver-modbus/src/driver.rs` — `ModbusDriver` impl of `Driver`; transport-agnostic.
- `crates/driver-modbus/src/address.rs` — parse `<unit>/<area>/<addr>:<count>:<type>` → `ModbusAddress`.
- `crates/driver-modbus/src/connection.rs` — `Connection` enum: `Tcp { host, port }` | `Rtu { device, baud, parity, data_bits, stop_bits }`.
- `crates/driver-modbus/tests/integration.rs` — sim-modbus-gated end-to-end coverage.
- `examples/sim-modbus/Cargo.toml` + `src/main.rs` — minimal Modbus TCP slave harness (writable holding registers, deterministic input registers).
- `wiki/drivers/modbus-integration.md`.

Add to workspace `members`.

### Wire crate

**`tokio-modbus = "0.16"`** (https://github.com/slowtec/tokio-modbus). Mature, async, supports both TCP and RTU transports out of the box. Pin the version + document in the wiki.

### Address shape

```text
<unit-id>/<area>/<addr>[:<count>][:<datatype>]
```

Where:
- `<unit-id>` — Modbus unit identifier (1..247; 255 for TCP-only).
- `<area>` — one of `coils` (FC1/5/15), `discrete` (FC2), `input` (FC4), `holding` (FC3/6/16).
- `<addr>` — register/coil address (0-based).
- `<count>` — optional, default 1; for multi-register reads (string, 32-bit, 64-bit).
- `<datatype>` — optional, default depends on area: `bool` (coils, discrete), `u16` (input, holding); other valid: `i16`, `u32`, `i32`, `u32_le`, `i32_le`, `f32`, `f32_le`, `f64`, `string`.

Examples:
- `1/holding/100:2:f32` — float at register 100 on unit 1, big-endian.
- `1/coils/0` — coil 0 on unit 1.
- `2/holding/200:5:string` — 5-register UTF-8 string starting at 200.

### Capabilities

```rust
Capabilities {
    native_subscribe: false,   // Modbus has no pub/sub; poll only
    browse: false,             // No browse service in Modbus
    batch_read: true,          // Read-many registers in one FC
    batch_write: true,         // Write-multiple-coils / -registers
}
```

### Polling

- Per-unit poll cycle, driven by project driver config (`poll_rate_ms`, default 250ms).
- Group reads by adjacent register ranges (≤125 holding registers per FC3 read; ≤2000 coils per FC1 read). Within a group, one TCP frame per cycle.
- Coalesce overlapping ranges from the subscribed-tag set.

### Endianness

Default big-endian for multi-register integers/floats (Modbus convention). Address suffix `_le` selects little-endian. Document both in the wiki — endianness is the second-most-common new-user trap (after wrong unit-id).

### Test requirements

- `address.rs` unit tests: parse all area + datatype combinations; reject malformed.
- `driver.rs` mocked-tokio-modbus unit tests: read holding registers, write coils, error mapping (`ExceptionCode::IllegalDataAddress` → `DriverError::InvalidAddress`).
- `tests/integration.rs`: spawn `sim-modbus` TCP slave, connect, subscribe to several tags across `holding` and `coils`, assert reads return expected values; write a coil, assert sim-side observes the change.
- RTU: integration tests are TCP-only (RTU requires a serial pair, harder to CI); RTU coverage is unit-test only. Document this in the wiki.

### Acceptance criteria

- [ ] `cargo test -p openwebhmi-driver-modbus` green.
- [ ] `cargo test --workspace --all-features --locked` stays green.
- [ ] `examples/sim-modbus` runs; integration tests connect against it.
- [ ] `wiki/drivers/modbus-integration.md` documents: chosen crate + version + commit, area/datatype matrix, endianness defaults, known limitations.
- [ ] `docs/feature-matrix.md`: Modbus TCP and Modbus RTU rows flipped to "🟢 v1 (Phase 4, in development)".
- [ ] Manual smoke step added: launch sim-modbus, configure a Modbus driver, see register values in the runtime.
- [ ] All public Rust items have rustdoc.

### Out of scope (v1)

- **Modbus slave/server mode** — post-1.0.
- **Modbus ASCII** transport — post-1.0; ASCII is a serial-line variant that's largely deprecated.
- **Serial port enumeration UI** in the designer — Phase 5+; for v1 the user types the device path.
- **Function codes 7, 8, 11, 12, 17, 24** (diagnostic / report-slave-id) — post-1.0.

### Risks / gotchas

- **Endianness ambiguity** is the dominant new-user trap. Document both byte order (BE vs LE) and word order (CDAB vs ABCD vs DCBA) in the wiki — `tokio-modbus` exposes them; the address suffix maps to specific orderings.
- **Unit ID confusion**: TCP often uses 255 as a passthrough; RTU requires the actual slave id. The driver doesn't second-guess; the user provides the value.
- **Reconnect on idle drops**: Modbus TCP brokers often kill idle connections after 60s. Driver must reconnect transparently.
- **Slow slaves**: a Modbus slave with a 500ms response time will starve a 250ms poll cycle. Per-unit poll rate (configurable) avoids this. Document the gotcha.
- **RTU on macOS/Windows**: serial port support varies. v1 ships with `tokio-modbus`'s default `tokio-serial` backend; document the platform limitations.

## Codex log

*(codex — append working notes here)*

## Claude review

*(claude — after submission)*

## Verdict

*(claude — final disposition)*
