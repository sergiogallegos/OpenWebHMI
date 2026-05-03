---
status: active
last-validated: 2026-05-03
---

# ADS Secure Decision

## Summary

TwinCAT 3.1.4024+ routes added through XAE commonly use Secure ADS. Beckhoff's documentation clarifies that Secure ADS is router-to-router TLS, not a normal application-level TLS socket. OpenWebHMI should therefore add a Windows TwinCAT-router backend using Beckhoff `TcAdsDll` when TwinCAT is installed, while keeping the current pure Rust `ads-rs` backend for plain ADS/TCP targets.

## Current understanding

1. The 2026-05-02 first-contact run reached TCP port `48898` on a TwinCAT 3.1.4024.44 CX, but AMS requests timed out because the configured route contained a TLS block. Source: [ads-integration.md](ads-integration.md).
2. `crates/driver-ads/src/driver.rs` constructs `ads::Client::new((host, tcp_port), timeouts, source)` and therefore inherits the pure TCP transport behavior exposed by the `ads` crate.
3. The published `ads` 0.4.4 manifest has no TLS dependency or feature flag. Source: <https://docs.rs/crate/ads/0.4.4/source/Cargo.toml>.
4. Beckhoff documents Secure ADS as TLSv1.2 between TwinCAT routers. Existing ADS applications are not supposed to be modified; they use the route, and the router handles encrypted transport. Source: <https://download.beckhoff.com/download/Document/automation/twincat3/Secure_ADS_EN.pdf>.
5. Beckhoff also states Secure ADS is available only between ADS routers and incoming Secure ADS uses TCP port `8016`. Sources: <https://download.beckhoff.com/download/Document/automation/twincat3/Secure_ADS_EN.pdf> and <https://infosys.beckhoff.com/content/1033/secure_ads/6798095243.html>.
6. On the maintainer machine, `TcAdsDll.dll` and headers are installed under `C:\TwinCAT\AdsApi\TcAdsDll\` and `C:\TwinCAT\Common64\`. The local route file contains a Secure ADS route to `192.168.10.100.1.1`.

## Decision

Choose a TwinCAT-router backend for v1.0 validation on Windows. When `TcAdsDll` is available, OpenWebHMI should use the Beckhoff local ADS router API so Secure ADS routes work the same way XAE and other TwinCAT applications work. Keep `ads-rs` as the backend for plain ADS/TCP targets and non-Windows deployments.

Rejected alternatives:

- Plain-TCP workaround only: rejected as customer guidance. Manual route edits around XAE-managed Secure ADS routes were fragile during the 2026-05-02 validation run.
- Add direct TLS to `ads-rs`: rejected as the primary v1.0 path after reading Beckhoff's manual. Secure ADS is documented as router-to-router; a standalone TLS socket to port `8016` would need to behave like a router, not like a normal ADS client.
- Wait for upstream first: rejected for v1.0 because OpenWebHMI cannot make first-contact support depend on upstream review timing.

## Implementation direction

1. Add a Windows-only backend in `crates/driver-ads` that opens a local ADS port through `TcAdsDll` and performs symbol upload plus read/write/read-write calls through the local router.
2. Reuse the existing symbol parsing and `TagValue` encode/decode layers so backend choice is limited to the ADS I/O boundary.
3. Select the backend from config, for example `backend: "auto" | "ads_rs_tcp" | "twincat_router"`, with `auto` preferring `twincat_router` on Windows when `TcAdsDll` can be loaded.
4. Validate the backend against the current CX route (`192.168.10.100.1.1`) before marking ADS hardware-validated.
5. Keep the `ads-rs` upgrade/fork option for Linux or non-TwinCAT-router deployments, but treat it as a separate plain-TCP route-management problem.

## Evidence

- [ads-integration.md](ads-integration.md) — first-contact hardware log and current driver status.
- [crates/driver-ads/src/driver.rs](../../crates/driver-ads/src/driver.rs) — current `ads::Client::new` call site.
- [Cargo.toml](../../Cargo.toml) — workspace pin to `ads = "=0.4.4"`.
- Beckhoff Secure ADS manual: <https://download.beckhoff.com/download/Document/automation/twincat3/Secure_ADS_EN.pdf>.
- Beckhoff Secure ADS requirements: <https://infosys.beckhoff.com/content/1033/secure_ads/6798095243.html>.
- Beckhoff ADS deactivation / SecureOnly settings: <https://infosys.beckhoff.com/content/1033/tc3_grundlagen/6917981195.html>.
- `ads` 0.4.4 manifest: <https://docs.rs/crate/ads/0.4.4/source/Cargo.toml>.

## Open questions

1. Native `TcAdsDll` device-notification callback support is not implemented yet. The first Windows backend validates live updates through driver-owned polling over indexed ADS reads; replacing that with `AdsSyncAddDeviceNotificationReqEx` remains a follow-up if native notification semantics are required.
2. Why does the pure Rust TCP client receive `0x12 Port disabled` or time out when pointed at the local router on this machine, while XAE can monitor the PLC? Resolve by comparing `TcAdsDll` calls against raw TCP traffic or treating raw local-router TCP as unsupported for Secure ADS.
3. Does upstream `master` / `0.5` change the connection API enough to require an OpenWebHMI driver shim for the plain-TCP backend? Resolve by building `driver-ads` against the newer source.

## Related pages

- [ads-integration.md](ads-integration.md)
- [ads-sim-decision.md](ads-sim-decision.md)
- [docs/agents/tasks/CODEX-AD-ads-validation.md](../../docs/agents/tasks/CODEX-AD-ads-validation.md)
