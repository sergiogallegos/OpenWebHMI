# AGENTS.md

Codebase-wide rules for any agent working in this repository (Codex, Claude Code, or a human following the same playbook). Loaded automatically by Codex CLI and by Claude Code when the working directory is inside this repo.

Scope:

- This file — **codebase-wide code, test, and dependency rules**.
- `crates/AGENTS.md` — Rust-specific subset (auto-loaded under `crates/`).
- `apps/AGENTS.md`, `packages/AGENTS.md` — TypeScript-specific subsets (auto-loaded under those trees).
- `wiki/AGENTS.md` — engineering-wiki governance (auto-loaded under `wiki/`).
- `CLAUDE.md` — Claude-specific operating procedure: review/merge lifecycle, brief authoring, hand-off message format.
- `VISION.md` — what OpenWebHMI is, non-goals, what we won't merge.
- `docs/agents/README.md` — cross-agent task lifecycle (open → submitted → merged).

Read `VISION.md` before doing anything that might cross a "won't merge" line.

## Map

```
crates/             Rust workspace
  gateway/            single binary: tags, drivers, historian, alarms, auth, scripting
  driver-api/         Driver trait + supervisor
  driver-*            five v1 drivers (rockwell, opcua, modbus, mqtt, ads)
  historian/          time-series storage + read API
  alarm-engine/       state machine + journal
  auth/               local users, roles, JWT, per-view ACLs
  scripting/          CPython 3.11+ host + worker subprocesses
  project-store/      versioned project storage
  audit-log/          security event journal
  backup/             project + historian + alarm-journal export/import
  protocol/           wire types shared with TS via codegen
apps/               TypeScript apps (Vite/React/Tauri)
  runtime-web/        browser HMI runtime
  designer/           Tauri designer
  website/            public marketing + docs (Astro)
packages/           TypeScript libraries
  component-library/  HMI components (bindings, write-back tagPath pattern)
  protocol-ts/        TS protocol types (paired with crates/protocol)
docs/               user-facing docs (architecture, roadmap, contributing, etc.)
docs/agents/        cross-agent task board, log, briefs
wiki/               synthesized engineering knowledge (vendor quirks, decisions)
examples/           simulators (sim-rockwell, etc.)
scripts/            local dev + maintenance scripts
```

## Commands

Rust toolchain is pinned by `rust-toolchain.toml` (currently 1.95, edition 2024). Node version is pinned in `.github/workflows/ci.yml`. Package manager is **pnpm** — never `npm` or `yarn` in this repo.

Full validation matrix (debug builds; `--release` only when reproducing a perf issue):

```
cargo build --workspace --all-features --locked
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test --workspace --all-features --locked
cargo doc --workspace --no-deps
cargo fmt --check
pnpm -r typecheck
pnpm -r test
pnpm -r build              # for the apps/* and packages/* that have build scripts
```

For known-flaky integration tests, three consecutive green runs is the bar before claiming the run.

## Code rules

- **No `panic!`, `unwrap()`, `unreachable!()`, or `expect()` in production code.** Production = anything reachable from a non-test, non-startup, non-`main` path. Documented exceptions get a one-line `// invariant: ...` comment explaining the assertion. `tag-engine`'s `expect("rwlock poisoned")` is the existing precedent.
- **`#[expect(...)]` over `#[allow(...)]` for clippy lints.** `#[allow]` silences forever; `#[expect]` flips into a warning if the underlying code stops triggering the lint. When touching code that has an `#[allow]`, convert it.
- **Let chains over nested `if let`.** Rust 1.88+ / edition 2024 syntax. Prefer `if let Some(x) = opt && x.is_valid() { ... }` over the nested form.
- **Top-level imports only.** No `use foo::Bar;` inside function bodies, except to break a cyclic-import that can't be resolved structurally — rare, and gets a `// cyclic-import workaround` comment.
- **Full variable names.** `version` not `ver`; `tag_path` not `tp`; `historian` not `hist`. The codebase already follows this.
- **`#![deny(missing_docs)]` is the bar for every crate.** New crates land with this attribute from day one. `driver-rockwell`, `driver-modbus`, `driver-api`, `driver-opcua`, `driver-mqtt`, `driver-ads` are all on the list.
- **Comments explain WHY, not WHAT.** Default: no comments. Add one when the *invariant*, *workaround*, or *hidden constraint* would surprise a reader. Don't narrate well-named code. Don't reference the current task or callers — that belongs in the commit message.
- **If neighboring code does something differently than you're about to, find out why before deviating.** Three identical `spawn_blocking` wraps in `gateway/src/server.rs` are not three opinions — they're one shape, agreed-on. Match the neighbor unless you have a stated reason.
- **Ask "why this and not the alternative?" before non-obvious choices.** If the answer is "I don't know," spend five minutes investigating before writing the code.

## Tests

- **All changes must be tested. If you're not testing your changes, you're not done.** Behavior changes need behavior tests. Mechanical changes need at minimum compile + existing suite green on three consecutive runs.
- **Your test is NOT VALID if it passes without the fix.** Regression tests must demonstrably catch the bug they're guarding against. Run the test against the pre-fix code at least once.
- **Add to existing test files; don't fragment.** New tests join the closest existing file unless the new behavior is genuinely a new module's concern.
- **No flaky tests. No `sleep()`, `setTimeout()`, or wall-clock waits.** Use deterministic synchronization: channels, oneshots, `tokio::time::pause()` + `advance()`, explicit `JoinHandle::await`.
- **No hardcoded ports.** Bind to `127.0.0.1:0` and read the assigned port back from the listener.

## Dependencies

- **Never `cargo update` all dependencies.** Use `cargo update --precise <crate>@<version>` when a specific bump is needed. Lockfile drift across unrelated deps hides supply-chain regressions and inflates review surface.
- **`=`-pinned workspace versions stay pinned** until the task is explicitly a version bump. Current set: `tokio-modbus`, `async-opcua`, `rumqttc`, `rumqttd`, `prost`, `jsonpath-rust`, `ads`. Pinning is load-bearing.
- **License gate**: MIT, Apache-2.0, BSD, or compatible only. GPL/AGPL/SSPL deps do not land. See `VISION.md` for the full policy.
- **`Cargo.lock` diff is bounded.** Bumped crate + proc-macro counterpart + direct transitives. Wider = investigate before committing.

## Honesty

- **Never overstate what shipped.** Commits, PR descriptions, hand-off messages, and `## Codex log` entries describe what *actually* ran, not what was attempted. If something was deferred (manual smoke, hardware validation, a partial item), say so by name in the same sentence as the claim it qualifies.
- **"I tested it" only after running it.** Claiming a test passed without having executed it is a fabrication.
- **Own brief errors.** When a Claude-authored brief was wrong, the verdict says so by name. See CODEX-AH (`nTransMode = 4` vs the brief's `3`), CODEX-AJ (alarm-engine subscription path), CODEX-AL (ProjectStore identifier boundary). These are strong points, not embarrassments.
- **Don't undersell load-bearing items as "polish".** If a "v1.1 polish" item actually breaks demo-HMI headline behavior, it's a closeout blocker.

## Git

- Rust + TS changes can ship in the same commit when they're paired (protocol additions, designer/runtime changes that depend on each other). Don't artificially split.
- Commit messages: conventional-commits prefix (`feat`, `fix`, `docs`, `chore`, `refactor`, `test`) + crate or area scope. The `docs/agents/log.md` history is a good style reference.
- Lifecycle three-place updates (task frontmatter + `board.md` + `log.md`) commit together.
- Pushing to the remote is not automatic. Push when the maintainer explicitly asks, or when an unambiguous task convention requires it (backfilling a merge ref). See `docs/agents/README.md` for the full push policy.

## Don't

The full "will not merge" list lives in `VISION.md`. The frequently-tripped subset:

- No stub protocol implementations (drivers must speak the real wire protocol).
- No tests that pass without the fix.
- No `cargo update`-all lockfile drift.
- No telemetry / phone-home by default.
- No commercial vendor SDKs requiring NDAs or license keys.
- No fourth language (Rust + TS + Python is the set).
- No designer changes that alter the runtime contract without a paired runtime change in the same PR.
- No "I tested it" in a commit message when you didn't run it.

## See also

- `VISION.md` — what we're building, non-goals, won't-merge.
- `CLAUDE.md` — Claude-specific lifecycle, brief authoring, hand-off format.
- `docs/agents/README.md` — task lifecycle, status flow, who-edits-what.
- `docs/architecture.md` — system topology and component contracts.
- `docs/roadmap.md` — phase plan.
- `wiki/AGENTS.md` — engineering-wiki governance.
