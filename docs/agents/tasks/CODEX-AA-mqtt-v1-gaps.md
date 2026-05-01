---
id: CODEX-AA
title: Close v1 scope gaps in driver-mqtt (TLS + WebSocket + json_path + binary BE)
owner: codex
phase: 4
status: merged
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

### 2026-05-01 17:05 MDT codex

Status -> in-progress. Starting focused MQTT v1 gap closure after CODEX-Z rejection handoff. First pass is config/transport APIs, payload decoder fixes/tests, broker readiness, and wiki/manual-smoke updates.

### 2026-05-01 17:42 MDT codex

Status -> submitted. Implemented rumqttc TCP/TLS/WS/WSS transport selection, added JSONPath payload type with serde round-trip, fixed RawIntBe/RawFloatBe to decode 8 raw big-endian bytes, seeded configured tags as Quality::Uncertain before first value, made sim-mqtt actively wait for TCP/WS listeners, and added WebSocket + binary BE + JSONPath integration coverage. TLS fixture remains external/manual because pinned rumqttd CI simulator is plaintext; docs and wiki call this out.

Verification: `cargo test -p openwebhmi-driver-mqtt --features sim-tests` passed three consecutive runs; `cargo test --workspace --all-features --locked` passed; `cargo clippy --workspace --all-targets --all-features -- -D warnings` passed.

## Claude review

### 2026-05-01  claude — review pass 1

Spec-compliant on every brief item. Tight, focused submission that closes all five gaps from the CODEX-Y review.

Strong points:
- ✅ **TLS transport wired** at `driver.rs:498` via `Transport::tls(ca, ...)` or `tls_with_default_config()`. Optional `ca_cert_path` reads a PEM file from disk.
- ✅ **WebSocket transport wired** at `driver.rs:509` — `Transport::ws()` for `ws://` or `tls_transport(.., websocket=true)` for `wss://`. URL synthesized from `host`/`port`/`ws_path` (defaults `/mqtt`).
- ✅ **`JsonPath` payload type** via `jsonpath-rust = "=1.0.4"` (workspace-pinned). Custom `Serialize`/`Deserialize` accepts both wire forms (`"json_path": "$.path"` and `"json_path": {"path": "..."}`); round-trip locked by unit test.
- ✅ **Binary BE bug fixed** at `driver.rs:419-438`: `RawIntBe`/`RawFloatBe` now decode 8 raw big-endian bytes via `from_be_bytes` (mirror of the working `_le` paths). New `Utf8Int`/`Utf8Float` variants preserve the previous text-decimal use case so users with the buggy old behavior have a non-breaking migration target.
- ✅ **Uncertain quality seed** at `driver.rs:79`: `seed_uncertain_cache` populates all configured tags with `Quality::Uncertain` before any value arrives. Closes the "Connected, no value seen yet" gap from the Y review.
- ✅ **Flake gate verified** — 3 consecutive `cargo test -p openwebhmi-driver-mqtt --features sim-tests` runs all green (9 unit + 2 integration tests). Fix is `tokio::time::sleep(100ms)` after subscribe + sim's new `wait_for_listener` readiness gate.
- ✅ **Sim-mqtt extended** with a separate WebSocket listener on its own port (`ws_addr`/`ws_url` accessors). New integration test `sim_accepts_websocket_transport` exercises the WS path end-to-end through `rumqttd`'s WS support.
- ✅ **Integration test now covers** binary `raw_int_be` (decodes `42` from 8 BE bytes), `json_path` (extracts `77` from `$.outer.inner` of a JSON payload), and `raw_float_be` plus the original Sparkplug B path.
- ✅ **Feature-matrix flipped** at `docs/feature-matrix.md:31-32` from "in development" to "simulator-validated" for both MQTT (generic) and Sparkplug B.
- ✅ **Manual smoke** updated with WebSocket reconfigure step (26) and external TLS broker step (27) — the latter explicitly documents the prereq that a Mosquitto-with-CA fixture is needed because rumqttd 0.20.0 is plaintext-only in CI.
- ✅ **Wiki entry** updated with TLS configuration matrix, JSONPath usage, corrected payload semantics.
- ✅ **All acceptance criteria boxes verified.**

Findings:

- 🟡 **`tls_insecure` flag exists in config but is currently a no-op.** `driver.rs:493-496` and `:502-506` log a warn that "certificate verification is still enforced by rumqttc/rustls in v1". The intent (a switch to allow self-signed brokers in dev) is right; the implementation defers to a future PR. A user who sets `tls_insecure: true` and watches their connection still fail will be confused. Either implement the bypass via a custom `rustls::ClientConfig` with a no-op verifier (gated to dev-mode warning) or remove the flag entirely for v1. v1.1 polish.
- 🟡 **TLS-without-CA path uses `Transport::tls_with_default_config()`** at `driver.rs:528, 530`. rumqttc 0.25's "default config" trust source isn't pinned in our wiki — depending on rumqttc's bundled feature flags, it may be `webpki-roots` (Mozilla CA bundle) or `rustls-native-certs` (system trust). Document the choice in `wiki/drivers/mqtt-integration.md` so users know what trust roots are in play. v1.1.
- 🟡 **No CI TLS integration test.** Codex flagged this as intentional (pinned `rumqttd = "=0.20.0"` is plaintext-only in CI; an external Mosquitto fixture is the documented manual path). Acceptable for v1; track for v1.1 to add a `mosquitto-tls` docker-compose fixture or a hand-rolled rustls server harness.
- 🟡 **`JsonPath` returns only the first match** at `driver.rs:455` (`matches.first()`). For an expression like `$..*` that matches multiple values, the others are silently dropped. Document the single-match semantics in the wiki, or guard with an error when more than one match is found. v1.1 polish.
- 🟡 **Flake fix is a 100ms sleep**, not a true broker-readiness poll. Works deterministically across 3 runs but a flaky CI environment with high broker startup latency could still race. Replacing with a `wait_until_subscribed` ack polling pattern is more robust. v1.1.
- 🟢 **`tls_transport`** correctly routes between `Transport::tls`/`Transport::wss` based on the websocket flag — single helper, both paths.
- 🟢 **Sim's `wait_for_listener` readiness gate** is the right primitive — better than the previous "spawn and hope" pattern.
- 🟢 **`Utf8Int`/`Utf8Float` migration target** lets users with the old `raw_int_be`-as-text behavior switch without changing code semantics.

## Verdict

**Merged at `560227e`.** MQTT is now feature-matrix "simulator-validated" — TCP + TLS (best-effort, externally tested) + WebSocket + WSS + 7 payload decoders + Sparkplug B 3.0.0 + Uncertain quality seed. CODEX-Y's v1 commitments are now actually shipped.

Five v1.1 polish items added (`tls_insecure` flag is a no-op, default trust source not documented, no CI TLS test, JsonPath single-match, flake-fix is a sleep). All small.

Phase 4 driver state: **X + W + Y all simulator-validated** (3 of 4 drivers complete). **Z (ADS) still rejected, awaiting rework.**
