---
id: CODEX-AA
title: Close v1 scope gaps in driver-mqtt (TLS + WebSocket + json_path + binary BE)
owner: codex
phase: 4
status: open
created: 2026-05-01
last-update: 2026-05-01 claude
---

# CODEX-AA — MQTT v1 scope follow-up

## Brief

> **Phase 4 follow-up to [CODEX-Y](CODEX-Y-driver-mqtt.md).** Y merged with the v1 happy path (TCP + anonymous/username + generic UTF-8/raw_*_le + Sparkplug B), but four v1 commitments from the brief body didn't ship: TLS transport, WebSocket transport, `json_path` payload type, and binary big-endian payload decoders (the current `RawIntBe`/`RawFloatBe` decode ASCII text). The feature-matrix entry stays at "in development" until this lands.

### Goal

Close the v1 scope gaps so MQTT can flip to "🟢 v1 (Phase 4, simulator-validated)" in the feature matrix. Each gap is small and self-contained — this should be one focused commit, not a rewrite.

### Context to read first

- `crates/driver-mqtt/src/connection.rs` — `MqttTransport` enum + `MqttConfig`. `MqttTransport::Tls` and `WebSocket` variants exist; the driver explicitly rejects them at `driver.rs:60-64`.
- `crates/driver-mqtt/src/driver.rs:60-64` — the `UnsupportedType` early-return that needs to be replaced with real connect logic.
- `crates/driver-mqtt/src/driver.rs:389-425` — `decode_payload` where `RawIntBe`/`RawFloatBe` currently parse ASCII decimal instead of binary BE bytes.
- `crates/driver-mqtt/src/address.rs:8-22` — `PayloadType` enum where `JsonPath` needs to be added.
- `rumqttc` `Transport` API: `Transport::Tls(...)` takes a rustls `ClientConfig`; `Transport::Ws` for WebSocket.

### Files to modify

- `crates/driver-mqtt/Cargo.toml` — add `rustls`, `rustls-pemfile`, `webpki-roots` (or use `rustls-platform-verifier` for system-CA trust); add `jsonpath-rust` for `json_path`.
- `crates/driver-mqtt/src/connection.rs` — extend `MqttConfig` with optional TLS settings (`tls_insecure: bool`, `ca_cert_path: Option<PathBuf>`) and WS path (`ws_path: Option<String>`).
- `crates/driver-mqtt/src/driver.rs` — replace the `UnsupportedType` early-return with three branches that build the right `rumqttc::Transport`:
  - `Tcp` → existing path (already works)
  - `Tls` → `Transport::Tls(rumqttc::TlsConfiguration::Rustls(Arc::new(rustls_config)))`
  - `WebSocket` → `Transport::Ws` with the WS URL form
- `crates/driver-mqtt/src/driver.rs` — fix `decode_payload`:
  - `RawIntBe` decodes 8 raw big-endian bytes via `i64::from_be_bytes` (mirror `RawIntLe`'s 8-byte LE path)
  - `RawFloatBe` decodes 8 raw big-endian bytes via `f64::from_be_bytes`
  - Keep the ASCII-text decoders as new variants `Utf8Int` / `Utf8Float` if useful (or drop entirely if Codex doesn't think they're needed)
- `crates/driver-mqtt/src/address.rs` — add `PayloadType::JsonPath { path: String }` variant. Decoder evaluates the JSONPath expression against the JSON-parsed payload. Use `jsonpath-rust` (pin the version, document in wiki).
- `crates/driver-mqtt/tests/integration.rs` — add coverage:
  - TLS connect against a TLS-enabled `rumqttd` instance (or skip if rumqttd doesn't support TLS easily — document the limitation)
  - WebSocket connect against a `rumqttd` WS endpoint
  - Binary `raw_int_be` and `raw_float_be` payloads
  - `json_path` payload extracts a value from a nested JSON object
- `crates/driver-mqtt/tests/integration.rs` — **fix the flake**: the existing `sim_publishes_generic_and_sparkplug_updates` test failed once during the CODEX-Y review on a clean `cargo test --workspace --all-features --locked` run, passed on retry. Likely a broker-startup race. Wait for the broker to be reachable (e.g. dial-test the listening port) before the driver `connect`, or add a `tokio::time::sleep(Duration::from_millis(100))` after `sim.start_ephemeral`.
- `crates/driver-mqtt/src/driver.rs` — quality state for "Connected, no value seen yet → Uncertain": when subscribed but no message has arrived, `read()` should return `Quality::Uncertain` with a sentinel value (or the last-known value if any). Today it returns `NotConnected`.
- `wiki/drivers/mqtt-integration.md` — update with TLS configuration matrix, WS URL form, JSONPath usage examples, and the corrected payload-decoder semantics.
- `docs/feature-matrix.md` — flip MQTT (generic) and MQTT Sparkplug B from "in development" to "simulator-validated" on merge.

### Out of scope (still post-1.0)

- Client certificate auth.
- MQTT v5-only features (shared subs, request/response).
- Sparkplug B host application (NCMD/DCMD writes, primary-host coordination).
- Persistent sessions across gateway restarts.

### Test requirements

- `address.rs`: `JsonPath` parses; `payload_type: { json_path: "$.outer.inner" }` round-trips through serde.
- `driver.rs`: binary `raw_int_be` decodes correctly (`from_be_bytes` of `[0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x2A] → 42_i64`); binary `raw_float_be` decodes correctly.
- Integration: TLS handshake completes against a TLS broker fixture (or document why it's skipped); WS connect completes against a WS broker fixture; existing flake is fixed.

### Acceptance criteria

- [ ] `cargo test -p openwebhmi-driver-mqtt --features sim-tests` green on three consecutive runs (flake gate).
- [ ] `cargo test --workspace --all-features --locked` green.
- [ ] `cargo clippy --workspace --all-targets --all-features -- -D warnings` clean.
- [ ] Manual smoke step in `apps/designer/README.md` updated with a TLS connection example (using `mosquitto` or rumqttd-tls fixture; document the prereq).
- [ ] `docs/feature-matrix.md`: MQTT (generic) and MQTT Sparkplug B flipped to "simulator-validated".

### Risks / gotchas

- **rumqttd's TLS support varies by version.** If `rumqttd = "=0.20.0"` doesn't expose a clean TLS server fixture for tests, a `mosquitto` docker-compose or a hand-rolled rustls server may be needed. Document the choice.
- **`jsonpath-rust` is one of several JSONPath crates.** Pin a specific version; document version + commit in wiki. The crate's recursion depth on deeply-nested payloads may need a guard.
- **System CA roots vs bundled roots**: `rustls-platform-verifier` uses platform-native roots (good for production); `webpki-roots` bundles Mozilla roots (good for portability). Pick one with rationale documented.
- **Don't break the existing test** that already covers the v1 happy path.

## Codex log

*(codex — append working notes here)*

## Claude review

*(claude — after submission)*

## Verdict

*(claude — final disposition)*
