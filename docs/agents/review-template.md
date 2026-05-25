# Claude Review Template

Use this template for new `## Claude review` entries. Keep the review concise, cite files or symbols for every code claim, and write `not proven` when a claim was not independently verified.

The structured shape exists so every review hits the same nine questions in the same order. A reviewer who can't answer one of them honestly is the reviewer most likely to miss the issue that section would have surfaced.

## Skeleton

```md
### YYYY-MM-DD HH:MM  claude [model]

**Independent verification**
- `<command>` — pass/fail and one-line result

**What's being fixed**
- One line restating the bug or feature.

**Root cause confirmation**
- Confirmed/not investigated, with file:line or symbol citations.

**Fix appropriateness**
- Judgment on whether the change lands at the right layer, with citations.

**Test proof**
- Tests added or rerun, plus uncovered edge cases. Note flaky-test runs (three consecutive).

**Residual risk**
- Known limitations, not-proven claims, hardware gaps, manual-smoke deferrals, future follow-ups.

**Strong points (✅)**
- Citation-anchored positives worth preserving.

**Findings**
- 🟢 factual note (no action; context for the reader)
- 🟡 polish, non-blocking (v1.1 candidate; not a closeout blocker)
- 🟠 real concern, blocks merge unless fixed (or tracked as a follow-up task before merge)
- 🔴 defect, rejects (fundamental contract failure; brief amended, status returns to `open`)

**Acceptance criteria tally**
- ✅ Criterion copied from the brief — result
- 🟡 partially Criterion copied from the brief — missing piece
- ❌ Criterion copied from the brief — failed
- (deferred) Criterion copied from the brief — explicit owner/timing
```

## Why each section exists

- **Independent verification** — proves the reviewer ran the matrix and didn't take Codex's word. Lists *commands actually run*, not "tests pass". One line of result each.
- **What's being fixed** — single sentence. Forces the reviewer to articulate the contract before judging the implementation. If you can't write this line, you can't review the task.
- **Root cause confirmation** — names the file:line of the actual cause, separate from the symptom. Bugs that present in module A but originate in module B are the ones that come back.
- **Fix appropriateness** — answers "did the change land at the right layer?" The fix might work and still be wrong if it patches a symptom one layer above the real cause.
- **Test proof** — what the tests actually cover. Edge cases the test set doesn't reach get named here, not glossed.
- **Residual risk** — the section that catches everything else: hardware not exercised, manual-smoke deferred, environment mismatches, partial scope. The honesty contract from `CLAUDE.md` runs through this section.
- **Strong points / Findings / Acceptance tally** — the merge-or-not call, broken into the three views a future reader needs: what works, what's left, did the brief get satisfied.

## Worked example

The shape, with realistic OpenWebHMI commands and a plausible Phase 4 component-library task.

### 2026-05-22 14:00  claude [Opus 4.7]

**Independent verification**
- `cargo fmt --check` — clean.
- `cargo clippy --workspace --all-targets --all-features --locked -- -D warnings` — clean.
- `cargo test --workspace --all-features --locked` — 412 passed, 0 failed.
- `pnpm -r typecheck` — clean.
- `pnpm -r test` — 187 passed, 0 failed.
- `pnpm --filter component-library build` — succeeded; bundle size unchanged.
- Three consecutive runs of `cargo test -p driver-mqtt --test integration` — 3/3 green (known-flaky surface).
- Manual smoke of the 19-step designer checklist — **deferred to maintainer**.

**What's being fixed**
- `Slider` / `Dropdown` / `ToggleSwitch` write to a hardcoded `"value"` target instead of the bound tag path, breaking write-back on any binding that isn't named `value`.

**Root cause confirmation**
- Confirmed: `packages/component-library/src/Slider.tsx:74` and the matching lines in `Dropdown.tsx:81`, `ToggleSwitch.tsx:62` send `{ target: "value" }` literals.
- The correct pattern already exists on `NumericInput.tsx:91` — accepts an optional `tagPath?: string` prop with the binding path as fallback. See [`docs/agents/notes/binding-write-asymmetry.md`](notes/binding-write-asymmetry.md) for the underlying architectural gap.

**Fix appropriateness**
- Appropriate layer: the bug is component-side write dispatch, not in the binding resolver. Mirrors the existing `NumericInput` fallback shape exactly.
- Considered fixing in the binding system instead (exposing the bound path for write-back). That's the v1.1 architectural fix; doing it now would expand scope past the brief.

**Test proof**
- Vitest cases added in `Slider.test.tsx`, `Dropdown.test.tsx`, `ToggleSwitch.test.tsx` cover: explicit `tagPath` prop wins, binding-path fallback when prop absent, no-write when neither set.
- Pre-fix the new tests fail (`"value"` literal asserted vs the bound path) — confirmed by reverting the impl and re-running.

**Residual risk**
- The 19-step designer manual-smoke checklist (`apps/designer/README.md`) was not run in this environment; that's the maintainer's gate before the v0.3.0 tag.
- The architectural fix (binding system exposes write-back path) is deferred and tracked in [`docs/agents/notes/binding-write-asymmetry.md`](notes/binding-write-asymmetry.md).

**Strong points (✅)**
- Mirrors the existing `NumericInput` pattern instead of inventing a new one — neighbor-copy discipline.
- New tests fail against the pre-fix code (verified by revert), meeting the "regression test must catch the bug" bar.
- Brief gap owned in the verdict: the brief didn't mention the architectural-fix deferral, this review surfaces it.

**Findings**
- 🟢 The `NumericInput` precedent is documented in [`docs/agents/notes/binding-write-asymmetry.md`](notes/binding-write-asymmetry.md); future write-back components should consult it.
- 🟡 The optional-prop shape (`tagPath?: string`) repeats across four components. A shared `useWriteTarget()` hook would dedupe; tracked for v1.1.
- 🟠 Real concerns — none.
- 🔴 Defects — none.

**Acceptance criteria tally**
- ✅ Slider/Dropdown/ToggleSwitch accept an optional `tagPath` prop matching NumericInput.
- ✅ Falls back to the binding path when the prop is absent.
- ✅ Vitest coverage for all three components.
- ✅ Existing tests still pass.
- (deferred) Designer manual smoke — maintainer to run before v0.3.0 tag.
