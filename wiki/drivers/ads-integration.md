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

## Hardware validation log

### 2026-05-02 — first contact attempt against TwinCAT 3.1.4024.44 on CX-23F092 (192.168.10.100)

**Result: blocked at the route-policy layer.** The ADS driver compiled, connected to TCP `192.168.10.100:48898`, and sent AMS frames; the CX received them but never replied. Driver returns `remote fault ads: receiving reply (route set?): timed out`.

**Diagnosis:** The CX's `StaticRoutes.xml` had the source PC registered with a `<Tls>` block (TLS-wrapped ADS via Beckhoff "ADS Secure"). The `ads = 0.4.4` Rust crate connects via plain TCP only — it does not speak TLS-wrapped ADS. The CX silently drops plain-TCP frames from peers whose route entry requires TLS.

**Workarounds attempted:**
- Removing the `<Tls>` block from the CX's route entry for the source PC and restarting the TwinCAT system service — operationally fragile. Multiple route-cache desync issues surfaced during the attempt: the in-memory route table in `TcSysSrv` did not pick up the file edit reliably, and re-adding the route via XAE produced `"an item with the same key has already added"` errors. Recovery required restoring the original TLS-wrapped routes.
- The TwinCAT 4024 route-management UX (XAE Add Route + StaticRoutes.xml on both peers) is brittle when the file is edited manually alongside in-memory state. **For real customers this is a route-management trap, not a driver bug.**

**What we learned:**
- TLS-wrapped ADS is the enterprise default on TwinCAT 3.1.4024+ (the route is added with TLS automatically when paired via XAE's "Add Route" dialog).
- Plain-TCP routes still work but require manual XML edits on both peers plus a clean system-service restart on each — workflow that is too brittle for production deployment guidance.
- TCP connectivity to port `48898` is open even when TLS is required at the route layer, which is why the failure mode is "AMS frame timeout" rather than "TCP refused."

**Validation status as of this session:**
- ✅ Driver compiles and runs against `ads = 0.4.4`.
- ✅ TCP layer reaches the CX (`Test-NetConnection` confirms port 48898 open).
- ✅ AMS source NetId is correctly derived (`Source::Auto` produced `192.168.10.98.1.1` matching the local TwinCAT registry).
- ❌ AMS frame round-trip blocked by TLS-required route on the CX.
- ⏸️ Browse / read / write / notification / reconnect — all deferred until a non-TLS test PLC is available.

**Next attempt blocked on one of:**
- A test TwinCAT runtime (local on the same PC, or a separate non-secure test CX) that the maintainer can configure with plain-TCP routes from the start. **Local TwinCAT runtime requires HyperV or virtualization features enabled on the host PC** — not available in the current environment as of 2026-05-02.
- Or: TLS-wrapped ADS support added to the `ads` crate (upstream contribution) or implemented in a fork.

This entry is the first item in CODEX-AD's "Hardware validation log" scope.

## Open Questions

1. **TLS-wrapped ADS is a v1.0 risk, not v1.1 polish (re-prioritized 2026-05-02).** TwinCAT 3.1.4024+ defaults to TLS for new routes added via XAE. Any real Beckhoff customer running 4024+ will hit this on first contact. Options for closing the gap: (a) fork or contribute TLS support to `birkenfeld/ads-rs`; (b) document the workaround (manually configure plain-TCP routes on both peers, with explicit warnings about route-cache fragility); (c) wait for an upstream maintainer response. Decision required before v1.0 ships.
2. Real TwinCAT 3 validation remains open. Blocked by item 1 above; will be resolved by configuring an AMS route on a non-secure test target, browsing port `851`, reading `MAIN.nCounter`, writing a writable primitive, and observing an ADS notification.
3. Route-table discovery is not implemented. Explicit `host` plus `ams_net_id` is the supported v1 path until Windows registry and Linux `StaticRoutes.xml` parsing are designed.
4. ADS sum-up batch read/write is not implemented yet, even though the protocol supports it. The v1 driver exposes batch capability only after the gateway has a batch call surface to exercise.
5. Structured TwinCAT types are browsed as symbols but are not decoded as single OpenWebHMI structured tag values. Primitive child symbols remain the v1 path.
6. A deterministic CI ADS server harness is still open. The `ads` 0.4.4 crate provides a client-oriented surface, so the previous simulator-stub evidence was removed rather than treated as protocol validation.
7. **`birkenfeld/ads-rs` upstream maintenance status is a v1.0 risk** (re-prioritized 2026-05-02 alongside item 1). Single-maintainer crate; if upstream is unresponsive on a TLS contribution, OpenWebHMI's options narrow to fork-and-maintain or write a thin AMS/ADS layer in-tree. Mitigation tracking is part of CODEX-AD.

## Related Pages

- [docs/feature-matrix.md](../../docs/feature-matrix.md)
- [docs/roadmap.md](../../docs/roadmap.md)
- [docs/agents/tasks/CODEX-Z-driver-ads.md](../../docs/agents/tasks/CODEX-Z-driver-ads.md)
