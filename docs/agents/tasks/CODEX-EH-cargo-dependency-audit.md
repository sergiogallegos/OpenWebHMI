---
id: CODEX-EH
title: Audit and upgrade Rust dependencies with measured feature budgets
owner: codex
phase: 4
status: open
created: 2026-10-11
last-update: 2026-10-11 codex [GPT-6]
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

## Claude review

## Verdict
