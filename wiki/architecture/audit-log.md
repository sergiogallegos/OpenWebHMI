---
status: active
last-validated: 2026-05-03
---

# Audit log

## Summary
OpenWebHMI now has a SQLite-backed security audit journal in `crates/audit-log`, with gateway wire protocol support for administrator-only `audit.subscribe` and `audit.query`.

## Current understanding
1. `crates/audit-log` persists append-only `AuditEntry` rows with `schema_version = 1`, event kind, timestamp, actor metadata, and JSON payload. Source: `crates/audit-log/src/store.rs`.
2. Audit event payloads reuse `TagValue` from `openwebhmi-protocol` and `Role` from `openwebhmi-auth`, avoiding duplicate wire/domain copies. Source: `crates/audit-log/src/event.rs`.
3. The gateway appends audit events for login attempts, logout, WebSocket tag writes, project artifact saves, user admin actions, and script tag writes routed through `GatewayTagWriteSink`. Source: `crates/gateway/src/server.rs`, `crates/gateway/src/script_writes.rs`.
4. The old scripting `tracing::info!` tag-write audit line was removed; script writes now audit through the gateway sink as `WriteSource::Script`. Source: `crates/scripting/src/worker.rs`.
5. `audit.subscribe` and `audit.query` require an administrator session server-side; unauthorized clients receive an error before the audit store is consulted. Source: `crates/gateway/src/server.rs`.

## Evidence
- Code: `crates/audit-log/src/event.rs`, `crates/audit-log/src/query.rs`, `crates/audit-log/src/store.rs`.
- Protocol: `crates/protocol/src/lib.rs`, `packages/protocol-ts/src/index.ts`.
- Tests: `crates/audit-log/tests/audit.rs`, `crates/gateway/src/script_writes.rs`, `crates/protocol/src/lib.rs`.
- Validation run: `cargo test -p openwebhmi-audit-log --all-features --locked`, `cargo test -p openwebhmi-gateway --all-features --offline`, focused clippy, and `pnpm --filter @openwebhmi/protocol typecheck` passed on 2026-05-03.

## Open questions
- Gateway integration tests should still exercise `audit.query` end-to-end over WebSocket with real admin/non-admin sessions.
- The store currently performs query filtering in Rust after loading rows; move high-volume filters into SQL before production scale testing.
- Per-event value-size caps and tamper-evident chains remain post-v1 work.

## Related pages
- `wiki/architecture/backup-restore.md`
- `docs/agents/tasks/CODEX-AE-audit-log.md`
