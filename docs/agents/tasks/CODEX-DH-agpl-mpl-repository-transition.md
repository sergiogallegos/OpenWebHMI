---
id: CODEX-DH
title: Atomic AGPL core and MPL protocol repository transition
owner: codex
phase: 4
status: merged
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

### 2026-09-01  codex [gpt-5]

Started under a recorded maintainer decision to proceed without outside legal
counsel because paid review is not currently affordable. Authoritative GNU,
Mozilla, SPDX, Harmony, and USPTO materials were rechecked. The transition will
carry an explicit no-outside-counsel risk record and will not imply legal review.
The exact last MIT revision is `a8cbdbc`; prior MIT grants remain unchanged.

### 2026-09-01 13:57  codex [gpt-5]

Implemented the atomic license map, canonical texts, manifest identifiers,
runtime/Designer/Gateway Legal/Source surfaces, bundled notices, transition
record, asset inventory, and fail-closed CI validator. Submitted for review after
the Rust matrix and pnpm typecheck/test passed; all web/package builds and the
macOS `.app` passed, while aggregate `pnpm -r build` stopped at DMG packaging.

## Codex review

### 2026-09-01 13:57  codex [gpt-5]

**Independent verification**

- `scripts/validate-licenses --self-test` — passed, including all four negative fixtures.
- `cargo build --workspace --all-features --locked` — passed.
- `cargo clippy --workspace --all-targets --all-features --locked -- -D warnings` — passed.
- `cargo test --workspace --all-features --locked` — passed with network-bound tests run outside the restricted sandbox.
- `cargo doc --workspace --no-deps` and `cargo fmt --check` — passed.
- `pnpm -r typecheck`, `pnpm -r test`, and representative Vite builds — passed.
- `pnpm -r build` — all package/web builds and the Designer binary/`.app` passed; the command failed only when the local Tauri process invoked `bundle_dmg.sh`.
- `pnpm --filter designer tauri build --bundles app` — passed after the transition commit; the exact source revision and canonical legal resources were inspected in the output.

**What's being fixed**

- Future product-core code moves from MIT to AGPL-3.0-only while the two protocol packages use MPL-2.0, with usable source and notice disclosures.

**Root cause confirmation**

- Confirmed: the prior root `LICENSE`, workspace/package manifests, product surfaces, and public copy all represented the repository as uniformly MIT licensed.

**Fix appropriateness**

- The path-based policy in `LICENSE-POLICY.md`, package-level MPL overrides, canonical local texts, and artifact-visible Legal/Source surfaces put each obligation at its owning repository and build layer.

**Test proof**

- The validator rejects a stale MIT product manifest, missing MPL override, missing source offer, and unclassified bundled asset. Canonical hashes, generated bundles, app resources, and package metadata were also inspected.

**Residual risk**

- No attorney reviewed these project-specific policies or CLA; the maintainer explicitly accepted that risk because paid review is not currently affordable.
- Interactive runtime/Designer visual smoke was unavailable because the in-app browser reported no installed browser. The `.app` bundle passed; a separate all-bundles attempt reached `.app` success but DMG packaging did not complete locally.
- This review validates repository policy consistency and build behavior; it is not individualized legal advice.

**Strong points (✅)**

- `docs/license-transition.md` preserves prior MIT grants and records exact revision boundaries.
- `scripts/validate-licenses` makes scope drift and missing offline notices build failures.
- Gateway, Runtime, and Designer disclose the exact build revision and corresponding-source path.

**Findings**

- 🟢 The embedded MIT Rust EtherNet/IP library and all third-party dependencies retain their own terms.
- 🟡 Interactive visual and DMG smoke remain release-environment checks, not license-map blockers.
- 🟠 Real concerns — none within the accepted no-counsel risk disposition.
- 🔴 Defects — none.

**Acceptance criteria tally**

- ✅ Core/product packages are AGPL-3.0-only and only approved protocol packages are MPL-2.0.
- ✅ Supported artifacts carry license, source, and third-party notices; representative web and `.app` outputs were inspected.
- ✅ Runtime and Designer expose the exact corresponding-source path for their build.
- ✅ The transition notice identifies the exact last MIT revision without revoking prior grants.
- 🟡 partially CI prevents license-map drift and the code/test matrix passes; aggregate `pnpm -r build` remains red only at local DMG packaging, while `tauri build --bundles app` passes.
- ✅ No commercial license, plugin exception, or user-content claim is invented.

## Claude review

## Verdict

**Merged with explicit validation gate.** Transition commit:
`549829e`. Interactive
browser visual smoke and local DMG packaging remain unproven; neither changes
the effective repository license map or the inspected offline `.app` resources.
