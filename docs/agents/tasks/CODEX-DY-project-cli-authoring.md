---
id: CODEX-DY
title: Add headless project CLI and local browser preview service
owner: codex
phase: 4
status: open
created: 2026-09-27
last-update: 2026-10-24 codex [GPT-6]
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

### 2026-10-18 23:13 codex [GPT-6] — agent-assisted interface workflow clarification

The maintainer approved the explicit [agent-assisted HMI/SCADA workflow](../../planning/agent-authoring.md#agent-assisted-hmiscada-engineering)
and [pump-screen acceptance scenario](../../planning/agent-authoring.md#pump-screen-acceptance-scenario).
Original brief and open status are preserved; implementation has not started.

DY supplies catalog inspection, offline diagnostic/correction results and a documented
way to inspect or capture the rendered browser preview with trusted browser tooling.
Preview diagnostics/evidence identify the candidate revision and synthetic fixture.
The pump screen includes status, start/stop controls, speed setpoint, high-pressure
alarm and pressure trend. Validation must expose an intentionally invalid binding
and support its correction. No LLM is required in CI. MCP remains a deferred optional
adapter over these same services and permissions, not a DY implementation dependency.

### 2026-10-24 07:16 codex [GPT-6] — team repository coordination

The [team workflow](../../planning/project-engineering.md#team-repositories-and-agent-friendly-collaboration)
clarifies scaffolding for an independent HMI application repository, provider-neutral
README/AGENTS.md, ignore rules, offline schemas/catalogs, simulation and optional CI.
Document explicit developer Git initialization/remote/push actions and structured
agent validation/preview results. Prove clean-checkout reproduction with DX/DZ;
no implicit hooks, network access, production credentials or LLM are required.
Original brief and open status are preserved; implementation remains pending.

## Claude review

## Verdict
