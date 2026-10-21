---
id: CODEX-DU
title: Upgrade frontend tool families with compatibility and footprint checks
owner: codex
phase: 4
status: submitted
created: 2026-09-27
last-update: 2026-10-21 codex [GPT-6]
---

# CODEX-DU — Upgrade frontend tool families with compatibility and footprint checks

## Brief

### Goal

Bring remaining frontend frameworks/tooling onto supported releases without a blind all-package update.

### Context and dependencies

DK version inventory; DQ Monaco integration; CB/BY runtime quality; existing React component suite.

Authoritative contract: [engine and capacity plan](../../planning/engine-and-capacity.md).
Read root/scoped AGENTS.md and VISION.md before implementation. This task must not
represent planning targets as measured product capability.

### Required behavior

- Recheck latest supported versions at execution; checkpoint Vite/React plugin/Vitest/jsdom/Monaco together where compatibility requires it, then React/types/test adapters, Astro/compiler API, and pnpm separately.
- Preserve strict typechecking, frozen lockfile builds and official supported peer ranges; do not suppress conflicts or introduce unused state/chart/canvas libraries.
- Keep the website independent and use the compiler API version supported by Astro; the current TS5 exception is temporary and documented.
- Measure app/editor bundle sizes, dependency/feature changes and representative interactions; review pnpm lifecycle-build policy before major migration.

### Acceptance and validation

- [ ] Each compatible group passes clean frozen install, build, typecheck, existing tests and appropriate browser smoke before the next group.
- [ ] No unrelated Cargo changes or automatic protocol upgrades; inbound license/advisory and bundle reports are recorded.
- [ ] No inferred speedup from version numbers; publish before/after values if claiming improvement.

Run checks proportionate to the changes plus required workspace validation.
Behavior regressions must demonstrably fail without their fix. Mechanical code
changes require compile and three consecutive existing-suite passes. Use existing
test files, deterministic synchronization and ephemeral ports. Record hardware,
browser and cross-platform checks that were not performed by name.

### Out of scope

Do not implement adjacent tasks under this task's completion claim. No clustering,
MES expansion, license change, unbounded queues, telemetry, stub drivers or broad
Cargo dependency update. Preserve existing project/wire compatibility or ship an
explicit paired migration. Do not claim a performance gain without measurement.

## Codex log

### 2026-10-11 13:06 codex [GPT-6] — maintainer-directed modernization coordination

EI owns Google TypeScript policy, lint/formatter adoption and conformance; EK owns Bun execution trials and any explicitly gated package-manager migration. Coordinate compatible versions and avoid concurrent unrelated lockfile changes. Existing pnpm authority remains until EK validates and updates the complete policy.

### 2026-10-21  codex [GPT-6] — execution started

Maintainer-directed latest-stable upgrades are being executed together across EG/EH/DU; exact upstream versions and compatibility constraints are rechecked at execution. Edition 2024 and protocol contracts remain in scope for preservation. No validation result is claimed yet.

### 2026-10-21 22:54 codex [GPT-6] — submitted

The [upgrade evidence](../notes/2026-10-dependency-upgrade.md) inventories all frontend declarations. React 19.3.1, Vite 8.3.5/plugin-react 6.1.2, Vitest 5.0.5, jsdom 30.1.3, Monaco 0.57.0, Astro 7.4.2 and pnpm 12.14.0 are selected from current stable registry metadata. Product TypeScript 7.0.2 was already current. Website TypeScript moves to 6.0.2 because latest Astro check supports only major 5/6; no peer-range suppression is used. Node stays on its existing LTS family at latest patch 24.22.1. The machine-wide Node/pnpm commands were not replaced; validation used checksum-verified temporary executables matching project pins.

pnpm 12 uses strict peers, explicit esbuild lifecycle permission, narrow patched transitive overrides, exact verified release-age exclusions, and a Monaco-plugin package extension for its undeclared esbuild import. The clean build failed before the extension and passed afterward. The website's unused optional Sharp/libvips chain is excluded and Astro's documented passthrough service preserves its static SVG assets. Three final website builds passed.

Fresh frozen force installation, strict typechecking, recursive builds and three final recursive test runs passed (152 tests each). Frontend audit reports zero known vulnerabilities. Runtime/designer entry chunks grew from 447.73/523.28 kB to 527.67/586.03 kB (gzip 124.48/139.19 to 153.80/166.96 kB); no speed/footprint improvement is claimed. The unsigned debug macOS arm64 app bundle passed. Chrome smoke verified disposable local login/project loading, designer component tree/theme navigation, runtime connected view and modal Cancel. The existing fixed embedded-preview URL, Monaco worker/editing interactions, full manual checklist, Linux/Windows and signing/notarization were not validated. Paired Cargo changes are intentionally owned by EG/EH under the maintainer's combined upgrade request. Submitted as an uncommitted diff; no push.

### 2026-10-21 23:01 codex [GPT-6] — maintainer-directed publication

The maintainer approved staging, committing and pushing the validated combined upgrade to main. Submission status is retained for independent review; this instruction does not constitute an independently executed merge review.

## Claude review

## Verdict
