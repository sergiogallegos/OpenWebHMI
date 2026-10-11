---
id: CODEX-EK
title: Evaluate and standardize compatible frontend tooling on Bun
owner: codex
phase: 4
status: open
created: 2026-10-11
last-update: 2026-10-11 codex [GPT-6]
---

# CODEX-EK — Evaluate and standardize compatible frontend tooling on Bun

## Brief

### Goal

Evaluate Bun for development/build execution and standardize supported commands on it where measured compatibility permits. Make runtime execution and package-manager selection separate decisions.

### Evidence and sources

Root package.json pins pnpm 9.15.9 and Node >=24; .node-version is 24.21.0. CI uses setup-node and pnpm, and frontend scripts run Vite, Vitest, TypeScript, Astro and optional Tauri CLI. The shipped gateway is Rust, the HMI runs in browsers and scripts use Python; no Node production server was found in the inspected workspace manifests/CI.

[Bun runtime docs](https://bun.sh/docs/runtime) and [Node compatibility docs](https://bun.sh/docs/runtime/nodejs-compat) are starting sources, not proof that this repository's tools are compatible. Bun supports invoking Node-shebang CLIs with Bun explicitly; a bun run wrapper alone does not prove execution migrated.

### Required work and decision gates

- Inventory Node shebangs/APIs, subprocess launchers, package scripts, CI jobs, lifecycle/native installs and platform needs. Verify latest stable Bun and official support for each tool version at execution; pin the exact tested Bun release.
- Coordinate DU version groups first or freeze their versions for the trial, then record a matrix for tsc, Vite/React plugin, Vitest/jsdom, Astro/check/compiler, Monaco workers and optional Tauri/native packaging.
- Begin with a compatibility lane using existing pnpm lock/install resolution and explicit Bun runtime invocation for eligible CLIs. Keep Vitest, strict typechecking and pnpm authoritative while evaluating. Verify actual subprocess runtime; do not claim Bun adoption because bun delegates to Node.
- Measure clean/cached install separately from execution, build/test time, bundle output and memory on the same recorded environment. Correctness and offline deployability outrank speed; no speedup assumed from marketing.
- Promote proven commands to default Bun execution with documented Node fallback for specific blockers. Record unsupported versions/platforms, workers/process behavior and executable resolution.
- If full Bun package-manager adoption is justified, provide an explicit reviewed policy/lockfile migration stage: replace pnpm-only rules and CI/install commands together, test workspace resolution, frozen installs, lifecycle permissions, peer ranges, licenses/advisories and rollback. Maintain one authoritative lockfile/package manager; no partial dual-lockfile state.
- Neither Node nor Bun becomes a mandatory runtime dependency for deployed Rust engines or browser clients.

### Acceptance and validation

- [ ] Versioned compatibility matrix, actual runtime evidence, comparable measurements and concrete adopt/defer reasons are recorded.
- [ ] Supported Bun-default commands pass clean reproducible install, typecheck, existing Vitest suites three times, build and browser smoke; applicable Linux/macOS/Windows and Tauri gaps are named.
- [ ] Node fallback and rollback remain documented; package manager stays pnpm unless the explicit complete migration stage passes.
- [ ] CI and root/scoped instructions match the chosen execution/install policy without weakening tests or dependency budgets.

### Out of scope

Replacing Rust/Python production services, moving browsers to a server runtime, changing test runner to Bun tests, or adopting Bun's package manager without compatibility evidence.

## Codex log

## Claude review

## Verdict
