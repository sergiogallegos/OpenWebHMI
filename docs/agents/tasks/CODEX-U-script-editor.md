---
id: CODEX-U
title: Designer script editor — Monaco + Python syntax + system.* stubs
owner: codex
phase: 3
status: open
created: 2026-04-27
last-update: 2026-04-28 claude
---

# CODEX-U — Designer script editor

## Brief

> **Unblocked.** [CODEX-T](CODEX-T-scripting-host.md) merged at `f8b74a9`; [CODEX-V](CODEX-V-script-tag-write-routing.md) merged at `9db307e` (closeout fix). The `system.*` surface is `tag.read/write` + `util.now/log`; `on_tag_change` is the only active trigger.
>
> **Brief refreshed 2026-04-28** with two important changes the original draft missed:
> 1. **Script source is NOT saved through `ArtifactKind::Script { id }`.** That artifact is the JSON config envelope (`{ id, path, enabled, triggers }`) at `scripts/{id}.json`. The Python source lives separately at `scripts/{path-from-config}.py`. CODEX-U adds a new `ArtifactKind::ScriptSource { id }` variant to project-store that resolves the path from the loaded `ScriptConfig` and writes raw text (no JSON serialization).
> 2. **Script-event subscribe protocol doesn't exist yet.** CODEX-T's protocol additions (`ClientMessage::ScriptRun` + `ServerMessage::ScriptResult/ScriptError`) are for one-shot designer runs, correlated by `request_id`. The Recent errors pane needs a separate streaming subscription. CODEX-U adds `ClientMessage::ScriptSubscribe`/`ScriptUnsubscribe` + `ServerMessage::ScriptEvent`, mirroring the alarm subscribe pattern.

### Goal

A first-class script-authoring module in the designer. Monaco editor with Python syntax highlighting, auto-completion stubs for `system.*`, save through `project.save_artifact { Script(id) }`, and a "Recent errors" pane that surfaces script_error events from the gateway.

### Context to read first

- `apps/designer/src/modules/ViewEditor.tsx` — same module shape this task mirrors.
- `apps/designer/src/lib/designerClient.ts` — extend with `subscribeScriptEvents` for error surfacing.
- The `system.*` API surface from CODEX-T's deliverables.
- Monaco editor docs: https://microsoft.github.io/monaco-editor/

### Files to create / modify

**Backend (must land first; the editor is dead without it):**
- `crates/project-store/src/store.rs` — add `ArtifactKind::ScriptSource { id }`. The `artifact_path` resolver looks up the script id in the loaded `ScriptConfig` list and joins `scripts/{config.path}`; writes/reads raw bytes (NOT JSON). If the script id isn't registered, error with a clear message.
- `crates/project-store/src/store.rs` — `save_artifact` body branch: for `ScriptSource`, the body is `{ source: "..." }` and the `source` string gets written as raw UTF-8 to the `.py` file. Bump version + broadcast `ChangeAction::Updated` so the runtime/script-host can hot-reload.
- `crates/protocol/src/lib.rs` + `packages/protocol-ts/src/index.ts` — add `ClientMessage::ScriptSubscribe { project_id }`, `ClientMessage::ScriptUnsubscribe { project_id }`, `ServerMessage::ScriptEvent { project_id, script_id, event_kind, status?, message? }`. Mirror the `alarm.subscribe`/`alarm.event`/`alarm.unsubscribe` shape.
- `crates/gateway/src/server.rs` — `set_default_script_host` (analog to `set_default_alarm_engine`); subscribe handler taps `ScriptHost::subscribe_events()` and forwards `ScriptEvent::{Status, Log, Error}` to subscribed WS clients as `ServerMessage::ScriptEvent`. Project-id filter (only forward events for the subscribed project).
- `crates/gateway/src/main.rs` — register the spawned `ScriptHost` with `set_default_script_host` so the WS layer can fan out events.
- `crates/scripting/src/host.rs` — `ScriptEvent::Status`/`Log`/`Error` already exist; expose `project_id` on each variant (currently only `script_id`). The host needs to know which project a script belongs to for filtering.

**Frontend (the actual editor work):**
- `apps/designer/package.json` — add `monaco-editor`, `@monaco-editor/react`, and a Vite Monaco worker plugin (e.g. `vite-plugin-monaco-editor`).
- `apps/designer/src/lib/designerClient.ts` — add `loadScriptSource(projectId, scriptId)`, `saveScriptSource(projectId, scriptId, source)`, `subscribeScriptEvents(projectId, callback)` mirroring the existing alarm/view client patterns.
- `apps/designer/src/lib/systemStubs.ts` — TS literal with the `system.*` signatures + docstrings, fed to Monaco's completion provider.
- `apps/designer/src/modules/ScriptList.tsx` — sidebar list of project scripts (read from `project.scripts`); add / rename / delete via `project.save_artifact { Script(id) }` (the JSON config) plus `project.save_artifact { ScriptSource(id) }` (the source body).
- `apps/designer/src/modules/ScriptEditor.tsx` — Monaco editor; loads via `loadScriptSource`; 300ms-debounced save via `saveScriptSource`; status indicator (saved / saving / unsaved / error); auto-completion source from `systemStubs.ts`.
- `apps/designer/src/modules/ScriptErrorPane.tsx` — Recent errors pane subscribed via `subscribeScriptEvents`. Bounded buffer (last 20 per script). Click an error → Monaco "go to line" if a line number can be parsed from the traceback.
- `apps/designer/src/App.tsx` — switch logic: `selectedModule` gains `"scripts"`; `ProjectExplorer` surfaces Scripts.
- `apps/designer/src/modules/ProjectExplorer.tsx` — add Scripts tree entry.

### Behavior

#### ScriptEditor
- Loads source via `loadScriptSource(projectId, scriptId)` → `project.load_artifact { ScriptSource(id) }`.
- Monaco configured: `language: "python"`, `theme: "vs-dark"` (or follows OS preference), `tabSize: 4`.
- Auto-completion source: `systemStubs.ts` declares `system.tag.read`, `system.tag.write`, `system.util.now`, `system.util.log`, and `@system.on_tag_change` with docstrings.
- Save: 300ms-debounced via `saveScriptSource` → `project.save_artifact { ScriptSource(id), body: { source } }`. Same debounce pattern as ViewEditor / AlarmConfig.
- Save status indicator (saved / saving / unsaved / error).
- Hot-reload coordination: when the gateway receives a `ScriptSource` change, the script host should restart that worker (already wired for the `Tags` and `Alarms` artifacts; mirror it for `ScriptSource`). Out-of-scope hardening: don't restart on every keystroke — the 300ms client-side debounce is the natural rate-limit.

#### ScriptList
- Tree of scripts in the project (rendered from the loaded `project.scripts` array).
- Add: prompts for an id, creates a default `system.on_tag_change` skeleton via two writes (`Script(id)` for the config and `ScriptSource(id)` for the starter source).
- Rename: rewrites the config + moves the file. v1 acceptable to do this as delete-old + create-new; document the limitation.
- Delete: `project.delete_artifact { Script(id) }` AND `{ ScriptSource(id) }`.

#### Recent errors pane
- Subscribes via `subscribeScriptEvents(projectId, callback)`. The callback fires for every `ServerMessage::ScriptEvent` (status / log / error) for that project. The pane filters to errors only by default; toggle for full event stream.
- Bounded buffer: last 20 events per script, dropped FIFO. Across all scripts in the project, hard cap at 200.
- Each error row shows: timestamp, script_id, message (first line of traceback), expandable to full traceback.
- "Go to line": parses `line N` from the traceback when present and tells Monaco to scroll. If parsing fails, just open the script.

#### Prerequisites
Document in `apps/designer/README.md`:
- Python 3.11+ on PATH (gateway requirement, also needed for the worker subprocesses CODEX-T spawns).
- Optional: per-project `requirements.txt` for `numpy` / `pandas` / `scikit-learn`. Phase 4 adds first-class venv-per-project; v1 uses the system Python.

### Test requirements

Rust:
- `crates/project-store` test: round-trip a `ScriptSource` artifact (write source → read source → bytes match exactly). Verify the `.py` file lives at `scripts/{config.path}` and that JSON encoding is NOT applied.
- `crates/protocol` literal-form test: `ClientMessage::ScriptSubscribe`, `ClientMessage::ScriptUnsubscribe`, `ServerMessage::ScriptEvent` round-trip.
- `crates/gateway` test (or in `crates/scripting`): subscribing through the WS forwards Status/Log/Error events for the matching project_id only.

TS (Vitest):
- `ScriptEditor.test.tsx`: a `MockEditor` stub stands in for Monaco (Monaco's web-worker setup doesn't initialize cleanly in jsdom). The test verifies edit → debounced save callback fires.
- `ScriptList.test.tsx`: add/rename/delete fire the right gateway client calls (both `Script(id)` and `ScriptSource(id)` artifacts).
- `ScriptErrorPane.test.tsx`: synthetic `ScriptEvent` callbacks render rows; bounded buffer drops oldest after 20 per script.
- `systemStubs.test.ts`: snapshot test — locks the surface so a stub change is a deliberate code-review item.

Manual smoke (added to `apps/designer/README.md`, step #16+):
- Open the demo project's `derived-setpoint` script.
- Edit it: change the multiplier from `0.5` to `0.6`. Save.
- Watch the runtime — within 1s of the next `Pressure` update, `Setpoint` should track 0.6× pressure (verified by running the simulator with Pressure > 100).
- Introduce a typo (`sytem.tag.read` instead of `system.tag.read`); save; observe the `NameError: name 'sytem' is not defined` traceback in the Recent errors pane within seconds.
- Click the error → editor scrolls to the offending line.
- Fix the typo, save; observe the script returns to working order.

### Acceptance criteria

- [ ] `cargo test --workspace --all-features --locked` green (covers project-store + protocol + gateway/scripting subscribe forwarding tests).
- [ ] `cargo clippy --workspace --all-targets --all-features -- -D warnings` clean.
- [ ] `pnpm -r typecheck && test` green.
- [ ] Monaco bundle size acceptable (target: designer JS bundle stays under 5MB after the addition; Monaco's tree-shaking is OK with the right `vite-plugin-monaco-editor` config).
- [ ] Auto-completion of `system.tag.read("` shows the docstring.
- [ ] Manual smoke succeeds end-to-end (edit → save → loop closes; introduce typo → error appears in pane within seconds).

### Out of scope

- Python language server / type checking inside Monaco (post-1.0; would require Pyright in a worker).
- Script debugger / breakpoints (post-1.0).
- Script package manager UI (`pip install` from the editor) (post-1.0).
- Diff view between script versions (Phase 4).
- AI-assisted script generation (Phase 4+ — explicit feature, separately scoped).

### Risks / gotchas

- **Backend before frontend.** Land the project-store + protocol + gateway plumbing first — without `ArtifactKind::ScriptSource` and `script.subscribe`/`script.event` wire forms, the editor has nothing to talk to. Recommend two commits: backend first (Rust + protocol-ts mirror + tests), then frontend.
- **`save_artifact` raw-text branch.** Don't shoehorn Python source into JSON (`{ "source": "...\n..." }`) on disk — that loses readability and breaks `git diff` on `.py` files. The JSON envelope is the wire form; the on-disk file is plain text. The store branches on `ArtifactKind::ScriptSource` to extract `body["source"]` and write the string directly.
- **Hot-reload on `ScriptSource` change.** The script host needs to restart the affected worker when a `ScriptSource` change broadcasts. Mirror what `crates/scripting/src/host.rs` does today on tag/script-config changes (track this carefully — a missing restart would mean the editor saves but the worker keeps running the old code).
- **Project-id propagation in `ScriptEvent`.** Today, `ScriptEvent::{Status,Log,Error}` only carries `script_id`. The subscribe handler needs to filter by project_id, so add `project_id` to each variant. Touches `crates/scripting/src/host.rs`, the workspace tests, and any callers (gateway `main.rs`).
- **Monaco bundle size**: blindly importing `monaco-editor` ships ~5MB. Use `vite-plugin-monaco-editor` (or equivalent) to load only the languages you need (`python` only for now). Acceptable budget: designer JS bundle ≤ 5MB after addition.
- **Monaco in jsdom**: Monaco's web-worker setup doesn't initialize cleanly in jsdom. Tests should mock the editor or use a minimal `MockEditor` component. Don't fight Monaco in jsdom.
- **Tauri webview doesn't run Python.** The editor edits source; the worker subprocesses (CODEX-T) execute on the gateway side. Don't accidentally try to run Python in the Tauri webview.
- **Don't auto-format on save.** Python users have strong preferences (black, ruff, autopep8). Phase 4 adds an opt-in formatter setting.
- **Don't break the existing 5 host tests in `crates/scripting/tests/host.rs`.** Adding `project_id` to `ScriptEvent` requires updating the test fixtures. Default to `"phase1-demo"` or similar.

## Codex log

*(codex — append working notes here)*

## Claude review

*(claude — after submission)*

## Verdict

*(claude — final disposition)*
