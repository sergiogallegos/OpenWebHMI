---
status: active
last-validated: 2026-10-18
---

# External editors and the Designer share a project contract

## Summary

The accepted direction makes terminal agents and visual editing peer authoring
workflows over portable project source. A shared validation/revision service avoids
private database editing or agent-specific business logic. This decision is planned,
not implemented; the [contract](../../docs/planning/agent-authoring.md) defines scope.

## Current understanding

1. [ProjectStore](../../crates/project-store/src/store.rs) loads TOML/JSON artifacts
   and Python source paths; `save_artifact` updates the metadata version and sends
   change events. External filesystem writes bypass that path. Text files alone
   therefore do not provide supported two-way editing or conflict detection.
2. Browser-first authoring needs an explicit bridge to a terminal workspace. The
   [accepted local service](../../docs/planning/agent-authoring.md#local-and-remote-authoring)
   serves the browser and reconciles an authorized directory. Requiring every
   browser to access arbitrary local files or sharing the production filesystem
   would undermine portability and the existing gateway ownership boundary.
3. A watcher observes mutations, not an atomic multi-file project transaction.
   Content-based snapshots, validation and expected revisions must govern accepted
   drafts. [DR](../../docs/agents/tasks/CODEX-DR-draft-publish-integrity.md) remains
   the publication authority; external editing must not become live deployment.
4. A CLI and offline schemas cover the initial agent workflow without model SDKs
   or a required MCP service. This follows the [dependency and privacy policy](../../VISION.md).
   Provider-specific integrations may be adapters later, not a second project model.

## Agent workflow and MCP assessment

The [roadmap](../../docs/roadmap.md) already requires agent-assisted authoring for
v1; the [authoring plan](../../docs/planning/agent-authoring.md#delivery-and-proof)
defines a terminal-to-Designer release fixture. Deferring MCP does not defer the
ability to help an engineer create an HMI. Portable source, component/binding
catalogs, structured diagnostics and preview supply the initial workflow; these
remain planned capabilities.

[MCP exposes resources, tools and prompts](https://modelcontextprotocol.io/specification/draft/server/index).
An optional authoring adapter could expose project/catalog context and shared
validation, draft changes and preview operations to MCP clients. It would reuse
the same application services, revision checks and authorization as the CLI and
Designer. Protocol integration alone does not establish that generated screens
render correctly or shorten engineering time; the existing plan requires separate
workflow evidence. MCP remains deferred under the accepted plan.

The [2026-10-18 plan clarification](../../docs/planning/agent-authoring.md#agent-assisted-hmiscada-engineering)
makes the engineer/agent creation and visual-correction loop explicit. Its
[pump-screen acceptance scenario](../../docs/planning/agent-authoring.md#pump-screen-acceptance-scenario)
requires catalogs, binding-error correction, rendered preview evidence tied to a
candidate revision, concurrent-edit handling and engineer review before authorized
publication. This strengthens the existing v1 authoring requirement without
committing MCP implementation or claiming shipped behavior.

## Evidence

- [Current project storage](../../crates/project-store/src/store.rs): `load`,
  `save_artifact`, `bump_version`, `script_source_path` and filesystem layout.
- [Existing storage notes](project-store-on-disk-format.md).
- [Accepted authoring plan](../../docs/planning/agent-authoring.md) and
  [planning task DW](../../docs/agents/tasks/CODEX-DW-agent-authoring-direction.md).

## Open questions

- Exact CLI commands and JSON diagnostic version: DY resolves in a tested contract.
- Watcher adapter and stable-snapshot implementation: DZ must demonstrate missed
  event recovery and cross-platform behavior before choosing dependencies.
- Measured authoring time saved and agent interoperability: end-to-end fixtures
  plus recorded engineer/agent usability sessions resolve; no market-first claim.
- Whether to promote optional MCP support: the accepted plan calls for evaluation
  against the pump-screen scenario. Document which client capability the adapter
  adds beyond file/CLI access before changing release scope; MCP implementation
  remains deferred.

## Related pages

- [Architecture](../../docs/architecture.md)
- [Engine and capacity decision](engine-capacity-decision.md)
- [Task board](../../docs/agents/board.md)
