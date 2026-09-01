---
id: CODEX-DC
title: Website positioning and shipped-versus-planned claim reconciliation
owner: codex
phase: 4
status: open
created: 2026-09-01
last-update: 2026-09-01 codex [gpt-5]
---

# CODEX-DC — Website positioning and claim reconciliation

## Brief

### Goal

Align the public website with the current OpenWebHMI product direction while
preserving a clear boundary between shipped SCADA/HMI behavior, active work,
and optional post-1.0 manufacturing modules.

### Context to read first

- `VISION.md` and `docs/roadmap.md` for the binding v1.0 scope.
- `docs/feature-matrix.md` and CODEX-CI for implementation-status honesty.
- `docs/planning/manufacturing-platform.md` and
  `docs/planning/manufacturing-demo.md` for the forward direction.
- CODEX-CN, CODEX-CY, CODEX-DA, and CODEX-DB for Linux and demo claims that
  must remain planned until their evidence exists.

### Required behavior

- revise the homepage positioning so SCADA/HMI remains the shipped foundation
  and manufacturing operations are presented as an optional post-1.0 direction;
- visibly distinguish `Available`, `In development`, and `Direction` claims
  wherever adjacent wording could imply that planned behavior already ships;
- add concise demo-direction copy covering default offline simulation and an
  explicit real Rockwell profile, marked planned until CODEX-CY/DA prove it;
- reconcile About-page mission, extension points, protocol/hardware validation,
  AI/ML, driver, component-count, and platform claims with repository evidence;
- add the three manufacturing planning documents to the documentation index;
- reconcile Download-page Linux Designer packaging and current-demo language
  with actual artifacts; do not advertise an installer that cannot be obtained;
- retain the pre-alpha/not-for-production warning and the v1.0 MES boundary;
- use concise end-user language before stack or competitor-comparison detail.

### Tests and verification

- run the website typecheck and production build;
- validate every internal route and repository-document target added or changed;
- maintain a claim-evidence checklist in the Codex log for hardware, platform,
  driver, scripting/AI, demo, and manufacturing statements;
- inspect desktop and mobile layouts, navigation, headings, focus order, and
  contrast using the built site; record any unavailable visual smoke explicitly;
- verify generated HTML retains the pre-alpha warning and correct metadata.

### Acceptance criteria

- [ ] A visitor can tell what works now, what is being built, and what is only direction.
- [ ] No v1.0 copy presents OEE, MES, recipes, traceability, or other manufacturing modules as shipped scope.
- [ ] Demo copy accurately describes the planned offline/Rockwell dual-profile contract without claiming completion.
- [ ] About and Download claims match `VISION.md` and executed release evidence.
- [ ] The documentation index links all current manufacturing planning documents.
- [ ] Website typecheck/build and internal-link verification pass.

### Out of scope

- Creating mock product screenshots or fabricated manufacturing UI.
- Implementing the demo, Linux packages, drivers, AI/ML features, or modules.
- Rebranding OpenWebHMI away from its SCADA/HMI foundation.
- Publishing or deploying the website without explicit maintainer authorization.

### Risks and gotchas

- A future roadmap is useful marketing context but must not read like a feature list.
- Omarchy is a first showcase environment, not an exclusive dependency,
  supported-distribution claim, or official partnership.
- The Rockwell 24-hour real-hardware run is the binding pre-1.0 hardware gate;
  do not silently expand it to every v1 driver.

## Codex log

## Claude review

## Verdict
