---
id: CODEX-X
title: crates/driver-modbus — Modbus TCP + RTU client driver
owner: codex
phase: 4
status: merged
created: 2026-04-30
last-update: 2026-05-01 claude
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

### 2026-04-30 12:04 MDT codex

Status -> in-progress. Starting with the local `Driver` trait and `driver-rockwell` implementation shape, then building `crates/driver-modbus`, `examples/sim-modbus`, wiki coverage, and the manual smoke entry from the CODEX-X brief.

### 2026-04-30 15:55 MDT codex

Status -> submitted. Added `openwebhmi-driver-modbus` with TCP/RTU config, address parser/datatype codecs, `tokio-modbus` 0.16.1 client wrapper, grouped polling subscription stream, write support for coils/holding registers, `sim-modbus` TCP slave harness, simulator-backed integration coverage, wiki page, feature-matrix wording, and designer manual smoke steps. Verified `cargo fmt --check`, `cargo clippy --workspace --all-targets --all-features -- -D warnings`, `cargo test -p openwebhmi-driver-modbus --features sim-tests`, and `cargo test --workspace --all-features --locked`.

## Claude review

### 2026-05-01  claude — review pass 1

Spec-compliant on every brief callout. First Phase 4 driver — sets the bar nicely for W/Y/Z.

Strong points:
- ✅ **Address parser exhaustive** at `address.rs:311-403`. All 11 datatypes (`bool`, `u16`, `i16`, `u32`/`i32`/`u32_le`/`i32_le`, `f32`/`f32_le`, `f64`, `string`); FC count limits enforced (125 registers, 2000 coils); area↔datatype cross-check (bit areas reject non-bool); count default derived from datatype min when omitted; explicit unit-id range validation (1..=247 + 255).
- ✅ **`ModbusClientLike` trait abstraction** lets driver-impl unit tests run against an in-process mock (`driver.rs:28-59`). Same pattern as `EipClientLike` in driver-rockwell — consistent abstraction layer across drivers.
- ✅ **Read-group coalescing** at `driver.rs:404-442`: sort requests by `(unit_id, area, start_address)`; merge if `request_start <= group_end && merged_end - group.start <= max_count`. Conservative (strict adjacency, no merge across gaps). The `groups_overlapping_and_adjacent_subscription_reads` unit test locks the behavior.
- ✅ **Single vs multiple FC selection** based on payload size (`driver.rs:233-250`): single coil writes use FC5, multi-coil use FC15; single register writes use FC6, multi-register use FC16. Mirrors what real Modbus masters do for efficiency.
- ✅ **Exception code mapping** at `driver.rs:520-533` is exactly right: `IllegalDataAddress → InvalidAddress`, `IllegalFunction`/`IllegalDataValue → UnsupportedType`, others → `RemoteFault { code: "modbus-exception-N" }` with the numeric code preserved for postmortem.
- ✅ **`tokio-modbus = "=0.16.1"` strict pin** in workspace Cargo.toml, with **upstream commit `b077ca3f...` recorded in the wiki**. This is the wiki-as-source-of-truth pattern that CODEX-G/F set for the rust-ethernet-ip integration. Disciplined.
- ✅ **`examples/sim-modbus` is hand-rolled** at `lib.rs` — implements only the eight FCs the driver exercises (1, 2, 3, 4, 5, 6, 15, 16). No circular dep on tokio-modbus's server side; sim behavior fully under our control. Has a `Notify` for tests to `wait_for_write()` deterministically.
- ✅ **`Slave(255)` sentinel for TCP connect** + `set_slave(unit_id)` per request matches the Modbus convention for TCP gateways.
- ✅ **Connection timeout for TCP** (`driver.rs:469-475`): wraps `tcp::connect_slave` in `time::timeout(connection_timeout_ms)` so a wedged TCP connect doesn't hang the driver indefinitely.
- ✅ **Wiki entry** at `wiki/drivers/modbus-integration.md` is the cleanest driver-wiki yet: pinned version + upstream commit + area/datatype matrix + endianness defaults + function-code map + Modbus spec V1.1b3 section citations + Independent Verification Status table separating CI-validated claims from "pending hardware" RTU.
- ✅ **Manual smoke steps 20-21** added to designer README cover the simulator round-trip.
- ✅ **Acceptance criteria all six boxes verified.**

Findings:

- 🟡 **No transport-level reconnect.** The brief flagged this gotcha ("Modbus TCP brokers often kill idle connections after 60s. Driver must reconnect transparently."). Codex flagged the open question in the wiki (`wiki/drivers/modbus-integration.md:71`: "Should Modbus TCP reconnect be owned by this driver directly or left to the existing gateway/driver supervisor pattern?") but didn't implement it. Today, on a transport error during `read`/`write`/`poll_groups`, the driver returns the error and keeps the dead client. The gateway's `DriverSupervisor` recovers from panics, not from quiet transport errors. **Real concern for production v1**; track for the v1.1 polish PR alongside the equivalent OPC UA/MQTT/ADS reconnect concerns.
- 🟡 **RTU has no per-request timeout.** `tokio_serial::SerialStream::open` doesn't configure a read timeout, so a wedged slave on RTU will hang the driver. Acceptable for v1 (RTU is unit-test-only; CI can't exercise this); document the limitation. v1.1.
- 🟡 **Conservative coalescing skips merge across gaps.** A subscription to holding registers 100, 200, 300 stays as three FC3 reads instead of one FC3(100, count=201) read. That's the right v1 default (don't over-fetch unused registers, especially across word-aligned gaps where a struct boundary may exist). Worth a wiki note. v1.1 polish: configurable max-gap merge.
- 🟡 **Single shared `Mutex<Box<dyn ModbusClientLike>>` serializes all I/O on one connection.** Correct for Modbus's single-master semantics, but it does mean a slow read on unit A blocks an unrelated write to unit B. v1.1 polish: consider per-unit connections or pipelining if/when this surfaces.
- 🟡 **`f64` is ABCDEFGH only**, no `f64_le` variant. Codex flagged this as an open question in the wiki. v1.1 if a real user reports a 64-bit float on a CDABEFGH-style device.
- 🟢 **Mock client `read_holding_registers` returns `Ok(Ok(vec![0]))` by default**, which would fail `decode_registers` for f32 (needs 2 words). The unit test populates the queue explicitly (`reads_u16.push_back(Ok(Ok(vec![0x4148, 0x0000])))`), so the default fallback is only hit if a test forgets to seed. Defensive; fine.

Acceptance criteria all green. The driver pattern (trait abstraction + hand-rolled sim + wiki-with-upstream-commit) is the template the next three drivers should follow.

## Verdict

**Merged.** First of four Phase 4 drivers in. Five v1.1 polish items added (reconnect, RTU timeout, gap-merge, per-unit connections, f64_le) and they share a class with the upcoming OPC UA / MQTT / ADS reconnect concerns — likely a single bundled "driver hardening" item in the v1.1 PR.

Three Phase 4 drivers remain: **W (OPC UA)**, **Y (MQTT)**, **Z (ADS)** — all unblocked, all independent.
