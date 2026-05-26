---
id: CODEX-AS
title: Widget export/import between projects — single-widget JSON round-trip
owner: codex
phase: 4
status: submitted
created: 2026-05-25
last-update: 2026-05-25 codex [gpt-5]
---

# CODEX-AS — Widget export/import between projects

## Brief

> Add designer actions to export a single placed widget (props + bindings + styles) as a JSON file and import it into another project. Widget export/import between projects is a standard SCADA designer UX feature; OpenWebHMI's component library is JSON-serializable already (the view schema is JSON), but there's no integrator-facing surface to lift one widget out and drop it into another project. Small UX win; lands on existing infrastructure.

### Goal

An integrator builds a polished `AlarmTable` widget configuration in Project A (sized, themed, bound to specific tags, custom column order), exports it to `alarm-table-template.owhmi-widget`, opens Project B, imports it, and the widget appears in the current view with all configuration preserved (binding paths may be invalid in Project B; the import surfaces this honestly rather than silently failing).

### Context to read first

- `apps/designer/src/modules/ViewEditor.tsx` — where the export/import actions get added (or right-click menu).
- `crates/protocol/src/` — `View` and widget schemas. Confirm the widget shape is fully JSON-serializable today (it should be — every existing component round-trips through `crates/project-store`).
- `crates/project-store/src/` — how views and widgets are persisted; pick a stable widget-extraction format.
- The component registry pattern in `packages/component-library/src/registry.ts` — used by both designer and runtime to look up widget types by string id. Import validates against this registry.

### Files to create / modify

- **Create** `apps/designer/src/modules/widget-io.ts` — `exportWidget(widget) → blob`, `importWidget(blob) → ImportResult`. `ImportResult` is `{ widget, warnings: string[] }` where warnings include "binding path `rockwell/TankLevel` does not exist in this project's tag namespace".
- **Modify** `apps/designer/src/modules/ViewEditor.tsx` — add "Export widget…" to the widget context menu; add "Import widget…" to the view toolbar.
- **Create** `crates/protocol/src/widget_io.rs` — `ExportedWidget { schema_version: u32, widget_type: String, props: Value, bindings: Vec<Binding>, exported_at: u64, openwebhmi_version: String }`. `#[non_exhaustive]`. `schema_version: u32` starts at `1`.
- **Modify** `packages/protocol-ts/src/` — TS mirror of `ExportedWidget`.
- **Create** `docs/widget-export-import.md` — integrator-facing doc with screenshots / GIFs (or placeholders) showing the export → import flow, expected warnings, format stability promise.
- **Create** `docs/agents/notes/widget-export-format.md` — agent note: schema-version policy, what makes a widget round-trippable, what *intentionally* doesn't round-trip (e.g. project-specific UUIDs).

### Behavior

- **Export**: right-click a widget → "Export widget…" → browser file-save dialog → `.owhmi-widget` JSON file. Filename suggestion is `<widget-type>-<view-name>.owhmi-widget`. The file is human-readable JSON (no compression, no binary blob).
- **Import**: toolbar "Import widget…" → file picker → validates the JSON against the registry (unknown widget type = hard error; show toast and abort), validates bindings against the current project's tag namespace (unknown paths = warning, not error), places the widget at a default position in the current view.
- **Schema-version policy**: `schema_version: 1` is the v1.0 shape. Future format changes increment and ship a migration. The doc commits to "v1.x widget exports import into v1.y where y ≥ x".
- **What round-trips**: widget type, position (x, y, w, h), all props (colors, sizes, labels), all bindings (with their full path strings), instance styles.
- **What does NOT round-trip**: project UUIDs (regenerated on import), parent-view id (the widget lands in the current view, not the original), any tag-namespace state.

### Test requirements

- **Vitest** in `apps/designer/src/__tests__/widget-io.test.ts`:
  - Round-trip: export a complex widget (every prop type), import into a blank project, assert byte-identical props.
  - Binding-warning path: export a widget with binding `rockwell/Foo`, import into a project with no `rockwell` driver, assert exactly one warning naming the path.
  - Unknown widget type: import a file with `widget_type: "NonExistent"`, assert hard error with a helpful message.
  - Schema-version mismatch: import `schema_version: 999`, assert hard error mentioning the version.
- **Vitest** snapshot-test the exported JSON of a representative widget (Trend or AlarmTable). The snapshot pins the export format; future changes update the snapshot and bump `schema_version`.
- **No Rust tests needed** if the shape is pure TS — but if `crates/protocol/src/widget_io.rs` is created (because Python scripts need to drive import/export), add a `crates/protocol/tests/widget_io.rs` round-trip.

### Acceptance criteria

- [ ] Export action present in designer; produces a `.owhmi-widget` file.
- [ ] Import action present in designer; validates and places the widget.
- [ ] Unknown widget type → hard error, no partial state.
- [ ] Unknown binding paths → warnings surfaced in the UI (toast or modal), widget still places.
- [ ] Schema version 1 documented; mismatch is hard error.
- [ ] `docs/widget-export-import.md` exists; `README.md` features list references it.
- [ ] `docs/agents/notes/widget-export-format.md` exists.
- [ ] Vitest tests pass three consecutive runs (file I/O can be flaky in CI).

### Out of scope

- **Multi-widget export (groups, whole views).** v1.x ships single-widget only. Whole-view export is the AF/AI backup path's job. Group export is a v1.2 brief.
- **Widget marketplaces / registries.** Out of scope. The format is just JSON files; an integrator can email them.
- **Backward compatibility older than v1.0.** v1.0 is the floor; nothing pre-1.0 exports cleanly.
- **Auto-binding-rebinding on import.** If the imported widget has `tank-1/Level` and the target project has `tank-2/Level`, the import does **not** auto-rebind. The integrator does it via the property panel after import. Auto-rebinding is a v1.2 feature.
- **Drag-from-file-system to import.** Toolbar action only. Drag-import is a v1.2 polish.

### Risks / gotchas

- **Bindings are the failure mode.** A widget that exports cleanly but won't import usefully (because every binding points at a tag that doesn't exist in the destination project) is the predictable user experience. The warning surface must be honest and informative, not just "imported successfully".
- **Schema version is a one-way contract.** Once v1.0 ships `schema_version: 1`, that shape is frozen — never reshape `schema_version: 1` files post-release. Add a `schema_version: 2` with a migration path if changes are needed.
- **Widget instance ids.** Most widgets have an instance UUID at runtime. Don't export it (regenerate on import) — exporting it would create id collisions when the same template is imported multiple times.
- **The exported `openwebhmi_version` field is informational only.** Don't gate import on version compatibility; the `schema_version` is the contract. Versions help debugging.
- **Filesystem APIs in browser vs Tauri.** The designer is Tauri-shelled; export goes through Tauri's save dialog. The browser-only mode (if any) needs the standard `<a download>` blob trick. Test both code paths if the designer supports both.

## Codex log

2026-05-25 codex: Implemented `.owhmi-widget` export/import helpers, designer export/import actions, TS and Rust `ExportedWidget` schema mirrors, import validation for schema/type/binding paths, docs, and format note. Verified with `pnpm --filter @openwebhmi/designer test`, `cargo test -p openwebhmi-protocol --test widget_io --locked`, `pnpm -r --if-present typecheck`, and build verification.

## Claude review

## Verdict
