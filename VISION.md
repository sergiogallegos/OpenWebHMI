# OpenWebHMI Vision

OpenWebHMI is an open-source, web-first SCADA/HMI platform for small and mid-size industrial systems — a credible alternative to Inductive Automation Ignition and Rockwell FactoryTalk Optix that a plant could actually run. MIT-licensed, gateway-centric, self-hosted. Three languages total (Rust gateway, TypeScript runtime/designer, Python scripting), five v1 drivers (Rockwell EtherNet/IP, OPC UA, Modbus TCP/RTU, MQTT incl. Sparkplug B, Beckhoff ADS).

This document defines what OpenWebHMI **is**, what it explicitly **is not**, and what we **will not merge**. The bullet lists are load-bearing — they describe the project's identity, not a snapshot of current code. Changes here are project-direction decisions, not implementation choices.

For implementation rules, see `AGENTS.md`. For the agent collaboration model, see `CLAUDE.md` and `docs/agents/README.md`.

## What v1.0 ships

- **Single gateway** per deployment. No clustering, no federation.
- **≤ 10,000 live tags** under one gateway.
- **≤ 50 concurrent runtime clients** per gateway.
- **Five drivers**: Rockwell EtherNet/IP (CompactLogix, ControlLogix), OPC UA (vendor-neutral), Modbus TCP/RTU (serial + TCP), MQTT (generic + Sparkplug B), Beckhoff TwinCAT (ADS).
- **Web-only runtime** (browser). Tauri desktop runtime is post-1.0.
- **Linux + macOS + Windows** for the gateway and designer.
- **Pre-1.0 hardware-validation gate**: 24-hour continuous run of `driver-rockwell` against real CompactLogix/ControlLogix hardware before the 1.0 tag.

These bounds are *the* design constraint. If anything in this repo implies bigger numbers, it's wrong and should be fixed. Larger deployments are a year-2+ conversation.

## What OpenWebHMI is not (and won't try to be in v1.0)

- A **hard-real-time** control platform. Use a PLC.
- A **safety-rated** (IEC 61508 / SIL-rated) system.
- A **replacement for vendor PLC engineering tooling** (Studio 5000, TwinCAT, TIA Portal). OpenWebHMI consumes PLCs; it doesn't program them.
- An **MES platform**. Recipes, OEE, batch (ISA-88), and traceability are explicit post-v1 (year-2+) scope and out of scope for the v1.0 headline.
- An **enterprise-scale** (>10K tag, >50 client, multi-gateway) system. The envelope is set above.
- A **cloud SaaS**. OpenWebHMI is self-hosted by design. Hosted offerings, if they happen, are post-1.0 and additive.

## What we won't merge

These are *contracts*, not preferences. A PR that lands code violating one of them is a regression even if the test suite is green.

### License & dependency hygiene

- **No non-MIT-compatible dependencies.** OpenWebHMI is MIT-licensed; transitively-pulled crates and npm packages must be MIT, Apache-2.0, BSD, or compatible. GPL/AGPL/SSPL dependencies do not land.
- **No commercial vendor SDKs requiring NDAs, license keys, or per-seat fees.** v1 driver protocols are reached via open implementations: `rust-ethernet-ip` for CIP/EtherNet/IP, `async-opcua` for OPC UA, `tokio-modbus` for Modbus, `rumqttc` + `prost` for MQTT/Sparkplug B, `ads` for Beckhoff ADS. New drivers follow the same rule.
- **Workspace-pinned versions stay pinned.** The current `=`-pinned set (`tokio-modbus`, `async-opcua`, `rumqttc`, `rumqttd`, `prost`, `jsonpath-rust`, `ads`) is load-bearing for driver-side compatibility. Unpinning requires a task brief, not a drive-by.
- **`Cargo.lock` diff is bounded.** Bumping one crate should touch that crate + its proc-macro counterpart + direct transitives. A wider lockfile diff is a red flag and needs investigation before commit. `cargo update` (no arguments) never lands.

### Privacy & operator trust

- **No telemetry by default.** No phone-home, no opt-out crash reporting, no usage analytics shipped with the gateway. If an operator wants to send diagnostics upstream, that's a deliberate config they enable per deployment.
- **No third-party calls during startup or normal operation.** The gateway runs in air-gapped plants. Anything that fails when there's no internet is a bug.
- **No bundled accounts or default credentials.** Auth is local-first, configured by the operator.

### Correctness & honesty

- **No stub implementations of protocols.** A driver that imports a protocol crate as a marker and exchanges JSON-line frames with a fake "simulator" is rejected even if the tests pass. The driver must speak the real wire protocol against the real (or upstream-vendored) library. See the CODEX-Z v1 rejection rationale.
- **No tests that pass without the fix.** Regression tests must demonstrably catch the bug they're guarding against — run the test against the pre-fix code at least once.
- **No `--release`-only correctness.** Code that relies on `debug_assertions` being off, or behaves differently between debug and release, is a bug. The development matrix is always debug builds.
- **No flaky tests.** No `sleep()`, `setTimeout()`, or wall-clock waits. Use deterministic synchronization (channels, oneshots, `tokio::time::pause()` + `advance()`).
- **No hardcoded ports.** Bind `127.0.0.1:0` and read the assigned port back from the listener.
- **No "I tested it" without running it.** Verification claims in commit messages, PR descriptions, and `## Codex log` entries describe what *actually* ran in the submitter's environment. If something was deferred (hardware, manual smoke), say so by name.

### Architectural invariants

- **Three languages total** — Rust, TypeScript, Python. Adding a fourth requires the kind of justification that goes in `docs/stack-rationale.md`, not a PR comment.
- **Drivers are in-process, owned by the gateway** in v1. Out-of-process drivers are post-1.0.
- **SQLite is v1 persistence** for project store, historian, auth, audit log, alarm journal. Pluggable backends are post-1.0.
- **JSON over WebSocket** is the wire protocol. No Protobuf, no gRPC, no REST-first redesign in v1.
- **One gateway per deployment.** Multi-gateway federation is post-v1.
- **No breaking driver or protocol changes without an in-PR migration path.** The protocol is `packages/protocol-ts`; drivers implement `driver-api`. Both have versioning discipline.

### Scope discipline

- **No MES (recipes, OEE, batch, traceability) in v1.0 code.** Planning notes are allowed in `docs/roadmap.md` Phase 5+; implementation is not.
- **No mobile-native runtime in v1.0.** Web-only.
- **No clustering / redundancy / hot-standby in v1.0.**
- **No designer features that change the runtime contract without a paired runtime change in the same PR.** Designer and runtime ship together.

## What earns a "yes, ship it"

The mirror of the above:

- New driver: real protocol against the real crate, simulator harness for CI, wiki entry that's honest about what's been proven (and what hasn't), designer manual-smoke step added.
- New component: bound to a real data path (read + write where applicable; see the `tagPath` write-back pattern in `packages/component-library`), gallery entry, vitest coverage, designer manual-smoke step.
- New gateway capability: behaviour test in the closest existing test file, wiki entry if a vendor/library quirk surfaced, docs update in the same PR if user-visible.
- Performance work: starts from a baseline number, lands with an after number from the same harness.

When in doubt, the bias is: ship a smaller thing that's honest about its scope, not a bigger thing with deferred verification.
