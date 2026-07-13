---
id: CODEX-BQ
title: Sparkplug B conformance — signed-int datatype decode, DEATH stale-marking, command writes
owner: codex
phase: 4
status: open
created: 2026-07-12
last-update: 2026-07-12 claude [Fable 5]
---

# CODEX-BQ — Sparkplug B conformance

## Brief

> **MEDIUM, integrator-facing** — four Sparkplug B conformance gaps in the MQTT driver. (a) The metric `datatype` field is parsed but ignored: Sparkplug encodes signed `Int8/16/32` as two's-complement inside `int_value` (a uint32), so `IntValue(value) => TagValue::Int(value as i64)` turns `-1i32` into `4294967295`, and any negative `Int64` (`LongValue > i64::MAX`) flips to a `TagValue::String` mid-stream. Reinterpret using the datatype. (b) The driver subscribes to NBIRTH/DBIRTH/DDATA only; NDEATH/DDEATH (mandatory stale-marking in Sparkplug B) never arrive, so a dead edge node's metrics stay Good forever, and node-level NDATA can't be addressed. Handle DEATH certificates to mark metrics stale and support node-level addressing. (c) `apply_birth` aborts the whole alias map if any BIRTH metric lacks a value, so a property-only BIRTH metric orphans all alias-only DDATA. Skip valueless metrics without discarding the alias map. (d) `write()` publishes `value.to_string()` bytes to `address.raw` as a literal topic — writing to a `raw_float_be` topic sends ASCII, and "writing" a Sparkplug metric publishes to the DDATA topic string instead of issuing an NCMD/DCMD. Encode writes per payload type and issue proper Sparkplug command messages.

### Goal

Signed Sparkplug integers decode to their true signed values. A dead edge node's metrics go stale on NDEATH/DDEATH. Node-level Sparkplug metrics (no device id) are addressable. A property-only BIRTH metric no longer wipes the alias map. Writes encode according to the target payload type, and a Sparkplug metric write issues an NCMD/DCMD command rather than publishing to the data topic.

### Context to read first

- `crates/driver-mqtt/src/sparkplug.rs`:
  - `Metric.datatype`, lines 38-40 — the `Option<u32>` Sparkplug datatype enum value, currently parsed but unused.
  - `metric_to_update`, lines 162-189 — the value mapping at 169-178: `IntValue(value) => TagValue::Int(value as i64)` (170) ignores signedness; `LongValue(value) if value <= i64::MAX` (171) then the `LongValue(value) => TagValue::String(...)` fallback (172) for anything larger — i.e. any negative Int64.
  - `apply_birth`, lines 115-134 — builds `aliases` in the `filter_map` closure (125-130) but `.collect::<Result<Vec<_>, _>>()?` (131) returns early on the first `MissingValue` error, so `self.aliases.insert(key, aliases)` (132) never runs and the whole map is lost. `metric_to_update` returns `Err(MissingValue)` (169) for a metric with no `value`.
  - `apply_data`, lines 137-159 — alias resolution; the node-level vs device-level key handling.
  - The Sparkplug B datatype enum values (Int8=1, Int16=2, Int32=3, Int64=4, UInt8=5 … per the spec) — you need this mapping to reinterpret `int_value`.
- `crates/driver-mqtt/src/driver.rs`:
  - `subscription_filters`, lines 183-208 — subscribes to NBIRTH/DBIRTH/DDATA topics only; no NDEATH/DDEATH.
  - `topic_mappings`, lines 210-244; `parse_birth_topic` (319-342) and `parse_data_topic` (344-354) — the topic classification that needs DEATH cases and node-level (`NDATA`) support.
  - `handle_publish`, lines 246-270 — routes birth vs data; add DEATH handling here.
  - `write()`, lines 134-145 — `client.publish(&address.raw, …, tag_value_to_payload(value))`: publishes `value.to_string()` bytes to the raw address as a topic, wrong for both raw-binary payloads and Sparkplug metrics.
  - `tag_value_to_payload`, lines 542-549 — the `to_string()` encoder that ignores the target `PayloadType`.
- `crates/driver-mqtt/src/address.rs`:
  - `parse_sparkplug`, lines 156-173 — requires exactly the 7-part `spB/v1.0/<group>/DDATA/<edge>/<device>/<metric>` device form; a node-level (6-part, no device) address can't be parsed.
  - `MqttAddressKind::Sparkplug`, lines ~125-138 — the `device_id` field (empty for node-level per `SparkplugKey`'s convention in sparkplug.rs 92-93).
- Sparkplug B 3.0.0 spec: the metric datatype enum, the NDEATH/DDEATH "bdSeq" stale-marking semantics, and the NCMD/DCMD command topic/payload structure. Reference the spec's datatype table for the signed-int reinterpretation.

### Files to create / modify

1. **Modify** `metric_to_update` (sparkplug.rs 162-189) to use `metric.datatype` when mapping `IntValue`/`LongValue`:
   - `Int8` → `i8` from the low bits; `Int16` → `i16`; `Int32` → `i32` — reinterpret the two's-complement `int_value` and widen to `i64`.
   - `UInt8/16/32` → widen unsigned (current behavior is fine for these).
   - `Int64` → reinterpret the `uint64` as `i64` (two's-complement), so negatives decode correctly instead of flipping to `String`.
   - `UInt64` → keep the `> i64::MAX` guard (that value genuinely doesn't fit `TagValue::Int`; keep the existing String/error behavior or map to a `DriverError`-carried quality, matching how the other drivers treat unrepresentable u64).
   - When `datatype` is absent, fall back to the current best-effort mapping and log once (unknown datatype).
2. **Modify** `apply_birth` (115-134) to skip valueless metrics (property-only BIRTH) without failing the whole payload: build the alias map from all `(alias, name)` pairs regardless of value presence, and only emit updates for metrics that have a value. A `MissingValue` on one metric must not discard the alias map.
3. **Modify** the driver topic handling (`subscription_filters`, `parse_birth_topic`/`parse_data_topic`/a new DEATH parser, `handle_publish`) to subscribe to and process NDEATH/DDEATH: on a DEATH certificate, mark that node's/device's metrics stale (emit Bad/Stale updates for the affected addresses, matching the quality variant the resilience work uses). Support node-level NDATA addressing.
4. **Modify** `crates/driver-mqtt/src/address.rs` `parse_sparkplug` (156-173) to accept the node-level (device-less) form as well as the 7-part device form, populating `device_id` empty for node-level — consistent with `SparkplugKey`'s empty-device convention.
5. **Modify** `write()` (driver.rs 134-145) and `tag_value_to_payload` to encode per the target `PayloadType` (raw big/little-endian bytes for `RawFloatBe`/etc., not ASCII), and for a Sparkplug metric address issue an NCMD/DCMD command message to the correct command topic with a proper Sparkplug payload, rather than publishing `to_string()` bytes to the metric's data-topic string.
6. **Add tests** to the existing `#[cfg(test)] mod tests` in `sparkplug.rs`, `driver.rs`, and `address.rs` (do not create new test files).

### Behavior

- Negative signed Sparkplug metrics decode to their true value (`-1i32` → `TagValue::Int(-1)`, not `4294967295`; negative `Int64` → correct negative, not a String).
- NDEATH/DDEATH marks the affected metrics stale; a dead edge node's tags no longer read Good indefinitely.
- Node-level Sparkplug metrics (no device id) parse and resolve.
- A property-only (valueless) BIRTH metric no longer discards the alias map; alias-only DDATA still resolves.
- Writing to a raw-binary topic sends the correct bytes; writing a Sparkplug metric issues NCMD/DCMD, not a data-topic publish.

### Test requirements

- **Signed-int decode tests** (sparkplug.rs): metrics with `datatype = Int8/Int16/Int32/Int64` carrying negative two's-complement payloads decode to the correct negative `TagValue::Int`. Prove the pre-fix path returns the wrong (large positive / String) value for the same input.
- **BIRTH-with-property-only-metric test** (sparkplug.rs): a BIRTH containing one valueless metric plus alias'd metrics still populates the alias map; subsequent alias-only DDATA resolves. Prove pre-fix loses the map (DDATA resolves to nothing).
- **DEATH stale-marking test** (driver.rs): a DDEATH/NDEATH for a known node marks its metrics stale (Bad/Stale updates emitted). Structure so the topic handling is unit-testable without a live broker (drive `handle_publish` or an extracted helper with a synthesized DEATH payload).
- **Node-level addressing test** (address.rs): the device-less Sparkplug form parses; the 7-part device form still parses; genuinely malformed forms still error. Update existing address tests for the widened contract.
- **Write-encoding test** (driver.rs): a `RawFloatBe` write produces the IEEE-754 big-endian bytes (not ASCII); a Sparkplug-metric write targets the NCMD/DCMD topic with a Sparkplug payload (assert against the `AsyncClient` publish seam — extract or spy on the publish call; no live broker).
- No `sleep()`, no hardcoded ports; if a broker is needed, `127.0.0.1:0` plaintext (see the TLS note). Full matrix clean: `cargo test -p openwebhmi-driver-mqtt`, workspace `clippy -D warnings`, `fmt --check`.

### Acceptance criteria

- [ ] `metric_to_update` uses `datatype` to decode signed `Int8/16/32/64` correctly; negatives no longer become large positives or Strings.
- [ ] `apply_birth` skips valueless metrics without discarding the alias map.
- [ ] NDEATH/DDEATH are subscribed and processed; dead nodes' metrics go stale; node-level NDATA addressing works.
- [ ] `parse_sparkplug` accepts the device-less node-level form; device form still parses; malformed still errors.
- [ ] `write()` encodes per `PayloadType` (raw bytes not ASCII) and issues NCMD/DCMD for Sparkplug metric writes.
- [ ] Signed-int, property-only-BIRTH, DEATH, node-level-address, and write-encoding tests fail pre-fix and pass after.
- [ ] No `sleep()`/hardcoded ports; tests added to existing modules.
- [ ] No `unwrap`/`expect`/`panic` on production paths introduced; new suppressions use `#[expect(...)]`.
- [ ] Codex log cites the Sparkplug B datatype enum values used and the NCMD/DCMD topic/payload structure implemented.

### Out of scope

- The event-loop reconnect / broker-drop resilience (CODEX-BL) — coordinate on the shared quality variant for "stale", but don't reimplement the reconnect loop here. If BL merges first, reuse its stale-marking helper; if this merges first, use the same variant BL will adopt and note it.
- Full Sparkplug host-application / STATE / primary-host semantics — v1 is a subscriber/consumer of edge-node data, not a full host app.
- Sparkplug `DataSet`/`Template`/`PropertySet` metric types beyond primitive scalars.
- Sequence-number (`seq`) gap detection and rebirth requests — a conformance nicety, separate brief if wanted.
- TLS validation (maintainer manual-smoke gate).

### Risks / gotchas

- **Two's-complement reinterpretation, not casting.** For `Int16`, the wire `int_value` (uint32) holds the value in its low 16 bits as two's-complement: `value as u16 as i16 as i64`. Get the widths right per datatype; a test with `-1`, `i8::MIN`, `i16::MIN`, `i32::MIN`, `i64::MIN` catches off-by-width errors.
- **DEATH doesn't carry the metric list.** An NDEATH/DDEATH certificate marks the whole node/device dead; you mark stale the metrics you already know from the prior BIRTH's alias map (scoped by `SparkplugKey`). Use the stored alias map to enumerate which addresses to mark — don't expect the DEATH payload to name them.
- **Node-level vs device-level key convention.** `SparkplugKey.device_id` is empty for node-level (sparkplug.rs 92-93). Keep `parse_sparkplug`, `parse_birth_topic`, `parse_data_topic`, and the new DEATH/NDATA parsers consistent with that convention so alias maps key correctly.
- **NCMD/DCMD payload is a real Sparkplug protobuf**, not a raw value — it carries a metric (name/alias + value + datatype) in the same `Payload` message shape. Reuse `encode_payload`/`Metric`; confirm the command topic form (`spB/v1.0/<group>/NCMD/<edge>` and `.../DCMD/<edge>/<device>`) against the spec. Confirm whether the metric write should carry a name or the alias.
- **Don't take the write bug's symptom narrowly.** It's two bugs: raw-payload writes send ASCII (fix `tag_value_to_payload` to honor `PayloadType`), and Sparkplug writes go to the wrong topic/shape (issue NCMD/DCMD). Fix both; don't just special-case Sparkplug and leave raw writes broken.
- **Coordinate the stale quality variant** with CODEX-BL so both use the same `Quality` value for "connection/node lost". Note the dependency in the Codex log.
- **Honesty:** end-to-end Sparkplug conformance against a real edge node (Ignition/Chariot, etc.) is a maintainer manual-smoke gate — say so; unit tests prove decode/encode/topic logic, not interop.

## Codex log

## Claude review

## Verdict
