# Working on OpenWebHMI as Claude

OpenWebHMI is an open-source SCADA/HMI platform — Rust gateway + Tauri designer + React/TS web runtime + Python scripting. Ignition / FactoryTalk Optix alternative. See `README.md` for the elevator pitch and `docs/architecture.md` for the v1 system design.

## Division of labor

OpenWebHMI uses a two-agent collaboration model:

- **Codex** primarily writes the code; may also review, merge, and push.
- **Claude** primarily authors task briefs, reviews submissions, merges, and updates the bookkeeping; may also write code and push.
- **Maintainer** routes messages between the two agents, makes strategic decisions, and runs manual smoke validations.

Either agent may run the review/merge lifecycle — Claude via `.claude/skills/openwebhmi-merge/`, Codex via `.agents/skills/openwebhmi-merge/`. The skill content is mirrored; only the agent-name conventions in the verdict/log lines differ.

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
3. Write the `## Claude review` section using the nine-part contract in [`docs/agents/review-template.md`](docs/agents/review-template.md): independent verification, what's being fixed, root cause confirmation, fix appropriateness, test proof, residual risk, strong points (✅), findings (🟢/🟡/🟠/🔴), acceptance-criteria tally.
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

When opening a new task (`CODEX-{next-letter}`), or when reviewing a task Codex
authored under explicit maintainer direction:

```yaml
---
id: CODEX-XY
title: <short title>
owner: codex
phase: <non-negative roadmap phase>
status: open
created: YYYY-MM-DD
last-update: YYYY-MM-DD claude [Opus 4.7]
---
```

The `last-update` field carries the underlying model in square brackets (e.g. `claude [Opus 4.7]`, `codex [gpt-5]`) so the maintainer can audit model-vs-quality over time. Same convention applies to entry headers inside `## Codex log` and `## Claude review` (`### YYYY-MM-DD HH:MM <author> [<model>]`) and to lines in `log.md`. See [`docs/agents/README.md`](docs/agents/README.md) for the full format spec.

Then sections: `## Brief` with goal + context to read first + files to create + behavior + test requirements + acceptance criteria + out of scope + risks/gotchas. Plus empty `## Codex log`, `## Claude review`, `## Verdict`. Mirror existing briefs (CODEX-X / W / Y / AB are the recent shape templates).

When the task is opened, also: add a row to `board.md`'s phase table, append a one-line entry to `log.md`, commit + push.

## Hand-off message format for Codex

After opening or amending a task brief, write a hand-off message the maintainer can paste to Codex. Format:
- Task ID + path to task file
- 2-3 sentence summary of what's required and why
- Specific constraints (pinned versions, scope limits, "don't shortcut X")
- Acceptance gate ("ping back when submitted")

Don't restate the entire brief — Codex reads the task file. The hand-off message is the bridge.

## Code quality and testing discipline

Cross-cutting rules. Apply to every PR Codex submits, every review Claude writes, and every commit either agent ships. These are *contracts*, not preferences — landing code that breaks one of them is a regression even if all tests pass.

### Honesty

- **Never overstate what you got done.** Commits, PR descriptions, hand-off messages, and Codex-log entries describe what *actually* shipped, not what was attempted. If something was deferred (manual smoke, hardware validation, a partial item), say so by name in the same sentence as the claim it qualifies. Pattern proven across CODEX-AJ / -AK / -AL / -AI: Codex called out "Manual smoke not run in this environment" in every submission; that's the bar.
- **"I tested it" only after running it.** Claiming a test passed without having executed it is a fabrication, not a shortcut. Codex's "three consecutive runs" log entries are the discipline.
- **Own brief errors.** When a Claude-authored brief was wrong, the verdict says so by name. See CODEX-AH (`nTransMode = 4` vs the brief's `3`), CODEX-AJ (alarm-engine subscription path the brief missed), CODEX-AL (ProjectStore identifier boundary). These are *strong points*, not embarrassments.

### Testing

- **All changes must be tested. If you're not testing your changes, you're not done.** Adapted from ruff/bun. Behavior changes need behavior tests; mechanical changes need at minimum a "does it still compile and the existing suite still passes" verification on three consecutive runs.
- **Your test is NOT VALID if it passes without the fix.** Regression tests must demonstrably catch the bug they're guarding against. Run the test against the pre-fix code at least once; if it passes, the test isn't testing what you think.
- **Add to existing test files; don't fragment.** New tests join the closest existing file unless the new behavior is genuinely a new module's concern. Pattern: AJ extended `crates/historian/tests/store.rs` rather than creating `recorder_spawn_blocking.rs`; AK extended the AJ shutdown test rather than creating a new file.
- **No flaky tests. No `sleep()`, `setTimeout()`, or wall-clock waits in tests.** Use deterministic synchronization (channels, oneshots, `tokio::time::pause()` + `advance()`, explicit `JoinHandle::await`). The repo's "known-flaky integration tests" history is the warning; don't add to it.
- **No hardcoded ports.** Bind to `127.0.0.1:0` and read the assigned port back from the listener.

### Rust code quality

- **No `panic!`, `unwrap()`, `unreachable!`, or `expect()` in production code.** Production = anything reachable from a non-test, non-startup, non-`main` execution path. Exceptions are explicit: `tag-engine`'s `expect("rwlock poisoned")` is an acceptable panic policy for non-recoverable state and is documented in code. New exceptions need a one-line comment explaining the invariant being asserted. Test code, startup config validation, and `fn main` are exempt.
- **`#[expect(...)]` over `#[allow(...)]` for clippy lints.** `#[allow]` silences forever; `#[expect]` flips into a warning if the underlying code stops triggering the lint. Codify the lint's *expected* presence; let the compiler tell you when reality changes. AJ landed `#[allow(dead_code)]` on `driver-ads/src/driver.rs` as an incidental — convert these to `#[expect]` as they're touched.
- **Let chains over nested `if let`.** Rust 1.88+ / edition 2024. Codebase is on 1.95 per `rust-toolchain.toml`. Prefer `if let Some(x) = opt && x.is_valid() { ... }` over `if let Some(x) = opt { if x.is_valid() { ... } }`.
- **Top-level imports only.** No `use foo::Bar;` inside function bodies (except to break a cyclic-import that can't be resolved structurally — rare, and gets a `// cyclic-import workaround` comment).
- **Full variable names, no abbreviations.** `version` not `ver`; `tag_path` not `tp`; `historian` not `hist`. The codebase already follows this; codifying it.
- **Comments explain WHY, not WHAT.** Default: no comments. Add one when the *invariant*, *workaround*, or *hidden constraint* would surprise a reader. Don't narrate well-named code. Don't reference the current task or callers — that belongs in the commit message. Pattern: AK's `// Dropping a JoinHandle does not cancel its task; AbortHandle is the cancel-only handle.` at the first AbortHandle insertion site — that's the shape.
- **No `--release` builds during development.** Release builds lack debug assertions and compile slower. The full validation matrix is `cargo build --workspace --all-features --locked` (debug), `cargo clippy --workspace --all-targets --all-features --locked -- -D warnings`, `cargo test --workspace --all-features --locked`, `cargo doc --workspace --no-deps`. Release builds only when reproducing a performance issue.
- **`#![deny(missing_docs)]` is the bar for every crate.** `driver-rockwell`, `driver-modbus`, `driver-api` already meet it; CODEX-AM brings the remaining drivers up. New crates land with this attribute from day one.

### Dependency management

- **Never `cargo update` all dependencies.** Use `cargo update --precise <crate>@<version>` when a specific bump is needed. Lockfile drift across unrelated dependencies hides supply-chain regressions and inflates review surface.
- **Workspace `=`-pinned versions stay pinned** until the task is explicitly a version bump. The current `=`-pinned set: `tokio-modbus`, `async-opcua`, `rumqttc`, `rumqttd`, `prost`, `jsonpath-rust`, `ads`. Pinning is load-bearing (driver-side compatibility); unpinning needs a brief.
- **`Cargo.lock` diff is bounded.** A dep bump should touch the bumped crate + its proc-macro counterpart + direct transitives. Anything wider is a red flag — investigate before committing.

### "Why this and not the alternative?"

- **Before making a non-obvious choice, ask the question pre-emptively.** If the answer is "I don't know," that's the cue to spend five minutes investigating before writing the code. Codex's pattern of catching brief gaps (AJ alarm-engine, AK SupervisorHandle::Drop, AL ProjectStore boundary) all came from asking this question; the brief described what to do, and the question surfaced what the brief missed.
- **If neighboring code does something differently than you're about to, find out why before deviating.** Three identical `spawn_blocking` wraps in `gateway/src/server.rs` are not three opinions — they're one shape, agreed-on, and your fourth wrap should match unless you have a stated reason. When in doubt, copy the neighbor.
- **Don't take a bug report's suggested fix at face value.** The reporter knows the symptom; you have to verify which layer to fix. The brief is similarly suggestive — if the brief's "Files to modify" list is incomplete (AJ missed the alarm-engine subscription path), update the brief in the verdict and own the gap, don't silently work around it.

## Phase ladder reminder (as of last commit)

- Phases 0, 1 complete (v0.1.0 tagged)
- Phases 2, 3 code-complete (awaiting manual smoke validation for v0.2.0 / v0.3.0 tags)
- Phase 4 in progress: drivers (X Modbus + W OPC UA + Y MQTT + AA MQTT-gaps merged); components (AB batch 2 merged, AC batch 3 status varies — read board.md)
- One driver still open at all times: **CODEX-Z** (ADS rework, awaiting Codex)

After Phase 4 driver + component slices complete, the remaining v1.0 ladder: plugin SDK, audit log, backup/restore, performance baseline, and the pre-1.0 hardware-validation gate (24h continuous run of `driver-rockwell` against real CompactLogix/ControlLogix).

## Project-specific gotchas

Surface-specific lore lives under [`docs/agents/notes/`](docs/agents/notes/) — load on demand when touching the relevant surface.

- [`notes/toolchain-drift.md`](docs/agents/notes/toolchain-drift.md) — Rust + Node version pinning; what to ask when verification claims don't reproduce.
- [`notes/python-tag-write-routing.md`](docs/agents/notes/python-tag-write-routing.md) — `system.tag.write` routes through `GatewayTagWriteSink` (CODEX-V plumbing); don't say "publish directly" in briefs.
- [`notes/binding-write-asymmetry.md`](docs/agents/notes/binding-write-asymmetry.md) — component bindings expose read paths but not write paths; the `tagPath?: string` prop pattern is the v1.0 workaround.
- [`notes/mqtt-tls-in-ci.md`](docs/agents/notes/mqtt-tls-in-ci.md) — `rumqttd 0.20.0` is plaintext-only; real TLS validation is a maintainer-run manual-smoke gate.

Don't restate these inline in briefs. Link to the note; that's the durable reference.
