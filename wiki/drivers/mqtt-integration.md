---
status: active
last-validated: 2026-05-01
---

# MQTT integration

## Summary

`driver-mqtt` wraps `rumqttc` as OpenWebHMI's MQTT client driver. v1 covers generic MQTT topics with scalar payload decoding, JSONPath extraction, TLS/WSS transport selection, and a Sparkplug B data-feed subset that resolves BIRTH alias maps before applying DDATA metric updates. `examples/sim-mqtt` embeds `rumqttd` as the plaintext TCP/WebSocket simulator broker for CI and manual smoke.

The brief named `rumqttc = "0.24"`. At implementation time the compatible current pair is `rumqttc` `0.25.1` plus `rumqttd` `0.20.0`, so OpenWebHMI pins those exact versions.

## Current understanding

- Dependency: `rumqttc` is pinned at `0.25.1`; simulator broker dependency `rumqttd` is pinned at `0.20.0`.
- Source identifiers: `rumqttc` crate archive commit `f1e9e8d558783f942993046679cdf3c8c3a3d36b`; `rumqttd` crate archive commit `c03ba8bbb785dc6cd7809ce14fc2845d14b6bb74`.
- Generic address shape is the concrete MQTT topic, for example `factory/line1/temperature`.
- Transport config supports `tcp`, `tls`, and `websocket`. WebSocket hosts use the rumqttc URL form, for example `ws://127.0.0.1:1884/mqtt` or `wss://broker.example.com:443/mqtt`.
- TLS and WSS use rumqttc's rustls transport. By default they use platform certificate roots; `ca_cert_path` points at a PEM CA bundle for private brokers. `tls_insecure` is accepted for config compatibility but logs a warning and does not disable certificate verification in v1.
- Sparkplug address shape is `spB/v1.0/<group_id>/DDATA/<edge_node_id>/<device_id>/<metric_name>`.
- Supported generic payload decoders are UTF-8 string, UTF-8 decimal integer/float, big-endian and little-endian 64-bit integer/float, and JSONPath extraction from JSON payloads. `RawIntBe`/`RawFloatBe` mean 8 raw big-endian bytes, not ASCII text.
- JSONPath payload config is `{ "json_path": "$.outer.inner" }`. OpenWebHMI pins `jsonpath-rust = 1.0.4`.
- Sparkplug B support is intentionally data-feed-only: DBIRTH builds the device alias map, DDATA resolves aliases to metric names, and NBIRTH is accepted for node-level alias state.
- MQTT writes publish the OpenWebHMI `TagValue` as UTF-8 text to the address topic. Sparkplug NCMD/DCMD command writes are deferred.

## Evidence

| Claim | Source | URL / path |
|---|---|---|
| `rumqttc` selected as the v1 wire crate | CODEX-Y task brief and implementation | `docs/agents/tasks/CODEX-Y-driver-mqtt.md`, `Cargo.toml` |
| Resolved `rumqttc` version is `0.25.1`; `rumqttd` version is `0.20.0` | Cargo lockfile | `Cargo.lock` |
| Upstream commits are recorded in crate VCS metadata | Cargo VCS metadata | `~/.cargo/registry/src/.../rumqttc-0.25.1/.cargo_vcs_info.json`, `~/.cargo/registry/src/.../rumqttd-0.20.0/.cargo_vcs_info.json` |
| Sparkplug B payload wire fields are first-party Rust `prost` types | OpenWebHMI implementation against the public specification | `crates/driver-mqtt/src/sparkplug.rs` |
| Address parser supports generic and Sparkplug B shapes | OpenWebHMI code | `crates/driver-mqtt/src/address.rs` |
| Sparkplug alias lifecycle is implemented and unit-tested | OpenWebHMI code | `crates/driver-mqtt/src/sparkplug.rs` |
| Simulator-backed integration validates generic and Sparkplug updates | OpenWebHMI test | `crates/driver-mqtt/tests/integration.rs` |
| JSONPath extraction uses a pinned parser crate | Cargo metadata and OpenWebHMI tests | `Cargo.toml`, `crates/driver-mqtt/src/driver.rs` |

### Independent verification status by OpenWebHMI

| Claim | OpenWebHMI verification | When |
|---|---|---|
| Generic and Sparkplug address parsing rejects malformed Sparkplug paths | ✅ verified by `address.rs` unit tests | CODEX-Y, 2026-05-01 |
| BIRTH alias maps resolve subsequent DATA aliases | ✅ verified by `sparkplug.rs` unit tests | CODEX-Y, 2026-05-01 |
| Alias collisions are scoped by edge/device key | ✅ verified by `sparkplug.rs` unit test | CODEX-Y, 2026-05-01 |
| DDATA before DBIRTH is ignored rather than guessed | ✅ verified by `sparkplug.rs` unit test | CODEX-Y, 2026-05-01 |
| Generic and Sparkplug updates flow through an embedded broker simulator | ✅ verified by `cargo test -p openwebhmi-driver-mqtt --features sim-tests` | CODEX-Y, 2026-05-01 |
| Binary BE decoders read 8 raw bytes, not ASCII text | ✅ verified by `driver.rs` unit tests and simulator integration | CODEX-AA, 2026-05-01 |
| JSONPath extracts nested JSON scalar payloads | ✅ verified by `address.rs`, `driver.rs`, and simulator integration tests | CODEX-AA, 2026-05-01 |
| WebSocket transport connects through the simulator broker | ✅ verified by simulator integration test | CODEX-AA, 2026-05-01 |
| TLS transport is selectable with default or PEM CA trust | ⚠️ config path implemented; handshake smoke uses external Mosquitto because the pinned `rumqttd` CI fixture stays plaintext | CODEX-AA, 2026-05-01 |

## Limitations

- v1 supports TCP, TLS, WS, and WSS client transports. TLS verification is not disabled by `tls_insecure`; the flag logs a deployment warning and remains a v1.1 hardening point if true insecure lab mode is still desired.
- The gateway does not act as a Sparkplug B primary host in v1 and does not send STATE/REBIRTH requests.
- NCMD/DCMD writes to Sparkplug edge nodes are post-1.0. Generic MQTT write publishes are supported.
- Persistent sessions, retained publish behavior, MQTT v5-specific features, and client certificate auth are out of scope for v1.
- Reconnect/resubscribe is left to the common v1.1 driver-hardening track; the v1 simulator test validates the happy-path broker flow.

## Related pages

- [`docs/feature-matrix.md`](../../docs/feature-matrix.md) — §1 Connectivity.
- [`docs/roadmap.md`](../../docs/roadmap.md) — Phase 4 driver expansion.
- [`docs/agents/tasks/CODEX-Y-driver-mqtt.md`](../../docs/agents/tasks/CODEX-Y-driver-mqtt.md) — implementation brief and review lifecycle.
