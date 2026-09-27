# Open projects and agent-assisted authoring

Status: accepted product direction, 2026-09-27; implementation pending. The Designer
is the visual engineering environment for an OpenWebHMI application. Engineers,
terminal tools and coding agents must be able to author the same project without
automating clicks or editing private gateway databases. This extends the
[engine and capacity plan](engine-and-capacity.md), not the runtime's control scope.

Product positioning: **Build visually. Bring your favorite coding agent.** The
engineering benefit to demonstrate is a shorter path from project source to a
reviewed working HMI: generating layouts/bindings/configuration/scripts, diagnosing
errors and iterating in the Designer. Manual authoring remains fully supported;
using an external agent is optional and subject to that tool's own requirements.
Public copy must label this workflow planned until DX/DY/DZ/DR pass. Do not claim
market-first status, universal agent compatibility or quantified time savings
without comparative research, interoperability evidence or measured user workflows.

## Shared project contract

Project source is a documented, versioned directory: `project.toml`, JSON view/tag/
alarm/theme and script metadata, ordinary `.py` sources, and referenced assets.
Retain existing formats where compatible; publish explicit migrations for changes.
Stable artifact/component identifiers, deterministic formatting and relative paths
make projects reviewable in Git. Git is optional and is not the runtime revision
authority. Keep published revisions, audit journals, caches and credentials outside
editable project source; source carries secret references, not resolved secrets.

The directory is authoritative for authoring content. A rebuildable index may
accelerate discovery, but opening a source checkout must not require a copied SQLite
file. Published snapshots and their audit history have their own durable authority;
rebuilding a source index must not invent publication history. Designer, CLI and
gateway use one versioned validation/command contract, with wire conformance checks.
Do not duplicate business rules in a provider-specific agent integration.

Publish machine-readable JSON schemas, a manifest specification, component/property
catalog, binding/expression grammar and Python API documentation/stubs. All required
schemas and documentation resolve offline. Validate references as well as shape:
tags, bindings, view IDs, assets, script registrations, triggers and component types.
Diagnostics include stable codes, severity, file and JSON pointer or line/column
where available. Unsupported schema versions or fields fail explicitly; only
documented extension fields may round-trip. Never silently discard authored data.

Designer saves must preserve supported content outside the selected edit, stable
IDs and Python source. Deterministic formatting applies to structured artifacts;
do not reformat unrelated files or rewrite Python on an unrelated visual edit.
Agents can create layouts, bindings, themes, configuration and scripts using the
same capabilities as engineers. Arbitrary React/TypeScript in a view document is
not a new execution channel. Custom component packages use the shared, versioned
Designer/runtime extension contract and explicit installation/build review; missing
components produce actionable diagnostics rather than silently changing a screen.

## CLI and Designer workflow

Provide a small Rust CLI as an adapter over shared project/application services.
Scaffolding and validation work without a running gateway, account, GUI, Node,
Python interpreter or agent subscription. Script syntax/execution checks that need
Python are separate, explicit commands and report interpreter requirements. Do not
execute project scripts, hooks, package installation or network access while opening
or validating a project. No bundled model SDK or mandatory external AI service.

The following is an illustrative command contract, **not installed commands today**;
final names/options are owned by CODEX-DY:

```text
owhmi project init ./line-a
# Engineer or external agent edits the files using any editor.
owhmi project validate ./line-a --json
owhmi project diff ./line-a --base <revision> --json
owhmi project preview ./line-a
owhmi project publish ./line-a --gateway <name> --expected-revision <revision>
```

Also support inspection/catalog export, explicit migration and deterministic export
of a validated revision. CLI output has a versioned JSON mode, documented exit codes,
noninteractive behavior, bounded resource use and actionable conflicts. Preview
reuses the web Designer/runtime and synthetic tag sources. Export/schema/validation
must not pull the web server into the domain library or start listeners implicitly.
The core generation/edit/validation loop does not require MCP. A future optional
MCP adapter can call the same services if a concrete workflow justifies it; no
provider-specific integration is a v1 dependency.

## Local and remote authoring

- **Local checkout:** a CLI development service serves the browser Designer and
  watches one explicitly selected workspace. Browser filesystem access is not a
  prerequisite. The service accepts only the selected root, authenticates sessions,
  enforces origin/host checks and binds to loopback by default. It is a separate
  authoring process, not a requirement on a low-resource production gateway.
- **Remote gateway:** the engineer/agent works in a local checkout, pulls or exports
  a revision, and submits a validated changeset through the authenticated gateway
  project protocol. The remote Designer observes accepted draft events. No shared
  network filesystem or SSH access to gateway internals is required.
- **Optional desktop:** a host adapter supplies workspace access but shares the
  project contract, reconciliation rules and renderer with browser authoring.

Raw filesystem edits do not currently go through ProjectStore's save/version/event
path. Watcher notifications therefore cannot be treated as committed revisions.
The development service must reconcile a complete validated snapshot, using content
identity and an expected base revision. Coalesce noisy/duplicate notifications and
detect missed events by rescan; metadata timestamps alone do not establish identity.
Multi-file changes become one accepted draft transaction only after validation and
a stable snapshot check. Watch mode may refresh a valid draft automatically;
explicit reconcile/validate remains available for deterministic agent workflows.

Dirty Designer buffers and external edits need three-way comparison against a known
base. Preserve both variants and show a conflict when they overlap; never use silent
last-writer-wins. Invalid/partial writes or Git merge markers remain visible as
diagnostics while the last valid preview continues. Undo must not revert another
writer's changes without revision checking. Batch renames, deletions and binding
updates use the same transaction/reconciliation path. Reject traversal, symlink
escapes and case-colliding project paths consistently across supported platforms.

## Draft, preview and production

Opening/editing files affects source/drafts only. Preview uses mock or recorded data
and does not connect to plant devices, run scripts or write tags implicitly. Explicit
script tests run in an isolated test context; subprocesses alone are not a security
sandbox. Live commissioning remains a separate authenticated and authorized mode.

Publishing is an explicit operation under CODEX-DR: validate the complete candidate,
check the expected revision, authorize the caller, record an audit event and switch
to an immutable published snapshot. Record authenticated principal, revision and
result; a client-supplied agent name is descriptive metadata, not proof of identity.
Editing credentials/roles, writing live tags and publishing are separate capabilities.
An engineer may deliberately grant deployment automation publish access; external
editing itself never grants it. Failed validation/conflicts leave production on its
prior revision. Rollback is an authorized publication, not an edit of audit history.

## Delivery and proof

| Task | Responsibility | Required evidence |
|---|---|---|
| DW | Direction and architecture reconciliation | Consistent plan, vision, architecture and board |
| DX | Portable project contract and offline schemas | Existing-project migration; stable round trips; checkout without private database |
| DY | CLI, scaffolding and local preview service | Headless generation/validation; structured errors; authenticated revision-checked remote submission |
| DZ | Designer external-edit reconciliation | Concurrent editor/agent conflicts, multi-file changes, undo, reconnect and browser workflow |
| DR (existing) | Draft/publish authority | CLI and Designer share atomic publication, authorization and rollback |

DX coordinates with DM schema extraction. DY's offline commands can land before
DR; publish is unavailable until DR's gates pass. DZ shares DQ's browser delivery
and DR's revision protocol. Do not make security fixes wait for authoring features.
MCP, a built-in AI chat interface and dynamic executable plugin loading are deferred.

The release fixture creates a project from terminal tools without launching the
Designer, adds tags/alarms, a custom layout/theme and a Python script, validates it,
and opens the same source in the browser Designer. An external change updates its
preview; a concurrent dirty edit conflicts without data loss. An invalid source
fails with file-level diagnostics. Runtime stays on the prior published revision
until explicit authorized publish; denied/stale publish and rollback are tested.
Use deterministic fixtures in CI without an LLM/API dependency. Record manual
agent-assisted usability and Linux/macOS/Windows browser runs separately from tests.
Before advertising time savings, compare a fixed authoring scenario with and without
agent assistance, including review/correction time and identical correctness criteria.
Record the agent/tool versions and limits rather than implying universal compatibility.

## Current evidence and limits

[ProjectStore](../../crates/project-store/src/store.rs) already loads TOML/JSON and
Python source paths, maintains version metadata in `_index.sqlite`, and broadcasts
changes through `save_artifact`. This is a useful starting point, not a supported
external-edit/CLI synchronization contract. Schemas, offline scaffolding, safe
reconciliation and draft publication above remain implementation tasks.
