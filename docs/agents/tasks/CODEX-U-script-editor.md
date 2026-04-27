---
id: CODEX-U
title: Designer script editor — Monaco + Python syntax + system.* stubs
owner: codex
phase: 3
status: open
created: 2026-04-27
last-update: 2026-04-27 claude
blocked-by: CODEX-T
---

# CODEX-U — Designer script editor

## Brief

> **Blocked by [CODEX-T](CODEX-T-scripting-host.md).** Needs the script lifecycle protocol (`script.run`, `script.error`) and `system.*` API surface to generate auto-completion stubs from.

### Goal

A first-class script-authoring module in the designer. Monaco editor with Python syntax highlighting, auto-completion stubs for `system.*`, save through `project.save_artifact { Script(id) }`, and a "Recent errors" pane that surfaces script_error events from the gateway.

### Context to read first

- `apps/designer/src/modules/ViewEditor.tsx` — same module shape this task mirrors.
- `apps/designer/src/lib/designerClient.ts` — extend with `subscribeScriptEvents` for error surfacing.
- The `system.*` API surface from CODEX-T's deliverables.
- Monaco editor docs: https://microsoft.github.io/monaco-editor/

### Files to create / modify

- `apps/designer/package.json` — add `monaco-editor`, `@monaco-editor/react`.
- `apps/designer/src/modules/ScriptEditor.tsx` — Monaco-backed editor.
- `apps/designer/src/modules/ScriptList.tsx` — sidebar list of project scripts.
- `apps/designer/src/lib/systemStubs.ts` — generated TS file declaring the `system.*` Python signatures (used to populate Monaco's auto-completion).
- `apps/designer/src/App.tsx` — surface Scripts in the project explorer.

### Behavior

#### ScriptEditor
- Loads the script body via `project.load → project.scripts[id]`.
- Monaco configured: `language: "python"`, `theme: "vs-dark"` (or follows OS preference), `tabSize: 4`.
- Auto-completion source: `systemStubs.ts` declares `system.tag.read`, `system.tag.write`, etc. with their docstrings.
- Save: 300ms-debounced `project.save_artifact { Script(id), body: { source } }` — same pattern as ViewEditor.
- Save status indicator (saved / saving / unsaved / error).

#### ScriptList
- Tree of scripts in the project.
- Add / Rename / Delete via `project.save_artifact` / `project.delete_artifact`.

#### Recent errors pane
- Subscribes to `script.error` events from the gateway (CODEX-T emits these on Python exceptions).
- Shows last 20 errors per script: timestamp, exception type, message, the line number Monaco can jump to via "Go to line".

#### Prerequisites
Document in `apps/designer/README.md`:
- Python 3.11+ on PATH (gateway requirement, also needed for the worker subprocesses CODEX-T spawns).
- Optional: per-project `requirements.txt` for `numpy` / `pandas` / `scikit-learn`. Phase 4 adds first-class venv-per-project; v1 uses the system Python.

### Test requirements

Vitest:
- `ScriptEditor.test.tsx`: renders Monaco (or a stub if Monaco's loader is hostile to jsdom), edit emits debounced save callback.
- `ScriptList.test.tsx`: add/rename/delete fire the right gateway client calls.
- `systemStubs.test.ts`: snapshot test — locks the surface so a stub change is a deliberate code review item.

Manual smoke (added to `apps/designer/README.md`):
- Open the demo project's `pressure_setpoint` script.
- Edit it: change the multiplier from `0.5` to `0.6`.
- Save.
- Watch the runtime — within 1s of the next `Pressure` update, `Setpoint` should update with the new multiplier.
- Introduce a typo (`sytem.tag.read` instead of `system.tag.read`); save; observe the error in the Recent errors pane within seconds.

### Acceptance criteria

- [ ] `pnpm --filter @openwebhmi/designer typecheck && test` green.
- [ ] Monaco bundle size acceptable (target: designer JS bundle stays under 5MB after the addition; Monaco's tree-shaking is OK with the right `vite-plugin-monaco-editor` config).
- [ ] Auto-completion of `system.tag.read("` shows the docstring.
- [ ] Manual smoke succeeds.

### Out of scope

- Python language server / type checking inside Monaco (post-1.0; would require Pyright in a worker).
- Script debugger / breakpoints (post-1.0).
- Script package manager UI (`pip install` from the editor) (post-1.0).
- Diff view between script versions (Phase 4).
- AI-assisted script generation (Phase 4+ — explicit feature, separately scoped).

### Risks / gotchas

- **Monaco bundle size**: blindly importing `monaco-editor` ships ~5MB. Use `vite-plugin-monaco-editor` (or equivalent) to load only the languages you need (`python` only for now).
- **Monaco in jsdom**: Monaco's web-worker setup doesn't initialize cleanly in jsdom. Tests should mock the editor or use a minimal `MockEditor` component.
- **Tauri webview Python execution**: the editor doesn't run Python — it edits source. The worker subprocesses (CODEX-T) execute on the gateway side. Don't accidentally try to run Python in the Tauri webview.
- **Don't auto-format on save** — Python users have strong preferences (black, ruff, autopep8). Phase 4 adds an opt-in formatter setting.

## Codex log

*(codex — append working notes here)*

## Claude review

*(claude — after submission)*

## Verdict

*(claude — final disposition)*
