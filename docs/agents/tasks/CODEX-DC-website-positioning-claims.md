---
id: CODEX-DC
title: Website positioning and shipped-versus-planned claim reconciliation
owner: codex
phase: 4
status: merged
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

### 2026-09-01  codex [gpt-5]

Started after CODEX-DG/DJ submission. This task updates product positioning and
shipped-versus-planned claims only. MIT remains the effective public license until
CODEX-DH/DI execute the coordinated AGPL/MPL transition; DC will not pre-claim it.
Interactive browser discovery found no available browser, so source, production
build, generated HTML, responsive CSS inspection, and link checks will be recorded;
pixel-level browser smoke remains explicitly deferred unless the browser becomes available.

### 2026-09-01  codex [gpt-5]

Reconciled the homepage, About, Docs, and Download pages. Claim-evidence check:
the 24-hour physical Rockwell run is the sole binding pre-1.0 hardware gate;
five protocols are labeled v1 targets rather than shipped drivers; CPython is
described as a timeout-governed worker surface rather than a complete sandbox or
preinstalled AI/ML environment; Linux Designer artifacts and the Omarchy showcase
remain explicit plans; the dual offline/real-Rockwell reference demo is labeled in
development with no silent fallback; and OEE/manufacturing modules remain optional
post-1.0 direction, not v1.0 or MES claims. The docs index now links all three
manufacturing planning documents. Website typecheck and production build passed
with zero diagnostics. Generated HTML retained current MIT metadata and pre-alpha
warnings, and all added repository targets exist. Internal routes are limited to
the five generated routes and existing static assets. Responsive source inspection
confirmed the new three-column product-path grid collapses to one column below
760px. Interactive desktop/mobile, focus-order, and pixel-level contrast smoke is
deferred because browser discovery returned no available browser.

## Codex review

### 2026-09-01  codex [gpt-5]

**Independent verification**

- Website typecheck and production build passed with zero diagnostics; generated HTML retained current MIT metadata and pre-alpha warnings.
- Full locked Rust workspace build, strict Clippy, tests, rustdoc, format, TypeScript typechecks, and TypeScript tests passed.
- All added document targets exist and every internal route maps to a generated page or existing static asset.
- In-app browser discovery returned no available browser; interactive desktop/mobile smoke was not run.

**What's being fixed**

- Reconcile public positioning with what ships now, what is in development, and what remains post-1.0 direction.

**Root cause confirmation**

- Confirmed: prior copy described five drivers as shipped, overclaimed Python isolation/AI packages and hardware coverage, omitted Linux Designer plans, and did not separate the dual-profile demo from available behavior.

**Fix appropriateness**

- Appropriate: the public pages now use explicit `Available`, `In development`, and `Direction` labels while keeping SCADA/HMI as the product foundation.

**Test proof**

- Generated output contains the updated homepage outcome, demo contract, Omarchy qualification, current MIT statement, unregistered-mark statement, and physical Rockwell gate.

**Residual risk**

- Pixel layout, keyboard focus order, and rendered contrast remain unproven until a browser is available.
- Repository-wide Tauri packaging remains blocked by the unrelated 2.11.5/2.10.1 dependency mismatch.

**Strong points (✅)**

- The same demo screens/logical tags are promised for offline and real Rockwell profiles, with explicit no-silent-fallback language.
- OEE and other manufacturing modules remain post-1.0 direction, not a v1.0 or MES claim.
- The responsive product-path grid collapses from three columns to one below 760px.

**Findings**

- 🟢 Omarchy is clearly a planned first recording environment, not a support guarantee or partnership.
- 🟡 Interactive visual smoke remains a release-content gate.
- 🟠 Real concerns — none blocking this claim-honesty update.
- 🔴 Defects — none.

**Acceptance criteria tally**

- ✅ A visitor can tell what works now, what is being built, and what is only direction.
- ✅ No v1.0 copy presents OEE, MES, recipes, traceability, or other manufacturing modules as shipped scope.
- ✅ Demo copy accurately describes the planned offline/Rockwell dual-profile contract without claiming completion.
- ✅ About and Download claims match `VISION.md` and executed release evidence.
- ✅ The documentation index links all current manufacturing planning documents.
- ✅ Website typecheck/build and internal-link verification pass.

## Claude review

## Verdict

**Merged with explicit validation gate.** Merge commit: `f635f4a` (placeholder; backfilled after commit).
Public claims now distinguish available behavior, in-development demo work, and
post-1.0 direction. Interactive desktop/mobile visual and accessibility smoke
remains required when a browser is available; no pixel-level pass is claimed.
