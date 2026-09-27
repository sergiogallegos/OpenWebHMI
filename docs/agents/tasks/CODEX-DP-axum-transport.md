---
id: CODEX-DP
title: Replace custom HTTP transport with Axum Hyper adapters
owner: codex
phase: 4
status: open
created: 2026-09-27
last-update: 2026-09-27 codex [GPT-6]
---

# CODEX-DP — Replace custom HTTP transport with Axum Hyper adapters

## Brief

### Goal

Use maintained HTTP infrastructure outside the engine and preserve the established JSON/WebSocket client contract.

### Context and dependencies

DN transport separation; BC/BD/BE/BF/BG and BU behavior contracts; CJ endpoint ownership.

Authoritative contract: [engine and capacity plan](../../planning/engine-and-capacity.md).
Read root/scoped AGENTS.md and VISION.md before implementation. This task must not
represent planning targets as measured product capability.

### Required behavior

- Select exact compatible Axum/Hyper/Tower versions after current license/advisory review; bound Cargo.lock drift.
- Port backup upload/download and WebSocket upgrade to one configured TLS/origin/auth policy with header-first authentication, pre-auth body limits, deadlines and connection/message/subscription caps.
- Retain request correlation, close/keepalive behavior and command results; own no business state in HTTP handlers.
- Coordinate BE security regressions and CJ health/metrics rather than duplicating endpoints; do not redesign application RPC as REST or add deployment services.

### Acceptance and validation

- [ ] Existing Rust/TS wire fixtures remain valid; malformed HTTP, oversized body/message, bad origin, unauthenticated upload and expired session tests fail before fixes.
- [ ] TLS parity, slow clients, cancellation, token expiry cleanup and shutdown tests pass with ephemeral ports.
- [ ] No custom HTTP request parser remains; minimal engine does not depend on HTTP crates.

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
