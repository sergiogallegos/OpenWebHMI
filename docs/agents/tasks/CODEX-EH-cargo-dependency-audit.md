---
id: CODEX-EH
title: Audit and upgrade Rust dependencies with measured feature budgets
owner: codex
phase: 4
status: submitted
created: 2026-10-11
last-update: 2026-10-21 codex [GPT-6]
---

# CODEX-EH — Audit and upgrade Rust dependencies with measured feature budgets

## Brief

### Goal

Inventory and selectively upgrade maintained Rust crates while reducing unnecessary dependency/features at the existing accepted engine boundaries. A version upgrade is not evidence of a lighter graph.

### Evidence and context

On 2026-10-11 Cargo.lock contains 662 external package/version entries and 56 package names with multiple versions across the entire workspace. These are lockfile inventory counts, not resolved minimal-runtime counts. Workspace Tokio enables `full`; async-opcua enables client/server together. An unused workspace PyO3 declaration exists although no PyO3 package appears in Cargo.lock; CI already flags this in its brief.

Read root AGENTS.md, VISION.md, [engine/capacity plan](../../planning/engine-and-capacity.md), DO, DM/DN, DT, BK and CK. DO owns feature isolation and profile architecture; this task owns version inventory, focused upgrades and clearly unused dependency cleanup.

### Required work

- Use locked Cargo metadata/tree to record direct, transitive, normal/build/dev and target-specific dependencies, duplicate versions and feature provenance. Separate Rust gateway/engine, optional services, simulator/test tools and Tauri host graphs. Include clean build time and artifact size where measurable.
- Produce a dated table of declared/resolved version, latest supported candidate verified from crates.io/upstream, security/support status, license, enabled features, consumers, compatibility risk and decision (upgrade/remove/defer with reason). Cover every used direct external dependency; do not describe unverified candidates as current releases.
- Prioritize security/maintenance needs, then compatible upgrades. Treat protocol pins tokio-modbus, async-opcua, rumqttc, rumqttd, prost, jsonpath-rust and ads as explicitly reviewed version-bump candidates with driver compatibility evidence; retain exact pins at approved new versions. Coordinate rust-ethernet-ip with BK.
- Upgrade one coherent family at a time using `cargo update -p <package> --precise <version>` and explicit manifest edits. Never run an unrestricted cargo update; explain necessary transitives and reject unrelated lockfile drift.
- Verify the PyO3 declaration is unconsumed before removal. Remove unused dependencies, keep maintained security/protocol libraries and leave DO to narrow Tokio/driver features without duplicating its scope.
- Measure before/after closures, duplicates and features under each currently available profile; identify planned profiles as unavailable until DO lands. Document runtime versus development-only cost and permissive inbound license/advisory results.

### Acceptance and validation

- [ ] Complete used-direct-dependency inventory, upgrade decisions and baseline/after reports with official source links and observation dates.
- [ ] Each upgrade group passes locked compile, relevant existing/regression suites and full root matrix before proceeding; mechanical edits require three existing-suite green runs.
- [ ] Pin changes have real protocol/simulator coverage, migration notes and named hardware checks not performed. No behavior test can pass without the behavior fix it claims to cover.
- [ ] Minimal domain std/core and narrow asynchronous-service targets remain consistent with DM/DN/DO; no dependency-count-driven rewrites of crypto or protocols.

### Out of scope

Blanket dependency updates, new transports/backends, claiming DO profile extraction complete or replacing real protocols with stubs.

## Codex log

### 2026-10-21  codex [GPT-6] — execution started

Maintainer-directed latest-stable upgrades are being executed together across EG/EH/DU; exact upstream versions and compatibility constraints are rechecked at execution. Edition 2024 and protocol contracts remain in scope for preservation. No validation result is claimed yet.

### 2026-10-21 22:54 codex [GPT-6] — submitted

All 36 used direct external Cargo dependencies were inventoried and moved to verified latest stable releases where newer versions exist. Existing exact driver pins remain exact. Unused PyO3 was removed after confirming it had no consumer or lockfile entry. The [upgrade evidence](../notes/2026-10-dependency-upgrade.md) records versions, licenses, features, consumer/dependency kinds, graph closures, duplicates, migrations and residual risks. Explicit manifest upgrades were resolved with a targeted Tokio update; precise rustls/rustls-webpki and JWT updates followed. No unrestricted cargo update ran. The wide lockfile diff reflects the explicitly upgraded families and their transitives, including OPC UA and Tauri; it is not presented as a one-crate patch.

Real-protocol simulator suites pass with OPC UA 0.19, Modbus 0.17 and WebSocket 0.30. Existing backup tests exposed the TOML document-parser migration before its correction (7 failures); all three final workspace suites pass afterward. The AWS-LC JWT backend is explicit. Exact TLS patches are rustls 0.23.46 and rustls-webpki 0.103.16.

The final inventory has 671 external package/version entries versus 662 before, with 62 versus 56 duplicate names. This is not a footprint reduction. The OSV scan retains findings in eight upstream-bound/unmaintained package versions, including production MQTT certificate validation and OPC UA RSA; the report identifies paths and advisory IDs. Axum 0.7.9 remains broker-owned because latest rumqttd 0.20 requires that major; latest Axum 0.8.9 is not falsely forced into it. DO feature/profile extraction and clean before/after Rust build-time measurement were not performed. Full locked build/clippy/test/doc/fmt matrix passed with three final test runs. Hardware and Linux/Windows validation remain unperformed. Submitted as an uncommitted diff for review, with the named measurement/upstream limitations; no push.

### 2026-10-21 23:01 codex [GPT-6] — maintainer-directed publication

The maintainer approved staging, committing and pushing the validated combined upgrade to main. Submission status is retained for independent review; this instruction does not constitute an independently executed merge review.

## Claude review

## Verdict
