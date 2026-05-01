---
status: active
last-validated: 2026-04-30
---

# Modbus integration

## Summary

`driver-modbus` wraps `tokio-modbus` as OpenWebHMI's Modbus TCP/RTU client driver. v1 is master/client only: the gateway polls Modbus slaves and writes coils/holding registers, while `examples/sim-modbus` provides a deterministic TCP slave for CI and manual smoke. RTU support is compiled and unit-covered through the shared transport config, but end-to-end RTU validation remains hardware or serial-pair gated.

## Current understanding

- Dependency: `tokio-modbus` is pinned at `0.16.1` in the workspace and resolved in `Cargo.lock`. The crate advertises async Modbus TCP and RTU support through default `tcp` and `rtu` features in its `Cargo.toml`.
- Source identifier: the crate archive contains `.cargo_vcs_info.json` with upstream commit `b077ca3f26c156915173cc5976585989d142a15d`.
- OpenWebHMI address shape is `<unit-id>/<area>/<addr>[:<count>][:<datatype>]`, for example `1/holding/100:2:f32`, `1/coils/0`, and `2/holding/200:5:string`.
- Unit ids `1..247` and `255` are accepted. `255` is useful for TCP gateway/passthrough setups; RTU users should provide the actual slave id.
- Areas map to Modbus function families: `coils` uses FC1/5/15, `discrete` uses FC2, `input` uses FC4, and `holding` uses FC3/6/16.
- Datatype defaults are area-specific: `bool` for `coils`/`discrete`, `u16` for `input`/`holding`.
- Valid register datatypes are `bool`, `u16`, `i16`, `u32`, `i32`, `u32_le`, `i32_le`, `f32`, `f32_le`, `f64`, and `string`. Bit areas accept only `bool`.
- Multi-register default byte/word order is ABCD: high word first, big-endian bytes inside each register. `_le` variants use CDAB word order for 32-bit values. `f64` is currently ABCDEFGH only.
- Strings are UTF-8 bytes packed two bytes per register, high byte first, and read until the first NUL byte.
- Polling subscriptions group overlapping and adjacent requests by unit id and area, respecting Modbus maximum quantities of 2000 coils and 125 registers per frame.
- Simulator validation covers TCP reads for coils and holding registers, writes to coils, grouped polling subscription updates, and the simulator's FC1/2/3/4/5/6/15/16 subset.

### Area and datatype matrix

| Area | Read FC | Write FC | Default type | Valid types |
|---|---:|---:|---|---|
| `coils` | 1 | 5 / 15 | `bool` | `bool` |
| `discrete` | 2 | read-only | `bool` | `bool` |
| `input` | 4 | read-only | `u16` | `bool`, `u16`, `i16`, `u32`, `i32`, `u32_le`, `i32_le`, `f32`, `f32_le`, `f64`, `string` |
| `holding` | 3 | 6 / 16 | `u16` | `bool`, `u16`, `i16`, `u32`, `i32`, `u32_le`, `i32_le`, `f32`, `f32_le`, `f64`, `string` |

## Evidence

| Claim | Source | URL / path |
|---|---|---|
| `tokio-modbus` selected as the v1 wire crate | CODEX-X task brief | `docs/agents/tasks/CODEX-X-driver-modbus.md` |
| Resolved version is `0.16.1` | Cargo lockfile | `Cargo.lock` |
| Upstream commit for the crate archive is `b077ca3f26c156915173cc5976585989d142a15d` | Cargo VCS metadata | `~/.cargo/registry/src/.../tokio-modbus-0.16.1/.cargo_vcs_info.json` |
| `tokio-modbus` has default TCP and RTU features | Upstream crate manifest | `~/.cargo/registry/src/.../tokio-modbus-0.16.1/Cargo.toml.orig` |
| FC1 max quantity 2000 coils and FC3/FC4 max quantity 125 registers | Modbus Application Protocol Specification V1.1b3, function-code sections 6.1, 6.3, and 6.4 | https://modbus.org/specs.php |
| Address parser, datatype decoding, and error mapping are implemented and unit-tested | OpenWebHMI code | `crates/driver-modbus/src/address.rs`, `crates/driver-modbus/src/driver.rs` |
| Subscription polling groups overlapping and adjacent reads | OpenWebHMI code | `crates/driver-modbus/src/driver.rs` |
| TCP simulator implements the v1 function-code subset | OpenWebHMI code | `examples/sim-modbus/src/lib.rs` |
| Simulator-backed integration validates TCP read/write/subscribe | OpenWebHMI test | `crates/driver-modbus/tests/integration.rs` |

### Independent verification status by OpenWebHMI

| Claim | OpenWebHMI verification | When |
|---|---|---|
| Address syntax rejects malformed unit/area/count/type combinations | ✅ verified by `address.rs` unit tests | CODEX-X, 2026-04-30 |
| Register decoding covers big-endian and `_le` 32-bit word order | ✅ verified by `address.rs` unit tests | CODEX-X, 2026-04-30 |
| Overlapping and adjacent subscription reads are coalesced | ✅ verified by `driver.rs` unit test | CODEX-X, 2026-04-30 |
| `ExceptionCode::IllegalDataAddress` maps to `DriverError::InvalidAddress` | ✅ verified by `driver.rs` unit test | CODEX-X, 2026-04-30 |
| TCP client can read holding registers/coils from the simulator and write coils back | ✅ verified by `cargo test -p openwebhmi-driver-modbus --features sim-tests` | CODEX-X, 2026-04-30 |
| RTU transport works against a real serial device | ⏳ pending — requires serial pair or hardware | Pre-1.0 hardware gate |

## Limitations

- OpenWebHMI does not ship Modbus slave/server mode in v1. The gateway is a client/master only.
- Modbus ASCII is out of scope for v1.
- RTU integration tests are not run in CI because they need a serial pair or hardware. TCP simulator tests are the merge gate.
- Browse is not supported. Modbus has no standard browse service, so users must configure addresses explicitly.
- Endianness support is intentionally conservative: ABCD is default; CDAB is exposed through `_le` 32-bit suffixes. Other vendor-specific orders such as BADC or DCBA need explicit future suffixes before use.

## Open questions

- Do we need `f64_le` before 1.0, or is 64-bit floating point rare enough in Modbus deployments to defer?
- Should Modbus TCP reconnect be owned by this driver directly or left to the existing gateway/driver supervisor pattern?
- Which serial-pair setup should CI use for RTU validation: `socat` pseudo-terminals, a small Rust loopback harness, or hardware-only validation?

## Related pages

- [`docs/feature-matrix.md`](../../docs/feature-matrix.md) — §1 Connectivity.
- [`docs/roadmap.md`](../../docs/roadmap.md) — Phase 4 driver expansion.
- [`docs/agents/tasks/CODEX-X-driver-modbus.md`](../../docs/agents/tasks/CODEX-X-driver-modbus.md) — implementation brief and review lifecycle.
