---
status: active
last-validated: 2026-05-01
---

# MQTT integration

## Summary

`driver-mqtt` wraps `rumqttc` as OpenWebHMI's MQTT client driver. v1 covers two modes: generic MQTT topics with simple payload decoding, and a Sparkplug B data-feed subset that resolves BIRTH alias maps before applying DDATA metric updates. `examples/sim-mqtt` embeds `rumqttd` as the simulator broker for CI and manual smoke.

The brief named `rumqttc = "0.24"`. At implementation time the compatible current pair is `rumqttc` `0.25.1` plus `rumqttd` `0.20.0`, so OpenWebHMI pins those exact versions.

## Current understanding

- Dependency: `rumqttc` is pinned at `0.25.1`; simulator broker dependency `rumqttd` is pinned at `0.20.0`.
- Source identifiers: `rumqttc` crate archive commit `f1e9e8d558783f942993046679cdf3c8c3a3d36b`; `rumqttd` crate archive commit `c03ba8bbb785dc6cd7809ce14fc2845d14b6bb74`.
- Generic address shape is the concrete MQTT topic, for example `factory/line1/temperature`.
- Sparkplug address shape is `spB/v1.0/<group_id>/DDATA/<edge_node_id>/<device_id>/<metric_name>`.
- Supported generic payload decoders are UTF-8 string, decimal integer/float, and little-endian 64-bit integer/float.
- Sparkplug B support is intentionally data-feed-only: DBIRTH builds the device alias map, DDATA resolves aliases to metric names, and NBIRTH is accepted for node-level alias state.
- MQTT writes publish the OpenWebHMI `TagValue` as UTF-8 text to the address topic. Sparkplug NCMD/DCMD command writes are deferred.

## Evidence

| Claim | Source | URL / path |
|---|---|---|
| `rumqttc` selected as the v1 wire crate | CODEX-Y task brief and implementation | `docs/agents/tasks/CODEX-Y-driver-mqtt.md`, `Cargo.toml` |
| Resolved `rumqttc` version is `0.25.1`; `rumqttd` version is `0.20.0` | Cargo lockfile | `Cargo.lock` |
| Upstream commits are recorded in crate VCS metadata | Cargo VCS metadata | `~/.cargo/registry/src/.../rumqttc-0.25.1/.cargo_vcs_info.json`, `~/.cargo/registry/src/.../rumqttd-0.20.0/.cargo_vcs_info.json` |
| Sparkplug B payload schema subset is vendored | OpenWebHMI proto | `crates/driver-mqtt/proto/sparkplug_b.proto` |
| Address parser supports generic and Sparkplug B shapes | OpenWebHMI code | `crates/driver-mqtt/src/address.rs` |
| Sparkplug alias lifecycle is implemented and unit-tested | OpenWebHMI code | `crates/driver-mqtt/src/sparkplug.rs` |
| Simulator-backed integration validates generic and Sparkplug updates | OpenWebHMI test | `crates/driver-mqtt/tests/integration.rs` |

### Independent verification status by OpenWebHMI

| Claim | OpenWebHMI verification | When |
|---|---|---|
| Generic and Sparkplug address parsing rejects malformed Sparkplug paths | ✅ verified by `address.rs` unit tests | CODEX-Y, 2026-05-01 |
| BIRTH alias maps resolve subsequent DATA aliases | ✅ verified by `sparkplug.rs` unit tests | CODEX-Y, 2026-05-01 |
| Alias collisions are scoped by edge/device key | ✅ verified by `sparkplug.rs` unit test | CODEX-Y, 2026-05-01 |
| DDATA before DBIRTH is ignored rather than guessed | ✅ verified by `sparkplug.rs` unit test | CODEX-Y, 2026-05-01 |
| Generic and Sparkplug updates flow through an embedded broker simulator | ✅ verified by `cargo test -p openwebhmi-driver-mqtt --features sim-tests` | CODEX-Y, 2026-05-01 |

## Limitations

- v1 supports plain TCP in the driver implementation. TLS and WebSocket are config-shaped but deferred to the driver-hardening pass.
- The gateway does not act as a Sparkplug B primary host in v1 and does not send STATE/REBIRTH requests.
- NCMD/DCMD writes to Sparkplug edge nodes are post-1.0. Generic MQTT write publishes are supported.
- Persistent sessions, retained publish behavior, MQTT v5-specific features, and client certificate auth are out of scope for v1.
- JSONPath payload decoding is not implemented yet; v1 supports the scalar decoders listed above.
- Reconnect/resubscribe is left to the common v1.1 driver-hardening track; the v1 simulator test validates the happy-path broker flow.

## Related pages

- [`docs/feature-matrix.md`](../../docs/feature-matrix.md) — §1 Connectivity.
- [`docs/roadmap.md`](../../docs/roadmap.md) — Phase 4 driver expansion.
- [`docs/agents/tasks/CODEX-Y-driver-mqtt.md`](../../docs/agents/tasks/CODEX-Y-driver-mqtt.md) — implementation brief and review lifecycle.
