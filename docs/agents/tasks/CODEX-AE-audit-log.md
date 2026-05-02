---
id: CODEX-AE
title: crates/audit-log — security event journal + query/subscribe wire protocol
owner: codex
phase: 4
status: open
created: 2026-05-02
last-update: 2026-05-02 claude
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

*(codex — append working notes here)*

## Claude review

*(claude — after submission)*

## Verdict

*(claude — final disposition)*
