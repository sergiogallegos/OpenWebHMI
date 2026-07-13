---
id: CODEX-BV
title: Config-driven data paths — stop hardcoding sqlite files into the working directory
owner: codex
phase: 4
status: open
created: 2026-07-12
last-update: 2026-07-12 claude [Fable 5]
---

# CODEX-BV — Config-driven data paths

## Brief

> The gateway hardcodes two SQLite files as cwd-relative paths while every other database is a CLI argument. `crates/gateway/src/main.rs` opens `AlarmJournal::open("openwebhmi-alarms.sqlite")` (line 327) and `HistorianStore::open("openwebhmi-history.sqlite")` (line 401) as bare relative paths, whereas `--auth-db` and `--audit-db` are proper `#[arg(long, default_value = …)]` fields on `Args` (lines 47, 50). As a result the alarm and history databases land wherever the process happened to be launched — the stray `openwebhmi-alarms.sqlite` and `openwebhmi-history.sqlite` files currently sitting in the repo root are the direct evidence. Add `--alarm-db` and `--history-db` CLI args (or derive all four from a single `--data-dir`) with sensible defaults, mirroring the existing `--auth-db`/`--audit-db` pattern.

### Goal

The alarm-journal and historian database paths are configurable on the command line and default to a predictable data location, consistent with `--auth-db`/`--audit-db`. Running the gateway from any directory no longer scatters `.sqlite` files into the cwd.

### Context to read first

- `crates/gateway/src/main.rs:29-66` — the `Args` clap struct. Note the established pattern: `auth_db: PathBuf` with `#[arg(long, default_value = "openwebhmi-auth.sqlite")]` (46-47) and `audit_db` (48-50). The new args mirror this exactly.
- `crates/gateway/src/main.rs:327` — `AlarmJournal::open("openwebhmi-alarms.sqlite")?` (hardcoded).
- `crates/gateway/src/main.rs:401` — `HistorianStore::open("openwebhmi-history.sqlite")?` (hardcoded).
- `crates/gateway/src/main.rs:92-93` — the `--audit-db` open with a `with_context` error message; mirror the error-context style for the new opens.
- `crates/gateway/src/main.rs:634-635` and `:756-757` — existing test constructions of `Args` set `auth_db`/`audit_db` to `tempdir` paths. Any new fields must be set here too (these break to compile otherwise — a useful forcing function).
- `.gitignore:52-54` — already contains `*.sqlite`, `*.sqlite-journal`, and `examples/projects/_index.sqlite`. **Correction to the brief's premise:** the stray files are already ignored by the `*.sqlite` glob (they are untracked, not committed). So the real fix is not writing them into the cwd in the first place; a `.gitignore` edit is at most adding the two explicit names as documentation, not a functional change. Do not claim the `.gitignore` change stops them from being committed — `*.sqlite` already does.

### Files to create / modify

1. **Modify** `crates/gateway/src/main.rs` — `Args`:
   - Add `alarm_db: PathBuf` and `history_db: PathBuf` fields with `#[arg(long, default_value = …)]`, mirroring `auth_db`/`audit_db`. **Or** add a single `data_dir: PathBuf` (`#[arg(long, default_value = …)]`) and derive all four db paths (`auth`, `audit`, `alarm`, `history`) beneath it. Pick one approach and document the choice + rationale in the Codex log ("why this and not the alternative"). The per-db-arg approach is a smaller diff and matches the existing two args; the `--data-dir` approach is cleaner long-term but changes the `--auth-db`/`--audit-db` defaults' base. If choosing `--data-dir`, keep `--auth-db`/`--audit-db` overridable and back-compatible.
   - Defaults should place the files under a data directory, not the bare cwd. Match whatever convention the existing `--auth-db`/`--audit-db` defaults imply — if they too default to cwd-relative names today, at minimum make the four consistent and document the default location; do not silently change where auth/audit land without noting it.

2. **Modify** `crates/gateway/src/main.rs:327` and `:401` — open the databases from `args.alarm_db` / `args.history_db` (or the derived paths), with a `with_context` error message mirroring line 92-93.

3. **Modify** the `Args` test constructions (`main.rs:634-635`, `:756-757`) — set the new fields to `tempdir`-based paths so tests write into a temp dir, not the repo.

4. **Modify** `.gitignore` (optional, documentation-only) — the `*.sqlite` glob (line 52) already covers both files; adding the explicit names is a readability nicety, not a functional fix. If added, keep it minimal and note in the Codex log that it is documentation, not the fix.

5. **Remove** the stray `openwebhmi-alarms.sqlite` and `openwebhmi-history.sqlite` from the repo root if they are present in the working tree (they are untracked, so `rm` them; confirm `git status` shows them gone and nothing tracked was removed).

### Behavior

- `--alarm-db PATH` / `--history-db PATH` (or `--data-dir DIR`) control where the alarm and history databases live.
- With no flags, the databases default to a predictable data location consistent with the other two dbs, not scattered by launch directory.
- Existing `--auth-db`/`--audit-db` behavior is unchanged (or, under `--data-dir`, remains overridable and back-compatible).

### Test requirements

- A test asserting the alarm and history db paths are configurable (constructing `Args` with explicit temp paths and confirming the databases open there).
- A test asserting the default paths resolve under the intended data directory (not the bare cwd). Since this is process-cwd-sensitive, assert on the *resolved path value* the code computes from `Args`, not by inspecting the filesystem relative to cwd.
- No `sleep()`, no hardcoded ports. Use `tempfile`/`tempdir` for any on-disk assertions.
- Full validation matrix clean (`cargo build`, `cargo clippy … -D warnings`, `cargo test --workspace --all-features --locked`, `cargo doc --workspace --no-deps`).

### Acceptance criteria

- [ ] `--alarm-db` and `--history-db` (or `--data-dir` deriving all four) exist on `Args` with `#[arg(long, default_value = …)]`, mirroring `--auth-db`/`--audit-db`.
- [ ] `main.rs:327` and `:401` open from the configured paths with `with_context` error messages.
- [ ] Chosen approach (per-db args vs `--data-dir`) and rationale documented in the Codex log.
- [ ] Test `Args` constructions updated; tests write to temp dirs.
- [ ] Configurability + default-location tests present and green.
- [ ] Stray `openwebhmi-alarms.sqlite` / `openwebhmi-history.sqlite` removed from the working tree; `.gitignore` already covers them via `*.sqlite` (any explicit-name addition noted as documentation-only).
- [ ] No `unwrap`/`expect`/`panic!` outside `fn main`/startup-config validation (startup validation is exempt per CLAUDE.md); full matrix clean.

### Out of scope

- Reworking `--auth-db`/`--audit-db` semantics beyond what a `--data-dir` approach necessarily touches (and if `--data-dir` is chosen, keep them back-compatible).
- A full config-file (`openwebhmi.toml`) layer for the gateway — this brief is CLI args only.
- Changing the SQLite schema, retention, or storage engine for either database.
- Relocating the project-store root or other data paths not named here.
- Environment-variable overrides for the new args (clap `env` attributes) unless trivially consistent with the existing pattern — otherwise a follow-up.

### Risks / gotchas

- **The `.gitignore` premise in the source brief is slightly off.** `*.sqlite` already ignores both stray files, so they were never at risk of being committed — the bug is *where they get written*, not whether git tracks them. State this correctly; don't overclaim a `.gitignore` fix.
- **Default location vs. cwd.** If `--auth-db`/`--audit-db` themselves default to cwd-relative names today, "under a data dir" for the two new args creates an inconsistency. Prefer making all four consistent; if that means changing the auth/audit defaults, that is a behavior change worth flagging explicitly in the Codex log (and possibly out of scope — confirm before changing where auth/audit land).
- **Test `Args` constructions are the compile forcing-function.** Adding fields breaks `main.rs:634-635` and `:756-757` until updated — good; it guarantees no test writes to the repo root. Set them to `tempdir` paths.
- **`fn main` / startup-config exemption.** Panics/`expect` in startup config validation are permitted per CLAUDE.md, but prefer `anyhow::Result` propagation with `with_context` to match the existing `--audit-db` open style.
- **Startup path creation.** If the default data dir doesn't exist, decide whether the gateway creates it or errors clearly; match whatever the auth/audit opens do today (don't introduce a novel behavior).

## Codex log

## Claude review

## Verdict
