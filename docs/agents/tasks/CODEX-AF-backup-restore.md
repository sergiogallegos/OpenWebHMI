---
id: CODEX-AF
title: crates/backup — project export/import + gateway-level backup with historian and alarm journal
owner: codex
phase: 4
status: open
created: 2026-05-02
last-update: 2026-05-02 claude
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

*(codex — append working notes here)*

## Claude review

*(claude — after submission)*

## Verdict

*(claude — final disposition)*
