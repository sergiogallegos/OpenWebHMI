---
id: CODEX-DH
title: Atomic AGPL core and MPL protocol repository transition
owner: codex
phase: 4
status: open
created: 2026-09-01
last-update: 2026-09-01 codex [gpt-5]
---

# CODEX-DH — AGPL/MPL repository transition

## Brief

### Goal

After CODEX-DG clears every ownership and policy gate, transition future
OpenWebHMI product code atomically to AGPL-3.0-only with MPL-2.0 protocol
packages, accurate source offers, artifact notices, and automated enforcement.

### Dependencies

- CODEX-DG merged with recorded legal-review disposition and final scope matrix.
- Coordinate with CODEX-CK release/license CI work and avoid duplicating or
  weakening its permissive inbound dependency gate.

### Required behavior

- install unmodified canonical license texts and the reviewed repository license map;
- set Cargo/npm SPDX expressions per the approved path matrix;
- add reviewed annotations/notices for files not unambiguously covered by package boundaries;
- preserve third-party license texts and the separate MIT Rust EtherNet/IP dependency;
- record the exact last-MIT and first-AGPL/MPL revisions without claiming revocation;
- add Legal/Source surfaces to Gateway-served runtime and Designer, including
  build version/commit and corresponding-source instructions suitable for offline installs;
- generate third-party and first-party license notices in supported artifacts;
- update contribution mechanics to the CODEX-DG-approved policy;
- enforce license mapping, stale claims, manifest values, notices, and source-offer
  presence in CI and repository validation;
- keep dependency acceptance limited to reviewed compatible inbound licenses.

### Tests and verification

- run the full Rust and pnpm validation matrix;
- build representative Gateway, Designer, runtime, and protocol packages and inspect notices;
- verify every tracked path resolves to the intended license or documented third-party exception;
- verify the Legal/Source surface from browser runtime and Designer release builds;
- prove CI fails on a stale MIT product manifest, missing MPL override, missing
  source offer, and unclassified bundled asset;
- run the artifact from an offline environment and confirm license/source information remains available.

### Acceptance criteria

- [ ] Core/product packages are AGPL-3.0-only and only approved protocol packages are MPL-2.0.
- [ ] Every release artifact carries accurate license, source, and third-party notices.
- [ ] Runtime and Designer expose the exact corresponding-source path for their build.
- [ ] The transition notice identifies the last MIT revision accurately.
- [ ] CI prevents license-map drift and the complete validation matrix passes.
- [ ] No commercial license, plugin exception, or user-content claim is invented.

### Out of scope

- Commercial licensing, pricing, activation, telemetry, or per-server fees.
- Relicensing the independently published MIT Rust EtherNet/IP library.
- Altering third-party dependency terms.
- Starting until CODEX-DG's legal/policy gates are closed.

## Codex log

## Claude review

## Verdict
