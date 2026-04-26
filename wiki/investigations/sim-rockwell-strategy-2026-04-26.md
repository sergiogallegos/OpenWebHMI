---
status: active
last-validated: 2026-04-26
---

# Rockwell simulator strategy

## Summary

Phase 1 will reuse and adapt the upstream `rust-ethernet-ip` 0.7.0 simulator (`src/bin/plc_sim.rs`) as the OpenWebHMI `examples/sim-rockwell` harness. This is lower cost and higher fidelity than introducing OpenENER or building a clean-room CIP responder from scratch, because it exercises the same request shapes the upstream crate already uses for its own simulator regression path.

## Current understanding

1. `rust-ethernet-ip` 0.7.0 includes a binary target named `plc_sim` at `src/bin/plc_sim.rs`; the normalized crate manifest lists that binary explicitly. Source: upstream crate manifest at `rust-ethernet-ip` 0.7.0 / commit `592bfa716309e3388cf8143c4095622d6302a7f6`, inspected locally from Cargo registry and via docs.rs.
2. The upstream simulator already implements the minimum EtherNet/IP envelope needed for client connection and unconnected messaging: `RegisterSession` (`0x0065`) and `SendRRData` (`0x006F`). Source: `src/bin/plc_sim.rs` in upstream 0.7.0.
3. The upstream simulator already implements CIP read/write tag services (`0x4C`, `0x4D`) and returns AB scalar encodings for `BOOL`, `DINT`, `REAL`, and `STRING`. Source: `src/bin/plc_sim.rs` in upstream 0.7.0.
4. `rust-ethernet-ip` tag-group subscriptions call `read_tags_batch`, which builds a CIP Multiple Service Packet (`0x0A`) and parses a Multiple Service Response (`0x8A`). Source: upstream `src/lib.rs` `read_tags_batch`, `build_multiple_service_packet`, and `parse_multiple_service_response`.
5. Therefore OpenWebHMI should adapt upstream `plc_sim` rather than import OpenENER. The adaptation adds OpenWebHMI's required tags (`Counter`, `Setpoint`, `Pressure`, `Heartbeat`), tag behavior updates, `Sim.toml` config, CLI flags, and Multiple Service Packet responses for batch reads.

## Evidence

- Upstream manifest source: https://docs.rs/crate/rust-ethernet-ip/0.7.0/source/Cargo.toml
- Upstream simulator source: https://docs.rs/crate/rust-ethernet-ip/0.7.0/source/src/bin/plc_sim.rs
- Upstream client source: https://docs.rs/crate/rust-ethernet-ip/0.7.0/source/src/lib.rs
- Local inspected crate path: `$CARGO_HOME/registry/src/.../rust-ethernet-ip-0.7.0/`
- Upstream source identifier carried from `wiki/drivers/rust-ethernet-ip-integration.md`: `592bfa716309e3388cf8143c4095622d6302a7f6`

## Decision

Use option 1 from CODEX-G: upstream test-harness reuse. Copy/adapt the responder logic into `examples/sim-rockwell` with attribution in the README. Do not add OpenENER for Phase 1, because its C build toolchain would add contributor friction before we have evidence that upstream's simulator is insufficient. Do not implement a new responder from scratch, because the upstream implementation already covers the envelope and scalar tag operations that CODEX-F needs.

## Limitations

- This simulator is a fixture, not a conforming CompactLogix emulator.
- It supports the subset OpenWebHMI needs for Phase 1: connect, scalar read/write, and batch reads used by tag-group subscriptions.
- It does not model real firmware quirks, routing/backplane behavior, UDT metadata, produced/consumed tags, safety I/O, or motion.
- Any simulator pass remains weaker evidence than the pre-1.0 real-hardware gate documented in `docs/roadmap.md`.

## Open questions

- CODEX-F may discover that `rust-ethernet-ip` uses connected messaging or additional services for a specific wrapper path. That would be resolved by expanding `examples/sim-rockwell` only as far as the driver tests require.
- Batch writes are not needed for the initial CODEX-F tag-group tests. If driver-rockwell uses `write_tags_batch`, add Multiple Service write replies in the simulator with focused tests.

## Related pages

- [`wiki/drivers/rust-ethernet-ip-integration.md`](../drivers/rust-ethernet-ip-integration.md)
- [`docs/roadmap.md`](../../docs/roadmap.md)
- [`docs/agents/tasks/CODEX-G-sim-rockwell.md`](../../docs/agents/tasks/CODEX-G-sim-rockwell.md)
