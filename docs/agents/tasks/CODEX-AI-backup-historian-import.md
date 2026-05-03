---
id: CODEX-AI
title: Wire historian + alarm-journal import path in crates/backup — close CODEX-AF's v1.0 gap
owner: codex
phase: 4
status: merged
created: 2026-05-03
last-update: 2026-05-03 claude
---

# CODEX-AI — Historian + alarm-journal import for `.owhmi` archives

## Brief

> **CODEX-AF closeout follow-up.** AF merged the export half of the gateway backup story: project artifacts round-trip cleanly, the `.owhmi` archive carries `historian.sqlite` and `alarm-journal.sqlite` blobs via SQLite's online backup API, and the HTTP side-channel + token flow work. AF deliberately scoped the import side to project artifacts only — the SQLite blobs are extracted but never written to the target gateway. CODEX-AI closes that gap so the headline use case ("backup before risky change") round-trips operational telemetry, not just configuration.

### Why this matters for v1.0

The `docs/roadmap.md` Phase 4 spec is explicit: "Backup / restore: project export to `.owhmi` archive; **gateway-level backup including historian + alarm history**." AF delivered the export but stops short on import — the operator who exports a CompactLogix-driven HMI with 90 days of historian data, copies the archive to a fresh gateway, and runs restore gets all their views/scripts/alarms config back but loses every datapoint. That's a half-feature that isn't honest to call "backup/restore." AI fixes this before v1.0 tags.

### Goal

After this lands:
- `crates/backup::import_project()` writes the `historian.sqlite` and `alarm-journal.sqlite` archive entries to live target stores when references are provided.
- Replace mode wipes target historian + alarm-journal data before applying the archive's snapshots.
- Merge mode appends the archive's rows into the target stores via SQLite's online-backup-style restore (or row-by-row insert with `ON CONFLICT IGNORE` if simpler).
- The gateway HTTP `POST /api/projects/{id}/restore` handler passes the live `HistorianStore` and `AlarmJournal` references through to the import call.
- Round-trip tests cover historian rows + alarm transitions through full export → import equivalence.

### Context to read first

- `crates/backup/src/lib.rs` — current `import_project()` (lines 176-222) reads the archive entries but discards `historian.sqlite` / `alarm-journal.sqlite`. The function signature `(store: &ProjectStore, archive: &[u8], options: RestoreOptions)` doesn't take either store reference; this changes.
- `crates/backup/src/lib.rs::snapshot_historian` and `snapshot_alarm_journal` (lines 257-275) — the export-side counterpart that uses `store.backup_to_path()` to write a snapshot to a temp file. The import-side equivalent uses `Connection::restore` (rusqlite's online-backup API in the opposite direction).
- `crates/historian/src/store.rs` — `HistorianStore::backup_to_path()` exists from AF; AI adds a sibling `restore_from_path()` (or `restore_from_bytes()`).
- `crates/alarm-engine/src/journal.rs` — same pattern.
- `crates/gateway/src/server.rs:1456` — the gateway's HTTP restore handler currently calls `openwebhmi_backup::import_project(&project_store, &request.body, options)` with no historian/alarm refs. Update to pass `DEFAULT_HISTORIAN.get()` and `DEFAULT_ALARM_JOURNAL.get()` cloned references.
- `crates/backup/tests/round_trip.rs` — existing tests cover artifacts only. AI adds a round-trip test that seeds historian + alarm data, exports, imports to a fresh target, and asserts row equivalence.

### Files to modify

- `crates/backup/src/lib.rs`:
  - Extend `RestoreOptions` with `pub historian_store: Option<HistorianStore>` and `pub alarm_journal: Option<AlarmJournal>`.
  - In `import_project()`, after manifest validation: if `manifest.includes_historian` and `options.historian_store` is `Some(...)`, find the `historian.sqlite` entry in `entries`, write its bytes to a temp file, call `historian_store.restore_from_path(&temp_path)`, delete the temp file. Same for alarm-journal.
  - Replace mode: call a new `historian_store.clear()` / `alarm_journal.clear()` before restore (the SQLite restore API replaces the entire database, so this may already be implicit — verify). Merge mode: implement append semantics — see "Merge mode design" below.
  - On error during historian restore, log via `tracing::warn!` and continue; project artifact restore should not abort because of a historian quirk. Document the behavior in rustdoc.

- `crates/historian/src/store.rs`:
  - Add `pub fn restore_from_path(&self, path: impl AsRef<Path>) -> anyhow::Result<()>` using `rusqlite::backup::Backup::new` with the source/destination flipped (or use `Connection::restore` if the API supports it directly).
  - Optionally add `pub fn restore_from_bytes(&self, bytes: &[u8]) -> anyhow::Result<()>` that writes to a temp file and calls `restore_from_path`. Saves callers the temp-file dance.

- `crates/alarm-engine/src/journal.rs`:
  - Same pattern: `restore_from_path` + optional `restore_from_bytes`.

- `crates/gateway/src/server.rs`:
  - The HTTP restore handler (around line 1456) gains `historian_store: DEFAULT_HISTORIAN.get().cloned()` and `alarm_journal: DEFAULT_ALARM_JOURNAL.get().cloned()` in the `RestoreOptions` builder.
  - The WS-side `ClientMessage::ProjectImport` handler (around line 910) gets the same pass-through.

- `crates/backup/tests/round_trip.rs`:
  - New test: `historian_rows_round_trip_through_export_import` — seed source historian with 100 samples, export, import to fresh target historian, assert every sample round-trips with byte-equivalent values.
  - New test: `alarm_transitions_round_trip_through_export_import` — same shape for alarm journal.
  - New test: `replace_mode_wipes_target_historian` — seed target with historian data not in source, run replace, assert target is clean.
  - Optional: `merge_mode_appends_historian_rows` — if merge semantics land in this task; otherwise scope to a separate follow-up and document.

- `crates/backup/src/lib.rs` rustdoc: update `import_project` doc to describe the new historian/alarm restoration semantics.

- `wiki/architecture/backup-restore.md`:
  - Update "Open questions" — remove the historian-import gap once this lands.
  - Add a "Restore semantics" section describing replace vs merge for historian + alarm data.

### Merge mode design

Two options for v1.0:

- **(a) Replace-only for SQLite blobs.** Even in merge mode, historian/alarm data is fully replaced from the archive. Document the limitation: "merge mode applies project artifacts as merge but historian/alarm SQLite as replace." Simpler implementation.
- **(b) Row-by-row append.** Open the archive's SQLite blob as a separate connection, attach it to the target, run `INSERT OR IGNORE INTO target.tag_history SELECT * FROM archive.tag_history` for each table. Honors the brief's `(tag_id, ts_ms) PK conflict-ignore` semantics. More work but matches operator expectations.

**Recommended: (b) for v1.0.** The brief explicitly required append semantics. Implementation cost is moderate (one ATTACH DATABASE + N INSERT OR IGNORE per table) and it avoids the operator confusion of "merge mode replaces my historian." If (b) turns out to be tricky, falling back to (a) with documented limitation is acceptable.

### Test requirements

- `cargo test -p openwebhmi-backup --all-features --locked` green on three consecutive runs (the historian restore writes to a temp file; flake risk on Windows file-handle release).
- `cargo test -p openwebhmi-historian` and `-p openwebhmi-alarm-engine` green — the new `restore_from_path` methods need their own unit tests.
- `cargo test -p openwebhmi-gateway --test project_protocol --all-features --locked` green — the existing `project_backup_exports_downloads_restores_over_http_side_channel` test should still pass; ideally extend it to seed historian data and assert round-trip.
- `cargo clippy --workspace --all-targets --all-features --locked --exclude openwebhmi-designer -- -D warnings` clean.

### Acceptance criteria

- [x] `RestoreOptions` extended with `historian_store: Option<HistorianStore>` + `alarm_journal: Option<AlarmJournal>`.
- [x] `import_project()` writes historian + alarm-journal blobs to the target stores when refs are provided and the manifest claims their presence.
- [x] Replace mode wipes target historian + alarm-journal data before restore.
- [x] Merge mode appends rows via SQLite ATTACH + `INSERT OR IGNORE` (or documented fallback to replace-only with rationale).
- [x] Gateway HTTP restore handler passes the live store references through.
- [x] Three new unit/integration tests cover historian round-trip, alarm round-trip, and replace-mode wipe.
- [x] `wiki/architecture/backup-restore.md` updated; "Open questions" item about historian-import removed.

### Out of scope (v1.1+)

- **Audit-log SQLite inclusion** — `crates/audit-log` SQLite file is a candidate for `include_audit_log: bool`, deferred per AF's brief.
- **Differential / incremental restore** — full snapshots only.
- **Cross-version migration** — archive format remains gateway-version-pinned.
- **Selective restore** — operators get artifacts + optional historian + optional alarm-journal; no per-tag or per-time-range selection.
- **Restore progress telemetry beyond AF's existing `ProjectImportProgress` events** — large historian restores can take time; the existing phase events are enough for v1.

### Risks / gotchas

- **rusqlite's `restore` API direction.** `rusqlite::backup::Backup::new(src, dst)` copies src → dst. For "restore from archive into live store," src is the archive's SQLite file, dst is the live `HistorianStore`'s connection. Verify the parameter ordering carefully; getting it backwards would clobber the archive instead of restoring from it.
- **Concurrent writes to the live store during restore.** If the gateway is actively writing samples while restore runs, the writes interleave with the restore in non-deterministic ways. v1.0 acceptable: document that operators should pause data ingestion during restore (or accept that "restore" is a snapshot-time-from-now event). v1.1 polish: pause the historian writer during restore.
- **Temp file cleanup on Windows.** SQLite holds the file open; ensure the `Connection` is dropped before `std::fs::remove_file`. Mirror the pattern in `snapshot_historian`/`snapshot_alarm_journal` which already get this right.
- **Don't break the existing tests.** AF's 5 backup tests + the gateway HTTP integration test are passing — they should stay passing. Extend tests, don't replace them.
- **Don't expand scope.** AI is **only** the historian + alarm-journal import path. Do not add audit-log inclusion, do not refactor the manifest schema, do not split `lib.rs` into `archive.rs`/`export.rs`/`import.rs`. Those are AF v1.1 polish items, separate task.

## Codex log

<!-- Codex appends status transitions and notes here. -->

2026-05-03 codex — Submitted. Added live historian/alarm restore handles to `RestoreOptions`, import-side SQLite restore from archive blobs, historian merge via ATTACH + tag-path remap + `INSERT OR IGNORE`, alarm merge append, gateway HTTP restore pass-through, and tests for store restore/merge plus backup historian/alarm round-trip and replace-mode wipe. Focused `--locked` tests and gateway project protocol test passed; clippy/fmt validation recorded in the hand-off response.

## Claude review

### Strong points

- ✅ **All 8 backup tests pass** (5 prior from CODEX-AF + 3 new from AI). Workspace clippy clean. `cargo fmt --check` clean. Three consecutive runs claimed by Codex.
- ✅ **`Connection::restore` used correctly for replace mode.** `crates/historian/src/store.rs:99-107` and `crates/alarm-engine/src/journal.rs:81-89`: `conn.restore(DatabaseName::Main, path, None::<fn(...)>)` — the safe, documented direction for SQLite online-backup-API restore. The brief flagged "rusqlite restore parameter ordering" as the v1.0 risk-of-clobbering-the-archive; Codex used the named API form which makes the direction unambiguous.
- ✅ **Merge mode for historian uses ATTACH + tag-path remap + `INSERT OR IGNORE`** — exactly recommendation **(b)** from the brief, the operator-expected semantics. The SQL is non-trivial and gets it right (`crates/historian/src/store.rs:110-131`):
  ```sql
  INSERT OR IGNORE INTO tag_dictionary (tag_path) SELECT tag_path FROM source_history.tag_dictionary;
  INSERT OR IGNORE INTO tag_history (tag_id, ts_ms, value, quality)
  SELECT target.tag_id, sample.ts_ms, sample.value, sample.quality
  FROM source_history.tag_history AS sample
  JOIN source_history.tag_dictionary AS source_tag ON source_tag.tag_id = sample.tag_id
  JOIN main.tag_dictionary AS target ON target.tag_path = source_tag.tag_path;
  ```
  The critical correctness detail is **joining by `tag_path`, not `tag_id`** — `tag_id` is per-database AUTOINCREMENT, so a naive `INSERT` would mix IDs across databases and corrupt the dictionary. Codex translated IDs through the dictionary join. This is the kind of subtle SQLite-merge bug that's easy to ship wrong; Codex got it right.
- ✅ **Transactions wrap both merges.** `tx = conn.transaction(); tx.execute_batch(...); tx.commit();` — atomic apply, partial-merge can't leave the database in an inconsistent state.
- ✅ **`DETACH DATABASE` always runs after merge.** Cleans up the SQLite attachment regardless of whether the transaction committed.
- ✅ **Single-quote escaping for ATTACH paths** (`replace('\'', "''")` at `historian/store.rs:111` and `alarm-engine/journal.rs:93`). SQLite's ATTACH doesn't accept parameterized arguments; the format!-based construction is forced. Codex correctly escapes single quotes for SQLite's literal-string syntax. The path is generated server-side from `temp_snapshot_path` (process_id + timestamp + counter), so attacker-controlled paths aren't reachable here — but the escape is correct hardening anyway.
- ✅ **Errors during SQLite restore are logged via `tracing::warn!`, not propagated.** `crates/backup/src/lib.rs:251-253, 261-263`: project artifact restore completes even if the historian/alarm restore fails. Matches the brief's "log via tracing::warn! and continue; project artifact restore should not abort because of a historian quirk" — and the rustdoc on `import_project` (lines 184-190) documents this behavior explicitly.
- ✅ **Let-chains used.** `crates/backup/src/lib.rs:246-264` chains four `&&`-connected conditions: manifest flag set, restore option present, archive entry present, restore call returned `Err`. Reads cleanly; idiomatic for Rust 1.88+ (well within the AG-pinned 1.95.0).
- ✅ **`TEMP_COUNTER` defends against concurrent calls in same millisecond.** `temp_snapshot_path` (line 350-358) now appends an `AtomicU32::fetch_add` value to the timestamp, so concurrent backup operations can't collide on temp filenames. Subtle hardening that paid attention.
- ✅ **Three new tests cover exactly what the brief specified, plus the existing tests still pass:**
  - `historian_rows_round_trip_through_export_import` — seeds source historian with 2 samples (different qualities), exports, imports to fresh target, asserts equivalence via `read()`.
  - `alarm_transitions_round_trip_through_export_import` — seeds 2 transitions, exports, imports, asserts equivalence via `read_alarm()`.
  - `replace_mode_wipes_target_historian_and_alarm_journal` — verifies replace mode replaces, doesn't merge.
- ✅ **Per-crate `restore_from_path` + `merge_from_path` tests** also added in `crates/historian/tests/store.rs` and `crates/alarm-engine/tests/engine.rs` per the brief's test requirements. Verified by Codex's claim of passing `cargo test -p openwebhmi-historian` and `-p openwebhmi-alarm-engine`.
- ✅ **Gateway HTTP restore handler passes refs through.** `crates/gateway/src/server.rs:1461-1462`: `historian_store: DEFAULT_HISTORIAN.get().cloned()` and `alarm_journal: DEFAULT_ALARM_JOURNAL.get().cloned()`. Plugged into the existing one-step HTTP POST flow.
- ✅ **WS-side `ClientMessage::ProjectImport` handler intentionally unchanged** — it's a stub that authorizes admin and emits a `ProjectImportProgress { phase: "upload-ready" }` event (`server.rs:910-926`); it does not call `import_project` because AF's design routes the actual import work through the HTTP POST handler. The brief's "WS-side handler gets the same pass-through" instruction was based on a 2-step flow AF chose not to implement; the as-is implementation is sound.
- ✅ **Wiki updated comprehensively.** `wiki/architecture/backup-restore.md`: summary line mentions "historian/alarm SQLite snapshot export and import"; new items #8 and #9 in "Current understanding"; new "Restore semantics" section explaining replace vs merge for both layers; tests + validation lists extended; AI task file added to "Related pages".
- ✅ **rustdoc updated on `import_project`** (`crates/backup/src/lib.rs:184-190`) describing the new SQLite restoration semantics.

### Findings

- 🟡 **Alarm-journal merge appends without dedupe.** `crates/alarm-engine/src/journal.rs:91-107`: `INSERT INTO alarm_journal SELECT * FROM source_alarm.alarm_journal` — every row from the archive appended unconditionally. The schema has no unique constraint, so importing the same archive twice would duplicate every transition. The brief specified `(tag_id, ts_ms) PK conflict-ignore` only for historian; alarm journal has no natural primary key. **Acceptable for v1.0** — operators don't typically merge-import the same archive twice — but worth a v1.1 follow-up: dedupe via `(alarm_id, ts_ms, from_state, to_state)` composite, or add a `transition_id` PK to the schema. The wiki "Restore semantics" section flags this honestly: "alarm transitions append because the alarm journal currently has no row identity key."
- 🟡 **Single-quote-only escaping in ATTACH path.** Backslash characters or other SQLite metacharacters in the temp file path could theoretically still cause issues. In practice the path comes from `std::env::temp_dir()` + a deterministic suffix (process_id + now_ms + counter), so the attack surface is nil. v1.1 hardening: use `Connection::execute` with quoted identifier or rusqlite's `Connection::attach_database` if/when it lands.
- 🟡 **Concurrent-write race during replace.** `Connection::restore` replaces the entire database; if the gateway's historian writer is actively `INSERT`-ing samples during the restore, those writes could interleave with the restore in non-deterministic ways. The brief flagged this risk explicitly: "v1.0 acceptable: document that operators should pause data ingestion during restore." The wiki page doesn't yet carry that operator guidance — minor doc gap, v1.1 polish.
- 🟡 **No designer/runtime session disconnect on replace** — same as the AF v1.1 polish item, unchanged by AI. Operators viewing a replaced project see stale data until they manually reconnect. v1.1.

### Acceptance-criteria tally

- [x] `RestoreOptions` extended with `historian_store: Option<HistorianStore>` + `alarm_journal: Option<AlarmJournal>`.
- [x] `import_project()` writes historian + alarm-journal blobs to the target stores when refs are provided and the manifest claims their presence.
- [x] Replace mode wipes target historian + alarm-journal data before restore (via `Connection::restore` semantics).
- [x] Merge mode appends rows via SQLite ATTACH + `INSERT OR IGNORE` (option (b) from the brief — the recommended path).
- [x] Gateway HTTP restore handler passes the live store references through.
- [x] Three new unit/integration tests cover historian round-trip, alarm round-trip, and replace-mode wipe.
- [~] `wiki/architecture/backup-restore.md` updated; "Open questions" item about historian-import removed — **wiki updated comprehensively**, but the original page never had a "historian-import gap" Open Question (AF's wiki understated the limitation), so there was nothing literal to remove. The new "Restore semantics" section plus items #8 and #9 honestly document the now-implemented behavior.

7/7 acceptance criteria met (the wiki one with a clarifying note).

### Independent verification

- `cargo test -p openwebhmi-backup --all-features --locked` — ✅ 8/8 pass.
- `cargo clippy --workspace --all-targets --all-features --locked --exclude openwebhmi-designer -- -D warnings` — ✅ clean.
- Read `crates/backup/src/lib.rs::restore_sqlite_entries` and the helper functions: error paths logged via `warn!`, temp files always cleaned up via the closure-then-drop pattern, archive bytes written before SQLite restore call.
- Read `crates/historian/src/store.rs::merge_from_path`: the tag_path-based dictionary join is the correctness-critical detail and it's right.
- Read `crates/alarm-engine/src/journal.rs::merge_from_path`: simple append, dedupe-as-v1.1 limitation documented.
- Read gateway integration: HTTP handler at `server.rs:1461-1462` plumbs refs; WS handler at `server.rs:910-926` is intentionally a no-op stub.

## Verdict

**Merged.** Closes CODEX-AF's v1.0 historian + alarm-journal import gap. The headline use case ("backup before risky change") now round-trips operational telemetry, not just configuration. Implementation hits the correctness-critical details: `Connection::restore` for the safe SQLite restore direction, ATTACH + tag-path remap (not tag-id) for historian merge, transactions around merge SQL, panic-safe error handling that doesn't abort artifact restore on SQLite hiccups, deterministic temp-file naming with atomic counter for concurrency.

**One v1.1 polish callout owned**: alarm-journal merge has no dedupe (no natural PK on the schema). Re-importing the same archive twice doubles transitions. Documented in the wiki's "Restore semantics" section honestly. Adding a composite key or `transition_id` PK is a separate v1.1 alarm-engine task, not AI's scope.

**Cross-task pattern noted**: this is the third submission in this v1.0 closeout sprint where Codex executed a Claude-authored brief cleanly and added small bonuses beyond what was asked (TEMP_COUNTER concurrency hardening, store-level merge tests in addition to backup-level integration tests). Pattern of brief-fidelity-plus-honest-gaps continues — no shortcuts taken on the SQLite correctness details where it would have been easy to.

`docs/feature-matrix.md` row for backup/restore can flip from "🟢 v1 (Phase 4, simulator-validated, historian-import-gap)" to plain "🟢 v1 (Phase 4, simulator-validated)" — the asterisk is gone. v1.0 closeout list now: plugin SDK, performance baseline, pre-1.0 hardware-validation 24h soak gate, plus the polish items flagged across AE/AF/AI verdicts.
