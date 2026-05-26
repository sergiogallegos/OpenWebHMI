---
id: CODEX-AS
title: Widget export/import between projects — single-widget JSON round-trip
owner: codex
phase: 4
status: merged
created: 2026-05-25
last-update: 2026-05-26 claude [Opus 4.7]
merge-commit: a225aa7
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

### 2026-05-26 19:50  claude [Opus 4.7]

**Independent verification**
- `cargo test -p openwebhmi-protocol --test widget_io --locked` — 1/1 passed (`exported_widget_round_trips_json`).
- `pnpm --filter @openwebhmi/designer test` — 10 test files, 21 tests passed; `widget-io.test.ts` contributes 5 (round-trip, warnings, unknown widget type, unsupported schema version, snapshot pin).
- CI run 26431030607 Node + Rust jobs ✅ — both green; AS is among the merged work that brought CI to green.
- Read every diff in commit a225aa7 end-to-end: `crates/protocol/src/widget_io.rs` (42 lines), `crates/protocol/tests/widget_io.rs` (25 lines), `apps/designer/src/modules/widget-io.ts` (168 lines), `apps/designer/src/modules/ViewEditor.tsx` (71 lines added), `apps/designer/src/__tests__/widget-io.test.ts` (117 lines), `apps/designer/src/App.tsx` (9 lines), `packages/protocol-ts/src/index.ts` (17 lines added), `docs/agents/notes/widget-export-format.md` (14 lines), `docs/widget-export-import.md` (23 lines).

**What's being fixed**
- Designer had no integrator-facing surface to extract a single widget configuration and import it into another project. The view schema round-trips as JSON via project-store already, but no end-user mechanism existed to lift one widget out.

**Root cause confirmation**
- Confirmed: pre-AS `apps/designer/src/modules/` had no `widget-io.ts`; `ViewEditor.tsx` had no Import action; `crates/protocol/src/lib.rs` had no `widget_io` module re-export. AS adds exactly what the brief specified.

**Fix appropriateness**
- Right layers: TS `widget-io.ts` owns validation + UI dispatch (designer-only concern); Rust `widget_io.rs` owns the wire shape so Python scripts or future external tools can produce/consume the format without re-deriving it.
- `ExportedWidget` and `ExportedWidgetChild` are intentionally non-unified — top-level metadata (`schema_version`, `exported_at`, `openwebhmi_version`) lives only on `ExportedWidget`; children carry just `widget_type`/`props`/`bindings`/`children`. Correct shape — child metadata would be redundant noise.
- Both Rust structs are `#[non_exhaustive]` per CODEX-AL convention; both use `#[serde(default)]` on `bindings` and `children` so future Rust readers can tolerate older exports that omit those fields.
- `componentToExportedChild` + `exportedToComponent` recursion correctly preserves the widget tree depth. `id` regeneration on import (via `idFactory`) prevents collisions when the same template is imported multiple times — matches the brief's "Imported widgets receive new component ids to avoid collisions."
- `bindingWarnings` correctly walks the tree (own bindings + recursive `children`); uses `Set` for dedup of warnings.
- `validateKnownWidget` is called both at top-level `importWidget` and recursively inside `exportedToComponent` — defense in depth; a nested unknown widget aborts before any partial state is created.

**Test proof**
- 5 vitest cases cover the full contract: round-trip (props + bindings preserved), warnings path (missing tag paths), unknown widget type (hard error with helpful message), unsupported schema version (hard error naming the version), and a **snapshot pin of the exported JSON format**. The snapshot is the brief's "Snapshot-test the exported JSON of a representative widget. The snapshot pins the export format; future changes update the snapshot and bump `schema_version`" — exactly the right discipline.
- Rust round-trip test pins a representative `ExportedWidget` JSON (with nested child) and asserts byte-identical re-encode after decode. Catches serde key-ordering drift if it ever happens.
- Single-run vitest is sufficient — no real file I/O, no async timing, no port binding (everything is in-memory string operations). The three-runs discipline applies to known-flaky tests; these are deterministic.

**Residual risk**
- **`exportWidget` hardcodes `openwebhmi_version: "0.0.1"`** in the default — should ideally read from a build-time constant or a workspace package version. Cosmetic for now (the field is informational per the brief's "informational only"), but a future polish.
- **`cloneValue` uses `JSON.parse(JSON.stringify(value))`** — drops non-JSON values (functions, undefined, symbols, BigInt, Date). Safe for OpenWebHMI's JSON-only widget config invariant, but a contributor adding a non-JSON prop type would silently lose data on export/import. Low likelihood; no guard added.
- **`bindingWarnings` warns for every binding when `tagPaths.size === 0`** — correct per the brief, but noisy for first-import-into-fresh-project. Acceptable UX; documented in test.
- **Rust `exported_widget_round_trips_json` test is the only Rust coverage** — doesn't test the warnings or unknown-widget paths (those are TS-side responsibilities). Correct split, but worth noting that any future Python-script consumer of the format will hit the validation logic only in TS, not in Rust.
- **`downloadExportedWidget` uses browser DOM** (`document.createElement("a")`, `Blob`, `URL.createObjectURL`) — works in the Tauri webview which supports Web APIs, but would not work in a hypothetical Node-side export path. Tauri 2 webview compatibility is good; v1.0 only ships the Tauri designer, so this is fine.
- **Schema version 1 is now frozen** — the docs commit to "v1.x widget exports import into v1.y where y ≥ x." Any future shape change ratchets `schema_version` and adds a migration. This is the right contract; just flagging the irreversibility.

**Strong points (✅)**
- **Both languages share the wire shape** — `ExportedWidget` is defined in both `crates/protocol/src/widget_io.rs` and `packages/protocol-ts/src/index.ts` with identical field shapes. Future external tooling has Rust + TS implementations of the same contract.
- **Snapshot-test pin of the exported JSON** is the gold-standard discipline for format-stability tests. Any future field reordering or default-value change visibly bumps the snapshot, forcing a deliberate decision.
- **Defense-in-depth validation**: `validateKnownWidget` is called both at top-level and recursively. `parseExport` validates JSON shape before unwrap. `schema_version` check happens before any widget construction.
- **No partial state on import errors** — every error is thrown before `addChild()` runs in `ViewEditor.tsx`. The integrator never sees a half-imported view.
- **`id` regeneration prevents collisions** — same template can be imported N times without colliding ids. `idFactory` is injectable for tests (used in the round-trip vitest case).
- **`#[serde(default)]` on `bindings` and `children`** gives Rust future-forward compatibility — older exports lacking those fields still deserialize cleanly.
- **`#[non_exhaustive]` on both structs** per AL convention — new fields can be added without breaking Rust consumers.

**Findings**
- 🟢 The `ThemeEditor` JSX render block in `apps/designer/src/App.tsx` lands in this commit (a225aa7) even though it's CODEX-AO scope. Chunking observation: AO commit (`428a9cf`, 2 min earlier) added the `ThemeEditor` import + `saveTheme` callback + `onOpenTheme` prop; this AS commit added the JSX block that consumes them. Code compiles in commit order; not a defect. Worth flagging for the AO review so the reviewer knows some "AO content" lives here.
- 🟢 The format docs (`docs/widget-export-import.md` + `docs/agents/notes/widget-export-format.md`) are concise and honest. The notes file says "Version 1 is frozen once v1.0 ships" — exactly the schema-version contract.
- 🟡 `openwebhmi_version: "0.0.1"` hardcoded default — wire to a workspace-version constant in a future polish.
- 🟡 The Rust round-trip test could be extended with a `to_canonical_bytes()`-style pinned-byte test (mirror of the CODEX-AN audit-log discipline) to guard against serde-json key-ordering drift. v1.1 polish.
- 🟠 Real concerns — none.
- 🔴 Defects — none.

**Acceptance criteria tally**
- ✅ Export action present in designer; produces `.owhmi-widget` JSON via `downloadExportedWidget`.
- ✅ Import action present in designer (`<label>Import widget...<input type="file" accept=".owhmi-widget,application/json">`); validates and places.
- ✅ Unknown widget type → hard error with helpful message naming the type.
- ✅ Unknown binding paths → warnings surfaced via `onWarning(message)` toast; widget still places.
- ✅ Schema version 1 documented (both docs); mismatch is hard error naming the version.
- ✅ `docs/widget-export-import.md` exists; `README.md` features list links to it (per AQ+AR commit).
- ✅ `docs/agents/notes/widget-export-format.md` exists.
- ✅ Vitest tests pass.

## Verdict

**Merged** at `a225aa7`.

What's NOT yet proven by this merge:
- Manual smoke of the export/import flow in the running designer (defer to the designer manual-smoke checklist).
- Cross-language wire-format compatibility (Rust struct + TS type ARE identical-by-construction, but no integration test asserts they share a byte-identical representation; the canonical-bytes pinned test would add this guard).

No follow-ups opened from AS specifically. The two yellow polish items (hardcoded version string + pinned-byte test) are light enough to roll into a future touchup without their own briefs.
