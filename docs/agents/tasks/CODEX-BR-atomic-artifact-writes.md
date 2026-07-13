---
id: CODEX-BR
title: Atomic artifact writes — real rename atomicity, fsync, non-colliding temp files
owner: codex
phase: 4
status: open
created: 2026-07-12
last-update: 2026-07-12 claude [Fable 5]
---

# CODEX-BR — Atomic artifact writes

## Brief

> `atomic_write` in `crates/project-store/src/store.rs` (lines 503-517) removes the destination file **before** renaming the temp over it, defeating the atomic replace that `fs::rename` already provides on Unix. A crash in the window between the remove and the rename loses the artifact entirely — the `.tmp` survives but the committed file is gone. The temp path is deterministic (`path.with_extension("…tmp")`), so two concurrent saves of the same artifact race on the same `.tmp` and produce a torn write. There is no `fsync`, so the "durable" claim doesn't survive power loss. Make the write genuinely atomic and crash-safe: unique temp, `fsync` the temp, rename directly over the destination (no pre-remove), then `fsync` the containing directory.

### Goal

`atomic_write` guarantees that a reader of `path` always observes either the complete previous contents or the complete new contents — never a missing file, never a partial write — even across a crash or concurrent save of the same artifact. The write is durable to disk before the function returns.

### Context to read first

- `crates/project-store/src/store.rs:503-517` — the current `atomic_write`. Note the `if path.exists() { let _ = fs::remove_file(path); }` on lines 512-514 that must be deleted, and the deterministic `tmp` path on lines 504-510.
- `crates/project-store/src/store.rs:179` — the sole caller (`ProjectStore::save`-path), for the concurrency context: multiple saves of the same project can run concurrently.
- `crates/project-store/tests/store.rs` — the **external** integration test file (not a `#[cfg(test)] mod tests` inside `store.rs`). Extend these tests here:
  - `concurrent_saves_do_not_lose_version_updates` (line 177) — the existing concurrency test.
  - `stray_tmp_file_does_not_replace_committed_artifact` (line 199) — writes a `.toml.tmp` alongside the artifact and asserts a load still returns the committed file. A unique-temp scheme must not regress this.
- `docs/agents/notes/toolchain-drift.md` — environment-mismatch discipline for platform-specific fs behavior.

### Files to create / modify

1. **Modify** `crates/project-store/src/store.rs` — rewrite `atomic_write`:
   - Write to a **unique** temp path in the same directory as `path` (same filesystem, so the rename stays atomic). Uniqueness must survive two concurrent same-artifact saves: include a per-write nonce (e.g. process id plus an atomic counter, or a random suffix), or adopt a vetted `tempfile`-style approach that creates an `O_EXCL` temp. Keep the file in the destination directory — `std::env::temp_dir()` is a different filesystem and breaks `rename` atomicity.
   - `fsync` the temp file (`File::sync_all`) after writing its bytes, before the rename.
   - `fs::rename(&tmp, path)` **directly over** the destination. Delete the `if path.exists() { remove_file }` block — `rename` replaces atomically on Unix; the pre-remove is the bug.
   - `fsync` the containing directory after the rename so the rename itself is durable (open the parent dir and `sync_all`). On platforms where directory fsync is unsupported, tolerate the error rather than fail the write.
   - On any error, best-effort remove the unique temp so a failed write doesn't leak a stray `.tmp`.
   - No `unwrap()`/`expect()`/`panic!` on any non-test path; propagate via the existing `anyhow::Result` return.

2. **Modify** `crates/project-store/tests/store.rs` — extend the two existing tests listed above (add to them; do not create a new test file per repo discipline):
   - A concurrent-same-artifact test: spawn N tasks that all save the **same** artifact id concurrently; after all complete, loading the artifact returns a well-formed, fully-parseable body (never a torn/partial file), and no `.tmp` remains in the directory.
   - A crash-simulation test: exercise the write path such that a temp exists but the rename has not happened (e.g. drop/interrupt before rename, or directly assert the invariant by leaving a unique temp on disk), then assert a load returns **either** the old committed contents **or** the new — never a missing artifact.

### Behavior

- Reader observes old-or-new, never missing, never partial.
- Two concurrent saves of the same artifact do not corrupt the destination or collide on a shared temp path.
- A leftover unique temp from a failed/crashed write never replaces or shadows the committed artifact (the existing `stray_tmp` guarantee must still hold — a unique-suffixed temp is easier to keep out of the load glob, not harder).
- Bytes are durable (temp fsync + rename + dir fsync) before return.

### Test requirements

- The crash-simulation regression test must **fail against the pre-fix code** (the pre-remove path can leave the destination missing). Confirm it fails before the fix, passes after. Note the before/after result in the Codex log.
- No `sleep()` / wall-clock waits — synchronize concurrent tasks with `JoinHandle::await` / channels, not timers.
- No hardcoded ports (not relevant here, but the discipline stands for any listener).
- `cargo test -p openwebhmi-project-store` green; full matrix (`cargo build`, `cargo clippy --workspace --all-targets --all-features --locked -- -D warnings`, `cargo test --workspace --all-features --locked`, `cargo doc --workspace --no-deps`) clean.

### Acceptance criteria

- [ ] `atomic_write` writes to a unique per-write temp in the destination directory.
- [ ] The `if path.exists() { remove_file }` pre-remove is deleted; `fs::rename` replaces atomically with no prior remove.
- [ ] Temp file is `fsync`ed before rename; the containing directory is `fsync`ed after rename (dir-fsync failure tolerated where unsupported).
- [ ] Failed writes best-effort clean up their unique temp; no stray `.tmp` leaks on the happy path.
- [ ] Concurrent-same-artifact test proves no corruption and no shared-temp collision.
- [ ] Crash-simulation regression test fails pre-fix, passes post-fix (before/after noted in Codex log).
- [ ] Existing `stray_tmp_file_does_not_replace_committed_artifact` still passes unchanged in intent.
- [ ] No `unwrap`/`expect`/`panic!` on the production path; full validation matrix clean.

### Out of scope

- Changing the artifact serialization format or the `.toml` layout.
- Introducing a heavy transactional store or WAL — this is a single-file atomic-replace fix.
- Touching callers other than `atomic_write` itself; the caller at line 179 keeps its signature.
- Windows-specific atomic-replace semantics beyond making the Unix path correct and not regressing on Windows compile (document any `#[cfg]` split with a WHY comment).

### Risks / gotchas

- **Same-filesystem temp is load-bearing.** `rename` is only atomic within one filesystem; a temp under `std::env::temp_dir()` can be on a different mount and silently degrade to copy-and-delete. Keep the temp beside the destination.
- **Directory fsync portability.** `File::open(dir).sync_all()` behaves differently across platforms; some return `EINVAL`. Tolerate the error (log at most) rather than fail the artifact write — the rename already committed.
- **Don't reintroduce a deterministic temp name.** The whole point is that concurrent same-artifact saves must not share a temp path. If reaching for `tempfile`, keep the `=`-pinned dependency discipline (per CLAUDE.md) — a new dep needs a bounded `Cargo.lock` diff; prefer a small in-crate nonce if it avoids a new dependency.
- **The stray-tmp load guard.** The existing test asserts a bad `.tmp` doesn't get loaded. A random-suffixed temp is naturally excluded from any `*.toml` load glob; confirm the load path filters temps by more than an exact `.tmp` extension if it currently relies on the deterministic name.
- **`if let` chains over nested `if let`** and top-level imports only, per CLAUDE.md Rust discipline.

## Codex log

## Claude review

## Verdict
