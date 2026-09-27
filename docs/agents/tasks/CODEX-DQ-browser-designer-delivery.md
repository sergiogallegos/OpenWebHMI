---
id: CODEX-DQ
title: Deliver offline browser designer and configured gateway routing
owner: codex
phase: 4
status: open
created: 2026-09-27
last-update: 2026-09-27 codex [GPT-6]
---

# CODEX-DQ — Deliver offline browser designer and configured gateway routing

## Brief

### Goal

Make the browser build a deployable authoring product across supported browsers and operating systems.

### Context and dependencies

DP transport; CC editor integrity; CN browser matrix; BZ connection behavior; DK browser build baseline.

Authoritative contract: [engine and capacity plan](../../planning/engine-and-capacity.md).
Read root/scoped AGENTS.md and VISION.md before implementation. This task must not
represent planning targets as measured product capability.

### Required behavior

- Serve designer/runtime static assets under a documented same-origin HTTPS/WSS deployment; remove hardcoded production localhost assumptions and preserve configurable remote gateways.
- Bundle Monaco loader/workers/fonts/assets locally with no startup CDN or telemetry calls; define and test a restrictive usable CSP.
- Separate desktop capabilities behind host adapters; browser import/export works without Tauri. Keep source/license surfaces accessible offline.
- Document setup, authentication/session expiry, reconnect and troubleshooting; add numbered manual-smoke steps.

### Acceptance and validation

- [ ] Browser network-denial test proves no third-party requests; Python editor loads and saves offline against a local gateway.
- [ ] Chrome/Edge, Firefox and Safari perform connect/open/edit/save/preview/reconnect with recorded OS/browser versions; automated evidence where supported.
- [ ] HTTPS/WSS origin and session tests pass; no default credentials or unsecured backup side channel.

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
