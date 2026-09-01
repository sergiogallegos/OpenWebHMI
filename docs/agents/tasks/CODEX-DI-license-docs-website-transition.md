---
id: CODEX-DI
title: AGPL/MPL public documentation, website, and transition communication
owner: codex
phase: 4
status: open
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

## Claude review

## Verdict
