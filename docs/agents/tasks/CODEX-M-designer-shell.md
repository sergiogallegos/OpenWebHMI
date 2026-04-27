---
id: CODEX-M
title: apps/designer — Tauri shell, project explorer, form-based view editor
owner: codex
phase: 2
status: open
created: 2026-04-26
last-update: 2026-04-26 claude
blocked-by: CODEX-I, CODEX-J, CODEX-K, CODEX-L
---

# CODEX-M — `apps/designer` (Tauri shell + form-based editor)

## Brief

> **Blocked by [CODEX-I](CODEX-I-project-store.md), [CODEX-J](CODEX-J-view-schema.md), [CODEX-K](CODEX-K-component-library-v1.md), [CODEX-L](CODEX-L-runtime-view-loading.md).** This is the Phase 2 exit-criterion deliverable. Don't start until I/J/K/L are merged. May scaffold the Tauri shell in parallel with K/L.

### Goal

The Phase 2 Designer MVP. Authors a 5-screen HMI without hand-editing JSON. Per the de-risked Phase 2 split: **Required = form-based editor, no canvas**. Canvas + drag/drop are Phase 2 stretch / Phase 4. Phase 2 exit criterion is met when an author can build the Phase 1 demo's runtime page (Pressure value + driver-status indicator + a couple of additional rows) entirely through this designer, save it through the gateway, and watch the runtime hot-reload it.

### Context to read first

- `docs/architecture.md` §4.10 (Designer / IDE).
- `docs/roadmap.md` Phase 2 — the Required vs Stretch split.
- `crates/project-store/src/types.rs` (CODEX-I) — the project model.
- `packages/component-library/src/registry.ts` (CODEX-K) — the components you spawn.
- `apps/runtime-web/src/ViewRenderer.tsx` (CODEX-L) — same renderer the designer uses for live preview.

### Files to create

```
apps/designer/
├── package.json                       @openwebhmi/designer
├── tsconfig.json
├── vite.config.ts
├── index.html
├── src-tauri/
│   ├── Cargo.toml                     openwebhmi-designer (binary, Tauri v2)
│   ├── tauri.conf.json
│   ├── build.rs
│   └── src/main.rs                    minimal Tauri main; no business logic
└── src/
    ├── main.tsx
    ├── App.tsx                        layout + routing between modules
    ├── modules/
    │   ├── ConnectGateway.tsx         pick a gateway URL, log in (Phase 3 will add auth)
    │   ├── ProjectExplorer.tsx        tree of views, drivers, tags
    │   ├── TagBrowser.tsx             list driver-published tags + memory tags
    │   ├── ViewEditor.tsx             form-based view-tree editor (no canvas)
    │   ├── PropertyPanel.tsx          edit selected component's props + bindings
    │   └── PreviewPane.tsx            embeds ViewRenderer pointing at the gateway's preview project
    ├── lib/
    │   ├── designerClient.ts          extends gatewayClient with project.save_artifact, view.open, etc.
    │   └── viewMutations.ts           pure functions to mutate a View tree (add/remove/move components)
    └── __tests__/                      vitest tests for viewMutations + module shells
```

Add `apps/designer` to `pnpm-workspace.yaml`. Add `apps/designer/src-tauri` to the workspace `Cargo.toml` `members`. Update `.gitignore` to cover `apps/designer/src-tauri/target/` and `apps/designer/src-tauri/gen/`.

### Behavior — required modules

#### ConnectGateway
- Pick a gateway URL (default `ws://localhost:8080`).
- (Phase 3 will add login; v1 is anonymous.)
- On successful WS handshake, advance to the rest of the UI.

#### ProjectExplorer
- Tree showing: `Project > Views > [view ids]`, `Project > Drivers > [driver ids]`, `Project > Tags`.
- Click a view → opens it in `ViewEditor` and `PreviewPane`.
- Right-click → "Add View" / "Rename" / "Delete". Save through `project.save_artifact`.

#### TagBrowser
- Shows all known tag paths the gateway is publishing (driver-backed + memory).
- Drag a tag onto a component prop in the property panel → creates a binding. (Phase 2 still uses a "Bind" button; full drag-drop is Phase 2 stretch.)

#### ViewEditor (the heart of Phase 2)
**Form-based, not canvas-based.** UI is a vertical column of nested editable cards:
- Each component renders as a card with: kind label, id, "Edit" / "Delete" / "Move ↑↓" / "Add Child (if Container)" buttons.
- Click "Edit" → property panel populates with that component's props + bindings.
- Click "Add Child" on a Container → modal picks from `componentRegistry`; new component spawns with defaults.
- Component positioning is via numeric x/y/width/height fields in the property panel (no drag/drop yet).
- All mutations go through `viewMutations.ts` pure functions, then `project.save_artifact { kind: View(id), body }`.

#### PropertyPanel
- For the selected component:
  - Editable fields generated from the component's `propsSchema`.
  - For each `bindableProps` entry: a "Bind" button that pops the tag picker, plus a "Constant" / "Tag" / "Expression" mode switch.
  - Save on every change (debounced to 300ms) → `project.save_artifact`.

#### PreviewPane
- An iframe (or embedded webview tab) pointing at the runtime app, loaded with `?project=<current-project-id>&view=<current-view-id>`.
- Reloads on `project.changed` events the designer receives. (Phase 2 hot reload validation.)

### Behavior — explicitly NOT in this task

- ❌ Visual canvas (react-konva). Phase 2 stretch.
- ❌ Drag/drop component placement.
- ❌ Snap-to-grid, undo/redo, multi-select.
- ❌ Theme editor UI.
- ❌ Tauri auto-updater.
- ❌ Cross-platform installer packaging (Phase 4).

### Test requirements

Vitest:
- `viewMutations.test.ts`: every pure mutation function (add/remove/move/duplicate component, set prop, set binding) round-trips and preserves the rest of the tree.
- `ProjectExplorer.test.tsx`: renders a synthetic project, click handlers fire as expected.
- `PropertyPanel.test.tsx`: editing a prop value calls the save callback (with debounce mocked off).

Manual smoke (documented as a checklist in `apps/designer/README.md`):
- Connect to gateway running with the Phase 1 demo project.
- Open the auto-created `home` view (or create one if it doesn't exist).
- Add a `Container` with two children: a `Label` ("Pressure") and a `ValueDisplay` bound to `rockwell-1/Pressure`.
- Save. Open `apps/runtime-web` in a browser at the same project — the value renders live.
- Edit the `Label` text; observe runtime hot-reloads within 1s.

### Acceptance criteria

- [ ] `pnpm --filter @openwebhmi/designer build` produces a Tauri-bundled binary on macOS and Windows (CI matrix expansion may slip to Phase 4).
- [ ] `pnpm --filter @openwebhmi/designer typecheck` and `test` exit 0.
- [ ] `cargo build -p openwebhmi-designer` (the Tauri shell) exits 0 on macOS.
- [ ] The manual smoke above succeeds end-to-end.
- [ ] All public TS modules carry TSDoc.

### Out of scope

- Linux build of the designer (post-1.0).
- Auth (Phase 3).
- Script editor / Monaco integration (Phase 3).
- Alarm config UI (Phase 3).

### Risks / gotchas

- **Tauri v2 install prerequisites** vary across platforms. Document them clearly in `apps/designer/README.md`. If a contributor can't build the designer, the friction kills the contribution loop.
- **The form-based editor is genuinely usable.** Don't let "no canvas" convince you to ship a worse experience. Add hover affordances on the component cards, keyboard navigation, and a visible "save status" indicator (saved / saving / unsaved).
- **`project.save_artifact` save thrash**: every keystroke in PropertyPanel must NOT trigger a save. Debounce to 300ms; combine multiple field edits within the window into one save.
- **Hot reload conflict**: if the designer is editing a view while the project changes from another client, the gateway's last-write-wins semantics will silently lose one author's edits. Phase 2 doesn't solve this (multi-developer locking is post-1.0); the designer should at minimum warn when it detects a `project.changed` for the view being edited from a version the user didn't author.
- **Don't reinvent the runtime renderer.** The PreviewPane points at `apps/runtime-web` via iframe. Don't fork the renderer.

## Codex log

*(codex — append working notes here)*

## Claude review

*(claude — after submission)*

## Verdict

*(claude — final disposition)*
