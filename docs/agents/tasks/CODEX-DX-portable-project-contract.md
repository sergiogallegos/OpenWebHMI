---
id: CODEX-DX
title: Define portable project source and offline schema contract
owner: codex
phase: 4
status: open
created: 2026-09-27
last-update: 2026-09-29 codex [GPT-6]
---

# CODEX-DX — Define portable project source and offline schema contract

## Brief

### Goal

Make one portable, versioned text project editable by both the Designer and external tools without a private database.

### Context and dependencies

DM schema separation; BC/BR path and persistence safety. Coordinate revision identities with DR before changing persisted formats.

Authoritative contract: [agent authoring plan](../../planning/agent-authoring.md).
Read root/scoped AGENTS.md and VISION.md before implementation.

### Required behavior

- Specify existing TOML/JSON/Python/assets source layout, stable IDs, relative paths, secret references and explicit migrations.
- Make source discovery/index reconstruction work from a clean checkout; preserve published/audit authority separately.
- Provide offline machine-readable schemas, manifest/component/binding catalogs and shared semantic validation with structured diagnostics.
- Preserve supported fields and Python source through unrelated Designer edits; reject unsupported data explicitly. Define custom component manifests through the paired Designer/runtime extension contract, not arbitrary executable view properties.

### Acceptance and validation

- [ ] Existing fixtures migrate without content loss; checkout-only open works without copying SQLite metadata.
- [ ] Repeated structured serialization is stable; layout/script/config round trips preserve IDs and unrelated content.
- [ ] Invalid references, unknown versions/fields, missing components, path escapes and cross-platform case collisions have actionable errors.

Run appropriate repository checks. Behavior regressions must fail against pre-fix
code; use deterministic synchronization and ephemeral ports. Paired Designer/runtime
contract changes land together. Record unperformed browser, OS and hardware checks.

### Out of scope

No mandatory AI provider, built-in chat or MCP server, automatic publication,
implicit script execution/PLC writes, dynamic executable plugin loader, private
SQLite editing API, license change or broad dependency updates. Do not claim
planned features are implemented. Keep optional authoring tools out of Edge runtime.

## Codex log

### 2026-09-29 22:37 codex [GPT-6] — project engineering scope coordination

The [project engineering plan](../../planning/project-engineering.md) records
maintainer-directed scope clarification. Original brief and status are preserved.

Specify Git-ready source for all supported layouts, actions, tags, drivers, alarms, themes, scripts and deployment requirements. Keep binary assets ordinary files; exclude secrets/history/caches/venvs/build output. TOML/JSON/Python remain base formats; custom TS/TSX/CSS packages use EF, not arbitrary executable view properties or XML. Coordinate script/dependency metadata with EB/EC and target manifests with ED.

## Claude review

## Verdict
