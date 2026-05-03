---
status: active
last-validated: 2026-05-03
---

# Backup restore

## Summary
`crates/backup` implements the `.owhmi` archive primitive and the gateway exposes the transfer side-channel: tar.gz export/import, schema_version 1 manifest, project artifact round-trip, path traversal rejection, replace-mode orphan removal, optional SQLite online-backup snapshots, one-shot download tokens, administrator restore auth, and optional audit events.

## Current understanding
1. Archives are gzip-compressed tar streams with `manifest.json`, `artifacts/` entries, and optional `historian.sqlite` / `alarm-journal.sqlite` payloads. Source: `crates/backup/src/lib.rs`.
2. The manifest starts at `schema_version = 1`, records the project id, export timestamp, artifact list, and optional database inclusion flags. Source: `crates/backup/src/lib.rs`.
3. Import rejects absolute paths, drive-letter paths, backslash roots, and `..` components before reading archive entries. Source: `crates/backup/src/lib.rs`; test: `crates/backup/tests/round_trip.rs`.
4. Export/import can emit `AuditEvent::ProjectExport` and `AuditEvent::ProjectImport` when supplied an AE `AuditLog`. Source: `crates/backup/src/lib.rs`.
5. `project.export` creates a one-shot five-minute URL at `/api/projects/{id}/backup/{token}`; the HTTP handler consumes the token and checks the requesting peer IP. Source: `crates/gateway/src/server.rs`.
6. `POST /api/projects/{id}/restore` imports the uploaded archive with administrator bearer-token authorization. Source: `crates/gateway/src/server.rs`.
7. Historian and alarm-journal inclusion use `rusqlite`'s online backup API through `HistorianStore::backup_to_path` and `AlarmJournal::backup_to_path`. Source: `crates/historian/src/store.rs`, `crates/alarm-engine/src/journal.rs`.

## Evidence
- Code: `crates/backup/src/lib.rs`, `crates/protocol/src/lib.rs`, `packages/protocol-ts/src/index.ts`.
- Tests: `crates/backup/tests/round_trip.rs`, `crates/gateway/tests/project_protocol.rs`.
- Validation run: `cargo test -p openwebhmi-backup --all-features --locked`, `cargo test -p openwebhmi-gateway --test project_protocol --all-features --locked`, focused clippy, `cargo test -p openwebhmi-protocol --all-features --locked`, and protocol TS checks passed on 2026-05-03.

## Open questions
- Merge mode intentionally preserves orphan target artifacts; referential integrity warnings remain a follow-up.
- The HTTP side-channel is intentionally small and purpose-built; revisit once the gateway adopts a general HTTP framework.

## Related pages
- `wiki/architecture/audit-log.md`
- `docs/agents/tasks/CODEX-AF-backup-restore.md`
