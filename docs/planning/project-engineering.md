# Project engineering and deployment

Status: accepted direction, 2026-09-29; implementation pending. This plan completes
[open authoring](agent-authoring.md) with application scripting and deployment.
[VISION.md](../../VISION.md) owns scope; the [engine plan](engine-and-capacity.md)
owns dependency and performance budgets. No capability below is certified by this
planning change.

## Product and engine boundaries

OpenWebHMI delivers a complete engineering environment and SCADA/HMI runtime.
Engine separation serves internal modularity, deterministic testing, replaceable
adapters and future UI choices. A small independent host remains required proof
that the gateway is not the only possible composition. Building a third-party
product SDK or ecosystem is not a v1 deliverable.

AGPL-3.0-only remains the engine/product license; compliant reuse, including in
other products, is accepted. The MPL wire-package policy is unchanged. No extra
restriction on reuse or implied relicensing is introduced.

The pure domain layer targets zero third-party dependencies (Rust std/core).
Async orchestration may use narrowly enabled Tokio. Drivers, storage, transport,
Python and UI remain adapters with reviewed dependencies. Moving functionality
across boundaries requires a stated responsibility and dependency-graph evidence;
dependency counts do not justify replacing maintained security/protocol libraries.

## One project in the Designer and terminal

The source directory is the engineering artifact. All authored layouts, bindings,
actions, themes, driver/tag/alarm configuration, script registrations and deployment
requirements have documented text representations. Binary assets remain ordinary
referenced files. Preserve existing formats and migrate explicitly.

Illustrative layout, finalized by DX and DY:

```text
my-machine/
  project.toml
  views/
  tags/
  alarms/
  drivers/
  themes/
  scripts/
  assets/
  tests/
  deployment/
  README.md
  AGENTS.md
  .gitignore
```

A new project is ready for optional git init and a GitHub repository. Scaffold
provider-neutral authoring instructions, local preview/test commands, a simulator
fixture and an optional CI validation example. Do not initialize a remote, push,
require GitHub credentials or run repository hooks when opening a project. Ignore
credentials, private connection settings, history, caches, .venv, generated exports
and runtime state. Source stores named secret references and public target profiles.
Git history is useful for collaboration; it is not publication/audit authority.

An external terminal agent works in this directory while the Designer remains
open, just as an editor and terminal can share a checkout. Valid complete changes
refresh the draft preview; invalid intermediate edits preserve the last valid
preview with file-level diagnostics. Concurrent edits use three-way reconciliation.
Visual saves preserve supported externally authored content and Python formatting.
Offline catalogs, schemas, API stubs, structured validation/test results and logs
make creation, updates and debugging accessible without GUI automation or an LLM
in CI. Agent use is optional; built-in chat and MCP remain deferred.

## Interface authoring and extensions

Use TOML for the manifest, JSON for structured artifacts, Python for application
scripts and normal asset files. No additional XML representation is planned.
Simple visibility, formatting and enabled-state behavior uses declarative bindings;
navigation/dialogs use built-in actions. They do not require Python or a gateway
round trip solely to update local presentation.

Custom interface components use the paired TypeScript Designer/runtime extension
contract with declared properties, bindings, events, version requirements and scoped
styles. Source may include TS/TSX and CSS inside an explicitly built component
package; authored view JSON does not embed arbitrary executable HTML/JavaScript.
Opening or validating a project never installs or executes an unreviewed component
package; visual preview renders only explicitly installed trusted components.
Missing components fail with diagnostics; preview and runtime use the same renderer.
The v1 component SDK must prove custom component creation, manual property editing,
agent-authored bindings, export and runtime rendering in one fixture. This does not
promise Designer round-tripping of arbitrary web applications.

## Optional Python application scripting

Python extends a project with event-driven behavior, data transformations, small
application workflows and PDF/CSV jobs. ML, training infrastructure, model serving
and built-in AI assistance are not v1 goals. Python can be absent when scripting is
disabled; it never becomes a dependency of the pure engine or offline schema checks.

Continue CPython worker subprocesses and JSON-RPC. Scripts are normal .py files;
registrations declare entry points, input/output schemas, capabilities and triggers.
Provide typed API stubs and shared test fixtures to agents and the Designer. Distinguish:

- Short tag/timer/alarm handlers with bounded event queues and explicit overload behavior.
- Explicit UI-invoked jobs with authenticated inputs, progress, cancellation and results.
- Longer report/export jobs with separate worker/queue budgets and bounded artifacts.

Each class declares execution timeout, concurrency, memory/CPU policy and shutdown
behavior. Worker crashes must not crash the gateway. A subprocess is not a security
sandbox: document supported OS enforcement and trusted-code assumptions. Never
claim filesystem/network isolation without implementation and tests. Opening,
validation and passive preview do not execute scripts; explicit test mode uses
simulated tags, temporary artifact storage and test databases, with no plant access.

The gateway authorizes every exposed command. UI jobs carry authenticated caller
identity and cannot exceed both the caller's and script's grants; scheduled jobs use
an explicit service identity. Identity is not trusted from a script argument. Tag
writes use the existing gateway write sink and driver path, not direct TagStore
publication. Database, file and network access must be explicitly configured; do
not advertise arbitrary third-party Python as safely sandboxed.

V1 proof includes a UI-invoked CSV export and a basic PDF report from deterministic
sample data. Select any PDF dependency through the existing license gate. Download
artifacts through authenticated, size-bounded storage with cleanup and expiry;
return artifact identifiers rather than arbitrary host paths. Runtime and Designer
show job failures and structured logs. This is lightweight reporting, not the
post-v1 manufacturing reporting product.

CL retains ownership of existing system.* and timer/alarm trigger completeness.
EB owns UI jobs, execution budgets, diagnostics, report artifacts and explicit script
testing; it must publish a supported-trigger table with CL/CI. View-open/close Python
hooks and general messaging remain deferred. Do not show unsupported completions.

## Python environments and packages

EC supplies optional managed environments per project, keyed by the resolved
runtime/dependency identity. Declare the Python version range, direct requirements,
resolved versions/hashes and platform compatibility. Build an environment explicitly
before activation; opening a project never runs pip, downloads packages or hooks.
No new package-manager dependency is selected by this plan.

Recreate environments for each target OS/architecture. Never commit or ship a local
.venv as a portable runtime. Provide offline bundles with compatible reviewed wheels,
integrity metadata and interpreter prerequisites. Missing/incompatible dependencies
fail preflight before publication. Report required disk space; clean incomplete
preparations; keep the previous revision/environment usable after failure or rollback.
Keep environments for in-flight jobs until their defined drain/cancel policy finishes.
Support explicit selection of a preprovisioned interpreter/environment for air-gapped
installations. No Python environment is installed on scripting-disabled Edge builds.

Database integration starts after v1 (EE): named connections, parameterized queries,
bounded result sizes, timeouts, transaction/cancellation semantics and credentials
outside project source. The first adapter is a project-owned local SQLite database;
never expose internal auth/project/historian tables as an unrestricted script DB.
External database adapters and outbound HTTP connectors need their own reviewed
scope. They are optional project services, outside the engine.

## Preview and deployment

The Designer and CLI expose the same operations and versioned results:

1. Preview locally with simulated or recorded data; explicit script tests use a test context.
2. Publish to an installed gateway using a named target profile and authenticated session.
3. Export/import a validated versioned .owhmi package for offline transfer.

Publishing checks expected revision, target engine/schema/component compatibility,
required drivers/services, Python readiness, secret references and storage capacity.
Show a resource manifest and diff before an explicit authorized publication. Include
referenced assets and declared dependencies; distinguish project resources from
gateway-wide credentials, users, certificates, device connections and history.
Never silently overwrite gateway-wide configuration during project import.

DR is the only atomic publication authority. ED adds target selection, preflight,
package assembly/import and post-publish verification around it, not a parallel
publisher. Interrupted uploads, denied/stale requests, dependency failures and failed
activation preserve the prior revision. Verification reports the activated revision
and readiness without writing to PLCs. Rollback is explicit and authorized, uses a
compatible retained environment, and does not pretend to undo external side effects.

Gateway download/install/upgrade is separate from publishing project content. ED
provides a target OS/architecture setup path with release/package verification,
prerequisite instructions and a connection/readiness check. An uninstalled target
gets actionable setup steps. Automatic remote OS provisioning, privilege escalation,
fleet management and cloud deployment are deferred. A local demo can use a separately
installed gateway; it does not require building an executable per HMI project.

Agents can inspect targets, validate, explain failures and prepare packages through
the same commands and permission model. Automation can publish only when deliberately
granted that capability; source-edit access alone cannot publish or write plant tags.
No AI service or provider account is required for any engineering or deployment step.

## Desktop performance decision

DV packages the shared interface in Tauri with small native host adapters. Measure
macOS, Windows and Linux separately against the engine plan's client budgets,
including startup, project opening, typing, drag/resize, large trees, memory, focus,
shortcuts, accessibility and DPI. Record hardware and webview versions. Diagnose
and optimize failures first. If representative workloads still fail, record a
separate decision comparing continued shared-UI optimization with a native UI on
an affected OS. Native rewrites remain optional and require a new scoped task;
engine contracts, project files and browser authoring must remain compatible.

## Delivery and acceptance

| Task | Scope | Gate |
|---|---|---|
| EA | Reconcile these planning decisions | Consistent vision, plans, design, tasks and board; no implementation claim |
| DM/DN/DO | Pure domain, engine API and optional adapters | Gateway plus independent proof host; dependency graph and minimal build evidence |
| DX/DY/DZ | Git-ready source, terminal tools and live Designer reconciliation | Clean checkout, agent-equivalent edits, conflicts and stable round trips |
| EF | Paired custom component SDK | One component authored, packaged, previewed and rendered through shared contracts |
| CL with CI | Existing scripting API and trigger completeness | Explicit supported/deferred table and truthful docs/completions |
| EB | Bounded script jobs and lightweight PDF/CSV artifacts | Authorized job lifecycle, deterministic tests and paired UI download smoke |
| EC | Reproducible optional Python environments | Offline target preparation, failure recovery and revision rollback |
| ED with DR | Target-aware publish and offline deployment | Target preflight, export/import and authorized atomic activation/rollback |
| DV | Optional desktop, Phase 5 | Three-platform interaction evidence and optional native-UI decision |
| EE | Named local database scripting, Phase 5 | Bounded parameterized access to a project-owned database |

V1 sequence: foundation/security and DX contracts; CL/EB and EC can develop against
shared contracts; DR and EC gate final ED activation. DY/DZ integrate job diagnostics
and the final preview/export/publish fixture. A clean checkout must reproduce a
manual-or-terminal-authored project with a tested report job, preview it, export it,
import it into a compatible offline gateway and publish/rollback through DR.
Record Linux/macOS/Windows evidence separately; deterministic CI fixtures need no
agent service, internet connection or physical PLC. Existing hardware/capacity gates
remain in force. Optional desktop and database adapters remain post-v1.

## References and limits

Ignition is a workflow reference, not a parity or benchmark claim:
[project export/import](https://docs.inductiveautomation.com/docs/8.3/platform/projects/project-export-and-import)
and [deployment practices](https://docs.inductiveautomation.com/docs/8.3/tutorials/ignition-8-deployment-best-practices)
help distinguish project transfers and gateway resources. OpenWebHMI adds its own
shared CLI/Designer contracts and optional terminal-agent workflow.
[Python venv documentation](https://docs.python.org/3/library/venv.html) explains why
environments are recreated rather than copied. The [AGPL text](https://opensource.org/license/AGPL-3.0)
defines reuse terms. No vendor comparison here claims tested feature parity.
