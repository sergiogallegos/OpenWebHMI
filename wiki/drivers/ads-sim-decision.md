---
status: active
last-validated: 2026-05-03
---

# ADS Simulator Decision

## Summary

OpenWebHMI should not ship a fake ADS simulator. A deterministic CI ADS fixture is technically feasible only if it speaks AMS/ADS frames. The v1.0 decision is to defer a standalone `examples/sim-ads` until the project can either adapt real frame-handling code from the upstream `ads` test harness or validate an external Beckhoff-compatible service; until then, TwinCAT hardware validation remains the proof for ADS.

## Current understanding

1. The OpenWebHMI driver exercises real `ads` client calls for symbol upload, handle-based read/write, and notifications. A JSON-line or other invented dialect would not validate those paths. Sources: [crates/driver-ads/src/driver.rs](../../crates/driver-ads/src/driver.rs) and [ads-integration.md](ads-integration.md).
2. The published `ads = 0.4.4` public crate surface is client-oriented. `src/lib.rs` exports `client`, `notif`, `udp`, `ports`, `index`, `file`, `strings`, and `symbol`; there is no public `server` module. Source: <https://docs.rs/crate/ads/0.4.4/source/src/lib.rs>.
3. The upstream crate does contain a private test server under `src/test/mod.rs` that parses ADS headers, handles read/write/read-write, emits notifications, and implements sumup test paths. Because it is compiled only under `#[cfg(test)]`, OpenWebHMI cannot reuse it directly as a dependency API.
4. The upstream `examples/timing_server.rs` is a fixed-byte timing responder, not a symbol-aware PLC simulator. It is useful as evidence of frame shape, not as an OpenWebHMI CI fixture.
5. Beckhoff's official ADS library is available as C++ source for non-Windows clients, but it is a client library rather than a mock TwinCAT runtime. Source: <https://github.com/Beckhoff/ADS>.

## Decision

Defer the ADS CI simulator for v1.0. Building it correctly means implementing enough AMS/ADS server behavior for:

- symbol metadata upload for `MAIN.bRunning`, `MAIN.nCounter`, `MAIN.fSetPoint`, and `MAIN.sStatus`;
- symbol handle lookup and release;
- handle-based read/write;
- add/delete notification and asynchronous device notification frames;
- reconnect behavior and bounded handle cleanup.

That is real protocol work, but likely smaller if seeded from the upstream crate's private test harness. The estimated minimum is several hundred lines for a narrow smoke fixture and over 1000 lines if it grows into a reusable symbol-table simulator with robust negative cases. Because CODEX-AD's immediate gate is production validation, hardware proof has higher value than rushing an unmaintained simulator.

## Evidence

- [crates/driver-ads/src/driver.rs](../../crates/driver-ads/src/driver.rs) — current real client calls under validation.
- `ads` exported modules: <https://docs.rs/crate/ads/0.4.4/source/src/lib.rs>.
- `ads` example list, including `timing_server.rs`: <https://docs.rs/crate/ads/0.4.4/source/examples/>.
- Local crate source inspected at `%USERPROFILE%\.cargo\registry\src\...\ads-0.4.4\src\test\mod.rs`; it contains the private AMS/ADS test server but is not exported by the crate.
- Beckhoff official ADS client library: <https://github.com/Beckhoff/ADS>.

## Open questions

1. Can the upstream private test server be proposed upstream as a reusable dev-support module without exposing unstable internals? Resolve by opening an `ads-rs` discussion or PR once TLS fork work starts.
2. What is the smallest correct symbol-upload payload required for the OpenWebHMI browse path? Resolve by capturing a TwinCAT symbol upload for the smoke PLC and comparing it with `crates/driver-ads/src/symbols.rs`.
3. Can a Dockerized Beckhoff-compatible ADS service be run legally and deterministically in CI? Resolve by testing Beckhoff/ADS plus a mock runtime configuration, if such a runtime can be provided without TwinCAT XAR.

## Related pages

- [ads-integration.md](ads-integration.md)
- [ads-tls-decision.md](ads-tls-decision.md)
- [docs/agents/tasks/CODEX-AD-ads-validation.md](../../docs/agents/tasks/CODEX-AD-ads-validation.md)
