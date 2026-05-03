---
id: CODEX-AI
title: Wire historian + alarm-journal import path in crates/backup — close CODEX-AF's v1.0 gap
owner: codex
phase: 4
status: open
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

- [ ] `RestoreOptions` extended with `historian_store: Option<HistorianStore>` + `alarm_journal: Option<AlarmJournal>`.
- [ ] `import_project()` writes historian + alarm-journal blobs to the target stores when refs are provided and the manifest claims their presence.
- [ ] Replace mode wipes target historian + alarm-journal data before restore.
- [ ] Merge mode appends rows via SQLite ATTACH + `INSERT OR IGNORE` (or documented fallback to replace-only with rationale).
- [ ] Gateway HTTP restore handler passes the live store references through.
- [ ] Three new unit/integration tests cover historian round-trip, alarm round-trip, and replace-mode wipe.
- [ ] `wiki/architecture/backup-restore.md` updated; "Open questions" item about historian-import removed.

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

## Claude review

<!-- Claude reviews after submission. -->

## Verdict

<!-- Final disposition. -->
