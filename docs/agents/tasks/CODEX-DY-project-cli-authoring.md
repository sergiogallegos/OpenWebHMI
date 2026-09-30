---
id: CODEX-DY
title: Add headless project CLI and local browser preview service
owner: codex
phase: 4
status: open
created: 2026-09-27
last-update: 2026-09-29 codex [GPT-6]
---

# CODEX-DY — Add headless project CLI and local browser preview service

## Brief

### Goal

Enable engineers and terminal agents to generate, inspect, validate, preview and submit the same projects used by the Designer.

### Context and dependencies

DX project contract; DN shared services where needed; DQ preview assets. Offline commands can land first; DR gates all remote publication.

Authoritative contract: [agent authoring plan](../../planning/agent-authoring.md).
Read root/scoped AGENTS.md and VISION.md before implementation.

### Required behavior

- Build a small Rust CLI adapter with init, inspect/catalog, validate, diff, explicit migrate/export and preview operations. Final command naming is documented with examples.
- Provide versioned JSON results and exit codes; core offline commands require no GUI, running gateway, Node, Python, model SDK or network. Separate explicit Python checks from project validation.
- Scaffold example source and concise provider-neutral project instructions, schemas and Python API documentation; a specific agent product is not required.
- Serve the browser editor for an explicitly selected local workspace using loopback defaults, authenticated sessions, origin/host checks and bounded/path-safe access. Use synthetic preview data; never run scripts or contact PLCs implicitly.
- Pull/submit revisions via the existing authenticated project protocol; publish requires explicit action, capability and expected revision under DR. Do not implement a parallel transport or private database writer.

### Acceptance and validation

- [ ] Deterministic headless fixture creates a complete project with layout, bindings, alarms and a Python source file; invalid inputs produce stable JSON errors.
- [ ] Offline validation/scaffolding opens no network connections and executes no project code, hooks or installs. Missing Python is reported only for commands requiring it.
- [ ] Local service access/path/origin tests and paired browser smoke pass; stale or denied publication leaves runtime unchanged.

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

Scaffold README, provider-neutral AGENTS.md, .gitignore, sample test/simulation data and optional CI validation instructions. Do not create/push remotes or run hooks implicitly. Expose shared EB script-test/log results and ED target/export/preflight operations with structured CLI output; core offline commands still need no Python/Git/agent.

## Claude review

## Verdict
