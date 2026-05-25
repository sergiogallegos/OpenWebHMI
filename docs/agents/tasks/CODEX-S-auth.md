---
id: CODEX-S
title: crates/auth — local users + roles + JWT sessions + per-view ACLs
owner: codex
phase: 3
status: merged
created: 2026-04-27
last-update: 2026-04-27 20:06 claude
merge-commit: 7eff30a
---

# CODEX-S — `crates/auth`

## Brief

### Goal

Authentication and authorization for the gateway. Local username/password (bcrypt), JWT session tokens, four built-in roles (`Administrator | Designer | Operator | Viewer`), per-view ACLs. Login UI in runtime-web; user management in the designer.

This is **security-critical** work. Don't roll your own crypto. Don't store passwords plain.

### Context to read first

- `docs/architecture.md` §4.9 (Auth) and §7 (Security boundaries).
- `docs/roadmap.md` Phase 3 — auth deliverables.
- The existing gateway WS handshake — currently anonymous; this task adds an auth phase before any other ClientMessage is accepted.

### Files to create / modify

- `crates/auth/Cargo.toml`
- `crates/auth/src/lib.rs`
- `crates/auth/src/users.rs` — bcrypt hashing, user CRUD.
- `crates/auth/src/sessions.rs` — JWT issue + verify (use `jsonwebtoken` crate).
- `crates/auth/src/roles.rs` — role + permission model.
- `crates/auth/src/acl.rs` — per-view ACL evaluation.
- `crates/auth/tests/` — unit tests.
- `crates/protocol/src/lib.rs` — `auth.login`, `auth.logout`, `auth.result`, `auth.required` error code.
- `packages/protocol-ts/src/index.ts` — mirror.
- `crates/gateway/src/server.rs` — gate every ClientMessage on a valid session except `auth.login` and `ping`.
- `apps/runtime-web/src/modules/Login.tsx` — login form; stores JWT in `localStorage`.
- `apps/designer/src/modules/UserAdmin.tsx` — add/remove users, assign roles, edit per-view ACLs.

Add `crates/auth` to workspace `members`. Add `jsonwebtoken`, `bcrypt`, `uuid` to workspace deps.

### Roles + permissions

| Role | Read tags | Write tags | Read views | Author project | Manage users |
|---|:-:|:-:|:-:|:-:|:-:|
| `Administrator` | ✅ | ✅ | ✅ | ✅ | ✅ |
| `Designer` | ✅ | ✅ | ✅ | ✅ | ❌ |
| `Operator` | ✅ | ✅ (per-view ACL) | ✅ | ❌ | ❌ |
| `Viewer` | ✅ | ❌ | ✅ | ❌ | ❌ |

Per-view ACLs override the role for **write tags** specifically. A view can name `allowedRoles: ["Administrator", "Designer"]` to deny operators from writing tags within it.

### Wire protocol

```rust
ClientMessage::AuthLogin { username, password },
ClientMessage::AuthLogout,

ServerMessage::AuthResult {
    session_token: Option<String>,    // JWT; None on failure
    user_id: Option<String>,
    roles: Vec<String>,
    error: Option<String>,
},

// New error code:
ServerMessage::Error { code: "auth.required", ... }
ServerMessage::Error { code: "auth.forbidden", ... }
```

WebSocket-level: client sends `Authorization: Bearer <jwt>` as a query param `?token=<jwt>` on the WS URL. Gateway extracts it during the upgrade and binds the session to the connection. Stale JWTs return `auth.required` and the gateway closes the connection.

### TLS

Gateway accepts a `--tls-cert` + `--tls-key` flag pair. When supplied, binds `wss://` instead of `ws://`. Use `rustls` (no openssl). For dev, ship a script that generates a self-signed cert.

### Test requirements

- Bcrypt round-trip: hash a password, verify it matches; reject the wrong password.
- JWT issue + verify with the project secret; reject tampered tokens; reject expired tokens.
- ACL: an Operator can write tags on a view with no ACL; cannot write on a view with `allowedRoles: ["Designer"]`.
- Gateway integration test: connect without a token → `auth.required` on first non-login message; connect with a valid token → succeeds.

### Acceptance criteria

- [ ] `cargo test -p openwebhmi-auth` green.
- [ ] `cargo test --workspace --all-features --locked` green; gateway integration tests now seed a default user.
- [ ] First-run gateway creates an `admin` user with a printed-once password (logged at INFO so the operator can capture it).
- [ ] `apps/runtime-web` requires login before opening any view.
- [ ] `apps/designer/src/modules/UserAdmin.tsx` lets an Administrator add/remove users and edit roles.
- [ ] **TLS path verified** with a self-signed cert: `wss://` connection succeeds when `--tls-cert/--tls-key` are supplied.

### Out of scope

- SSO / OIDC / SAML (post-1.0).
- AD / LDAP (post-1.0).
- 2FA / MFA (post-1.0).
- Per-tag ACLs (post-1.0; v1 is per-view).
- Password reset email flow (post-1.0; admin can manually reset via UserAdmin).
- Audit log export (Phase 4).

### Risks / gotchas

- **Don't roll your own crypto.** Use `bcrypt` and `jsonwebtoken` crates. Do not implement HMAC by hand. Do not invent a session token format.
- **Bcrypt cost**: pin `bcrypt::DEFAULT_COST` (currently 12). Don't hardcode a smaller cost for "performance".
- **JWT secret rotation**: the gateway's signing secret lives in `gateway.toml` (or env var). Document rotation procedure. Phase 3 doesn't implement rotation; just don't paint into a corner.
- **Don't log passwords**, even at trace level. Don't include them in error messages.
- **TLS cert paths must be readable by the gateway process only**. Document the file-permissions requirement.
- **Default admin password is printed once** — clearly log "ROTATE THIS IMMEDIATELY". A `--admin-password` CLI flag overrides for ops scripts.

## Codex log

*(codex — append working notes here)*

### 2026-04-27 19:27  codex
Started. Taking CODEX-S next because auth gates every websocket operation and is the highest-leverage Phase 3 baseline. Reading architecture security notes, gateway websocket handshake/message loop, protocol mirrors, and runtime/designer module structure before editing.

### 2026-04-27 19:52  codex
Submitted. Added `crates/auth` with bcrypt local users, JWT sessions, role permissions, and per-view write ACL evaluation. Wired gateway first-run admin bootstrap, `auth.login`/`auth.logout`, query-token sessions, role-gated websocket messages, user-management messages, and rustls `--tls-cert/--tls-key` serving. Added runtime login, designer login + UserAdmin, protocol Rust/TS mirrors, `View.allowedRoles`, and a local self-signed TLS helper script. Verification: `cargo test -p openwebhmi-auth`; `cargo test --workspace --all-features --locked`; `cargo clippy --workspace --all-targets --all-features -- -D warnings`; `cargo fmt --all -- --check`; `pnpm -r typecheck`; `pnpm -r test`; `pnpm -r build`; WSS smoke with `scripts/dev-self-signed-cert.sh` + OpenSSL returned `HTTP/1.1 101 Switching Protocols`.

## Claude review

*(claude — after submission)*

## Verdict

*(claude — final disposition)*
