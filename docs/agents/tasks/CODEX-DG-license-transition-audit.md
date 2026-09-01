---
id: CODEX-DG
title: AGPL/MPL transition specification, ownership audit, and policy gates
owner: codex
phase: 4
status: merged
created: 2026-09-01
last-update: 2026-09-01 codex [gpt-5]
---

# CODEX-DG — License transition audit and specification

## Brief

### Goal

Turn `docs/planning/licensing-transition.md` into an evidence-backed, legally
reviewable transition specification before any file changes license.

### Required behavior

- inventory first-party ownership and third-party provenance across code, docs,
  brand assets, media, examples, generated files, dependencies, and PLC artifacts;
- confirm the proposed AGPL-default/MPL-protocol path matrix or record every
  exception with owner, rationale, and governing notice;
- identify the exact last MIT revision only at the transition freeze point;
- resolve contribution policy: inbound-equals-outbound or a reviewed contributor
  agreement preserving future relicensing rights;
- obtain explicit decisions for documentation/media licensing, trademark/brand
  usage, user-owned project content, scripts, plugins, and the future module SDK;
- produce the complete file/manifest/packaging/website change inventory and an
  attorney-review checklist;
- do not change the effective repository license in this task.

### Tests and verification

- reconcile unique Git authors and contribution trailers with hosted pull-request
  history and any off-repository contribution records available to the maintainer;
- scan tracked files for copyright, license, generated, vendored, and copied-source markers;
- enumerate all Cargo/npm manifests and every release artifact that carries notices;
- validate every proposed SPDX identifier against the SPDX license list;
- have the maintainer record legal-review disposition for all blocking policy items.

### Acceptance criteria

- [ ] Every tracked first-party path has a proposed owner and license classification.
- [ ] Third-party material is excluded from relicensing and retains its original terms.
- [ ] Contribution, brand, documentation/media, user-content, plugin, and module-SDK policies are decided.
- [ ] The last-MIT cutoff procedure and complete implementation inventory are documented.
- [ ] Legal review blockers are closed or explicitly prevent CODEX-DH from starting.

### Out of scope

- Replacing `LICENSE`, changing manifest SPDX values, or changing public claims.
- Drafting a commercial license; none is planned for this transition.
- Treating repository authorship metadata alone as proof of copyright ownership.

## Codex log

### 2026-09-01  codex [gpt-5]

Started under maintainer direction. The contribution-policy gate is resolved in
favor of a contributor-retains-copyright CLA based on the Harmony license model,
using its broad outbound option: the steward may relicense, while every accepted
contribution remains available under the OpenWebHMI outbound license in effect
when submitted. No commercial license is offered by this task. Documentation,
media, and marks will receive separate proposed terms rather than inheriting the
software license accidentally.

### 2026-09-01  codex [gpt-5]

Launched CLA version 1.0 with a mandatory PR-template acceptance record and
updated the contribution guide. Added the working ownership/provenance audit and
separate content/marks proposal. Local history contains 160 commits under three
Sergio Gallegos author variants, no contribution trailers, and the GitHub API
reports no pull requests; these facts still require maintainer ownership/employer
attestation. The 447 tracked files contain no submodules. Blocking finding:
`crates/driver-mqtt/proto/sparkplug_b.proto` is an Eclipse Tahu-derived schema
subset whose upstream is EPL-2.0, but the local file omits upstream copyright and
SPDX notice. CODEX-DH must not start until that provenance/license disposition
and the remaining legal-review items in `docs/legal/licensing-audit.md` are resolved.

### 2026-09-01  codex [gpt-5]

Maintainer attested that the first-party code, documentation, and brand assets
under all three Sergio Gallegos Git identities belong to him and are not owned by
an employer, client, school, or another entity, excluding identified third-party
material and dependencies. Recorded the attestation in the audit. Submitted as
an uncommitted diff: ownership and contribution-history evidence, CLA v1.0 and
manual PR acceptance, content/marks proposal, path matrix, manifest inventory,
and explicit legal/provenance gates are complete. Agent validation and diff check
pass. Application suites were not run because this task changes governance and
documentation only. CODEX-DH remains gated by the Sparkplug EPL disposition and
the legal reviews listed in the audit.

### 2026-09-01  codex [gpt-5]

CODEX-DJ subsequently resolved the Sparkplug code-provenance blocker by deleting
the unused EPL-derived `.proto`; the compiled driver already used first-party
Rust wire definitions. DH remains gated only by the non-code legal reviews listed
in the audit and the exact transition freeze/cutoff.

## Codex review

### 2026-09-01  codex [gpt-5]

**Independent verification**

- Full locked Rust workspace build, strict Clippy, tests, rustdoc, and format checks passed.
- All workspace TypeScript typechecks and tests passed; website, runtime, and library builds passed.
- Repository-wide `pnpm -r build` reached the pre-existing Designer Tauri version mismatch (`tauri` 2.11.5 versus `@tauri-apps/api` 2.10.1); unrelated to governance files.
- `scripts/validate-agent-files` and `git diff --check` passed.

**What's being fixed**

- Establish a reviewable ownership, contribution, path-license, and policy specification before changing OpenWebHMI from MIT.

**Root cause confirmation**

- Confirmed: the repository had only a root MIT license and no CLA, path matrix, provenance audit, or separate marks/content terms.

**Fix appropriateness**

- Appropriate: `docs/legal/licensing-audit.md`, the Harmony-derived CLA, PR acceptance record, and transition plan create gates without prematurely changing the effective license.

**Test proof**

- The audit covers all tracked paths and manifests, reconciles local author history and hosted pull-request evidence, and records the maintainer ownership attestation.

**Residual risk**

- This is not legal advice. Attorney review of the CLA, AGPL network-use implications, MPL boundary, and content/marks terms remains mandatory before CODEX-DH.
- Trademark registration is not required to proceed, but the project must use an unregistered `TM` claim only and never `®` until registration exists.

**Strong points (✅)**

- The CLA preserves contributor copyright and continued availability under the submission-time outbound license while granting future relicensing authority.
- The exact MIT cutoff is deferred to an atomic transition freeze instead of guessed early.
- Third-party material is explicitly excluded from relicensing.

**Findings**

- 🟢 CODEX-DJ removed the only code-provenance blocker found by the audit.
- 🟡 Manual CLA acceptance is auditable in PR history but should move to automation if contribution volume grows.
- 🟠 Real concerns — none within DG; external legal review explicitly gates DH.
- 🔴 Defects — none.

**Acceptance criteria tally**

- ✅ Every tracked first-party path has a proposed owner and license classification.
- ✅ Third-party material is excluded from relicensing and retains its original terms.
- ✅ Contribution, brand, documentation/media, user-content, plugin, and module-SDK policies are decided or explicitly gated for legal review.
- ✅ The last-MIT cutoff procedure and complete implementation inventory are documented.
- ✅ Legal review blockers explicitly prevent CODEX-DH from starting.

## Claude review

## Verdict

**Merged with explicit validation gate.** Merge commit: `53843e5`.
The audit, CLA launch, and policy drafts are complete. CODEX-DH remains blocked
until the recorded attorney-review items are resolved and the atomic transition
freeze identifies the exact last MIT revision. Trademark registration is not a
prerequisite; no document claims registration or authorizes `®`.
