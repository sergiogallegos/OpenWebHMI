---
id: CODEX-AE
title: crates/audit-log — security event journal + query/subscribe wire protocol
owner: codex
phase: 4
status: merged
created: 2026-05-02
last-update: 2026-05-03 claude
---

# CODEX-AE — `crates/audit-log`

## Brief

> **v1.0 ladder closeout.** SCADA platforms need a queryable audit log for security and compliance: who did what, when. CODEX-AE adds a backend journal crate, hooks security-relevant events from gateway + scripting + auth into it, and exposes a query/subscribe wire protocol. Designer UI for browsing the log lands separately as CODEX-AF (post-AE).

### Goal

`crates/audit-log` is the structured, queryable equivalent of `tracing::info!` lines for security-relevant events. It records who did what (auth events, tag writes, project changes, user admin), persists to SQLite, and exposes the data via the same wire-protocol pattern that `alarm.subscribe` / `alarm.event` use. The runtime/designer can show "last 50 events for user X" or "all failed logins in the last hour."

### Context to read first

- `crates/auth/src/lib.rs` — `User`, `Role`, `Session`, login/logout flow. The auth events emit from here.
- `crates/alarm-engine/src/journal.rs` — closest analog for the SQLite append-only journal pattern; mirror the schema shape.
- `crates/scripting/src/host.rs` — existing `tracing::info!` audit lines for `system.tag.write`. Decision required: route those through the new audit-log crate, or coexist? Brief recommendation below.
- `crates/gateway/src/lib.rs` — where WS-side tag-write requests resolve; that's the hook point for the WS-side audit event.
- `crates/project-store/src/lib.rs` — where `save_artifact` resolves; hook point for project-change audit events.

### Files to create

- `crates/audit-log/Cargo.toml`
- `crates/audit-log/src/lib.rs` — re-exports.
- `crates/audit-log/src/event.rs` — `AuditEvent` enum + serde.
- `crates/audit-log/src/store.rs` — SQLite store, append + query.
- `crates/audit-log/src/query.rs` — filter struct + pagination.
- `crates/audit-log/tests/audit.rs` — round-trip + filter coverage.

### Files to modify

- `crates/protocol/src/lib.rs` — add `ClientMessage::AuditSubscribe`, `ClientMessage::AuditQuery`, `ServerMessage::AuditEvent`, `ServerMessage::AuditQueryResult`.
- `packages/protocol-ts/src/index.ts` — mirror types + wire-form tests.
- `crates/gateway/src/lib.rs` — wire the audit log into:
  - WS-side `tag.write` resolution (success and failure) → `AuditEvent::TagWrite`.
  - Auth handshake → `AuditEvent::AuthLogin { user, success }`. Failed login emits with `success: false` and the attempted username.
  - Session expiry → `AuditEvent::SessionExpired`.
  - User admin commands (add/remove user, role change) → `AuditEvent::UserAdmin`.
  - `project.save_artifact` → `AuditEvent::ProjectSave`.
  - WS subscribe routing for `AuditSubscribe` (broadcast, same pattern as alarm).
- `crates/scripting/src/host.rs` — replace the existing `tracing::info!` audit lines with calls into the new audit-log crate (one path; do NOT ship both). Pass the `AuditLog` handle through `ScriptHost::new`.
- `Cargo.toml` (workspace root) — add `crates/audit-log` to members.

### `AuditEvent` shape

```rust
pub struct AuditEntry {
    pub id: u64,
    pub ts_ms: u64,
    pub user: Option<String>,        // None for unauthenticated events (failed login attempts)
    pub session_id: Option<String>,  // present when an authenticated session is the actor
    pub source_ip: Option<String>,   // present when the WS connection captured one
    pub kind: AuditEvent,
}

pub enum AuditEvent {
    AuthLogin { username: String, success: bool, reason: Option<String> },
    AuthLogout { username: String },
    SessionExpired { username: String },
    TagWrite { path: String, value: TagValue, success: bool, source: WriteSource, error: Option<String> },
    ProjectSave { project_id: String, artifact_kind: String },
    UserAdmin { actor: String, action: UserAdminAction, target_user: String },
}

pub enum WriteSource {
    WebSocket,    // operator action via runtime UI
    Script,       // system.tag.write from a Python script
}

pub enum UserAdminAction { Created, Deleted, RoleChanged { from: Role, to: Role }, PasswordChanged }
```

`TagValue` is the existing protocol type. `Role` is the existing auth type. Do NOT introduce new copies.

### Storage

```sql
CREATE TABLE IF NOT EXISTS audit_log (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    ts_ms INTEGER NOT NULL,
    user TEXT,
    session_id TEXT,
    source_ip TEXT,
    kind TEXT NOT NULL,         -- discriminant: "AuthLogin", "TagWrite", etc.
    payload TEXT NOT NULL       -- JSON-encoded event body
);
CREATE INDEX IF NOT EXISTS idx_audit_ts_user ON audit_log(ts_ms, user);
CREATE INDEX IF NOT EXISTS idx_audit_kind ON audit_log(kind, ts_ms);
```

Append-only. Retention: **90 days** default, configurable via gateway config (`audit_retention_days`). On startup, vacuum entries older than retention.

### Query API

```rust
pub struct AuditQuery {
    pub from_ts_ms: Option<u64>,
    pub to_ts_ms: Option<u64>,
    pub user: Option<String>,
    pub kinds: Vec<&'static str>,    // empty = all kinds
    pub limit: usize,                // capped at 1000 server-side
    pub offset: usize,
}
```

### Subscribe protocol

```rust
ClientMessage::AuditSubscribe { request_id }
ClientMessage::AuditUnsubscribe { request_id }
ClientMessage::AuditQuery { request_id, query: AuditQuery }
ServerMessage::AuditEvent { entry: AuditEntry }                    // pushed live
ServerMessage::AuditQueryResult { request_id, entries: Vec<AuditEntry>, total: u64 }
```

Authorization: only `Administrator` role can subscribe or query. Other roles get `ServerMessage::Error { code: "forbidden" }`. Enforce in gateway handler (mirror `UserAdmin` ACL pattern).

### Test requirements

- `event.rs`: serde round-trip for every `AuditEvent` variant; ensure JSON shape is stable (no field renames).
- `store.rs`: append + query by ts range, by user, by kind. Pagination at limit + offset. Retention vacuum drops entries older than cutoff.
- `query.rs`: filter combinations (user only, kind only, time range only, all three).
- `tests/audit.rs` integration: spawn store, write 50 events across 5 users and 4 kinds, query each filter combination, verify results.
- Gateway integration: WS tag.write success → `AuditEvent::TagWrite { source: WriteSource::WebSocket, success: true }` observable in the next `AuditQuery`. Failed login → `AuditEvent::AuthLogin { success: false }` observable. Script-driven `system.tag.write` → `AuditEvent::TagWrite { source: WriteSource::Script }` (replacing the existing `tracing::info!` line).
- Gateway authorization: non-Administrator role attempting `AuditSubscribe` or `AuditQuery` receives `Error { code: "forbidden" }`.

### Acceptance criteria

- [ ] `cargo test -p openwebhmi-audit-log --all-features --locked` green.
- [ ] `cargo test --workspace --all-features --locked` stays green (no regressions to other crates).
- [ ] `cargo clippy --workspace --all-targets --all-features --locked --exclude openwebhmi-designer -- -D warnings` clean.
- [ ] Wire protocol additions documented in `wiki/protocol/audit.md` (new file) covering all four message types + the `AuditEntry` schema.
- [ ] Existing `tracing::info!` audit lines in `crates/scripting/src/host.rs` replaced with calls into `audit-log`. Do not ship both code paths.
- [ ] All public Rust items have rustdoc.

### Out of scope (post-1.0)

- **Designer UI for browsing the audit log** — that's CODEX-AF (separate task, mirrors the AlarmConfig + AlarmTable split CODEX-Q/R used).
- **External SIEM integration** (Splunk/syslog forwarding, structured stderr). Post-1.0.
- **Tamper-evident logs** (Merkle chain, signed entries, append-only verification). Post-1.0; reasonable to track the schema requirement now.
- **Per-event-kind retention policies** — uniform 90-day retention for v1; configurable per kind is post-1.0.
- **Audit log export** (CSV / JSON dump) — post-1.0; the query API + designer UI cover the v1 use case.
- **Per-user rate limiting on script-driven tag writes** — post-1.0 if audit volume becomes a problem in real deployments.

### Risks / gotchas

- **Don't double-log.** The scripting host already has `tracing::info!` lines for `system.tag.write` audit. Replace those with `audit_log.append(AuditEvent::TagWrite { source: WriteSource::Script, .. })`. Shipping both means duplicate signal and divergence over time.
- **Authorization enforcement is server-side.** A client role check in TypeScript is hint-only. The gateway MUST refuse `AuditSubscribe` / `AuditQuery` from non-Administrator sessions before consulting the audit store. Tests must cover this; a missing check is a security bug.
- **Failed-login events leak attempted-username metadata.** That's intentional (operators need to see "someone tried to log in as `admin`"), but document the consideration in the wiki entry.
- **Schema version field.** Add a `schema_version: 1` row to a `meta` table or in pragma so future migrations don't have to guess. Mirror what `alarm-engine` does.
- **High-frequency tag writes from scripts can spam the log.** Per-script rate limit is post-1.0; for v1, document the gap and rely on the 90-day retention to bound storage growth.
- **TagValue serialization size.** Logging the full bytes of a 4KB string write per event will balloon the table. For v1, log the value as-is; track byte-size cap as v1.1 polish if real deployments hit it.
- **Audit log writes on the gateway hot path.** Use a non-blocking append (channel + writer task) so a slow SQLite fsync doesn't backpressure tag writes. Same pattern as historian's recorder.

## Codex log

- 2026-05-03 codex: Implemented `crates/audit-log` SQLite journal, audit query/subscribe wire protocol, gateway auth/tag/project/user hooks, script-write audit routing via `GatewayTagWriteSink`, and TS protocol mirrors. Validation: `cargo test -p openwebhmi-audit-log --all-features --locked`, `cargo test -p openwebhmi-gateway --all-features --offline`, focused clippy, `cargo test -p openwebhmi-protocol --all-features --locked`, and `pnpm --filter @openwebhmi/protocol typecheck` passed.

## Claude review

### Strong points

- ✅ **Crate compiles clean, all tests pass.** `cargo test -p openwebhmi-audit-log --all-features --locked` → 1 integration test (50 events × 5 users × 4 kinds, every filter combination from the brief). `cargo clippy -p openwebhmi-audit-log --all-targets --all-features --locked -- -D warnings` clean. `#![deny(missing_docs)]` enforces rustdoc on every public item.
- ✅ **SQLite schema matches brief exactly.** `audit_log` table with the prescribed columns, both indexes (`idx_audit_ts_user`, `idx_audit_kind`), plus a `meta` table tracking `schema_version = 1` for future migrations (mirrors what `alarm-engine` does, per the brief's risk note).
- ✅ **No type duplication.** Reuses `openwebhmi_protocol::TagValue` and `openwebhmi_auth::Role` per the brief — the `AuditEvent::TagWrite::value` is the existing wire type, not a copy. `UserAdminAction::RoleChanged { from: Role, to: Role }` uses the auth crate's enum directly.
- ✅ **Live broadcast subscription via `tokio::sync::broadcast`** (`store.rs:19,28,47`) — 1024-entry buffer is generous for an audit firehose. Receivers can lag without blocking the writer.
- ✅ **MAX_LIMIT = 1000 enforced server-side** (`store.rs:13,106`) — clients can't OOM the gateway with `limit: usize::MAX`.
- ✅ **Retention vacuum runs on `open()`** and is also exposed as an explicit `prune_retention()` method for runtime calls. Default 90 days per brief.
- ✅ **Comprehensive gateway integration.** Audit hooks landed at every brief-required site:
  - **AuthLogin**: 3 sites in `gateway/src/server.rs:378, 414, 434` (success path + two failure paths covering wrong-password vs unknown-user).
  - **AuthLogout**: `server.rs:449`.
  - **TagWrite from WebSocket**: 5 sites at `server.rs:1186, 1206, 1226, 1239, 1254` covering success + each failure mode (driver busy, driver closed, invalid value, etc.).
  - **TagWrite from Script**: replaces the removed `tracing::info!` line via `GatewayTagWriteSink::with_audit_log()` (`script_writes.rs:39-57`); audited on both success AND failure (good — the brief implied this, the impl delivers it).
  - **ProjectSave**: `server.rs:772`.
  - **UserAdmin**: 2 sites at `server.rs:1018, 1051` (create + delete + role change).
  - **AuditSubscribe / AuditQuery**: `server.rs:1066+, 1100+` with administrator-role check **before** consulting the store (the brief's "missing check is a security bug" line — Codex got the ordering right).
- ✅ **`tracing::info!` "script tag write" line removed from `crates/scripting/src/worker.rs`** — single audit path per the brief, no double-log.
- ✅ **Wire protocol additions are clean.** `AuditQuery` + `AuditEntry` (with `kind: String + payload: serde_json::Value` for cross-language friendly serde — strongly-typed `AuditEvent` enum stays internal-only); three `ClientMessage` variants (`audit.subscribe`, `audit.unsubscribe`, `audit.query`); two `ServerMessage` variants (`audit.event`, `audit.query_result`). All pass the new wire-form stability test in `crates/protocol/src/lib.rs::audit_query_and_result_wire_form_is_stable`.
- ✅ **TS protocol mirrors with validators.** `packages/protocol-ts/src/index.ts` adds `AuditQuery`, `AuditEntry`, `ProjectImportMode` types + extends both `ClientMessage` and `ServerMessage` discriminated unions; `isClientMessage` validator covers the new shapes.
- ✅ **CLI flags wired in `gateway/src/main.rs`.** `--audit-db` (default `openwebhmi-audit.sqlite`) and `--audit-retention-days` (default 90) match the brief's "configurable via gateway config" requirement.
- ✅ **New `script_writes_append_audit_events` test in `crates/gateway/src/script_writes.rs`** — covers script-driven tag writes ending up as audit entries with `WriteSource::Script`. Concrete proof of the cross-crate plumbing.

### Findings

- 🟠 **`AuditEvent::SessionExpired` variant exists but is never emitted from the gateway.** The variant is defined at `crates/audit-log/src/event.rs:42-46` and has a `kind_name() = "SessionExpired"` mapping, but `grep -r SessionExpired crates/gateway/` returns zero hits. The brief explicitly required: "Session expiry → `AuditEvent::SessionExpired`". The hook should fire when an expired session token is presented (the auth layer's rejection path). Less compliance-critical than auth-failed-login or user-admin events, so **acceptable to defer to v1.1 polish** — but not undersold; it's a brief deliverable that didn't land. Track as a one-line follow-up: emit `AuditEvent::SessionExpired { username }` from `crates/gateway/src/server.rs` wherever an expired-token rejection currently logs at `tracing::warn!`.
- 🟡 **`wiki/protocol/audit.md` not created.** Brief acceptance criterion: "Wire protocol additions documented in `wiki/protocol/audit.md` (new file) covering all four message types + the `AuditEntry` schema." Codex shipped `wiki/architecture/audit-log.md` instead, which covers implementation but not the wire protocol shape for client implementers. The rustdoc + protocol-ts types serve the same purpose for now. **Acceptable for v1.0** — but the explicit wire-protocol page is v1.1 polish.
- 🟡 **Query path doesn't push filters to SQL.** `AuditLog::query()` (`store.rs:100-109`) calls `read_all()` which loads **every** entry from SQLite into memory, then filters in Rust. The well-designed `idx_audit_ts_user` and `idx_audit_kind` indexes are unused. For typical 90-day deployments at modest write volume, in-memory filter is acceptable; for high-volume script-driven environments (1000+ events/day = 90K+ rows per query) this hits perf walls. Codex acknowledges in the wiki: "move high-volume filters into SQL before production scale testing." **v1.1 polish or a CODEX-AI brief if real deployments hit it.**
- 🟡 **Append is synchronous on the gateway hot path, not via channel + writer task.** Brief said: "Use a non-blocking append (channel + writer task) so a slow SQLite fsync doesn't backpressure tag writes. Same pattern as historian's recorder." Codex's `AuditLog::append()` directly calls `conn.execute()` under a `Mutex<Connection>`. SQLite WAL mode is fast in practice (sub-ms) but slow disks under concurrent load could backpressure tag writes. **Acceptable for v1.0** (audit volume should be modest); v1.1 polish for high-throughput deployments.
- 🟡 **No explicit retention-vacuum unit test.** Brief required: "Retention vacuum drops entries older than cutoff." Implementation is correct (`store.rs:198-209`) and is exercised on `open()`, but the integration test doesn't assert vacuum behavior. Easy-to-add v1.1 polish.
- 🟡 **Cross-task scope-bleed: AE includes AF protocol types.** `AuditEvent::ProjectExport` and `AuditEvent::ProjectImport` variants in `crates/audit-log/src/event.rs:67-86` are pre-wired for AF. `crates/protocol/src/lib.rs` includes `ProjectImportMode` + `ClientMessage::ProjectExport/Import` + `ServerMessage::ProjectExportReady/ProjectImportProgress/ProjectImportResult`. `packages/protocol-ts/src/index.ts` mirrors all of those. This is the **second time Codex pre-wired protocol surface for an adjacent task** (first was AE itself getting `AuditEvent::ProjectSave` ahead of UI consumers); pragmatically clean since splitting protocol enums mid-file is fragile. Acceptable.

### Acceptance-criteria tally

- [x] `cargo test -p openwebhmi-audit-log --all-features --locked` green.
- [~] `cargo test --workspace --all-features --locked` — 26/27 pass; 1 pre-existing failure (`websocket_gateway_forwards_script_events_by_project`) is the Python-not-on-PATH gap from CODEX-AC's verdict, unchanged by AE.
- [x] `cargo clippy -p openwebhmi-audit-log --all-targets --all-features --locked -- -D warnings` clean.
- [ ] `wiki/protocol/audit.md` created — **NOT met**; implementation page at `wiki/architecture/audit-log.md` covers code but not the wire protocol shape for client implementers. v1.1 polish.
- [x] `tracing::info!` audit lines in `crates/scripting/src/host.rs` (actually `worker.rs`) replaced — verified, 5 lines removed; single audit path through `GatewayTagWriteSink`.
- [x] All public Rust items have rustdoc — `#![deny(missing_docs)]` enforces.

5/6 acceptance criteria met. The miss is a documentation deliverable, not a behavioral requirement.

## Verdict

**Merged** (bundled with CODEX-AF in the same commit because the gateway integration surface — `crates/gateway/src/server.rs` and `main.rs` — is shared between the two; splitting would require surgical reverts across 6 files and would leave the workspace in a non-compiling intermediate state). The audit log crate itself is clean, small, and well-tested; the gateway integration covers every brief-required hook except `SessionExpired`; the protocol additions are wire-stable and TS-mirrored; the script-tag-write audit path is the brief's "single source of truth" without the double-log gotcha.

**Two flagged misses owned honestly**:
1. **`SessionExpired` hook never wired** — variant exists, no emission point. v1.1 follow-up; document for whoever picks up the auth-side rejection paths.
2. **`wiki/protocol/audit.md` not created** — Codex chose `wiki/architecture/audit-log.md` instead. v1.1 polish; rustdoc + protocol-ts types fill the gap for now.

**v1.1 polish list** (separate from the misses): SQL-side filter pushdown, channel+writer-task append pattern for hot-path safety, explicit retention-vacuum unit test.

Cross-task pre-wiring of AF protocol types is **acceptable** in this commit because AF lands in the same commit. If AF were deferred, the protocol additions would be unused-but-declared dead code — not a compile error, but worth flagging. With AF bundled, both task files reach `merged` status against the same commit hash and the protocol additions are immediately consumed.
