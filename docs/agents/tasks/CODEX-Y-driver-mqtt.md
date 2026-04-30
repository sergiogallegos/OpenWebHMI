---
id: CODEX-Y
title: crates/driver-mqtt — MQTT (generic + Sparkplug B) driver
owner: codex
phase: 4
status: open
created: 2026-04-30
last-update: 2026-04-30 claude
---

# CODEX-Y — `crates/driver-mqtt`

## Brief

> **Phase 4 scope expansion (2026-04-30).** v1 driver set expanded; MQTT (both generic and Sparkplug B) bumped from post-1.0 to v1.

### Goal

MQTT pub/sub support, both **generic** (raw topic-to-tag mapping) and **Sparkplug B** (the industrial-IoT MQTT spec used heavily in IIoT/cloud deployments). Generic MQTT covers field devices, IoT sensors, and ad-hoc broker integrations. Sparkplug B is what most cloud-deployed Ignition installations actually use.

### Context to read first

- `crates/driver-api/src/trait_def.rs` — the `Driver` trait. MQTT's pub/sub model needs a careful read of the trait: `read_many` returns the **last value seen** on the subscribed topic (cached). Subscriptions deliver updates via the standard `DriverUpdate` channel.
- Sparkplug B spec: https://www.eclipse.org/tahu/spec/sparkplug_spec.pdf (cite section numbers in the wiki).

### Files to create

- `crates/driver-mqtt/Cargo.toml`
- `crates/driver-mqtt/src/lib.rs` — re-exports.
- `crates/driver-mqtt/src/driver.rs` — `MqttDriver` impl of `Driver`.
- `crates/driver-mqtt/src/address.rs` — parse `<topic>` (generic) or `spB/v1.0/<group>/DDATA/<edge>/<device>/<metric>` (Sparkplug B).
- `crates/driver-mqtt/src/connection.rs` — TCP, TLS, WebSocket transports.
- `crates/driver-mqtt/src/sparkplug.rs` — Sparkplug B birth/death/data parsing + alias map.
- `crates/driver-mqtt/tests/integration.rs` — mosquitto-broker-gated end-to-end coverage.
- `crates/driver-mqtt/proto/sparkplug_b.proto` — Sparkplug B protobuf schema (vendored from Eclipse Tahu).
- `examples/sim-mqtt/Cargo.toml` + `src/main.rs` — generic MQTT publisher emitting test topics + a Sparkplug B edge-of-network publisher emitting BIRTH/DATA messages.
- `wiki/drivers/mqtt-integration.md` — covers both modes; cites Sparkplug B spec sections.

Add to workspace `members`.

### Wire crate

- **`rumqttc = "0.24"`** (https://github.com/bytebeamio/rumqtt) for the MQTT client. Async, supports v3.1.1 + v5.
- **`prost = "0.13"`** + **`prost-build`** for the Sparkplug B protobuf decode. Vendor `sparkplug_b.proto` from Eclipse Tahu (https://github.com/eclipse-tahu/tahu).

Pin both versions; document in wiki.

### Address shape

#### Generic mode

```text
<topic>
```

Topic strings as-is. Wildcards (`+`, `#`) are valid in the *subscription* but a tag address is a single concrete topic. Examples:

- `factory/line1/temperature`
- `iot/sensor/42/value`

The driver subscribes to the configured topics on connect; values arrive as raw payload bytes. Decoding is per-tag config: `payload_type` ∈ `{ utf8_string, json_path, raw_int_le, raw_int_be, raw_float_le, raw_float_be, sparkplug_metric }`.

#### Sparkplug B mode

```text
spB/v1.0/<group_id>/DDATA/<edge_node_id>/<device_id>/<metric_name>
```

The driver tracks NBIRTH/DBIRTH messages to build the `alias_id → metric_name` map; subsequent NDATA/DDATA messages reference metrics by alias for compactness, and the driver resolves them back to names. Quality is set per Sparkplug B's `is_historical` / `is_transient` flags + connection state (NDEATH → all metrics on that edge go `Quality::Bad`).

### Capabilities

```rust
Capabilities {
    native_subscribe: true,    // Pub/sub native to MQTT
    browse: true,              // Sparkplug B BIRTH messages enumerate metrics
    batch_read: false,         // No "read N topics now" — values arrive when broker pushes
    batch_write: true,         // Multiple PUBLISHes per cycle is fine
}
```

`browse` returns:
- Generic mode: configured topics flat in a single `TagNode`.
- Sparkplug B mode: tree of `<group>/<edge>/<device>/<metric>` from BIRTH messages received so far.

### Connection modes

- **Anonymous** (no auth).
- **Username/password**.
- **TLS** (v1 supports trust-the-system-CA, no client certificates).
- **WebSocket** (`wss://broker.example.com/mqtt`).

Client certificate auth is **post-1.0** (same scope rule as OPC UA).

### Quality mapping

| Event | Quality |
|---|---|
| Connected, fresh value received | `Good` |
| Connected, no value seen yet | `Uncertain` |
| Disconnected | `Bad` (all subscribed topics) |
| Sparkplug B: NDEATH received | `Bad` for all metrics under that edge |
| Sparkplug B: DDEATH received | `Bad` for all metrics under that device |

### Test requirements

- `address.rs` unit tests: parse generic topics + Sparkplug B `spB/v1.0/...` paths; reject malformed Sparkplug B paths.
- `sparkplug.rs` unit tests: alias-map birth → data resolution; out-of-order DDATA before DBIRTH ignored with warn; alias collision across edges handled correctly.
- `tests/integration.rs`: spawn `sim-mqtt` (publishes both generic + Sparkplug B), connect via `rumqttc`, assert generic topic values flow through; assert Sparkplug B metric values flow through with names (not aliases) post-BIRTH.
- Reconnect: kill the broker process, restart, assert the driver reconnects and re-subscribes within configured backoff.

### Acceptance criteria

- [ ] `cargo test -p openwebhmi-driver-mqtt` green.
- [ ] `cargo test --workspace --all-features --locked` stays green.
- [ ] `examples/sim-mqtt` runs (uses an embedded `mosquitto` via `docker-compose` OR a Rust in-memory broker like `rumqttd`; document the choice in the wiki).
- [ ] `wiki/drivers/mqtt-integration.md` covers: generic mode, Sparkplug B mode, payload decoders, alias-map lifecycle, NDEATH semantics.
- [ ] `docs/feature-matrix.md`: MQTT (generic) and MQTT Sparkplug B rows flipped to "🟢 v1 (Phase 4, in development)".
- [ ] Manual smoke step added: launch sim-mqtt, configure both a generic and Sparkplug B tag, see live updates in the runtime.
- [ ] All public Rust items have rustdoc; Sparkplug B parsing module has spec section citations as doc comments.

### Out of scope (v1)

- **MQTT v5 features** beyond what's needed for v3.1.1 parity: shared subscriptions, message expiry, request/response correlation. Use v3.1.1 by default; v5 is a connection-config flag.
- **Sparkplug B host application** (the gateway acts as Sparkplug B "primary host" — STATE/REBIRTH coordination): post-1.0. v1 treats Sparkplug B as a one-way data feed.
- **Sparkplug B writes to edge nodes** via NCMD/DCMD messages: post-1.0 (the v1 driver subscribes only).
- **Client certificate auth**: post-1.0.
- **Persistent sessions** across gateway restarts: post-1.0; v1 uses fresh sessions on reconnect.
- **Retain message handling**: v1 honors retained messages but doesn't publish with retain. Post-1.0 adds the publish flag.

### Risks / gotchas

- **Sparkplug B alias map is per-session and fragile.** If the gateway misses an NBIRTH (connect after the edge already published BIRTH), the driver doesn't know what aliases mean. Solution: send STATE message requesting REBIRTH on connect → out-of-scope for v1, document as a limitation.
- **Topic explosion.** A wildcard subscription to `#` will pull every topic on the broker. The driver subscribes only to configured tag topics; document the design.
- **JSON path payloads.** Generic mode supports `payload_type: json_path` with a JSONPath expression. Use `serde_json` + `jsonpath-rust`; pin versions.
- **TLS hostname verification.** Always on by default. Self-signed broker certs need an explicit "skip verify" flag (`tls_insecure: true`); refuse to enable it without an explicit warn at startup.
- **MQTT keep-alive vs. tag staleness.** rumqttc's keep-alive is connection-level. A topic that hasn't published in an hour shows the last known good value; the runtime should communicate this is "last seen at" data. Quality stays `Good` until disconnect. Document.
- **Sparkplug B is a moving target.** Pin to spec version 3.0.0 (Eclipse-Tahu); cite the version in wiki/code.

## Codex log

*(codex — append working notes here)*

## Claude review

*(claude — after submission)*

## Verdict

*(claude — final disposition)*
