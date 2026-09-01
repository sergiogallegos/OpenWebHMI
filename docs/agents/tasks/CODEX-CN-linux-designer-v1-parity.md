---
id: CODEX-CN
title: Linux Designer v1 parity and three-platform release proof
owner: codex
phase: 4
status: open
created: 2026-09-01
last-update: 2026-09-01 codex [gpt-5]
---

# CODEX-CN — Linux Designer v1 parity

## Brief

### Goal

Close the gap between the `VISION.md` v1.0 contract and actual Designer release
proof: Linux, macOS, and Windows must all build, package, launch, connect to a
gateway, open/edit/save a project, and preview the browser runtime.

### Dependencies

- Coordinate with CODEX-CK (CI/release hardening) and CODEX-CI (docs honesty).
- Do not duplicate their generic CI or prose corrections; own the missing Linux
  Designer functionality and end-to-end release evidence.

### Required behavior

- inventory Tauri/webview/native dependencies per platform;
- add a three-OS CI build/package matrix with supported artifact formats;
- exercise gateway connection, project open/save, preview, and clean shutdown;
- document unsupported optional native integrations without weakening core parity;
- reconcile installer/release instructions with actual artifacts.

### Tests

- automated smoke at the lowest stable UI boundary on all three CI operating systems;
- existing Rust and pnpm validation matrix;
- maintainer-run launch smoke recorded separately where CI cannot drive a GUI.

### Acceptance criteria

- [ ] All three platforms produce supported Designer artifacts.
- [ ] Core authoring workflow has automated evidence on all three platforms.
- [ ] No Windows-only or macOS-only dependency is required by the core Designer.
- [ ] Docs state only what the executed validation proves.

### Out of scope

- Desktop runtime; v1 runtime remains browser-based.
- Platform-specific industrial integrations that can remain optional adapters.

## Codex log

## Claude review

## Verdict

