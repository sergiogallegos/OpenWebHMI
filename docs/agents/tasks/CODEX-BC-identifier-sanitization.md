---
id: CODEX-BC
title: Identifier sanitization at the ProjectStore boundary — close path-traversal / arbitrary file R/W
owner: codex
phase: 4
status: open
created: 2026-07-12
last-update: 2026-07-12 claude [Fable 5]
---

# CODEX-BC — Identifier sanitization at the ProjectStore boundary (Tier-1 SECURITY)

## Brief

> **v1.0 blocker.** `ProjectStore` joins attacker-controllable `project_id`, view ids, and script ids straight into filesystem paths with zero sanitization, and honors absolute `script.path` values — a remotely-reachable path-traversal / arbitrary file read+write. Add a strict identifier allowlist enforced at the store boundary, reject absolute/traversing script paths, canonicalize every resolved path and verify it stays under the store root before any filesystem operation, and move the backup restore handler's `project_id` equality check to *before* the destructive delete/write. **Scope-locked to the sanitization boundary** — no ProjectStore schema redesign, no `ProjectId` newtype (that is the CODEX-AL v1.1 follow-up), no backup archive-format change.

### Goal

Every path derived from a client- or archive-supplied identifier is provably confined to the ProjectStore root. A `project_id`, view id, or script id containing `..`, `/`, `\`, a drive prefix, a leading dot, or an empty string is rejected with a clear error *before* any `fs` call. An absolute or traversing `script.path` in a project manifest is rejected. The backup restore HTTP handler validates `manifest.project_id == <url project_id>` before it deletes or writes anything. Four regression tests — traversal `project_id`, `../` view id, absolute `script.path`, and a malicious backup manifest — each fail against the pre-fix code and pass after.

### Context to read first

- `crates/project-store/src/store.rs:251-283` — the vulnerable core. `artifact_path()` (251-262) joins `format!("{id}.json")` for `View`/`Script` kinds; `script_source_path()` (264-279) returns `path.to_path_buf()` verbatim when `script.path` `is_absolute()`, and otherwise joins it under the project dir with no `..` check; `project_dir()` (281-283) is `self.inner.root.join(project_id)` — a `project_id` of `../../etc` escapes the root directly.
- `crates/gateway/src/server.rs:894-963` — `ClientMessage::ProjectSaveArtifact` (894-934) and `ProjectReadArtifact` (935-963) call `save_artifact` / `read_artifact` with the client's `project_id` and `artifact`. `ProjectDeleteArtifact` (965+) is the same shape. These are reachable by any authenticated `AuthorProject` / `ReadViews` session — and in the no-auth serve modes, by anyone who can open a socket.
- `crates/backup/src/lib.rs:195-243` — `import_project()`. In `Replace` mode it calls `store.delete_project(&manifest.project_id)` at line 214, then loops `store.save_artifact(&manifest.project_id, artifact.kind.clone(), body)` at 224. `validate_archive_path()` (428) only validates `artifact.path` (the *entry* path inside the tar), **not** the `manifest.project_id` nor the derived write location the `ArtifactKind` resolves to.
- `crates/gateway/src/server.rs:1676` — the restore handler's `manifest.project_id != project_id` guard fires *after* `import_project` has already run (delete + writes at 214/224 complete inside the `spawn_blocking` at 1665). The check is cosmetic; the damage is already done.
- CODEX-AL (`docs/agents/tasks/CODEX-AL-api-polish.md`) — its Codex log records that ProjectStore schema structs still carry identifiers as bare `String`, documented as a v1.1 boundary in `wiki/architecture/api-surface-stability.md`. That boundary is exactly the surface this brief hardens; build on it, don't redo it.
- `docs/architecture.md` §7 (trust model) — operators are untrusted; this is the enforcement gap.
- `VISION.md` — SCADA gateways sit on OT networks; arbitrary file write on the gateway host is a plant-level compromise.

### Files to create / modify

1. **Modify** `crates/project-store/src/store.rs`:
   - Add a private `fn validate_identifier(id: &str) -> anyhow::Result<()>` that rejects any id not matching a strict allowlist. Recommended rule: non-empty, bounded length (e.g. ≤ 128 bytes), every byte in `[A-Za-z0-9_-]`, and explicitly reject `.`, `..`, and any id beginning with `.`. Do not depend on a regex crate if a byte-scan is clearer — match the crate's existing dependency posture.
   - Enforce `validate_identifier` on `project_id` in `project_dir()` (or at every public entry that takes a `project_id` — pick the single chokepoint so no path escapes it), and on the `id` field of the `View` / `Script` / `ScriptSource` `ArtifactKind` variants inside `artifact_path()`.
   - In `script_source_path()` (264-279): reject `script.path` when it `is_absolute()` or when any component is `..` (or otherwise non-`Normal`, mirroring `validate_archive_path`'s `Component::Normal` scan). A registered script must resolve under the project's `scripts` dir.
   - After resolving any `PathBuf`, canonicalize the parent (the file itself may not exist yet on write) and assert the result is still prefixed by `self.inner.root` before returning it. A resolved path that escapes the root is an error, not a best-effort clamp.
2. **Modify** `crates/backup/src/lib.rs`:
   - In `import_project()` validate `manifest.project_id` with the ProjectStore identifier rule **before** the `delete_project` at 214. Reject a manifest whose `project_id` fails validation before any destructive op.
   - Confirm the per-artifact `ArtifactKind` ids are validated by the ProjectStore boundary (they flow through `save_artifact` → `artifact_path`); if `save_artifact` is not the enforcement chokepoint, make it one so a crafted `ArtifactKind::View { id: "../x" }` in a manifest cannot escape.
3. **Modify** `crates/gateway/src/server.rs`:
   - Move the `manifest.project_id != project_id` equality check to *before* `import_project` runs. Parse the manifest (or thread the expected `project_id` into `import_project` so it rejects a mismatch before deleting/writing). The URL-supplied `project_id` must gate the destructive path, not post-hoc it.
4. **Do NOT**:
   - Introduce a `ProjectId` / `ScriptId` newtype — that is the explicit CODEX-AL v1.1 follow-up. Keep the `String` shape; sanitize at the boundary.
   - Change the backup archive format, the wire protocol, or `ArtifactKind`'s variants.
   - Weaken `validate_archive_path` — it stays; this brief adds the *derived-location* and *identifier* checks it doesn't cover.

### Behavior

- A `ProjectSaveArtifact` / `ProjectReadArtifact` / `ProjectDeleteArtifact` with `project_id = "../../etc"` (or containing `/`, `\`, `..`, `:`, or empty) is rejected with an error before any `fs` call; nothing outside the store root is touched.
- A `View`/`Script`/`ScriptSource` artifact whose `id` contains `..` or a separator is rejected identically.
- A project whose registered script has an absolute `path` (e.g. `/etc/cron.d/x`) or a `..`-traversing relative path yields an error from `script_source_path`, not a read/write of that path.
- A backup restore whose `manifest.project_id` mismatches the URL `project_id`, or whose `manifest.project_id` fails identifier validation, is rejected **before** `delete_project` runs — no destructive side effect on mismatch.
- Legitimate identifiers (`main`, `line-3_press`, `HMI-Overview`) continue to work unchanged. No behavior change for well-formed projects.

### Test requirements

- Add regression tests to the closest existing test files — `crates/project-store/`'s existing store tests for the identifier + script-path cases, and `crates/backup/`'s existing tests for the malicious-manifest case. Do not fragment into new files unless the crate has no existing home.
- **Each regression test must fail against the pre-fix code.** Run every new test once with the sanitization reverted and confirm it exercises the traversal (e.g. asserts a file was written outside a `tempdir` root, or that the escape path resolved) — a test that passes without the fix is not testing the fix.
  - `project_id = "../escape"` (and one with `..` mid-path) is rejected; no file created outside the temp root.
  - `ArtifactKind::View { id: "../../x" }` is rejected.
  - A registered script with an absolute `path` and one with a `..` relative `path` are both rejected by `script_source_path`.
  - `import_project` with `manifest.project_id = "../evil"` (Replace mode) does **not** delete/write outside the root and returns an error before the destructive step; and a manifest whose `project_id` mismatches the expected id aborts before `delete_project`.
- Use `tempfile`/`tempdir` roots (no hardcoded paths). No `sleep`/wall-clock waits.
- Full Rust matrix clean: `cargo build --workspace --all-features --locked`, `cargo clippy --workspace --all-targets --all-features --locked -- -D warnings`, `cargo test --workspace --all-features --locked`, `cargo doc --workspace --no-deps`.

### Acceptance criteria

- [ ] `validate_identifier` (or equivalently-named chokepoint) rejects empty, over-length, `.`, `..`, leading-dot, and any id containing a byte outside `[A-Za-z0-9_-]`, enforced on `project_id` and on `View`/`Script`/`ScriptSource` artifact ids.
- [ ] `script_source_path` rejects absolute and `..`-traversing `script.path`; a resolved script path is asserted to stay under the store root.
- [ ] Every path returned by `artifact_path` / `script_source_path` is confirmed (canonicalized-parent + prefix check) to be under `self.inner.root` before any `fs` op.
- [ ] `import_project` validates `manifest.project_id` before `delete_project`; a mismatch or invalid id aborts before any destructive op.
- [ ] The gateway restore handler's `project_id` equality gate runs before `import_project`'s delete/write, not after.
- [ ] Four regression tests (traversal `project_id`, `../` view id, absolute `script.path`, malicious backup manifest) each fail without the fix and pass with it.
- [ ] No `ProjectId`/`ScriptId` newtype introduced; `String` shape preserved. No archive-format or wire-protocol change.
- [ ] Full Rust matrix clean (build + clippy + test + doc); no new `#[allow]` (use `#[expect]` with a reason if a lint must be suppressed).
- [ ] `validate_archive_path` unchanged and still enforced.

### Out of scope

- **`ProjectId` / `ScriptId` / `ViewId` newtypes.** The CODEX-AL v1.1 follow-up; this brief sanitizes the `String` boundary only.
- **ProjectStore schema redesign** or moving identifiers off `String` in the on-disk format.
- **Backup archive format changes** or signing (BH/BE territory for the audit/transport concerns).
- **Auth/permission model changes** — a valid `AuthorProject` session is still allowed to author; this brief only stops it (and unauthenticated callers in no-auth modes) from escaping the store root.
- **Symlink hardening beyond canonicalization** — if the store root itself contains attacker-planted symlinks that is a separate threat; canonicalize-and-prefix-check is the v1.0 bar.

### Risks / gotchas

- **Canonicalize the parent, not the file.** On a *write* the target file does not exist yet, so `Path::canonicalize` on it fails. Canonicalize the parent directory (which must exist or be created under the root) and join the validated leaf. Confirm the prefix check uses canonicalized paths on both sides (the root may itself be a symlink on macOS — `/tmp` → `/private/tmp`), or the check will false-positive in tests.
- **Windows path shapes.** `validate_archive_path` already rejects `:` (drive prefix) and `\`. Mirror that in the identifier rule so a `C:` or `\\` id can't slip through on non-Windows CI and then bite on a Windows host.
- **The chokepoint must be single.** If validation is sprinkled at call sites, a future call site will forget it. Put it at `project_dir()` / `artifact_path()` so *every* derived path passes through it. Confirm no public method reaches the filesystem bypassing that chokepoint.
- **Restore ordering.** `import_project` currently reads the manifest inside `spawn_blocking` and the equality check is in the async handler after the join. Either parse+validate the manifest's `project_id` before spawning the destructive work, or thread the expected `project_id` into `import_project` so it bails before `delete_project`. Don't just reorder the async check — the delete happens inside the blocking task.
- **Test the escape, not just the rejection.** A test asserting `Err(...)` proves rejection but not confinement. At least one test should confirm that against the *unfixed* code a file lands outside the temp root (i.e. the vulnerability is real), so the fix's value is demonstrable per the repo's "your test is not valid if it passes without the fix" rule.
- **No `unwrap`/`expect`/`panic` on the reachable path.** These paths are remotely reachable; return `anyhow::Result` errors, don't panic. Canonicalization failures are errors to surface, not `unwrap`s.

## Codex log

## Claude review

## Verdict
