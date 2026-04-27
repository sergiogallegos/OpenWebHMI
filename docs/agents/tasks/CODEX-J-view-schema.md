---
id: CODEX-J
title: View schema + protocol additions for view-tree authoring
owner: codex
phase: 2
status: merged
created: 2026-04-26
last-update: 2026-04-27 claude
---

# CODEX-J — View schema + view protocol additions

## Brief

### Goal

Define the on-disk + over-the-wire format for **views** (HMI screens). Once this lands, designer (CODEX-M) and runtime (CODEX-L) can compose against a stable schema. This task only delivers the schema and protocol additions — no rendering, no editor.

### Context to read first

- `docs/architecture.md` §4.10 (Designer modules — what authors a view), §4.11 (HMI Runtime — what renders a view), §4.12 (Component Library — what's bound to a view's components).
- `crates/protocol/src/lib.rs` — current wire-protocol types.
- `packages/protocol-ts/src/index.ts` — TS counterpart.
- `crates/project-store/src/types.rs` (delivered by CODEX-I) — where view types live; this task extends them.

### Files to create / modify

- `crates/project-store/src/types.rs` — add `View`, `Component`, `ComponentProps`, `Binding` types (extending the file from CODEX-I).
- `crates/protocol/src/lib.rs` — add `view.open`, `view.close`, `view.definition` messages.
- `packages/protocol-ts/src/index.ts` — mirror the additions; type guards for the new variants.
- `packages/protocol-ts/src/index.test.ts` — round-trip tests for view-related wire forms.
- `docs/architecture.md` — add §4.10.1 "View schema v1" subsection documenting the format.

### View schema v1

A view is a tree of components. Components have `id`, `kind` (which component from the library), `props` (component-specific JSON), and optional `bindings` (which props read from which tag paths).

```rust
pub struct View {
    pub id: String,                   // unique within project, slug-like ("home", "line-1")
    pub title: String,                // human-readable
    pub root: Component,
    pub schema_version: u32,          // 1 in this task
}

pub struct Component {
    pub id: String,                   // unique within view; auto-generated if not provided
    pub kind: String,                 // "Label", "ValueDisplay", "Container", etc.
    pub props: serde_json::Value,     // component-specific JSON (validated against component's schema in the designer)
    pub bindings: Vec<Binding>,       // which props read from tags
    pub children: Vec<Component>,     // for container-style components
}

pub struct Binding {
    pub prop: String,                 // prop name on this component (e.g. "value", "label", "color")
    pub source: BindingSource,
}

#[serde(tag = "kind", rename_all = "snake_case")]
pub enum BindingSource {
    Tag { path: String },             // most common: read from a tag
    Expression { code: String },      // Phase 3 placeholder; Phase 2 ignores this variant in the runtime
    Constant { value: serde_json::Value },  // unbound default
}
```

**Why a tree, not a flat list:** containers nest. A `Container` component renders its children at relative positions. Phase 2's form-based editor manipulates this tree directly.

**Why JSON for `props`:** every component declares its own props schema (Phase 2's component library does this). The designer validates props against the schema before save; the wire format stays loose.

### Wire protocol additions

Extend `ClientMessage`:

```rust
#[serde(rename = "view.open")] ViewOpen { project_id: String, view_id: String },
#[serde(rename = "view.close")] ViewClose { project_id: String, view_id: String },
```

Extend `ServerMessage`:

```rust
#[serde(rename = "view.definition")] ViewDefinition {
    project_id: String,
    view_id: String,
    version: u64,                     // matches the project version that produced this view
    view: View,
},
```

When a `view.open` message arrives, the gateway responds with the current `view.definition`. When the project version changes (via `project.changed` from CODEX-I), the gateway pushes a fresh `view.definition` to all clients with that view open.

Note: this is *additive*. Existing `tag.subscribe` / `tag.update` flow is unchanged. The runtime opens a view, then subscribes to the tag paths the view's bindings reference.

### Test requirements

Rust:
- Round-trip serde for `View`, `Component`, `Binding`, `BindingSource`.
- A view tree with 3-level nesting (Container > Container > ValueDisplay) round-trips.
- `ClientMessage::ViewOpen` and `ServerMessage::ViewDefinition` round-trip with literal wire-form assertions (mirror Rust serde output exactly, like the existing tests).

TS (vitest in `packages/protocol-ts`):
- Mirror the literal wire-form assertions for the new variants.
- Type guards reject malformed view definitions (missing `kind`, missing `id`).

### Acceptance criteria

- [ ] `cargo test -p openwebhmi-protocol` and `cargo test -p openwebhmi-project-store` green with the new types.
- [ ] `pnpm --filter @openwebhmi/protocol test` green with the mirrored cases.
- [ ] `cargo test --workspace --all-features --locked` stays green.
- [ ] `docs/architecture.md` §4.10.1 added; one-line wiki/log entry.
- [ ] No view-rendering code in this task. Don't get pulled into CODEX-L's territory.

### Out of scope

- Component-specific prop schemas (CODEX-K owns those).
- View-rendering logic (CODEX-L).
- Drag/drop editing (Phase 2 stretch / Phase 4).
- Themes (Phase 2 stretch / Phase 4).
- Expression-binding evaluation (Phase 3 scripting).

### Risks / gotchas

- **Don't over-design the schema.** v1 should support exactly the 6 components CODEX-K is building. Adding speculative fields before there's a use case for them creates migration debt.
- **`props` is intentionally `Value`.** Tempting to type per-component-kind, but that ties the schema crate to the component library. Keep them decoupled — designer validates at the application layer.
- **Component `id` uniqueness scope** is per-view (not per-project). A `Label` with `id: "title"` in `home.json` doesn't collide with `id: "title"` in `line1.json`. Document this.
- **Binding ordering**: `bindings: Vec<Binding>` is a list, not a map. If two bindings target the same prop, the *last* one wins. Document this.

## Codex log

*(codex — append working notes here)*

### 2026-04-26 20:09  codex
Started alongside CODEX-I. Holding the schema/protocol edits locally because the files overlap with project-store protocol work; keeping CODEX-K separate in a worker because its package scope is disjoint.

### 2026-04-27 09:32  codex
Submitted. Added view schema v1 types to `crates/project-store`: `View`, `Component`, `Binding`, and `BindingSource` with tree-shaped components, loose JSON props, ordered bindings, and tag/expression/constant sources. Extended Rust protocol with `view.open`, `view.close`, and `view.definition`; extended TS protocol types and guards with matching view and project artifact shapes.

Documented view schema v1 in `docs/architecture.md` §4.10.1 and appended the wiki log entry. Verification: `cargo test -p openwebhmi-protocol`, `cargo test -p openwebhmi-project-store`, `pnpm --filter @openwebhmi/protocol test`, `pnpm --filter @openwebhmi/protocol build`, and full workspace cargo checks pass.

## Claude review

### 2026-04-27  claude — review pass 1

Spec-compliant. View / Component / Binding / BindingSource match the brief exactly: tree (not flat), `props: serde_json::Value` (loose, decoupled from component-library), per-view component-id uniqueness, binding-list-with-last-write-wins. Wire-protocol additions (`view.open`, `view.close`, `view.definition`, `project.subscribe`/`unsubscribe`/`load`/`save_artifact`/`changed`/`save_result`/`snapshot`) all round-trip through the test suite with literal wire-form assertions on the load-bearing variants.

Strong points:
- ✅ A 3-level nested view (Container > Container > ValueDisplay) round-trips in `view_tree_round_trips`.
- ✅ Literal wire-form assertion on `project.save_artifact` and `view.open` — matches the discipline established by Phase 0's protocol tests.
- ✅ `ArtifactKind` is adjacently tagged (`{ kind: "view", id: "home" }`) so future variants don't break existing parsers.
- ✅ The project-store types live in one place and the protocol crate re-exports the wire-relevant subset — no duplication.

Findings:
- 🟡 **`crates/protocol` now depends on `crates/project-store`** (`Cargo.toml:15`). Direction is non-obvious — typically a wire-protocol crate is the lower-level dep, not the consumer. The pragmatic justification: project-store owns the canonical Rust types; protocol re-exports the subset that lives on the wire. The cost: every consumer of `openwebhmi-protocol` now pulls SQLite/rusqlite/anyhow as transitive deps, even if they never touch the project store. For Phase 2 only the gateway consumes this surface, so it's a non-issue in practice. **Track for cleanup**: either flatten the wire-relevant types into protocol with project-store re-exporting from protocol, or split a `crates/wire-types` for the truly shared shapes.
- 🟢 Test for project `save_artifact` wire form catches the `request_id` skip-when-None semantics implicitly (it's `Some` in the test). A second test asserting `request_id` omission when `None` would be cheap insurance.

Acceptance criteria met. No view-rendering code introduced (correctly stayed in CODEX-J's lane).

## Verdict

**Merged** at the next commit. The protocol → project-store dep direction is the only architectural note; not blocking, tracked here for cleanup when CODEX-L/M land.
