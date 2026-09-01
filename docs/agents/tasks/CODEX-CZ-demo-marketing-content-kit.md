---
id: CODEX-CZ
title: Demo marketing capture pipeline and public content kit
owner: codex
phase: 5
status: open
created: 2026-09-01
last-update: 2026-09-01 codex [gpt-5]
---

# CODEX-CZ — Demo marketing content kit

## Brief

### Goal

Use the real default-bundled manufacturing demo as the reproducible source for
website screenshots, documentation, YouTube videos, release announcements,
social posts, and presentation material.

### Dependencies

- CODEX-CY initial demo slice and named deterministic capture scenes.
- Website/documentation asset conventions and applicable branding guidance.

### Required behavior

- capture manifest fixes demo version, seed, logical time, scenario step,
  viewport, locale, theme, role, and Offline Simulation/Rockwell source profile;
- automated or documented capture for approved desktop and responsive views;
- screenshot catalog with alt text, captions, and feature/story mapping;
- YouTube run-of-show outlines that follow the same tutorial project;
- reusable short post/release-note prompts grounded in shipped behavior;
- version/staleness check so changed UI identifies media needing regeneration;
- license/privacy review for every bundled and generated asset.
- first-video run of show for Omarchy Linux: standard install, offline demo,
  Linux Designer, explicit Rockwell profile, real updates, controlled disconnect,
  honest degraded quality, and explicit return to simulation;
- environment manifest recording OS/kernel/session, OpenWebHMI build, browser,
  controller/firmware, and network topology for the hardware video.

### Tests and verification

- regenerate each named screenshot from a clean standard distribution;
- compare required scene metadata and dimensions deterministically;
- verify public content contains no private reference identifier, brand, address,
  tag, alarm text, production value, or proprietary drawing;
- verify every advertised interaction is reproducible in the shipped product.

### Acceptance criteria

- [ ] Website, docs, posts, and video plans use the same shipped demo project.
- [ ] Named captures are reproducible without PLC hardware or internet access.
- [ ] Hardware-specific videos identify the Rockwell profile and validated
      controller accurately; other captures remain reproducible offline.
- [ ] The first public video proves Gateway, Designer, and browser runtime on
      the recorded Omarchy configuration without presenting Omarchy as an
      exclusive dependency or official partnership.
- [ ] The kit includes desktop, responsive, normal, fault, and degraded-quality stories.
- [ ] Alt text/captions and version metadata accompany every approved screenshot.
- [ ] No mock-only feature is marketed as shipped behavior.

### Out of scope

- Publishing posts/videos or uploading to external services without explicit
  maintainer authorization.
- Customer/reference footage, logos, screenshots, or process data.
- Fabricating advanced-module visuals before those modules ship.

## Codex log

## Claude review

## Verdict
