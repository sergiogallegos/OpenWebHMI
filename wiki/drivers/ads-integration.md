---
status: active
last-validated: 2026-05-03
---

# Beckhoff ADS Integration

## Summary

`driver-ads` now has two real ADS backends: the pure Rust `ads = 0.4.4` ADS-over-TCP backend for plain routes, and a Windows TwinCAT-router backend that dynamically loads Beckhoff `TcAdsDll.dll` for XAE-created Secure ADS routes. Unit tests verify OpenWebHMI address parsing, AMS configuration, symbol mapping, primitive encode/decode, read/write routing, and subscription plumbing; the TwinCAT-router backend passed a live smoke against CX-23F092 on 2026-05-03.

## Current understanding

1. OpenWebHMI ADS addresses use `<port>:<symbol>`, for example `851:MAIN.nCounter`. `crates/driver-ads/src/address.rs` preserves TwinCAT symbol case and rejects malformed ports or unsupported symbol characters.
2. The plain-TCP backend uses the `ads` crate's production client APIs: `ads::Client::new`, `Client::device`, `ads::Handle::new`, `Handle::read`, `Handle::write`, `ads::symbol::get_symbol_info`, `Device::add_notification`, and `Device::delete_notification`. This is the real ADS-over-TCP path, not a custom test protocol.
3. The Windows TwinCAT-router backend dynamically loads `TcAdsDll.dll` from the TwinCAT install or PATH, opens a local ADS port with `AdsPortOpenEx`, uploads symbols with `AdsSyncReadReqEx2`, and uses ADS handle read/write calls through `AdsSyncReadWriteReqEx2`, `AdsSyncReadReqEx2`, and `AdsSyncWriteReqEx`. Live updates currently use driver-owned polling over symbol index group/offset reads, not Beckhoff native notification callbacks.
4. Configuration requires `host` and a six-octet `ams_net_id`. `backend` defaults to `auto`, which prefers the TwinCAT-router backend on Windows when `TcAdsDll.dll` loads and falls back to `ads_rs_tcp`; `twincat_router` and `ads_rs_tcp` are explicit choices. `tcp_port` defaults to Beckhoff ADS-over-TCP port `48898`; `ports` defaults to `[851]`, the first TwinCAT 3 PLC runtime port.
5. The source AMS address can be `auto`, `request`, or an explicit `{ net_id, port }`. This is relevant to the `ads_rs_tcp` backend; the TwinCAT-router backend uses the local router's own port assignment.
6. Symbol browsing caches per-port symbol tables from ADS upload metadata. Primitive BOOL, integer, REAL/LREAL, and STRING values map to OpenWebHMI `TagValue` values with little-endian encoding.
7. The `ads_rs_tcp` backend subscriptions use ADS device notifications in `ServerOnChange` mode with the configured cycle time. The TwinCAT-router backend currently provides the same stream surface via polling reads at `poll_rate_ms`.
8. Secure ADS is now tracked as a v1.0 decision. Beckhoff documents Secure ADS as router-to-router TLS; normal applications should use the local TwinCAT router. Source: [ads-tls-decision.md](ads-tls-decision.md).
9. The ADS CI simulator is deferred unless it speaks real AMS/ADS frames. The `ads` crate has private test-server code but no public server API, so OpenWebHMI should not reintroduce a fake dialect. Source: [ads-sim-decision.md](ads-sim-decision.md).
10. Verification status is deliberately split:
   - CI/unit verified: parser, config defaults, symbol type mapping, primitive value codecs, mocked read/write, mocked subscription update flow.
   - Compile verified: the real `ads` 0.4.4 client/symbol/notification APIs compile against the driver.
   - Hardware first-contact attempted: TCP connectivity and source NetId derivation were verified, but AMS round-trip was blocked by a TLS-required route.
   - Hardware smoke verified through TwinCAT router: live symbol upload, browse, read, write, and update streaming against a Secure ADS route.
   - Not yet real-hardware verified: reconnect recovery, handle leak behavior, and native `TcAdsDll` notification callback support.

## Evidence

- `crates/driver-ads/Cargo.toml` pins the driver crate to the workspace `ads` dependency.
- `Cargo.toml` pins `ads = "=0.4.4"` in workspace dependencies.
- `crates/driver-ads/src/driver.rs` contains the native ADS client implementation and notification guard.
- `crates/driver-ads/src/twincat_router.rs` contains the Windows `TcAdsDll.dll` backend.
- `crates/driver-ads/src/connection.rs` contains AMS NetId/source policy and port-list configuration.
- `crates/driver-ads/src/symbols.rs` contains ADS symbol metadata and primitive value mapping.
- `docs/agents/tasks/CODEX-Z-driver-ads.md` records the rejected JSON-line submission and the rework requirement to use the real `ads` crate API.
- Beckhoff official ADS client source: <https://github.com/Beckhoff/ADS>.
- Beckhoff Information System ADS documentation: <https://infosys.beckhoff.com/>.
- Rust `ads` crate documentation: <https://docs.rs/ads/0.4.4/ads/>.
- ADS TLS decision: [ads-tls-decision.md](ads-tls-decision.md).
- ADS simulator decision: [ads-sim-decision.md](ads-sim-decision.md).

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
- A Windows TwinCAT-router backend using `TcAdsDll`.
- Or a test TwinCAT runtime (local on the same PC, or a separate non-secure test CX) that the maintainer can configure with plain-TCP routes from the start. **Local TwinCAT runtime requires HyperV or virtualization features enabled on the host PC** — not available in the current environment as of 2026-05-02.

This entry is the first item in CODEX-AD's "Hardware validation log" scope.

### 2026-05-03 — TwinCAT-router backend smoke against CX-23F092 (192.168.10.100.1.1)

**Result: passed for browse/read/write/update streaming through the local TwinCAT router.** `cargo run --example hardware-smoke -p openwebhmi-driver-ads -- --host 127.0.0.1 --net-id 192.168.10.100.1.1 --source request --subscribe-seconds 3` used `backend: auto`, dynamically loaded Beckhoff `TcAdsDll.dll`, and reached the CX through the XAE-created Secure ADS route.

Observed symbols included `MAIN.bRunning`, `MAIN.nCounter`, `MAIN.fSetPoint`, and `MAIN.sStatus`. Reads returned `Bool(true)`, incrementing `Int(...)`, `Real(50.0)`, and `String("Running")`. Writing `75.0` to `851:MAIN.fSetPoint` returned `Ok` and read back as `Real(75.0)`. The update stream delivered 24 good-quality updates over 3 seconds for `MAIN.bRunning` and changing `MAIN.nCounter`.

Runbook result table:

| Step | Status | Note |
| --- | --- | --- |
| Browse symbols | pass | 14 symbols uploaded from ADS port 851 through `TcAdsDll`. |
| Read `MAIN.nCounter` | pass | Value read successfully and changed between samples. |
| Read primitives | pass | BOOL, INT, REAL, and STRING symbols decoded into `TagValue`. |
| Write `MAIN.fSetPoint` | pass | Write returned `Ok`; read-back returned `Real(75.0)`. |
| Update stream | pass-with-limitation | Driver stream delivered changing values at `poll_rate_ms`; native `TcAdsDll` notification callback remains a follow-up. |
| Reconnect recovery | not-run | Needs repeated runtime stop/start or gateway reconnect pass. |
| Handle leak check | not-applicable-to-current-backend | Current TwinCAT-router update stream does not allocate ADS notification handles; native callback implementation will need this test. |

## Open Questions

1. Native `TcAdsDll` notification callbacks are not implemented. The 2026-05-03 TwinCAT-router backend validates the driver stream with polling reads, so an exact native notification proof still requires `AdsSyncAddDeviceNotificationReqEx` / `AdsSyncDelDeviceNotificationReqEx` support or a conscious v1 decision that polling is acceptable for the router backend.
2. Reconnect recovery remains open. Resolve by stopping/restarting the TwinCAT runtime or reconnecting the gateway repeatedly against the same Secure ADS route and verifying quality recovery.
3. Route-table discovery is not implemented. Explicit `host` plus `ams_net_id` is the supported v1 path until Windows registry and Linux `StaticRoutes.xml` parsing are designed.
4. **ADS sumup batch read/write is not wired into `driver-ads` yet.** The current driver issues sequential `Handle::read` and `Handle::write` calls per tag on the `ads_rs_tcp` backend and sequential read/write calls on the TwinCAT-router backend. For an HMI polling 50 ADS tags at 100 ms, that can become 50 request/reply round trips per poll cycle. The `ads` crate exposes sumup-capable `Device::read_multi`, `write_multi`, `write_read_multi`, `add_notification_multi`, and `delete_notification_multi` APIs, and Beckhoff documents ADS Sum Commands as the protocol-level batching path. Decision criteria for v1.1: if a real deployment shows latency degradation above roughly 30 ADS tags, prioritize sumup; otherwise defer to v2.
5. Structured TwinCAT types are browsed as symbols but are not decoded as single OpenWebHMI structured tag values. Primitive child symbols remain the v1 path.
6. A deterministic CI ADS server harness is still open. The `ads` 0.4.4 crate provides a client-oriented public surface, so the previous simulator-stub evidence was removed rather than treated as protocol validation. Source: [ads-sim-decision.md](ads-sim-decision.md).
7. **`birkenfeld/ads-rs` upstream maintenance status is a v1.0 risk.** Snapshot from 2026-05-03: latest upstream `master` commit/push was 2026-03-05; the GitHub API reported 0 open non-PR issues and 1 open PR; public package mirrors still list 0.4.4 as the latest published crates.io release from 2024-10-03 while upstream README advertises `ads = "0.5"`. Mitigation: keep OpenWebHMI's wrapper boundary in `crates/driver-ads/src/` narrow enough that the transport dependency is replaceable by a fork or a thin AMS/ADS layer.

## Related Pages

- [docs/feature-matrix.md](../../docs/feature-matrix.md)
- [docs/roadmap.md](../../docs/roadmap.md)
- [docs/agents/tasks/CODEX-Z-driver-ads.md](../../docs/agents/tasks/CODEX-Z-driver-ads.md)
- [ads-tls-decision.md](ads-tls-decision.md)
- [ads-sim-decision.md](ads-sim-decision.md)
