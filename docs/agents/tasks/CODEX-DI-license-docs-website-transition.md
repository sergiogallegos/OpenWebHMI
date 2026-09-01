---
id: CODEX-DI
title: AGPL/MPL public documentation, website, and transition communication
owner: codex
phase: 4
status: merged
created: 2026-09-01
last-update: 2026-09-01 codex [gpt-5]
---

# CODEX-DI — License documentation and website transition

## Brief

### Goal

Make every public repository and website statement explain the approved
AGPL-core/MPL-protocol transition accurately, consistently, and in plain language.

### Dependencies

- CODEX-DG final policy and wording boundaries.
- CODEX-DH implementation must be ready to land in the same coordinated release window.
- Coordinate with CODEX-DC/DE/DF so website positioning, demo media, and social
  metadata do not reintroduce stale MIT claims.

### Required behavior

- update VISION, README, contributor guidance, architecture, roadmap, feature
  matrix, stack rationale, design-system planning, release/install docs, and all
  other current-license claims found by repository-wide search;
- update website homepage, About, Download, Docs, footer, README, metadata, badges,
  and link targets to describe the license split and source location accurately;
- publish a transition FAQ identifying the last MIT and first AGPL/MPL revisions;
- state that OpenWebHMI remains open source, free for commercial plant use,
  self-hostable, air-gap capable, modifiable, and free of activation/per-server fees;
- state that no alternative commercial license is currently offered;
- explain AGPL core and MPL protocol scope without presenting summaries as legal advice;
- link to exact license texts and the repository path map;
- remove stale claims that brand assets, all repository files, or all packages are MIT;
- use only CODEX-DG-approved language for user projects, scripts, plugins, brand,
  third-party assets, and network-source obligations.

### Tests and verification

- run website typecheck/build and repository documentation/link validation;
- search source and built output for stale MIT badges, copy, metadata, and footer text;
- verify every license link resolves to the correct file and protocol exceptions
  remain visible from package pages and generated artifacts;
- review desktop/mobile website layouts and accessible names for license/source links;
- have a second reviewer reconcile every FAQ statement against the canonical
  license text and CODEX-DG-approved legal wording.

### Acceptance criteria

- [ ] Repository docs and the built website agree with the implemented license map.
- [ ] Visitors can understand AGPL core, MPL protocols, prior MIT revisions, and the absence of a commercial license.
- [ ] Commercial use is not incorrectly prohibited and prior MIT rights are not described as revoked.
- [ ] No stale MIT badge, footer, package claim, metadata field, or brand statement remains.
- [ ] All public legal summaries link to canonical terms and carry appropriate disclaimers.
- [ ] Website and documentation validation pass.

### Out of scope

- Giving individualized legal advice or interpreting a customer's deployment.
- Drafting a commercial license or pricing page.
- Publishing the website before CODEX-DH is ready for the coordinated transition.

## Codex log

### 2026-09-01  codex [gpt-5]

Started in the coordinated CODEX-DH transition window. Public summaries will
link canonical terms, retain commercial-use and prior-MIT-rights accuracy, state
that no commercial license is offered, and identify OpenWebHMI as an unregistered
mark without using the registration symbol.

### 2026-09-01 13:57  codex [gpt-5]

Updated repository and website copy to the implemented AGPL/MPL policy, added
canonical-policy and transition links, retained accurate commercial-use and
prior-MIT language, and removed stale uniform-MIT and brand-license claims.

## Codex review

### 2026-09-01 13:57  codex [gpt-5]

**Independent verification**

- `pnpm --filter website typecheck` and `pnpm --filter website build` — passed with zero diagnostics.
- `scripts/validate-licenses --self-test` — passed, including stale-copy checks.
- `scripts/validate-agent-files` and `git diff --check` — passed.
- Repository and built-output searches — no stale first-party MIT badge, footer, metadata, or package claim found.

**What's being fixed**

- Public documentation must explain the new split license accurately without claiming that commercial use is forbidden or earlier MIT rights were revoked.

**Root cause confirmation**

- Confirmed: README, website pages/footer, package metadata, brand guidance, and planning documents contained uniform-MIT language that would become false at transition.

**Fix appropriateness**

- Canonical terms live at repository root and public pages link back to those terms and the path map, while short website copy stays readable and avoids personalized interpretations.

**Test proof**

- Astro typecheck/build and targeted source/generated-output searches verify the published routes, links, license split, prior-revision wording, and absence of stale first-party MIT claims.

**Residual risk**

- Interactive desktop/mobile visual and accessible-name smoke was unavailable because the in-app browser reported no installed browser.
- No second human or attorney reviewed the legal summaries; this Codex review reconciled them against canonical GNU, Mozilla, SPDX, Harmony, and USPTO materials under the maintainer's accepted risk decision.

**Strong points (✅)**

- The website clearly distinguishes permitted commercial use from the absence of an alternative commercial license.
- Prior MIT grants, user-owned content, third-party terms, and unregistered marks are handled separately rather than collapsed into the software license.

**Findings**

- 🟢 `OpenWebHMI™` is used without the registration symbol and the policy does not claim registration.
- 🟡 Interactive responsive-layout smoke remains a release-environment check.
- 🟠 Real concerns — none within the accepted no-counsel risk disposition.
- 🔴 Defects — none.

**Acceptance criteria tally**

- ✅ Repository docs and the built website agree with the implemented license map.
- ✅ Visitors can understand AGPL core, MPL protocols, prior MIT revisions, and the absence of a commercial license.
- ✅ Commercial use is not incorrectly prohibited and prior MIT rights are not described as revoked.
- ✅ No stale first-party MIT badge, footer, package claim, metadata field, or brand statement remains.
- ✅ Public summaries link canonical terms and state that they are general information, not legal advice.
- ✅ Website and documentation validation pass.

## Claude review

## Verdict

**Merged with explicit visual-smoke gate.** Coordinated transition commit:
`a8cbdbc` (temporary parent ref; backfilled immediately after commit creation). Interactive
desktop/mobile browser smoke remains unproven because no browser is available in
this environment; source, typecheck, build, and generated-output checks passed.
