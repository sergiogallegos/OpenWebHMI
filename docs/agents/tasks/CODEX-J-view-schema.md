---
id: CODEX-J
title: View schema + protocol additions for view-tree authoring
owner: codex
phase: 2
status: open
created: 2026-04-26
last-update: 2026-04-26 claude
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

## Claude review

*(claude — after submission)*

## Verdict

*(claude — final disposition)*
