---
id: CODEX-CI
title: Docs-vs-code reconciliation — make architecture.md / README / feature-matrix match the shipped surface
owner: codex
phase: 4
status: open
created: 2026-07-12
last-update: 2026-07-12 claude [Fable 5]
---

# CODEX-CI — Docs-vs-code reconciliation

## Brief

> The public docs advertise a Phase-3-complete system the code does not match: CPython-via-PyO3 (reality is a `python3` subprocess over JSON-RPC), a full `system.*` scripting stdlib (reality is 4 RPC methods), four script triggers (reality is one), a CPU/memory/network resource sandbox (reality is a wall-clock timeout), rich tag data types (reality is `Bool | Int | Real | String`), and an "empty workspace" note that is 21 crates stale. This is a **docs-only honesty task** — edit the docs so every capability claim matches the code, reframing unshipped items as roadmap/deferred. **Do not change code.** This is the VISION.md "Correctness & honesty" contract applied to the docs surface: a contributor grepping the docs must find accurate claims, not aspirational ones.

### Goal

Every capability claim in `README.md`, `docs/architecture.md`, `docs/feature-matrix.md`, and `docs/roadmap.md` either describes what the code actually does today or is explicitly marked roadmap/deferred/post-1.0. A reviewer grepping the docs for "PyO3", the unshipped `system.*` families, the unshipped triggers, the resource-sandbox language, the extended tag-type list, and the "empty workspace" note finds each one either removed or reframed as not-yet-shipped. No code files change.

### Context to read first

- [`VISION.md`](../../../VISION.md) §"Correctness & honesty" (~47-54) — the honesty contract this task enforces; and §"No stub implementations" — the same principle the docs must not violate by overselling.
- [`CLAUDE.md`](../../../CLAUDE.md) §"Honesty" — "Never overstate what you got done"; docs are part of what ships.
- Code that is the source of truth for each claim (read these before editing the matching doc line):
  - `crates/scripting/src/worker.rs:1,42-59` — the header comment says "CPython worker subprocess"; `Command::new(python).arg("-u").arg(runner)` spawns a `python3` **subprocess**. There is no PyO3 embed. Grep confirms: no `.rs` file references `pyo3`.
  - `crates/scripting/src/rpc.rs:99-114` — `RpcMethod` has exactly **4** variants: `tag.read`, `tag.write`, `util.now`, `util.log`. No alarm/db/http/subscribe/send_message methods exist.
  - `crates/scripting/src/triggers.rs:14-28,37-43` — `OnTimer` / `OnAlarm` / `OnButtonClick` are `/// Stub` variants and are explicitly `filter_map`-ed out in `tag_change_paths`; only `OnTagChange` is wired.
  - `crates/scripting/src/host.rs:25-33` — the only enforced limit is `handler_timeout` (a wall-clock `Duration`). No CPU/memory/network limiting.
  - `crates/protocol/src/lib.rs:204-215` — the wire `TagValue` is `Bool(bool) | Int(i64) | Real(f64) | String(String)`. No typed ints, no `Bytes`, no `Array`, no `Struct`.
  - `crates/project-store/src/types.rs:286-305` — `BindingSource::Expression { code }` is documented in-code as a "Placeholder for Phase 3 expression evaluation" — a stored string with no evaluator.
  - `Cargo.toml:3-25` — workspace `members` lists **21** crates/apps/examples. (Also `Cargo.toml:61` declares a **dangling** `pyo3 = "0.22"` workspace dependency that no crate consumes — see Risks/gotchas.)
- The doc lines to correct (exact locations in Files to modify below).
- CODEX-CL (`docs/agents/tasks/CODEX-CL-scripting-system-library-gap.md`) — the paired task that decides implement-vs-defer per `system.*` item. Coordinate: CI reframes docs to match the *current* code; if CL lands first and implements some items, CI's reframing follows CL's decisions. Sequence-independent because each is corrected against whatever the code says at merge time.

### Files to create / modify

Docs only. For each item below, verify the cited code first, then edit the doc so the claim is accurate. Use the repo's existing roadmap markers (`🟡 post-1.0`, `(Phase N)` qualifiers, "roadmap", "deferred", "not yet implemented") rather than inventing new notation.

1. **`README.md`**
   - Line ~53 (Stack list): "Scripting: Python 3.11+ via PyO3, run in worker subprocesses" — remove "via PyO3". The mechanism is a `python3` subprocess over JSON-RPC on stdin/stdout, not a PyO3 embed. Reword to describe subprocess + JSON-RPC IPC.
   - Line ~66 (Stack-vs-incumbents table, Scripting row): "CPython 3.11+ in worker subprocesses" — keep "CPython 3.11+ in worker subprocesses" (accurate) but ensure nothing on this line or line ~69's prose implies an in-process/embedded (PyO3) integration.
   - Line ~111 (Targeted parity): "Python scripting with `system.tag`, `system.alarm`, `system.db`, `system.http` libraries" — reframe. `system.tag` (read/write) and `system.util` (now/log) ship; `system.alarm`, `system.db`, `system.http` do not. Mark the unshipped families as roadmap (tie to CODEX-CL's decision if it has landed).
   - Line ~126 (Building callout): the "Pre-Phase-0 … the Cargo workspace is currently empty (`members = []`)" note is stale — the workspace has 21 members and builds. Replace with an accurate build note (`cargo build --workspace` / `cargo test --workspace` run today).

2. **`docs/architecture.md`**
   - Line ~120 (§4.2 Tag Engine, "Data types"): the list `Bool, Int8/16/32/64, Uint8/16/32/64, Real32, Real64, String, Bytes, Array<T>, Struct(UDT)` overshoots the wire type. State the shipped `TagValue` set (`Bool | Int (i64) | Real (f64) | String`) as v1, and mark the wider type system (typed ints, `Bytes`, `Array`, `Struct`/UDT) as roadmap/post-1.0.
   - Line ~125 ("Expression tags — derived … recomputed when inputs change"): reframe as roadmap — the stored form exists (`BindingSource::Expression`) but there is no evaluator.
   - Line ~216 (§4.7 Scripting Host): "Embeds CPython via PyO3 in a pool of worker subprocesses" — correct. It runs `python3` as a subprocess and speaks JSON-RPC over stdio; it does **not** embed via PyO3. Rewrite this bullet and the "Why subprocesses" bullet (~217) so they describe subprocess isolation without the PyO3 claim.
   - Line ~219 (Triggers list): only `on_tag_change` is implemented. Mark `on_timer`, `on_alarm`, `on_button_click`, `on_view_open`, `on_view_close` as roadmap (coordinate with CODEX-CL).
   - Lines ~220-229 (stdlib code block): only `system.tag.read`, `system.tag.write`, `system.util.now`, `system.util.log` ship. Mark `system.tag.subscribe`, `system.alarm.ack`, `system.db.query`, `system.http.get/post`, `system.util.send_message` as roadmap (coordinate with CODEX-CL).
   - Line ~231 (§4.7) and line ~363 (§7 Security boundaries): both promise "resource limits (CPU, memory, network)". Only a per-handler wall-clock timeout exists. Reword to "process isolation with a per-handler wall-clock timeout; CPU/memory/network resource limits are roadmap hardening" — do not claim an enforced resource sandbox.

3. **`docs/feature-matrix.md`**
   - Line ~46 ("Expression tag (derived) … 🟢 v1 (Phase 3)"): no evaluator ships. Downgrade/qualify to reflect the stored-placeholder reality (e.g. `🟡 post-1.0` or a "stored, not yet evaluated" qualifier) — match whichever marker the file already uses for not-yet-shipped rows.
   - Line ~49 ("User-Defined Types (UDTs / Object Types) … 🟢 v1"): the wire `TagValue` has no `Struct`/UDT. Reframe as roadmap.
   - Line ~71 ("Expression bindings … 🟢 v1 (Phase 3)"): same reason as line ~46 (`BindingSource::Expression` is an un-evaluated placeholder). Qualify.
   - Note: the extended data-type list (typed ints / `Array` / `Struct`) lives in **architecture.md ~120**, not in a feature-matrix row — fix it there (item 2). Only reframe the feature-matrix rows that assert a capability the code lacks.

4. **`docs/roadmap.md`**
   - Line ~98 ("Scripting (Python via PyO3)"): drop "via PyO3"; describe the subprocess/JSON-RPC mechanism.
   - Line ~100 (Phase 3 triggers `on_tag_change, on_timer, on_alarm, on_button_click`): only `on_tag_change` shipped. Distinguish shipped vs. remaining-for-a-later-slice (point at CODEX-CL).
   - Line ~101 (`system.tag.*, system.alarm.*, system.db.*, system.util.*` libraries): `system.tag` (read/write) and `system.util` (now/log) shipped; `system.alarm.*`, `system.db.*`, and the http family are not. Reframe.
   - Line ~103 ("Per-script resource limits"): only a wall-clock timeout ships; reframe as roadmap hardening.

5. **(Optional, recommended) `docs/agents/notes/docs-honesty.md`** — a short surface note: "docs claims are checked against code at review time; every capability claim in README / architecture / feature-matrix / roadmap must cite or match a code path, and unshipped items use the roadmap markers." Mirror the format of the existing `docs/agents/notes/*.md` files. Keep it under ~30 lines. If added, do not restate it inline elsewhere — link to it.

### Behavior

- No source code, `Cargo.toml`, or test file changes. This task's diff is confined to `README.md`, `docs/architecture.md`, `docs/feature-matrix.md`, `docs/roadmap.md`, and (optionally) a new `docs/agents/notes/` file.
- Every corrected claim uses the existing roadmap/deferred vocabulary; no capability is described as present that the cited code does not provide.
- The correction is faithful to VISION.md's scope: items VISION.md already lists as roadmap/post-1.0 (e.g. UDTs, expression evaluation) are marked as such, not deleted — the roadmap intent stays, only the "already shipped" framing is removed.

### Test requirements

- No automated tests (docs-only). Verification is a grep sweep, recorded in the Codex log:
  - `grep -rin "pyo3" README.md docs/` returns no claim that scripting embeds/uses PyO3 (a roadmap mention that explicitly says "not PyO3" is fine; an assertion that it *is* PyO3 is not).
  - `grep -rn "members = \[\]\|currently empty" README.md docs/` returns nothing describing the workspace as empty.
  - Grepping for `system.alarm`, `system.db`, `system.http`, `system.tag.subscribe`, `system.util.send_message`, `on_timer`, `on_alarm`, `on_button_click` in the four docs finds each one only inside roadmap/deferred framing, never asserted as shipped.
  - `scripts/validate-agent-files` passes if the optional note is added under `docs/agents/notes/` (it only validates task/board/log files, but run it to be safe).
- Codex log records the before/after wording for each corrected line and confirms each was checked against the cited code path.

### Acceptance criteria

- [ ] No "PyO3" claim remains in `README.md` or `docs/architecture.md` or `docs/roadmap.md` that asserts the scripting host embeds/uses PyO3; the subprocess + JSON-RPC mechanism is described accurately.
- [ ] The `system.*` stdlib is described as `tag.read`/`tag.write`/`util.now`/`util.log` shipped; `alarm.*`/`db.*`/`http.*`/`tag.subscribe`/`util.send_message` marked roadmap (consistent with CODEX-CL's decision if landed).
- [ ] Script triggers are described as `on_tag_change` shipped; the rest roadmap.
- [ ] The scripting sandbox is described as process isolation + per-handler wall-clock timeout; no enforced CPU/memory/network resource-limit claim (architecture.md ~231 and ~363 both corrected).
- [ ] Tag data types in architecture.md ~120 reflect the shipped `Bool | Int | Real | String` wire type; typed ints / `Bytes` / `Array` / `Struct`(UDT) marked roadmap. Expression tags/bindings reframed as un-evaluated placeholder in architecture.md ~125, feature-matrix ~46 and ~71, and UDTs at feature-matrix ~49.
- [ ] The "empty workspace / `members = []`" note in README.md ~126 is replaced with an accurate build note (21 workspace members).
- [ ] No code, `Cargo.toml`, or test file modified.
- [ ] Codex log lists each corrected line with the source-of-truth code path it was checked against.

### Out of scope

- **Changing any code**, including deleting the dangling `pyo3 = "0.22"` workspace dependency (`Cargo.toml:61`). Flag it in the Codex log so a follow-up cleanup task can remove it, but this task does not touch `Cargo.toml`.
- **Implementing any `system.*` method or trigger** — that is CODEX-CL. CI only makes the docs honest about the current state.
- **Rewriting the docs' structure, tone, or marketing framing** beyond correcting the specific false/stale capability claims. Minimal, surgical edits.
- **Touching the designer script-editor stubs** (`apps/designer/src/lib/systemStubs.ts`) — they already list only the 4 shipped methods + `on_tag_change` and are honest. No change needed.
- **Correcting claims in other docs** (`docs/stack-rationale.md`, `docs/scale-estimates.md`, wiki pages) unless they repeat one of the five specific overselling claims above; if they do, note it in the log for a follow-up rather than expanding this task's scope.

### Risks / gotchas

- **Reframe, don't delete the roadmap intent.** UDTs, expression evaluation, and the wider `system.*` library are real v1-target/post-1.0 goals per VISION.md and roadmap.md. The bug is the "already shipped" framing, not the ambition. Keep the goal; fix the tense.
- **Coordinate with CODEX-CL.** If CL lands first and implements, say, `system.alarm.ack` + `on_timer`, then CI marks those as shipped, not roadmap. Re-check the code (not the sibling task's brief) at merge time — the code is the source of truth.
- **The `python3`-subprocess mechanism is not a weakness to hide.** VISION.md's crash-isolation rationale (a segfaulting C extension must not take down the gateway) is exactly why subprocess isolation is correct. Describe it as the deliberate design it is — the only error is calling it PyO3.
- **The dangling `pyo3 = "0.22"` workspace dep is misleading but out of scope here.** It is declared in `Cargo.toml` and consumed by nothing. Removing it is a code change; flag it for follow-up, do not silently reframe docs around a dep that shouldn't exist.
- **"Process isolation, not a resource sandbox" is the exact honest phrasing** for the scripting limits — a wall-clock timeout bounds a single handler's runtime; it does not cap CPU, memory, or network. Don't let "sandbox" language survive in a way that implies enforced resource ceilings.
- **Honesty discipline applies to this task's own log.** If a claim turns out already-accurate on a re-read, say so; don't "correct" a line that was fine just to show activity. The deliverable is accuracy, not edit count.

## Codex log

## Claude review

## Verdict
