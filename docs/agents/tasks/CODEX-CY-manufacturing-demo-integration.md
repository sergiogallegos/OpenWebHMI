---
id: CODEX-CY
title: Default-bundled fictional manufacturing demo and product-ready release gate
owner: codex
phase: 5
status: open
created: 2026-09-01
last-update: 2026-09-01 codex [gpt-5]
---

# CODEX-CY — Manufacturing demo integration

## Brief

### Goal

Assemble the planned first-party modules into the fictional five-station demo
specified by `docs/planning/manufacturing-demo.md` and expose it through the
Designer's template workflow.

This is a product-ready release gate: the standard OpenWebHMI distribution must
bundle the initial demo locally by default. It cannot require an internet
download, plugin installation, or checkout of another repository.

The project has two explicit data profiles: default Offline Simulation and an
opt-in Rockwell Demo PLC profile. Both drive the same logical tags, screens,
alarms, facts, and reports. Real-mode failure must remain failed; automatic
simulation fallback is prohibited.

### Dependencies

- Initial slice: CODEX-CP and CODEX-CR plus stable v1 alarms/trends/security.
- Manufacturing expansion: CODEX-CQ, CS, CT, CU, CV, CW, and CX.
- Deliver in slices; do not block the useful initial demo on OEE.

### Required behavior

- generic Load/Assemble/Inspect/Test/Unload simulation and assets;
- standard release packaging plus first-run `Open Demo` and new-project entry;
- explicit Offline Simulation / Rockwell Demo PLC selector and persistent banner;
- protocol-neutral logical tag contract shared by both profiles;
- stable named capture scenes for documentation, website, post, and video use;
- overview, simulated command, alarms, trends, diagnostics, roles;
- later line layout, production, state, faults, reports, downtime, and OEE;
- deterministic scenario reset and explicit simulation banner/origin;
- clone/remove-module workflow, tutorial metadata, screenshot-safe fictional data.

### Tests

- create/import/startup, deterministic scenarios, module enable/disable, guarded
  writes, quality failure, report reproduction, backup/restore, and cross-platform
  Designer/Gateway coverage.

### Acceptance criteria

- [ ] A new user reaches a functioning offline HMI from `Create from template`.
- [ ] The normal release artifact includes the demo by default and first run
      offers it without network access or extra installation.
- [ ] No private reference identifier, asset, value, alarm, tag, or brand appears.
- [ ] Failed real data never switches to simulated values.
- [ ] A project created and learned offline can select the configured Rockwell
      profile without changing its views or manufacturing-module configuration.
- [ ] The demo is both a documentation sample and automated integration fixture.
- [ ] Named scenes fix seed, logical time, viewport, locale, theme, and role so
      public screenshots and videos can be regenerated from the shipped runtime.

### Out of scope

- SPC, energy, equipment health, traceability, Andon, or MES until their own
  domain plans graduate from exploratory/deferred status.
- Customer PLC programs, plant addresses, credentials, proprietary tag maps, or
  support for every Rockwell controller/firmware combination.

## Codex log

## Claude review

## Verdict
