---
id: CODEX-AP
title: Recipes — process-control recipe management (load/save/apply named tag-value sets)
owner: codex
phase: 4
status: open
created: 2026-05-25
last-update: 2026-05-25 claude [Opus 4.7]
---

# CODEX-AP — Recipes (process-control recipe management)

## Brief

> Add first-class "recipes" to OpenWebHMI: a named set of `(tag_path, value)` tuples that an operator can load to a machine en bloc. This is the canonical SCADA feature for batch production, model changeover, and parameterized machine startup. Recipes are a built-in concept in every mature SCADA platform; OpenWebHMI's v1.0 stack has everything it needs (tag-write routing, audit-log, project-store, components) to land this cleanly. **Post-v1.0 brief — v1.1 target.**

### Goal

An operator opens an HMI view, picks a recipe from a dropdown (e.g. "Product A — 5L batch"), presses "Apply", and the gateway writes all tags in that recipe in a single audited transaction. The integrator authors recipes in the designer (named groups of tag/value pairs). Recipes persist per-project, version with the project, and every load/save/apply fires audit events.

This brief is **flat scope**: ad-hoc recipes (a fixed set of tag/value pairs with a name and description). It is explicitly **not** S88-style batch sequencing or recipe state machines — those are post-1.x.

### Context to read first

- `crates/project-store/src/` — artifact pattern for view/script. Recipe is a peer artifact kind.
- `crates/protocol/src/` — message shapes. New: `recipe.list`, `recipe.get`, `recipe.save`, `recipe.delete`, `recipe.apply` client→server. New: `RecipeApplied { recipe_id, results }` server→client.
- `crates/gateway/src/` — where `GatewayTagWriteSink` dispatches writes. The recipe-apply path routes through this sink so the per-driver write queue invariants from [`docs/agents/notes/python-tag-write-routing.md`](../notes/python-tag-write-routing.md) hold automatically.
- `crates/audit-log/src/event.rs` — add new `AuditEvent` variants for recipe events.
- `packages/component-library/src/components/` — the new `RecipeSelector` component lives alongside the existing 25.
- `apps/designer/src/modules/` — the new `RecipeEditor` module joins peers like `AlarmConfig.tsx`, `ScriptEditor.tsx`.

### Files to create / modify

**Backend:**

- **Create** `crates/recipes/` — new crate:
  - `Recipe { id, name, description, entries: Vec<RecipeEntry> }`
  - `RecipeEntry { tag_path: TagPath, value: TagValue }`
  - `RecipeStore` trait + SQLite implementation (mirror `HistorianStore` shape).
  - `RecipeApplyOutcome { successful: Vec<TagPath>, failed: Vec<(TagPath, String)> }`.
- **Modify** `crates/project-store/src/` — `Recipe` becomes a new `ArtifactKind`.
- **Modify** `crates/protocol/src/` — new message variants under `ClientMessage` and `ServerMessage`. All `#[non_exhaustive]`-friendly (AL convention).
- **Modify** `crates/audit-log/src/event.rs` — add `RecipeSaved`, `RecipeDeleted`, `RecipeApplied { recipe_id, successful_count, failed_count }` to the `AuditEvent` enum.
- **Modify** `crates/gateway/src/` — wire the recipe-apply handler: load recipe, dispatch each entry through `GatewayTagWriteSink`, collect results into a single `RecipeApplied` audit event + a `RecipeApplied` wire response.
- **Modify** `crates/scripting/src/` — Python `system.recipe.apply(name)`, `system.recipe.list()` for script-driven recipe loads.

**Frontend:**

- **Create** `packages/component-library/src/components/RecipeSelector.tsx` — runtime widget: dropdown of recipes + Apply button + status display (last apply outcome).
- **Create** `apps/designer/src/modules/RecipeEditor.tsx` — designer panel: list/create/edit/delete recipes; for each, a tag/value grid where the user picks tags from the tag browser and enters values.
- **Modify** `apps/designer/src/App.tsx` — register the Recipe Editor in the designer.
- **Modify** `packages/protocol-ts/src/` — TS counterparts for the new protocol messages.

**Docs:**

- **Create** `docs/recipes.md` — integrator-facing doc: what recipes are, how to author them, how to apply, audit-log traceability.
- **Create** `docs/agents/notes/recipe-apply-semantics.md` — agent note: how partial failures are handled, why the apply isn't atomic, what the audit-log records.

### Behavior

- **List**: `recipe.list` returns `[{id, name, description, entry_count}]` for the active project.
- **Get**: `recipe.get { id }` returns the full recipe including all entries.
- **Save**: `recipe.save { name, description, entries }` upserts (insert if new, replace if existing name). Returns the recipe id. Fires `RecipeSaved` audit event.
- **Delete**: `recipe.delete { id }` removes. Fires `RecipeDeleted`.
- **Apply**: `recipe.apply { id }` loads the recipe, dispatches each entry through `GatewayTagWriteSink`, returns `RecipeApplyOutcome { successful, failed }`. Fires a single `RecipeApplied { recipe_id, successful_count, failed_count }` audit event.
- **Apply semantics**: writes dispatch sequentially through the per-driver write queue in `entries` order. Failures (PLC reject, network error, unauthorized) are collected, *not* rolled back — process control is not transactional. The outcome surface tells the operator exactly which tags wrote and which didn't.
- **Authorization**: applying a recipe requires the `operator` role minimum; authoring requires `admin`. Reuse the `crates/auth` ACL surface from CODEX-S.

### Test requirements

- **Rust** in `crates/recipes/tests/store.rs`:
  - CRUD round-trip for `Recipe` against an in-memory SQLite.
  - Upsert semantics: saving with the same name replaces, doesn't append.
- **Rust** in `crates/gateway/tests/recipe_apply.rs`:
  - Apply a 5-entry recipe against a mock driver write sink; assert all 5 dispatch in order.
  - Apply a recipe where entry 3 fails (mock driver returns error); assert the outcome reports 2 successful and 3 failed (entries 3/4/5 fail because of … wait, no — entries 4/5 succeed since they go through a different per-driver queue). Adjust expectation to match the actual sequential semantics.
  - Apply fires exactly one `RecipeApplied` audit event with the correct counts.
- **TS** in `packages/component-library/src/__tests__/RecipeSelector.test.tsx`:
  - Dropdown shows recipes from the gateway.
  - Apply button POSTs and renders the outcome.
- **TS** in `apps/designer/src/__tests__/RecipeEditor.test.tsx`:
  - Author a recipe with 3 entries, save, reload, assert it round-trips.
- **Audit-log integration**: assert that the new `RecipeApplied` variant serializes through the hash chain (depends on CODEX-AN; if AN hasn't merged, document the dep in the Codex log and proceed with the non-hashed audit shape — CODEX-AN's migration will pick up the recipe events on its first run).
- Three-consecutive-runs discipline for the apply-flow tests (driver-mock concurrency is a known flake source).

### Acceptance criteria

- [ ] `crates/recipes` crate exists, builds, tests pass.
- [ ] Recipe is a `ProjectStore` artifact kind; recipes persist with the project.
- [ ] Wire protocol covers `list`/`get`/`save`/`delete`/`apply` plus the `RecipeApplied` server event.
- [ ] Gateway `recipe.apply` dispatches through `GatewayTagWriteSink` (driver-prefixed → driver write mpsc; memory tags → `TagStore::publish`).
- [ ] `RecipeSaved`, `RecipeDeleted`, `RecipeApplied` are audit-log events.
- [ ] Python `system.recipe.apply` / `list` work from scripts.
- [ ] `RecipeSelector` component (runtime) and `RecipeEditor` designer module ship.
- [ ] `docs/recipes.md` exists with an integrator-facing walkthrough.
- [ ] `docs/agents/notes/recipe-apply-semantics.md` exists.
- [ ] Designer manual-smoke checklist has a recipe step.

### Out of scope

- **S88 batch sequencing / recipe state machines.** Sequenced startup, hold/resume, batch-record reporting — separate post-1.x brief.
- **Recipe versioning / history.** Save replaces; no version table. v1.2 brief if customers ask.
- **Recipe import/export between projects.** Useful, lightweight follow-up — track as polish. Possibly bundled with CODEX-AS (widget export/import) if the JSON-export pattern generalizes.
- **Recipe parameter validation (units, ranges, types).** Beyond basic tag-type matching, no validation. Operator enters bad value → driver rejects → outcome reports the failure.
- **Bulk recipe execution (apply a sequence of recipes).** Out of scope; that's scheduler territory.
- **Recipe templates / inheritance.** Out of scope.

### Risks / gotchas

- **Partial-apply semantics are the whole story.** Process control isn't transactional — once a setpoint is written to a PLC, you cannot roll it back. The recipe operator must understand this; the `RecipeApplyOutcome` UI must make it impossible to miss which tags wrote and which didn't. **Do not** invent a fake "atomic" mode that pretends to roll back.
- **Driver queueing means non-atomic ordering across drivers.** Entries for the same driver write in `entries` order; entries for *different* drivers run in parallel through their respective queues. Document this in `notes/recipe-apply-semantics.md`. If an operator needs strict cross-driver ordering, it's a scripting task, not a recipe.
- **Audit-log dependency on CODEX-AN.** If CODEX-AN hasn't merged when CODEX-AP lands, the recipe events ship without hash-chain coverage and pick it up at CODEX-AN's migration. Document this in the Codex log; don't block on AN.
- **Designer Recipe Editor needs the tag browser**, which `TagBrowser.tsx` already exposes. Reuse — don't duplicate the tag discovery surface.
- **Recipe name collision across projects.** Recipes are project-scoped, not global. If we ever add cross-project recipe libraries (v1.x), names become ambiguous. Pin "project-scoped" in `docs/recipes.md` so the assumption is durable.
- **Permissions split is non-obvious.** Author = `admin`, apply = `operator`. If a future deployment wants "operator can author too", that's an ACL config change, not a code change. Document this in `docs/recipes.md`.

## Codex log

## Claude review

## Verdict
