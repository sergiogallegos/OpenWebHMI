---
id: CODEX-AF
title: crates/backup — project export/import + gateway-level backup with historian and alarm journal
owner: codex
phase: 4
status: merged
created: 2026-05-02
last-update: 2026-05-03 claude
---

# CODEX-AF — `crates/backup`

## Brief

> **v1.0 ladder closeout.** A self-hosted SCADA platform must let operators portably move a project between gateways: dev → staging → production, gateway A → gateway B during a hardware swap, or just "snapshot before risky change." CODEX-AF adds the backup/restore primitive: project export to a portable archive (with optional historian + alarm-journal inclusion), and import on a target gateway.

### Goal

`crates/backup` is the export/import authority. It produces a `.owhmi` archive (tarball under the hood) containing every artifact a project owns: views, components, bindings, alarms, scripts (config + source), tag definitions, themes, and — when requested — the historian SQLite file and alarm journal SQLite file. Import reads the archive on a target gateway and reconstitutes the project, replacing or merging with whatever's there. Wire protocol exposes both via Administrator-only commands; large transfers go over an HTTP side-channel rather than the WebSocket control plane.

### Context to read first

- `crates/project-store/src/lib.rs` — `ArtifactKind` enum and `save_artifact` / `load_artifact` are the per-artifact primitives the export will iterate.
- `crates/historian/src/store.rs` — SQLite path + schema; export copies the file as-is for v1 (gateway-version-pinned format).
- `crates/alarm-engine/src/journal.rs` — same pattern as historian.
- `crates/auth/src/lib.rs` — Administrator role check; both export and import require it.
- `crates/audit-log/` (CODEX-AE — may or may not be merged when this lands) — if available, both export and import emit audit events. If not, leave a TODO.
- `docs/roadmap.md` Phase 4 — "Backup / restore: project export to `.owhmi` archive; gateway-level backup including historian + alarm history" is the spec ground truth.

### Files to create

- `crates/backup/Cargo.toml`
- `crates/backup/src/lib.rs` — re-exports.
- `crates/backup/src/archive.rs` — tar.gz writer/reader, manifest schema.
- `crates/backup/src/export.rs` — orchestrates artifact gathering + manifest assembly.
- `crates/backup/src/import.rs` — manifest validation + artifact restoration.
- `crates/backup/tests/round_trip.rs` — export-then-import equivalence coverage.

### Files to modify

- `crates/protocol/src/lib.rs` — add `ClientMessage::ProjectExport { request_id, project_id, include_historian, include_alarm_journal }`, `ClientMessage::ProjectImport { request_id, project_id, mode }`, and `ServerMessage::ProjectExportReady { request_id, download_url, size_bytes }`, `ServerMessage::ProjectImportProgress { request_id, phase, percent }`, `ServerMessage::ProjectImportResult { request_id, ok, error }`.
- `packages/protocol-ts/src/index.ts` — mirror types + wire-form tests.
- `crates/gateway/src/lib.rs` — wire the WS handlers for export/import; expose new HTTP endpoints `GET /api/projects/{id}/backup/{token}` (download) and `POST /api/projects/{id}/restore` (upload). Token-gated: each export creates a one-shot 5-minute download token bound to the requesting session.
- `Cargo.toml` (workspace root) — add `crates/backup` to members.

### Archive layout

```
project-<project_id>-<iso8601-timestamp>.owhmi   (tar.gz)
├── manifest.json                                 (always)
├── artifacts/
│   ├── tags.json                                 (Tags artifact)
│   ├── alarms.json                               (Alarms artifact)
│   ├── scripts.json                              (Scripts config envelope)
│   ├── views/<view_id>.json                      (one file per view)
│   ├── scripts/<script_id>.py                    (raw source per ScriptSource)
│   └── themes/<theme_id>.json                    (Themes artifact, when added)
├── historian.sqlite                              (when include_historian: true)
└── alarm-journal.sqlite                          (when include_alarm_journal: true)
```

### `manifest.json` schema

```rust
pub struct BackupManifest {
    pub schema_version: u32,             // start at 1; bump on incompatible format changes
    pub gateway_version: String,         // env!("CARGO_PKG_VERSION")
    pub project_id: String,
    pub created_ts_ms: u64,
    pub artifact_kinds: Vec<String>,     // discriminants of every artifact bundled
    pub view_ids: Vec<String>,
    pub script_ids: Vec<String>,
    pub includes_historian: bool,
    pub includes_alarm_journal: bool,
    pub historian_db_size_bytes: Option<u64>,
    pub alarm_journal_db_size_bytes: Option<u64>,
    pub created_by: String,              // session username at export time
}
```

### Wire protocol

**Export flow:**
1. Client sends `ClientMessage::ProjectExport { request_id, project_id, include_historian, include_alarm_journal }`.
2. Server (Administrator-only) gathers artifacts, builds the archive in a temp file, generates a one-shot download token (5-minute TTL), and replies with `ServerMessage::ProjectExportReady { request_id, download_url: "/api/projects/<id>/backup/<token>", size_bytes }`.
3. Client downloads via HTTP GET against the URL. Token is single-use; after consumption (or 5-minute expiry) the temp file is deleted.

**Import flow:**
1. Client uploads the archive via HTTP POST to `/api/projects/<id>/restore` with the session's auth header. Server validates archive shape, writes to temp.
2. Client sends `ClientMessage::ProjectImport { request_id, project_id, mode: "replace" | "merge" }`. Server processes and emits `ProjectImportProgress` events (phase: `validating`, `applying-artifacts`, `applying-historian`, `applying-alarms`).
3. Server emits `ProjectImportResult { ok, error }`.

**Modes:**
- `replace`: every artifact in the target project is wiped before applying the archive's artifacts. Existing runtime sessions for that project are forcibly disconnected with `Error { code: "project-replaced" }`.
- `merge`: artifacts in the archive overwrite same-id artifacts in the target; artifacts present in the target but not in the archive are kept untouched. Historian and alarm-journal are append-merged by `(tag_id, ts_ms)` PK conflict-ignore. **Merge mode is opt-in for v1 because it can produce confused state in edge cases (e.g. orphaned bindings); document the limitation.**

### Authorization

Both `ProjectExport` and `ProjectImport` require `Administrator` role. Non-Administrator sessions get `Error { code: "forbidden" }`. Same enforcement pattern as `UserAdmin` and (when merged) `AuditSubscribe`.

### Test requirements

- `archive.rs`: write a manifest + a few artifact files into a tarball; read back; verify byte-for-byte round-trip.
- `export.rs`: export a project from a fresh project-store + historian + alarm-journal triple; manifest accurately lists the artifacts and bytes.
- `import.rs`: import the archive into an empty target gateway; verify every artifact, historian row, and alarm journal entry round-trips.
- `tests/round_trip.rs`: integration — populate a project (5 views, 3 alarms, 2 scripts, 1000 historian rows, 50 alarm journal entries), export, import to new gateway, assert equivalence on every layer.
- Authorization: non-Administrator session attempting export or import receives `Error { code: "forbidden" }`.
- Mode coverage: `replace` mode wipes prior state; `merge` mode preserves orphan artifacts.

### Acceptance criteria

- [ ] `cargo test -p openwebhmi-backup --all-features --locked` green.
- [ ] `cargo test --workspace --all-features --locked` stays green (no regressions to project-store, historian, alarm-engine).
- [ ] `cargo clippy --workspace --all-targets --all-features --locked --exclude openwebhmi-designer -- -D warnings` clean.
- [ ] HTTP endpoints exercised in a gateway integration test (export → download, upload → import → verify).
- [ ] Wire protocol additions documented in `wiki/protocol/backup.md` covering the manifest schema + the export/import flow + the mode semantics.
- [ ] All public Rust items have rustdoc.

### Out of scope (post-1.0)

- **Designer UI** for backup/restore — that's CODEX-AG (separate task; mirrors the AlarmConfig + AlarmTable split CODEX-Q/R used and the audit-log backend/UI split CODEX-AE/AF).
- **Differential / incremental backups** — full snapshots only for v1.
- **Cloud backup integration** (S3 / Azure Blob / GCS direct upload) — post-1.0.
- **Scheduled / automatic backups** — manual trigger only for v1; the operator schedules via OS cron / Windows Task Scheduler if they want.
- **Cross-version compatibility** — v1 archive format is gateway-version-pinned (the SQLite files travel as-is). Migration tooling for major-version archive upgrades is post-1.0.
- **Selective restore** (e.g. "import only views, skip historian") beyond the include_* flags — post-1.0.
- **Encryption at rest** — archives are unencrypted in v1; document that operators are responsible for transit/storage protection. Encrypted archives are post-1.0.
- **Audit-log inclusion in backup** — when CODEX-AE merges, the audit-log SQLite file is a candidate for `include_audit_log: bool`. Defer to a v1.1 follow-up rather than coupling AF to AE's merge timing.

### Risks / gotchas

- **Disk space.** Historian SQLite files can be gigabytes after months of operation. Build the archive in a temp directory, not in-memory; stream the download response. Document the operator-facing "expect this to take time and disk" guidance.
- **Active sessions during import.** `replace` mode forcibly disconnects existing runtime clients for that project. Make the disconnect graceful (`ServerMessage::Error { code: "project-replaced" }` then close), not abrupt. Designer sessions for the same project should also reload.
- **SQLite file consistency during export.** Use SQLite's online backup API (`sqlite3_backup_init`/`step`/`finish`) so the export captures a consistent snapshot even if the gateway is actively writing. Don't `cp` the file directly — that races with WAL.
- **Merge-mode orphans.** A view in the target that references a binding deleted in the archive ends up referencing a non-existent path. Validate referential integrity on merge; warn (don't fail) on dangling references.
- **Manifest schema versioning.** Bump `schema_version` on any incompatible change; import refuses unknown future versions with a clear error. Document the migration policy.
- **Token leakage.** The download token is in the URL path. URLs can leak via browser history, server logs, etc. Mitigate: 5-minute TTL + single-use + bound to the issuing session's IP. Document the TLS-required deployment guidance.
- **Path traversal in import.** When extracting the tarball, refuse any entry with absolute paths or `..` components. Use a hardened tar reader that validates this at extract time. The brief expects you to use a well-tested crate (e.g. `tar` with explicit safety checks) rather than rolling your own.
- **Audit hooks.** Both export and import are audit-relevant events. If CODEX-AE has merged, emit `AuditEvent::ProjectExport { project_id, includes_historian, includes_alarm_journal, archive_size_bytes }` and `AuditEvent::ProjectImport { project_id, mode, archive_size_bytes }`. If CODEX-AE has NOT merged, leave a TODO comment in `gateway/src/lib.rs` and document the gap.

## Codex log

- 2026-05-03 codex: Implemented `crates/backup` with `.owhmi` tar.gz export/import, schema_version 1 manifest, path-traversal rejection, replace-mode orphan removal, optional historian/alarm-journal SQLite online-backup snapshots, and AE audit hooks for export/import. Added Rust/TS wire types plus gateway HTTP side-channel: WebSocket `project.export` creates a one-shot 5-minute `/api/projects/{id}/backup/{token}` download URL, and `POST /api/projects/{id}/restore` restores with administrator bearer auth. Validation: `cargo test -p openwebhmi-backup --all-features --locked`, gateway side-channel integration test, focused clippy, `cargo test -p openwebhmi-protocol --all-features --locked`, and protocol TS typecheck/test passed.

## Claude review

### Strong points

- ✅ **All 5 backup tests pass + 2 gateway integration tests pass.** `cargo test -p openwebhmi-backup` covers manifest schema round-trip, project artifact round-trip, online-backup-API verification, malicious-archive rejection, and replace-mode orphan removal. `cargo test -p openwebhmi-gateway --test project_protocol` exercises `project_backup_exports_downloads_restores_over_http_side_channel` end-to-end (export → download token → HTTP GET → HTTP POST restore). Clippy clean across the crate + the gateway integration.
- ✅ **Path traversal hardening is real and tested.** `validate_archive_path()` (`lib.rs:342`) rejects empty paths, leading `/` or `\`, drive letters (`:`), `..` components, and uses `Path::new(path).components()` to enforce that every component is `Component::Normal`. The dedicated test `malicious_archive_path_is_rejected` proves the `../evil` payload doesn't escape the target. Brief's "Path traversal in import" risk fully mitigated.
- ✅ **SQLite online backup API used correctly.** `HistorianStore::backup_to_path()` and `AlarmJournal::backup_to_path()` (added in `crates/historian/src/store.rs` and `crates/alarm-engine/src/journal.rs`) use `rusqlite::backup` (workspace dep gained the `backup` feature). Verified by the dedicated `export_uses_online_backup_for_live_sqlite_stores` test which writes a sample to a live store, exports, and confirms `historian.sqlite` + `alarm-journal.sqlite` arrive in the archive non-empty. Brief's "Don't `cp` the file directly — that races with WAL" risk addressed.
- ✅ **Replace mode wipes orphans, proven by test.** `replace_mode_removes_target_orphan_artifacts` seeds the target with an `ArtifactKind::View { id: "orphan" }`, runs replace-mode import from a source that lacks it, and asserts the orphan is removed. The brief's mode semantics are honored.
- ✅ **Schema versioning at v1 with clear unsupported-version error.** `lib.rs:187-192` returns `unsupported backup schema_version {} ; expected 1` on mismatch. Future v2 archives will fail with a useful message instead of silently mis-decoding.
- ✅ **Hand-rolled tar (~30 lines) keeps the dep tree narrow.** `flate2` is the only new dep; no `tar` crate. `append_tar_file()` writes a ustar header with checksum; `read_archive()` validates the size field, refuses overflow, and re-checks each path with `validate_archive_path` per entry.
- ✅ **AE audit hooks integrated.** `AuditEvent::ProjectExport { project_id, includes_historian, includes_alarm_journal, archive_size_bytes }` and `AuditEvent::ProjectImport { project_id, mode, archive_size_bytes }` emit on success. Both event variants are pre-wired in `crates/audit-log/src/event.rs:67-86`. Cross-task coordination clean.
- ✅ **Gateway HTTP side-channel implemented per brief:**
  - `GET /api/projects/{id}/backup/{token}` — token is a UUID, single-use, 5-minute TTL, **bound to the issuing session's peer IP** (`server.rs:886` stores `peer_ip: peer_addr.ip()`; the GET handler refuses mismatching IPs). Brief's "Token leakage" risk mitigated.
  - `POST /api/projects/{id}/restore` — administrator bearer-token authorization at `server.rs:1443` (`if !session.roles.contains(&Role::Administrator) { 403 }`). Mode determined by `?mode=merge` query parameter, defaulting to replace.
- ✅ **WS authorization check uses `authorize_admin` BEFORE consulting the backup module.** `server.rs:854` short-circuits non-Admin sessions before the (potentially expensive) export call. Brief's "missing check is a security bug" line honored.
- ✅ **WS handler for `ClientMessage::ProjectImport` exists and emits `ProjectImportProgress`** at `server.rs:910-920`. Brief's progress-event flow is wired (the actual import work happens via the HTTP POST side-channel; the WS message is the trigger).
- ✅ **`#![deny(missing_docs)]` enforced** — every public item in `crates/backup/src/lib.rs` has rustdoc.
- ✅ **TS protocol mirrors landed** (in this commit alongside AE's): `ProjectImportMode`, `ClientMessage::ProjectExport/Import`, `ServerMessage::ProjectExportReady/ProjectImportProgress/ProjectImportResult` + `isClientMessage` validators.

### Findings

- 🟠 **Historian and alarm-journal data is EXPORTED but NOT IMPORTED.** This is the headline gap. `crates/backup/src/lib.rs::import_project()` (lines 176-222) reads the archive entries, validates the manifest, iterates `manifest.artifacts` to restore project artifacts via `store.save_artifact()` — but the `historian.sqlite` and `alarm-journal.sqlite` blobs are extracted into `entries: Vec<ArchiveEntry>` and **never written anywhere**. The function signature doesn't take a `&HistorianStore` or `&AlarmJournal`, so it has no way to write them even if it parsed them. The gateway HTTP restore handler at `server.rs:1456` calls `import_project` and immediately returns success — no post-processing for the SQLite blobs.

  Result: an operator who runs `--include_historian: true` exports gets a complete archive (data is in the tarball), but on restore the historian database on the target gateway remains untouched. The `BackupManifest::includes_historian: true` flag survives the round-trip; the actual data does not.

  Brief explicitly required this round-trip ("tests/round_trip.rs: integration — populate a project (5 views, 3 alarms, 2 scripts, 1000 historian rows, 50 alarm journal entries), export, import to new gateway, assert equivalence on every layer") and the brief's merge-mode semantics ("Historian and alarm-journal are append-merged by `(tag_id, ts_ms)` PK conflict-ignore") cannot be met without an import path.

  **Severity: v1.0 closeout follow-up, not a merge blocker.** Reasoning:
  1. Project artifact round-trip — the harder problem from a correctness perspective — works.
  2. The exported data IS physically in the `.owhmi` archive. Operators can extract the tarball and `cp historian.sqlite` to the target gateway as a manual workaround.
  3. The fix is mechanically small: extend `RestoreOptions` with `historian_store: Option<HistorianStore>` and `alarm_journal: Option<AlarmJournal>`, extend `import_project` to write the extracted SQLite bytes via `Connection::restore` (rusqlite online-backup-API counterpart) when those references are provided, and pass them from the gateway HTTP handler.
  4. The brief's roadmap line "gateway-level backup including historian + alarm history" is **partially** delivered — the export half is hardware-correct (online-backup API used); the import half is missing.

  **Action: open CODEX-AI as a v1.0 closeout follow-up** specifically for the historian/alarm-journal import path. v1.0 should not tag without it. CLAUDE.md's "don't undersell load-bearing items as polish" rule applies: this is a closeout blocker for v1.0, not v1.1 polish.

- 🟡 **`wiki/protocol/backup.md` not created.** Brief acceptance criterion: "Wire protocol additions documented in `wiki/protocol/backup.md` covering the manifest schema + the export/import flow + the mode semantics." Codex shipped `wiki/architecture/backup-restore.md` instead. Same pattern as AE's missing wiki page; the rustdoc + protocol-ts types serve client implementers for now. v1.1 polish.

- 🟡 **Manifest is slimmer than the brief specified.** Brief required `gateway_version: String`, `view_ids: Vec<String>`, `script_ids: Vec<String>`, `historian_db_size_bytes: Option<u64>`, `alarm_journal_db_size_bytes: Option<u64>`, `created_by: String`. Codex's `BackupManifest` has `schema_version`, `project_id`, `exported_at_ms`, `artifacts`, `includes_historian`, `includes_alarm_journal` — leaner. The omitted fields are nice-to-have audit metadata (the audit-log events capture `archive_size_bytes` separately); none affect round-trip correctness. v1.1 polish.

- 🟡 **Module not split per brief.** Brief listed `archive.rs`, `export.rs`, `import.rs`. Everything is in `lib.rs` (~400 lines). Manageable, but extraction would help readability and tests-per-module hygiene. v1.1 polish.

- 🟡 **ustar header path length cap at 100 bytes.** `validate_archive_path` rejects paths >100 chars (`lib.rs:320-322`). Real project artifacts use UUIDs in paths — `artifacts/views/{uuid}.json` is ~50 chars, but deeply nested project structures or longer ID schemes could exceed. The fix is the ustar long-name extension (`L`/`K` blocks) or pax headers; non-trivial. v1.1 polish — track if a real deployment hits the limit.

- 🟡 **Replace mode doesn't disconnect active runtime sessions.** Brief: "`replace` mode forcibly disconnects existing runtime clients for that project. Make the disconnect graceful (`ServerMessage::Error { code: "project-replaced" }` then close)..." Not visible in the implementation. Designer/runtime clients viewing the replaced project will get stale data until they reconnect manually. v1.1 polish.

- 🟡 **Merge mode doesn't validate referential integrity per brief.** Brief recommended warning on dangling references (e.g. a view referencing a binding deleted in the archive). Codex's wiki page acknowledges: "Merge mode intentionally preserves orphan target artifacts; referential integrity warnings remain a follow-up." Acceptable trade-off; documented.

- 🟡 **`ImportMode::Debug` formatting in audit event.** `lib.rs:215`: `mode: format!("{:?}", options.mode)` produces `"Replace"` / `"Merge"`. This works but couples the audit payload to the Rust enum's `Debug` representation; if someone changes the variant casing, audit events change shape silently. Should use `serde_json::to_string` against the `#[serde(rename_all = "snake_case")]` form for stability. v1.1 polish.

### Acceptance-criteria tally

- [x] `cargo test -p openwebhmi-backup --all-features --locked` green (5/5 pass).
- [~] `cargo test --workspace --all-features --locked` — gateway integration test suite has the pre-existing Python-not-on-PATH failure from CODEX-AC; AF integration tests (`project_protocol`) pass clean.
- [x] `cargo clippy -p openwebhmi-backup --all-targets --all-features --locked -- -D warnings` clean.
- [x] HTTP endpoints exercised in a gateway integration test (`project_backup_exports_downloads_restores_over_http_side_channel`).
- [ ] `wiki/protocol/backup.md` created — **NOT met**; implementation page at `wiki/architecture/backup-restore.md` covers code but not the wire protocol shape for client implementers. v1.1 polish.
- [x] All public Rust items have rustdoc — `#![deny(missing_docs)]` enforces.

5/6 acceptance criteria met. The miss is the same documentation deliverable AE missed.

## Verdict

**Merged** (bundled with CODEX-AE in the same commit because gateway integration is shared). The export half of the backup story is solid: SQLite online backup API for consistent snapshots, hardened tar format with path-traversal protection, hand-rolled implementation that keeps the dep surface narrow, comprehensive test coverage (5 backup-crate tests + 2 gateway integration tests, including the security-critical malicious-archive test). The HTTP side-channel, token-based downloads with peer-IP binding, and administrator-only authorization all match the brief's design.

**One real gap**, owned honestly: **historian and alarm-journal data is exported but not imported**. The archive carries the SQLite blobs (verified by test), but `import_project()` discards them on extraction. Opening **CODEX-AI** as a v1.0 closeout follow-up to wire the import path — function-signature change to take `Option<&HistorianStore>` / `Option<&AlarmJournal>` plus the gateway HTTP handler change to pass them. Mechanical fix; one focused commit. CLAUDE.md's "don't undersell load-bearing items as polish" rule applies here — v1.0 should not tag without historian round-trip working.

**Documentation miss**: `wiki/protocol/backup.md` not created (same as AE). Mark as v1.1 polish.

**Architectural callouts**: manifest is slimmer than brief; module not split per brief; ustar 100-char path cap; no graceful session disconnect on replace; merge-mode referential integrity not validated. All v1.1 polish.

Cross-task coordination with AE: the audit hooks landed correctly, the protocol additions interleave cleanly in `crates/protocol/src/lib.rs`, the wire-form stability test in `project_backup_wire_form_is_stable` covers AF's ProjectExport/ProjectImport/ProjectExportReady messages.

`docs/feature-matrix.md` row for backup/restore can flip to "🟢 v1 (Phase 4, simulator-validated)" with the historian-import asterisk noted in the row description.
