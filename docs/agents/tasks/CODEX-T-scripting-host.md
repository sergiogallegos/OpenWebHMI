---
id: CODEX-T
title: crates/scripting — CPython-3.11+ host with worker subprocesses + system.* RPC
owner: codex
phase: 3
status: open
created: 2026-04-27
last-update: 2026-04-28 claude
---

# CODEX-T — `crates/scripting`

## Brief

### Goal

Embed CPython 3.11+ as the user-scripting layer. Per `docs/architecture.md` §4.7, scripts run in **worker subprocesses** (not in-process), so a numpy segfault or torch CUDA crash in user code cannot take down the gateway. Workers connect to the gateway via JSON-RPC over stdin/stdout. Scripts call `system.tag.read()`, `system.tag.write()`, etc., which are Python shims that proxy to the gateway through that JSON-RPC channel.

This is the project's headline differentiator vs Ignition's frozen Jython 2.7. Get the architecture right; scope the v1 surface tight.

### Context to read first

- `docs/architecture.md` §4.7 (Scripting Host) — the design.
- `docs/stack-rationale.md` "Why Python (scripting host)" — the strategic context.
- `crates/tag-engine/src/lib.rs` — `TagStore::publish` is what `system.tag.write` ultimately calls.
- `crates/project-store/src/types.rs` — extend with `ScriptConfig`.

### Architectural seam

```
┌────────────────────────────────────────────────────────────┐
│                Gateway (Rust)                                │
│                                                              │
│   ┌──────────────┐      tokio::spawn      ┌──────────────┐  │
│   │ ScriptHost   │ ◀────────────────────▶ │ WorkerProc   │  │
│   │ (Rust)       │   stdin/stdout JSON    │ (CPython)    │  │
│   └──────────────┘                         └──────┬───────┘  │
│                                                   │           │
│                                                   ▼           │
│                                       ┌─────────────────────┐ │
│                                       │ user_script.py       │ │
│                                       │   import system      │ │
│                                       │   on_tag_change(tag) │ │
│                                       └─────────────────────┘ │
└────────────────────────────────────────────────────────────┘
```

The gateway-side `ScriptHost` owns the WorkerProc set. WorkerProc is a CPython process spawned with a stub harness Python script that:
1. Imports `system` (the OpenWebHMI Python module bundled with the gateway).
2. Loads the user script via `exec`.
3. Reads JSON-RPC frames from stdin, dispatches to the script's registered handlers, writes results to stdout.
4. The `system.*` functions in Python emit JSON-RPC calls **back** to stdout, which the gateway's WorkerProc reader handles.

### Files to create / modify

- `crates/scripting/Cargo.toml`
- `crates/scripting/src/lib.rs`
- `crates/scripting/src/host.rs` — `ScriptHost` (manages workers, dispatches triggers).
- `crates/scripting/src/worker.rs` — wraps `tokio::process::Child`, handles framing.
- `crates/scripting/src/rpc.rs` — request/response types for the JSON-RPC channel.
- `crates/scripting/src/triggers.rs` — `OnTagChange`, `OnTimer`, `OnAlarm`, `OnButtonClick` registration.
- `crates/scripting/python/system/__init__.py` — the Python-side shim that proxies to JSON-RPC.
- `crates/scripting/python/system/tag.py` — `system.tag.read`, `system.tag.write`.
- `crates/scripting/python/_runner.py` — the worker harness loaded by every WorkerProc.
- `crates/scripting/tests/host.rs` — round-trip + crash-recovery tests.
- `crates/project-store/src/types.rs` — `ScriptConfig`, `ArtifactKind::Script { id }`.
- `crates/protocol/src/lib.rs` — script lifecycle wire forms (script.run/result/error) for designer integration.

Add `crates/scripting` to workspace `members`. Add `pyo3 = "0.22"` to workspace deps. (PyO3 is used for the *embedding* of Python in test rigs; the worker subprocesses are normal CPython processes, not PyO3-embedded — they're spawned via `tokio::process::Command::new("python3")`.)

### v1 surface (intentionally minimal)

**Triggers**: `on_tag_change(tag)` only. Other triggers (`on_timer`, `on_alarm`, `on_button_click`) are stubbed in the API but are tracked as separate follow-up tasks if Phase 3 timelines hold.

**`system.*` libraries**:
- `system.tag.read(path) -> TagValue`
- `system.tag.write(path, value) -> None`
- `system.util.now() -> int` (epoch ms)
- `system.util.log(message)` (writes to gateway's tracing log)

That's it for v1. `system.alarm.*`, `system.db.*`, `system.http.*` follow once the surface is proven.

### Worker lifecycle

1. Gateway boot loads scripts from project. For each script, spawn a WorkerProc with `python3 -m _runner <script_path>`.
2. WorkerProc handshakes (sends `{"kind":"ready","script_id":"..."}` to stdout).
3. Gateway starts piping events: `{"kind":"trigger","trigger":"on_tag_change","args":{"tag_path":"...","value":...,"quality":"good","ts_ms":...}}`.
4. Worker calls the user-registered handler. If the handler calls `system.tag.read("path")`, the worker emits `{"kind":"rpc","method":"tag.read","args":{"path":"..."},"id":"req-1"}` to stdout.
5. Gateway reads the RPC, executes against `TagStore`, writes the response back to the worker's stdin.
6. Worker resolves the Python `system.tag.read` call with the response value.

**Crash recovery**: if a worker exits unexpectedly (segfault, exception, `os._exit`), the host respawns it with exponential backoff (250ms → 8s, ±25% jitter, mirroring `DriverSupervisor` from CODEX-E).

### Resource limits

For v1: per-worker `tokio::time::timeout` on each trigger handler invocation (default 5s). Memory + CPU rlimits via `nix::sys::resource::setrlimit` on POSIX. Windows: no enforcement in v1 (document as a limitation).

### Test requirements

- A simple user script that registers `on_tag_change` and writes a derived tag works: gateway publishes a tag, worker fires the handler, handler calls `system.tag.write` for a different tag, gateway sees the new value.
- Worker crash mid-handler: kill the worker process; assert the host respawns within 1s and the next trigger fires the new worker.
- Handler timeout: a script that does `time.sleep(10)` is interrupted at 5s; an `error` log entry is written; the next trigger still fires.
- `system.util.log` writes to `tracing` at INFO level with the worker's script_id in the span.
- JSON-RPC round-trip on the framing: `id` correlation works for concurrent calls.

### Acceptance criteria

- [ ] `cargo test -p openwebhmi-scripting` green.
- [ ] `cargo test --workspace --all-features --locked` stays green.
- [ ] Phase 1 demo extended with one script: `on_tag_change("rockwell-1/Pressure")` → if value > 100, `system.tag.write("rockwell-1/Setpoint", value * 0.5)`. Manual smoke verifies the loop closes.
- [ ] Python prerequisites documented in `apps/designer/README.md` and a new `crates/scripting/README.md` (Python 3.11+ on PATH; pip-installable `numpy` + `pandas` recommended).
- [ ] All public Rust items have rustdoc; the `system.*` Python module has docstrings.

### Out of scope (split into follow-ups if Phase 3 schedule slips)

- `on_timer`, `on_alarm`, `on_button_click` triggers — separate task.
- `system.alarm.*`, `system.db.*`, `system.http.*` — separate task.
- Worker process pool (sharing one worker across multiple scripts) — Phase 5+.
- Sandboxing beyond rlimits (seccomp, AppArmor, Windows Job Objects) — post-1.0.
- Python venv per project — Phase 4.
- Hot-reload of script source without worker restart — post-1.0.

### Risks / gotchas

- **Python interpreter discovery**: don't hardcode `/usr/bin/python3`. Use `which`-style resolution; document the `OPENWEBHMI_PYTHON` env var override.
- **CPython on Windows**: the worker invocation differs; document the Windows command. Test on macOS first; Windows is "best effort" for v1.
- **Don't use PyO3's `Python::with_gil` from the gateway process** — that would defeat the subprocess isolation. PyO3 is only used inside the worker harness if you find it useful for the `system.*` shim; even there, plain `import json` + stdin/stdout is simpler and recommended for v1.
- **Buffered stdout deadlocks**: Python's stdout is line-buffered by default but blocks may collect. Set `PYTHONUNBUFFERED=1` in the worker env or use `sys.stdout.flush()` after every write.
- **JSON-RPC framing**: use `\n`-delimited JSON (one JSON object per line). Document this; don't invent length-prefixed framing.
- **Don't silently swallow Python exceptions** — write them to a `script_error` event the host logs at WARN, and surface to the designer's script editor (CODEX-U) on next view of the script.
- **Auth bypass on tag writes**: scripts call `system.tag.write` through the in-process `TagStore` directly, not through the role-gated WS path that CODEX-S guards. That's the correct design (the gateway *is* the trusted process), but every script-driven write must be logged at INFO with `script_id` + tag path + value so post-hoc audit can answer "why did Setpoint change at 03:14?". The journal goes through `tracing` for v1; a dedicated `script_audit` SQLite table can land in v1.1.

## Codex log

*(codex — append working notes here)*

## Claude review

*(claude — after submission)*

## Verdict

*(claude — final disposition)*
