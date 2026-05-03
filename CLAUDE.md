# Working on OpenWebHMI as Claude

OpenWebHMI is an open-source SCADA/HMI platform — Rust gateway + Tauri designer + React/TS web runtime + Python scripting. Ignition / FactoryTalk Optix alternative. See `README.md` for the elevator pitch and `docs/architecture.md` for the v1 system design.

## Division of labor

OpenWebHMI uses a two-agent collaboration model:

- **Codex** writes the code.
- **Claude** authors task briefs, reviews submissions, merges, and updates the bookkeeping.
- **Maintainer** routes messages between the two agents, makes strategic decisions, and runs manual smoke validations.

## How to resume any session

1. `git pull` first — the durable state is on origin, not local.
2. Read **`docs/agents/board.md`** — the entry point. The phase-specific table lists open tasks with their statuses (`open`, `in-progress`, `submitted`, `under-review`, `merged`, `rejected`). Anything not `merged` is in-flight.
3. For any non-merged row, open `docs/agents/tasks/CODEX-{ID}-{slug}.md` and read the frontmatter (`status:` is authoritative), the Brief, the Codex log, and any prior Claude review.
4. Read `docs/agents/log.md` for the chronological context — last ~20 lines usually recover the recent thread.

Don't re-derive state by reading every file. The agent docs are the durable handoff — `board.md` tells you what's open in 60 seconds. Trust it.

## Review and merge lifecycle

When Codex submits (status `submitted` in the task frontmatter):

1. Run the test matrix independently, don't trust Codex's verification claim. Standard matrix:
   ```
   cargo test --workspace --all-features --locked
   cargo clippy --workspace --all-targets --all-features -- -D warnings
   cargo fmt --check
   pnpm -r typecheck
   pnpm -r test
   ```
   Plus any task-specific extras (e.g. component-library `build`, designer `build:vite`, three consecutive runs for known-flaky integration tests).
2. Read the changed/new files — at minimum the impl module, the test file, and any wiki entry the brief asked for.
3. Write the `## Claude review` section with strong points (✅), findings (🟡 polish, 🟠 real concern), and acceptance-criteria tally.
4. Set frontmatter `status: merged`, write the `## Verdict` section, update `board.md` (move row to Done, record merge commit), append to `log.md`.
5. Commit + push. Backfill the merge commit hash into board.md and the verdict in a follow-up commit.

## When to reject vs fix-during-merge

**Reject** when the implementation is fundamentally wrong — the submitted code can't fulfill the brief's contract even when the tests pass. Example: CODEX-Z's first submission imported the `ads` crate only as a dead-code marker function and exchanged JSON-line frames with a stub sim; it didn't speak ADS at all and couldn't talk to a real Beckhoff PLC. Rejected with rationale, brief amended in-place, status returned to `open`.

**Fix during merge** when the bug is mechanical and mirrors an existing pattern. Examples:
- CODEX-U's `vite-plugin-monaco-editor` Node 22+ incompatibility — gated the plugin to `command === 'build'` (5-line vite.config.ts change).
- CODEX-AB's Slider/Dropdown/ToggleSwitch hardcoded `"value"` write target — added `tagPath?: string` props matching NumericInput's existing fallback pattern.

Document Claude-applied fixes transparently in the verdict.

**Always flag honestly:**
- Brief errors — when a Claude-authored brief was wrong (e.g. CODEX-T's "system.tag.write calls TagStore::publish directly", CODEX-Z's `ads = "0.7"` pin that doesn't exist on crates.io). Own them in the verdict.
- Verification mismatches — Codex's environment vs the local merge environment (Node version drift, missing system deps).
- Don't undersell load-bearing items as "polish" — if a "v1.1 polish" actually breaks the demo HMI's headline behavior, it's a closeout blocker, not polish.

## Brief authoring conventions

When opening a new task (`CODEX-{next-letter}`):

```yaml
---
id: CODEX-XY
title: <short title>
owner: codex
phase: <0..4>
status: open
created: YYYY-MM-DD
last-update: YYYY-MM-DD claude
---
```

Then sections: `## Brief` with goal + context to read first + files to create + behavior + test requirements + acceptance criteria + out of scope + risks/gotchas. Plus empty `## Codex log`, `## Claude review`, `## Verdict`. Mirror existing briefs (CODEX-X / W / Y / AB are the recent shape templates).

When the task is opened, also: add a row to `board.md`'s phase table, append a one-line entry to `log.md`, commit + push.

## Hand-off message format for Codex

After opening or amending a task brief, write a hand-off message the maintainer can paste to Codex. Format:
- Task ID + path to task file
- 2-3 sentence summary of what's required and why
- Specific constraints (pinned versions, scope limits, "don't shortcut X")
- Acceptance gate ("ping back when submitted")

Don't restate the entire brief — Codex reads the task file. The hand-off message is the bridge.

## Phase ladder reminder (as of last commit)

- Phases 0, 1 complete (v0.1.0 tagged)
- Phases 2, 3 code-complete (awaiting manual smoke validation for v0.2.0 / v0.3.0 tags)
- Phase 4 in progress: drivers (X Modbus + W OPC UA + Y MQTT + AA MQTT-gaps merged); components (AB batch 2 merged, AC batch 3 status varies — read board.md)
- One driver still open at all times: **CODEX-Z** (ADS rework, awaiting Codex)

After Phase 4 driver + component slices complete, the remaining v1.0 ladder: plugin SDK, audit log, backup/restore, performance baseline, and the pre-1.0 hardware-validation gate (24h continuous run of `driver-rockwell` against real CompactLogix/ControlLogix).

## Project-specific gotchas

- **Toolchain drift:** Rust is pinned by `rust-toolchain.toml` and CI Node is pinned in `.github/workflows/ci.yml`; include both versions when reporting environment-specific verification failures.
- **CODEX-V plumbing**: `system.tag.write` from Python scripts routes through `GatewayTagWriteSink` (driver-prefixed paths → per-driver write mpsc; memory-tag paths → `TagStore::publish`). When briefing Python-related tasks, don't say "publish directly" — that's the v1.0 brief error that prompted CODEX-V.
- **Read/write asymmetry in components**: bindings give the read path but not the write path. Write-back inputs need a separate `tagPath` config prop (NumericInput / Slider / Dropdown / ToggleSwitch all follow this). v1.1 architectural fix: extend the binding system to expose the bound path for write-back use.
- **MQTT TLS in CI**: `rumqttd 0.20.0` is plaintext-only. Real TLS testing uses an external Mosquitto-with-CA fixture (manual smoke step #27 in designer README).
