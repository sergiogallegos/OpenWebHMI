---
id: CODEX-DV
title: Package optional shared-UI desktop designer with responsiveness proof
owner: codex
phase: 5
status: open
created: 2026-09-27
last-update: 2026-09-27 codex [GPT-6]
---

# CODEX-DV — Package optional shared-UI desktop designer with responsiveness proof

## Brief

### Goal

Provide an installable designer only when the shared web interface meets explicit speed and platform gates.

### Context and dependencies

DQ/DR browser authoring; CN browser platform proof; CK release automation; DU tool compatibility.

Authoritative contract: [engine and capacity plan](../../planning/engine-and-capacity.md).
Read root/scoped AGENTS.md and VISION.md before implementation. This task must not
represent planning targets as measured product capability.

### Required behavior

- Reuse browser editor/project contracts with minimal Tauri host adapters for file dialogs and other justified native capabilities.
- Package macOS/Windows/Linux and verify shortcuts, focus, accessibility, DPI, reconnect and clean shutdown; local keychain integration must not weaken sessions.
- Measure cold startup, warm project open, input latency, drag frame times and total webview memory against the accepted client workload; lazy-load editor features and virtualize large trees.
- Native platform UI rewrites remain deferred. A missed performance budget triggers diagnosis and measured optimization, not a claim that webviews are inherently fast.

### Acceptance and validation

- [ ] Supported platform packages install/launch and pass the shared authoring/preview/publish smoke.
- [ ] Recorded client hardware, webview versions, 500-component view and large tag tree meet or explicitly fail the accepted interaction budgets.
- [ ] Browser build remains usable without native APIs; no automatic release publication or signing-key access.

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

## Claude review

## Verdict
