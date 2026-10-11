---
id: CODEX-EJ
title: Build local architecture HTML handbook excluded from website publishing
owner: codex
phase: 4
status: open
created: 2026-10-11
last-update: 2026-10-11 codex [GPT-6]
---

# CODEX-EJ — Build local architecture HTML handbook excluded from website publishing

## Brief

### Goal

Deliver an offline HTML handbook under `docs/private/architecture.html` covering actual design, architecture, code guidelines and diagrams. Interpret private here as local-only/non-published; public repository content is not confidential.

### Destination and publication contract

Generated `docs/private/` output must be ignored by Git, absent from apps/website public/src/dist, deployment bundles and uploaded CI artifacts, and never linked from public website routes. Add an explicit exclusion check so future docs imports cannot publish it accidentally. Track the generator and non-sensitive canonical sources outside the ignored output directory. Confidential source content, credentials and plant topology must remain outside the public repository.

### Context and evidence

No architecture HTML or scripts/build_architecture_docs.py currently exists. docs/architecture.md, accepted planning docs, root/scoped AGENTS.md and code are sources. CI owns docs-versus-code reconciliation; EJ owns presentation/generation and exclusion. Link rather than fork policies; distinguish implemented code, accepted target architecture and pending tasks throughout.

### Required content and implementation

- Provide table of contents, navigation, readable code/source references, diagram legends, responsive layout and print styling. Open from file:// without a server, CDN, telemetry or network requests.
- Cover system context, subsystem/container topology, deployment boundaries and actual driver reachability; Rust domain/service/adapter modules, TypeScript apps/libraries, optional Tauri shell and Python workers.
- Include UML component/package and representative type/trait/class diagrams matching the language, sequence diagrams for login/read/subscribe/write/ack and project edit/publish, activity/step workflows, alarm/subscription lifecycle state diagrams, persistence/data flows and shutdown/reconnect/error paths.
- Include Rust/TypeScript code guidelines, toolchain/format/test/dependency policy and platform constraints. Integrate EI after its guideline decision; label unimplemented policy and architecture explicitly.
- Generate from existing canonical docs plus verified code references, with source revision/date and an update command. Bundle reviewed local diagram assets or pre-render diagrams to SVG; avoid a mandatory extra production dependency or new language.
- Record diagram source text and source links. Do not invent class relationships or treat capacity targets as measured capability. Provide local links usable from the generated output.

### Acceptance and validation

- [ ] Offline HTML contains all requested architecture, guidelines and diagram categories with source provenance and implemented/planned labels.
- [ ] Browser visual review at desktop/mobile widths and print verifies readable diagrams, navigation and no broken local references; record environments checked.
- [ ] Deterministic regeneration plus structural/link checks pass. Check website build/deployment selection and Git ignore rules to prove generated/private assets are absent; demonstrate the exclusion check rejects a deliberately included fixture.
- [ ] No private output is committed, uploaded or published. Instructions explain that any tracked sources remain public in a public repository.

### Out of scope

Website publication, hosted authentication, rewriting canonical architecture, or claims that a hidden route/noindex protects confidential content.

## Codex log

## Claude review

## Verdict
