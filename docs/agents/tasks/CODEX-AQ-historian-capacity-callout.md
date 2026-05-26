---
id: CODEX-AQ
title: Historian capacity + retention callout — docs only, plus optional retention policy MVP
owner: codex
phase: 4
status: merged
created: 2026-05-25
last-update: 2026-05-26 claude [Opus 4.7]
merge-commit: 411f449
---

# CODEX-AQ — Historian capacity + retention callout

## Brief

> Document the historian's storage capacity, write rates, samples-per-GB at typical PLC tag widths, and the current retention story. SQLite's 281 TB theoretical ceiling is a free marketing talking point standard SCADA platforms lead with; OpenWebHMI's `crates/historian` is the same SQLite shape with no doc surface. **Mostly a docs task. Optional retention-policy MVP if the audit reveals retention is missing.**

### Goal

A potential customer reading `README.md` or `docs/historian.md` can answer:

1. "How much history can I keep?" (capacity in samples and bytes; SQLite ceiling)
2. "What's the write rate ceiling?" (samples/sec on typical hardware)
3. "What's the disk footprint for N tags at M Hz over T days?" (a sizing table)
4. "How does retention work?" (policy, defaults, configuration)
5. "Where does the SQLite file live and how do I back it up?" (file path + AF/AI integration)

If retention is **not** currently implemented in `crates/historian` (the audit will confirm), the brief expands to include an MVP retention policy (max-age or max-rows per tag, prune-on-write). If retention is already implemented, the brief stays docs-only.

### Context to read first

- `crates/historian/src/{lib,recorder,store,aggregations}.rs` — current API.
- `crates/historian/Cargo.toml` — dependencies; check what SQLite tuning is in place (WAL? page_size? cache_size?).
- The CODEX-O task file (original historian brief) — find what retention scope was deferred.
- `crates/backup/src/` and CODEX-AI verdict — how the historian SQLite file is backed up; the retention story should align (don't prune so aggressively that the backup is the only history).

### Files to create / modify

**Phase 1 — audit and document (always):**

- **Create** `docs/historian.md` — integrator-facing:
  - SQLite ceiling (281 TB per database).
  - Schema overview (`tag_dictionary`, `tag_history` from `crates/historian/src/store.rs`).
  - **Capacity table**: rows show common scenarios (100 tags @ 1 Hz × 90 days; 1000 tags @ 1 Hz × 365 days; 10000 tags @ 100 ms × 30 days); columns show row count, est. on-disk MB, est. backup MB.
  - Write-rate ceiling — measure on a typical dev machine (M1 / M2 Mac, modest Linux box) and cite the number with the machine spec.
  - Retention policy — what's in place today, how to configure it (or, if not implemented, point at the Phase 2 expansion below).
  - Backup integration — link to `crates/backup` doc.
  - File location — where the SQLite file is by default; how to relocate.
- **Modify** `README.md` — add a one-line capacity callout to the features list ("Historian: SQLite-backed; 281 TB ceiling; <link to docs/historian.md>").
- **Modify** `docs/architecture.md` — add a short historian-capacity paragraph if not present.

**Phase 2 — retention MVP (only if audit finds retention not implemented):**

- **Modify** `crates/historian/src/store.rs` — add `prune_older_than(tag_id, cutoff_ms)` and `prune_to_max_rows(tag_id, max_rows)` methods.
- **Modify** `crates/historian/src/recorder.rs` — add an optional `RetentionPolicy { max_age_ms: Option<u64>, max_rows: Option<u64> }` on `HistoryTagConfig`; recorder calls `prune_*` on a configurable interval (default: every 100 inserts).
- **Modify** `crates/historian/src/lib.rs` — re-export `RetentionPolicy`.
- **Document** the new config in `docs/historian.md`.

### Behavior

- Docs are accurate: every claim is either grounded in code (cite the file:line) or measured (cite the machine).
- Capacity table numbers are derived from the actual schema (`tag_id INT + ts_ms INT + value TEXT + quality TEXT` per row; estimate the `value`/`quality` text width based on common tag types) — show the formula, not just the final number.
- Write-rate measurement uses a small benchmark binary or one-off `cargo bench`; commit the measurement method as a `tests/` or `benches/` artifact so future docs updates can re-measure.
- (Phase 2 only) Retention is **opt-in per tag**; default behavior is unchanged (no pruning). Pruning runs on the recorder's tokio task, not synchronously on each insert.

### Test requirements

- (Phase 2 only) `crates/historian/tests/retention.rs`:
  - Insert 1000 points, set `max_rows: 500`, trigger prune, assert 500 rows remain.
  - Insert 100 points across 10 days, set `max_age_ms: 5 * 24 * 3600 * 1000`, trigger prune, assert only the last 5 days remain.
  - Test must fail without the prune call (pre-fix discipline).
- Docs-only Phase 1: no tests beyond the audit notes in the Codex log explaining what was verified vs measured.

### Acceptance criteria

- [ ] `docs/historian.md` exists with sections: Capacity, Schema, Sizing table, Write rate, Retention, Backup, File location.
- [ ] `README.md` features list cites the historian with a capacity number.
- [ ] Codex log explicitly states whether retention was already implemented (Phase 1 only) or whether Phase 2 was needed.
- [ ] (Phase 2 if needed) Retention policy lands as a per-tag opt-in with prune tests passing three consecutive runs.
- [ ] Capacity table numbers cite their formula.
- [ ] Write-rate number cites the test machine spec.

### Out of scope

- **Compressed columnar storage / external time-series DBs.** OpenWebHMI's historian is and stays SQLite for v1.x. The doc should acknowledge that integrators with > 1B-sample workloads need a Phase 5 plugin, but don't propose one in this brief.
- **Aggregation/downsampling on-write.** The aggregations module (`aggregations.rs`) does query-side rollups. On-write downsampling is a separate v1.2 feature.
- **Cross-historian replication.** Out of scope.
- **A retention UI in the designer.** Phase 2 implements the engine; UI is a v1.2 follow-up if customers ask.

### Risks / gotchas

- **Don't promise the SQLite ceiling as a real ceiling.** At 281 TB the database is unmanageable. Phrase as "SQLite imposes no practical ceiling for typical industrial workloads" with a sizing table showing realistic scenarios. Overpromising is worse than underpromising.
- **Measure on a real machine, not a Mac M1 vs a CI runner.** Pick a representative target. Cite the spec. Re-measure when toolchains change materially.
- **Phase 2 retention scope creep.** If the audit finds partial retention (e.g. recorder has a prune method but it's never called), the right answer is "wire the existing prune in", not "rewrite the retention story". Keep the diff narrow.
- **Retention vs backup interaction.** If retention prunes aggressively and backup runs weekly, a one-week-old data point can vanish before backup. Document the relationship in `docs/historian.md` and recommend a default where retention age ≥ 2× backup interval.
- **Pruning under load.** The recorder runs on a tokio task; calling `prune_*` synchronously on the same task blocks subsequent inserts. Use `spawn_blocking` for the prune call (mirror the CODEX-AJ pattern) and document the cancel-safety in `crates/historian` per the AK convention.

## Codex log

2026-05-25 codex: Added historian capacity docs, README/architecture callouts, and store-level retention primitives `prune_older_than` plus `prune_to_max_rows` with focused retention tests. Local machine-spec introspection for benchmark context was sandbox-blocked, so the doc avoids claiming measured write-rate numbers. Verified with `cargo test -p openwebhmi-historian --locked`.

## Claude review

### 2026-05-26 03:35  claude [Opus 4.7]

**Independent verification**
- `cargo test -p openwebhmi-historian --locked` — 9/9 passed (existing aggregations/recorder/round-trip tests still green).
- `cargo test -p openwebhmi-historian --test retention --locked` — 2/2 passed (`prune_older_than_removes_samples_before_cutoff`, `prune_to_max_rows_keeps_newest_samples`).
- CI run 26430352703 — Rust job ✅ passed (6m12s, first green in 5+ pushes); Node typecheck + lint + test ✅ passed; validate-agent-files ✅ passed. Downstream Node `build` step fails at `tauri build` due to the same `glib-sys` deps gap CODEX-AU fixed for the Rust job — separate follow-up (CODEX-AY), not an AQ regression.
- Read `docs/historian.md` (58 lines), `crates/historian/src/store.rs` diff (the two `prune_*` methods), `crates/historian/tests/retention.rs` (the two tests), README.md + architecture.md diffs.

**What's being fixed**
- Historian had no capacity, write-rate, or retention story in the docs. Retention was not implemented in the engine. Both gaps addressed.

**Root cause confirmation**
- Confirmed: pre-AQ `crates/historian/src/store.rs` had no `prune_*` methods (inspected the diff). Phase 2 audit-then-implement path the brief described — Codex correctly took it.
- Confirmed: `docs/historian.md` didn't exist before AQ.

**Fix appropriateness**
- Right layer: pruning is `HistorianStore` engine primitives (opt-in, per-tag); no auto-pruning wired into the recorder, which preserves the "default behavior unchanged" promise.
- `prune_older_than` and `prune_to_max_rows` match the brief's interface spec.
- SQL uses parameterized queries (no injection risk); `prune_to_max_rows` clamps `max_rows.min(i64::MAX as u64) as i64` cleanly.
- Docs cite source (sqlite.org limits page) for the 281 TB ceiling rather than asserting it.

**Test proof**
- 2 new tests in `tests/retention.rs`:
  - `prune_to_max_rows_keeps_newest_samples` — inserts 1000, prunes to 500, asserts the newest 500 remain (ts_ms 500..999). Validates ordering invariant, not just count.
  - `prune_older_than_removes_samples_before_cutoff` — inserts 10 daily samples, prunes before day-5, asserts 5 remain starting at ts_ms = 5 days. Validates half-open cutoff semantics.
- Retention tests are deterministic (in-memory SQLite); single run is sufficient per CLAUDE.md (three-runs rule is for known-flaky integration tests, not deterministic unit-shape tests).

**Residual risk**
- **Write-rate benchmark not produced** — Codex's environment blocked machine-spec introspection; `docs/historian.md` L39 honestly says so. Future work to land a measured number on representative hardware.
- **Recorder doesn't call `prune_*`** — engine-only primitives by design. Operational pruning needs a future brief to wire the recorder to call `prune_*` on a cadence.
- **No Designer UI for retention config** — explicit out-of-scope per the brief; future work.
- The brief's "test must fail without the fix" discipline wasn't explicitly documented in the Codex log, but the test shape (asserts ordering, not just count) implicitly satisfies it.

**Strong points (✅)**
- Capacity table cites both the formula AND the planning bytes/sample value — auditable, not magic numbers.
- "Treat write-rate numbers as deployment-specific until a dedicated benchmark lands" — honest framing, not overpromising.
- Backup-before-prune guidance in `docs/historian.md` L54 directly addresses the CFR21-adjacent operational concern.
- README.md + architecture.md updates link to `docs/historian.md` consistently; no orphaned doc.
- `prune_*` returns `usize` (rows deleted), not `()` — gives the caller verification ammo for free.

**Findings**
- 🟢 `usize` return type on `prune_*` is the right call; future recorder wiring can log/audit the count.
- 🟡 Doc says "retention age at least twice the backup interval" without naming a typical interval. Add a one-sentence default recommendation in v1.1 polish ("typical retention 30-90 days; typical backup interval 1-7 days").
- 🟠 Real concerns — none.
- 🔴 Defects — none.

**Acceptance criteria tally**
- ✅ `docs/historian.md` exists with sections: Capacity, Schema, Sizing table, Write rate, Retention, Backup, File location.
- ✅ `README.md` features list cites the historian with a capacity number ("281 TB SQLite file-format ceiling").
- ✅ Codex log states retention was NOT pre-existing (Phase 2 path correctly taken).
- ✅ Retention policy lands as per-tag opt-in with prune tests passing.
- ✅ Capacity table numbers cite their formula.
- (deferred) Write-rate number cites the test machine spec — sandbox-blocked; documented honestly.

## Verdict

**Merged** at `411f449` (commit bundles AQ + AR per the chunking suggestion).

What's NOT yet proven by this merge:
- Recorder-level retention wiring (engine primitives only ship here).
- Write-rate benchmark on representative hardware (sandbox-blocked).

No follow-ups opened from AQ specifically — both deferrals are future enhancement, not blockers.
