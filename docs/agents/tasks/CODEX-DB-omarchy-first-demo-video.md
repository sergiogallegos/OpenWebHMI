---
id: CODEX-DB
title: Omarchy Linux first public demo deployment and video proof
owner: codex
phase: 5
status: open
created: 2026-09-01
last-update: 2026-09-01 codex [gpt-5]
---

# CODEX-DB — Omarchy first demo video

## Brief

### Goal

Produce the reproducible technical setup and capture proof for the first public
OpenWebHMI product video: Gateway, Designer, and browser runtime on Omarchy Linux,
using both a real Rockwell demo PLC and the offline deterministic profile.

### Dependencies

- CODEX-CN Linux Designer/release proof.
- CODEX-CY default-bundled demo, CODEX-CZ content kit, and CODEX-DA Rockwell profile.
- The claimed OpenWebHMI build and Rockwell controller/firmware must already pass
  their respective release/hardware validation gates.

### Required behavior

- documented clean installation and launch on the selected Omarchy release;
- environment manifest: Omarchy version, kernel, desktop/session, browser,
  OpenWebHMI commit/build/artifacts, controller model/firmware, and network layout;
- offline-first launch with deterministic live simulated data;
- Linux Designer open/edit/preview workflow using the same demo;
- explicit authorized selection of the real Rockwell profile;
- real state/count/alarm/trend and guarded command-versus-actual demonstration;
- controlled disconnect showing bad/stale/disconnected quality and no fallback;
- explicit operator action to return to Offline Simulation;
- repeatable capture checklist, narration facts, screenshots, and video chapters.

### Verification

- execute the complete run of show twice from a clean demo reset;
- validate every on-screen claim against recorded build/hardware evidence;
- verify network exposure is limited to the intended local demo environment;
- verify no plant/customer address, credential, tag, asset, or controller project
  enters the capture or repository;
- verify captions and narration describe Omarchy as the first showcase, not the
  only supported Linux distribution or an official partner.

### Acceptance criteria

- [ ] The standard OpenWebHMI artifacts install and run on the recorded Omarchy configuration.
- [ ] Gateway, Designer, and browser runtime are all visibly demonstrated.
- [ ] Offline Simulation works without the PLC and remains the first-run default.
- [ ] Real Rockwell data uses the same project/screens after explicit profile selection.
- [ ] Controlled real disconnection stays visibly failed until explicit mode change.
- [ ] A second operator can reproduce the video from the runbook and manifests.

### Out of scope

- Publishing/uploading the video or posts without explicit maintainer authorization.
- Treating Omarchy-specific packages, shell configuration, or desktop styling as
  a core OpenWebHMI runtime dependency.
- Claiming support for Linux distributions or Rockwell firmware not validated elsewhere.

## Codex log

## Claude review

## Verdict

