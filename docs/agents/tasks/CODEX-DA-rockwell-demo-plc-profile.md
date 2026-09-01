---
id: CODEX-DA
title: Generic Rockwell demo PLC profile, controller package, and hardware proof
owner: codex
phase: 5
status: open
created: 2026-09-01
last-update: 2026-09-01 codex [gpt-5]
---

# CODEX-DA — Rockwell demo PLC profile

## Brief

### Goal

Let the default-bundled manufacturing demo run unchanged against a real
supported Rockwell CompactLogix or ControlLogix controller as well as the
offline deterministic simulator.

### Dependencies

- CODEX-CY demo integration and its protocol-neutral logical tag contract.
- The v1 Rockwell driver hardening and mandatory real-hardware soak gate.
- CODEX-CR command gateway before any demo PLC write is enabled.

### Required behavior

- explicit `Offline Simulation` and `Rockwell Demo PLC` profiles;
- offline is first-run default; switching is role-gated and visibly confirmed;
- generic fictional controller interface and logical-to-Rockwell tag mapping;
- versioned redistributable controller export/package or reproducible import
  instructions after license review;
- configurable controller route/address; no bundled plant IP or credentials;
- supported controller/firmware matrix;
- read and command allowlists, typed writes, audit, PLC permissives, and watchdog;
- bad/stale/disconnected real quality with no automatic simulation fallback;
- equivalent logical behavior across simulator and PLC for the supported demo flow.

### Hardware verification

- download/commission on each claimed controller/firmware pair;
- validate browse, scalar/structured reads used by the demo, guarded writes,
  alarms, count/state/model changes, reconnect, controller restart, and cable pull;
- run the documented endurance test and record update rate, write latency,
  reconnect, resource stability, and known limitations;
- confirm the same HMI views work before and after profile selection.

### Acceptance criteria

- [ ] A clean install runs offline immediately without Rockwell software/hardware.
- [ ] An authorized user can configure and select the real Rockwell profile
      without editing views or manufacturing module definitions.
- [ ] Real connection/tag failures remain visibly failed and never become simulation.
- [ ] Every claimed PLC/firmware combination has recorded hardware evidence.
- [ ] Controller artifacts and instructions are fictional, license-reviewed,
      redistributable, and contain no customer/reference content.
- [ ] PLC safety/interlocks remain authoritative; OpenWebHMI is supervisory only.

### Out of scope

- Shipping Studio 5000, Rockwell firmware, licensed vendor assets, customer PLC
  programs, plant network configuration, safety logic, or unrestricted PLC writes.
- Claiming support for untested controller/firmware combinations.

## Codex log

## Claude review

## Verdict

