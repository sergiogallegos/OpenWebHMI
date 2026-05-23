---
name: openwebhmi-merge
description: Review a submitted Codex task (status submitted), run the test matrix independently, write the verdict, and merge or reject. Updates board + log + task frontmatter in one commit. Use when the maintainer says "review CODEX-XX" or when a task file's status flips to submitted.
---

# openwebhmi-merge

The Codex-side review/merge lifecycle for OpenWebHMI tasks. This skill packages the procedure that previously lived inline in `AGENTS.md` so it can be invoked directly when the maintainer hands off a submitted task.

## When to use

- The maintainer says "review CODEX-XX", "merge CODEX-XX", or "Codex submitted CODEX-XX".
- A `docs/agents/tasks/CODEX-*.md` file has frontmatter `status: submitted` that you haven't reviewed yet.

Do **not** use this skill to author briefs (that's `openwebhmi-task-brief`, when it exists) or to ship code yourself (that's Codex).

## Read order

Don't re-derive state. The agent docs are the durable handoff.

1. `git pull` — durable state is on origin.
2. `docs/agents/board.md` — find the task row to confirm phase and current status.
3. The task file's **frontmatter** (`status:` is authoritative).
4. The task file's **Brief** (what was asked for, what's out of scope).
5. The task file's **Codex log** (what was done, what was deferred, what was questioned).
6. The diff: `git log --oneline <main>..HEAD` (if Codex worked on a branch) or `git show <commit>` for the merge candidate.
7. Changed files in priority order: impl module → tests → wiki entry (if the brief asked for one) → docs update (if user-visible).

`board.md` and `log.md` are summaries. The task file frontmatter is the source of truth — if `board.md` and frontmatter disagree, fix `board.md`.

## Test matrix (run independently)

Don't trust Codex's verification claim. Run the matrix yourself:

```
cargo build --workspace --all-features --locked
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test --workspace --all-features --locked
cargo fmt --check
pnpm -r typecheck
pnpm -r test
```

Task-specific extras as applicable:

- `pnpm --filter component-library build` for component-library changes.
- `pnpm --filter designer build:vite` for designer changes.
- Three consecutive runs for known-flaky integration tests; one green run is not enough.
- `pnpm --filter designer build` (Tauri) only when the brief calls for it — it's slow.
- The designer manual-smoke checklist (`apps/designer/README.md`) is a separate maintainer-run gate; do not claim it passed unless the maintainer says so.

Record what you ran in the `## Codex review` section. Environment mismatches between Codex's run and yours (Node version drift, missing system deps) get flagged honestly, not glossed.

## Verdict shape

Write a `## Codex review` section with three parts:

1. **Strong points** — what landed well. Lead with the load-bearing items (the thing the brief asked for that's now done correctly). Mark with `✅`.
2. **Findings** — issues found. Use the project convention:
   - `🟡 polish` — cosmetic, can be a v1.1 item. Don't undersell load-bearing items as polish; if a "polish" item actually breaks demo-HMI headline behavior, it's a closeout blocker.
   - `🟠 real concern` — needs fixing in this PR or a follow-up task.
3. **Acceptance-criteria tally** — go through the brief's acceptance criteria one by one, mark met / not-met / partially.

Then write the `## Verdict` section:

- Disposition: **Merged**, **Merged with explicit validation gate**, or **Rejected**.
- Merge commit hash (backfilled in a follow-up commit if not known at write time).
- What's NOT yet proven, named explicitly. If hardware validation or manual smoke is deferred, say so by name.
- Any follow-up task ids opened (e.g., "tracked as CODEX-AD").

## Three-place status update

Status changes commit together. Don't split them.

1. **Task file frontmatter**: set `status: merged` (or `rejected`), update `last-update: YYYY-MM-DD Codex`.
2. **`board.md`**: move the row from the phase table to the `## Done` section. Record the merge commit. Add a one-liner describing what landed.
3. **`docs/agents/log.md`**: append `YYYY-MM-DD HH:MM Codex CODEX-XX merged at <commit>` (newest at bottom, never edit prior entries).

If the merge commit hash isn't known at write time (you're about to commit the verdict, then commit the impl), backfill the hash into the verdict and `board.md` in a follow-up commit. Note the backfill in the commit message.

## When to reject vs fix-during-merge

**Reject** when the implementation is fundamentally wrong — the submitted code can't fulfill the brief's contract even when the tests pass. The canonical example is **CODEX-Z's first submission**: imported the `ads` crate only as a dead-code marker function and exchanged JSON-line frames with a stub sim. Tests passed; the driver didn't speak ADS and couldn't talk to a real Beckhoff PLC. Rejected with rationale, brief amended in-place, status returned to `open`.

Rejection rationale:

- Write a `## Verdict` section that names the fundamental issue.
- Amend the `## Brief` to address the gap (this is a Codex edit; the Brief is normally Codex-immutable but Codex owns brief corrections).
- Set frontmatter `status: open` (back to the start of the lifecycle).
- Update `board.md` and append to `log.md` with `rejected — brief amended`.

**Fix during merge** when the bug is mechanical and mirrors an existing pattern. Examples:

- CODEX-U's `vite-plugin-monaco-editor` Node 22+ incompatibility — gated the plugin to `command === 'build'` (5-line `vite.config.ts` change).
- CODEX-AB's Slider/Dropdown/ToggleSwitch hardcoded `"value"` write target — added `tagPath?: string` props matching NumericInput's existing fallback pattern.

Document Codex-applied fixes transparently in the verdict. Don't silently rewrite Codex's submission.

## Honesty checklist (before committing the verdict)

- [ ] Strong points are accurate (you actually verified them, not just took Codex's word).
- [ ] Brief errors, if any, are owned by name. See CODEX-AH, CODEX-AJ, CODEX-AL for shape.
- [ ] Environment mismatches between Codex's run and the local run are noted.
- [ ] Deferred validation (hardware, manual smoke) is named explicitly in the verdict, not glossed.
- [ ] Follow-up tasks opened for `🟠 real concern` items that aren't fixed in this PR.
- [ ] Manual-smoke gate is **not** claimed as passed unless the maintainer confirmed it.

## Commit & push

The verdict + board + log update commit together as the merge commit (or as a separate review commit if you're not the one merging the impl).

Push only when the maintainer explicitly asks ("ship it", "push the merge"), or when an unambiguous convention requires it (backfilling a merge ref). Otherwise leave it as a local commit and surface it for review. See `docs/agents/README.md` for the full push policy.

## See also

- `AGENTS.md` — the broader Codex operating procedure.
- `AGENTS.md` (root) — codebase-wide code/test/dependency rules used during review.
- `VISION.md` — the won't-merge contract; rejected submissions usually trip a `VISION.md` line.
- `docs/agents/README.md` — task lifecycle and status flow.
- `docs/agents/board.md` — current task state.
