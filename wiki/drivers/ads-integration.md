---
status: needs-review
last-validated: 2026-05-01
---

# Beckhoff ADS Integration

## Summary

`driver-ads` is a native Rust Beckhoff TwinCAT ADS client built on the crates.io `ads` 0.4.4 wire client. It does not use Beckhoff C++ FFI and does not use the rejected JSON-line simulator dialect. Unit tests verify OpenWebHMI address parsing, AMS configuration, symbol mapping, primitive encode/decode, read/write routing, and subscription plumbing; real TwinCAT 3 validation is still pending on a machine with an AMS route and runtime.

## Current understanding

1. OpenWebHMI ADS addresses use `<port>:<symbol>`, for example `851:MAIN.nCounter`. `crates/driver-ads/src/address.rs` preserves TwinCAT symbol case and rejects malformed ports or unsupported symbol characters.
2. The driver uses the `ads` crate's production client APIs: `ads::Client::new`, `Client::device`, `ads::Handle::new`, `Handle::read`, `Handle::write`, `ads::symbol::get_symbol_info`, `Device::add_notification`, and `Device::delete_notification`. This is the real ADS-over-TCP path, not a custom test protocol.
3. Configuration requires `host` and a six-octet `ams_net_id`. `tcp_port` defaults to Beckhoff ADS-over-TCP port `48898`; `ports` defaults to `[851]`, the first TwinCAT 3 PLC runtime port. Additional ADS ports can be supplied for NC or I/O runtime browse attempts.
4. The source AMS address can be `auto`, `request`, or an explicit `{ net_id, port }`. `request` delegates source-port assignment to the local AMS router; `auto` lets the Rust client derive a source NetId from the local IPv4 address.
5. Symbol browsing caches per-port symbol tables from ADS upload metadata. Primitive BOOL, integer, REAL/LREAL, and STRING values map to OpenWebHMI `TagValue` values with little-endian encoding.
6. Subscriptions use ADS device notifications in `ServerOnChange` mode with the configured cycle time. The returned stream owns a guard that deletes notification handles when dropped.
7. Verification status is deliberately split:
   - CI/unit verified: parser, config defaults, symbol type mapping, primitive value codecs, mocked read/write, mocked subscription update flow.
   - Compile verified: the real `ads` 0.4.4 client/symbol/notification APIs compile against the driver.
   - Not yet real-hardware verified: connecting to TwinCAT 3, route negotiation, live symbol upload, notification delivery, and write round-trip.

## Evidence

- `crates/driver-ads/Cargo.toml` pins the driver crate to the workspace `ads` dependency.
- `Cargo.toml` pins `ads = "=0.4.4"` in workspace dependencies.
- `crates/driver-ads/src/driver.rs` contains the native ADS client implementation and notification guard.
- `crates/driver-ads/src/connection.rs` contains AMS NetId/source policy and port-list configuration.
- `crates/driver-ads/src/symbols.rs` contains ADS symbol metadata and primitive value mapping.
- `docs/agents/tasks/CODEX-Z-driver-ads.md` records the rejected JSON-line submission and the rework requirement to use the real `ads` crate API.
- Beckhoff official ADS client source: <https://github.com/Beckhoff/ADS>.
- Beckhoff Information System ADS documentation: <https://infosys.beckhoff.com/>.
- Rust `ads` crate documentation: <https://docs.rs/ads/0.4.4/ads/>.

## Open Questions

1. Real TwinCAT 3 validation remains open. It will be resolved by configuring an AMS route, browsing port `851`, reading `MAIN.nCounter`, writing a writable primitive, and observing an ADS notification.
2. Route-table discovery is not implemented. Explicit `host` plus `ams_net_id` is the supported v1 path until Windows registry and Linux `StaticRoutes.xml` parsing are designed.
3. ADS sum-up batch read/write is not implemented yet, even though the protocol supports it. The v1 driver exposes batch capability only after the gateway has a batch call surface to exercise.
4. Structured TwinCAT types are browsed as symbols but are not decoded as single OpenWebHMI structured tag values. Primitive child symbols remain the v1 path.
5. A deterministic CI ADS server harness is still open. The `ads` 0.4.4 crate provides a client-oriented surface, so the previous simulator-stub evidence was removed rather than treated as protocol validation.

## Related Pages

- [docs/feature-matrix.md](../../docs/feature-matrix.md)
- [docs/roadmap.md](../../docs/roadmap.md)
- [docs/agents/tasks/CODEX-Z-driver-ads.md](../../docs/agents/tasks/CODEX-Z-driver-ads.md)
