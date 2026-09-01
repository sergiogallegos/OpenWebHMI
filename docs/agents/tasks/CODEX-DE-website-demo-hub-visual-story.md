---
id: CODEX-DE
title: Website demo hub and shipped-product visual story
owner: codex
phase: 5
status: open
created: 2026-09-01
last-update: 2026-09-01 codex [gpt-5]
---

# CODEX-DE — Website demo hub and visual story

## Brief

### Goal

Make the default-bundled manufacturing demo the website's canonical product
story using reproducible media and instructions generated from shipped behavior.

### Dependencies

- CODEX-DC website positioning and claim reconciliation.
- CODEX-CY initial default-bundled demo slice and deterministic scenes.
- CODEX-CZ approved marketing capture catalog.
- CODEX-CN for any Linux Designer download/install claim.
- CODEX-DA and CODEX-DB before publishing real-Rockwell or Omarchy proof.

### Required behavior

- add a first-class `/demo/` page and primary navigation path;
- replace the text-only homepage hero with a real, versioned product capture and
  a concise operator/integrator value proposition;
- present Offline Simulation as the no-PLC, no-internet default evaluation path;
- present Real Rockwell as an explicit opt-in profile using the same logical
  tags, screens, alarms, and trends, with no automatic fallback on disconnect;
- provide reproducible start/reset steps tied to released artifacts and the
  canonical demo version rather than mutable development commands;
- show a compact visual sequence covering line overview, Designer editing,
  browser runtime, alarms/trends, and degraded/disconnected quality;
- use only CODEX-CZ-approved images/video with alt text, captions, dimensions,
  provenance/version metadata, and responsive variants;
- add `Watch the demo` only when a public video URL exists and `Try the demo`
  only when the referenced artifacts and instructions are usable;
- identify Omarchy as the first recorded Linux showcase without narrowing the
  general Linux support contract.

### Tests and verification

- run website typecheck and production build;
- verify every demo instruction from a clean supported environment;
- exercise responsive layouts and keyboard navigation for the hero, media,
  navigation, mode explanation, and calls to action;
- verify images reserve dimensions, use appropriate formats/sizes, and do not
  create a material performance regression;
- confirm offline page content and local demo instructions do not require a PLC,
  internet connection, account, analytics, or third-party runtime call;
- run the CODEX-CZ privacy/license/staleness checks for every included asset.

### Acceptance criteria

- [ ] The homepage shows the real shipped product rather than a mock-only illustration.
- [ ] `/demo/` clearly explains and distinguishes Offline Simulation and Real Rockwell modes.
- [ ] A new evaluator can launch/reset the offline demo from released instructions without PLC hardware.
- [ ] Rockwell and Omarchy claims appear only after their recorded evidence exists.
- [ ] All media is reproducible, accessible, responsive, licensed, and version-linked to the demo.
- [ ] No CTA points to a missing artifact, unpublished video, or hypothetical hosted service.

### Out of scope

- Building demo behavior owned by CODEX-CY or the controller profile owned by CODEX-DA.
- Fabricating screenshots for modules that have not shipped.
- Uploading videos, publishing posts, or deploying the website without explicit authorization.
- Adding a hosted demo gateway or a telemetry/analytics dependency.

### Risks and gotchas

- The website must degrade cleanly when video is unavailable; the offline demo
  and static product story cannot depend on YouTube or another third party.
- A controlled Rockwell disconnect must remain visibly failed until an explicit
  profile change; website copy must not imply seamless automatic simulation fallback.

## Codex log

## Claude review

## Verdict
