---
id: CODEX-BE
title: Backup HTTP side-channel hardening — pre-auth body cap, header-first auth, TLS parity
owner: codex
phase: 4
status: open
created: 2026-07-12
last-update: 2026-07-12 claude [Fable 5]
---

# CODEX-BE — Backup HTTP side-channel hardening (Tier-1 SECURITY)

## Brief

> **v1.0 blocker.** The backup HTTP side-channel reads an unauthenticated request's entire body — sized by an uncapped `Content-Length` — into a single `vec![0u8; content_length - body.len()]` allocation *before* it checks the bearer token, so one unauthenticated POST with a huge `Content-Length` triggers a multi-GiB allocation (OOM / abort). It also never wraps its listener in TLS even when the WebSocket listener is TLS-configured, so admin bearer tokens and full project archives transit cleartext. Cap the body before allocating, verify auth from headers before reading the body, achieve TLS parity with the WS listener, and correct the stale "intentionally unauthenticated WS" comment that mislabels what is now an authenticated HTTP channel. **Scope-locked to the backup channel.**

### Goal

An unauthenticated POST to the backup endpoint with an oversized `Content-Length` is rejected (413) before any large allocation. A request without a valid Administrator bearer token is rejected (401/403) *before* its body is read. When the gateway is started with `--tls-cert`/`--tls-key`, the backup listener speaks TLS too (or refuses to serve plaintext, or at minimum logs a loud startup warning — decide per the sub-decision below), so tokens and archives never cross the wire in cleartext. The stale comment at `main.rs:138` no longer claims the channel is an unauthenticated WS endpoint.

### Context to read first

- `crates/gateway/src/server.rs:1704-1765` — `read_http_request`. Headers are bounded (`32 * 1024` at 1719), but the body loop at 1743-1756 parses `Content-Length` with `.unwrap_or(0)` and **no upper bound**, then `vec![0u8; content_length - body.len()]` at 1750 allocates whatever the attacker declared. This runs for every request before any handler sees it.
- `crates/gateway/src/server.rs:1628-1646` — `handle_backup_restore` (name may differ; the restore handler). The bearer check is `bearer_session(&auth, &request)` at 1632 and the Administrator role check at 1643 — **both after** `read_http_request` has already read the full body into `request.body`. Auth is gated on data already accepted and allocated.
- `crates/gateway/src/server.rs:1811-1815` — `bearer_session`: reads `authorization` header, strips `Bearer `, verifies. This is header-only and cheap; it can and must run before the body is read.
- `crates/gateway/src/server.rs:213` — `serve_backup_http`, the accept loop for the side-channel. This is where a `TlsAcceptor` would wrap accepted streams, mirroring the WS listener.
- `crates/gateway/src/main.rs:138-154` — the spawn site. Line 138's comment "TODO Phase 3 auth/TLS: this Phase 0 endpoint is intentionally unauthenticated WS." is stale on two counts: the channel is HTTP (not WS) and it is now authenticated (bearer + Administrator). The listener is `TcpListener::bind` at 143 with no TLS, even when `tls_config` produced a `ServerConfig` for the WS path.
- `crates/gateway/src/main.rs:569-600` (`tls_config`) — how the WS `ServerConfig`/`TlsAcceptor` is built. Reuse the same `Arc<ServerConfig>` for the backup listener; do not build a second, divergent TLS config.
- [`docs/agents/notes/mqtt-tls-in-ci.md`](../notes/mqtt-tls-in-ci.md) — TLS-in-CI caveats; real TLS validation is a maintainer-run manual-smoke gate, so the automated tests here focus on the body-cap and header-first-auth behavior, and TLS parity is asserted structurally (the acceptor is wired) plus called out for manual smoke.
- `docs/architecture.md` §7 — "TLS via rustls in production. WSS + HTTPS only." The backup channel currently violates this.

### Files to create / modify

1. **Modify** `crates/gateway/src/server.rs`:
   - Add a bounded-body read to `read_http_request` (or a variant used by the backup path). Introduce a `max_body_bytes: usize` parameter (or a const with a configurable override). If declared `Content-Length` exceeds the cap, stop before allocating and signal an over-limit condition; the caller responds `413 Payload Too Large`. Default cap: a few MiB for general requests, with a higher explicit cap for the backup archive path (project archives are legitimately larger) — pick concrete numbers and make the backup cap configurable via a CLI flag (e.g. `--backup-max-bytes`, default e.g. 256 MiB) so large real projects still restore.
   - Never allocate `vec![0u8; content_length]` up front for an untrusted `content_length`. Read in bounded chunks (a fixed-size buffer, appending until `content_length` or the cap), or reject before the first large allocation. The existing 1024-byte chunk loop shape for headers is the pattern to mirror for the body.
   - Restructure the restore handler so `bearer_session` + Administrator-role check run on the **parsed headers before the body is consumed**. Concretely: parse the request line + headers, authenticate, and only then read (capped) body bytes. A 401/403 must be answerable without having read the body.
2. **Modify** `crates/gateway/src/server.rs` `serve_backup_http` (213): accept an optional `TlsAcceptor`. When present, wrap each accepted `TcpStream` before handing it to the HTTP request reader, exactly as the WS accept loop does.
3. **Modify** `crates/gateway/src/main.rs:138-154`:
   - Pass the WS `TlsAcceptor` (from `tls_config`) into `serve_backup_http` when TLS is configured. **Sub-decision (state it in the Codex log and pick one):** (a) always TLS-wrap the backup listener when TLS is on; (b) refuse to bind the backup listener in plaintext when TLS is on (fail-closed); or (c) TLS-wrap when possible and, if for some reason plaintext is unavoidable, emit a loud startup `warn!`. Preference order: (a) then (b); (c) only as a documented fallback. Cleartext admin tokens when the operator explicitly asked for TLS is the bug — don't leave that reachable silently.
   - Rewrite the stale comment at 138 to describe the channel accurately: authenticated (Administrator bearer) HTTP backup/restore side-channel, TLS-parity with the WS listener.
4. **Do NOT**:
   - Rewrite the hand-rolled HTTP parser into a framework (hyper/axum) — out of scope; harden the existing parser.
   - Change the backup archive format or the restore semantics (BC owns the manifest `project_id` ordering fix; coordinate but don't duplicate).

### Behavior

- Unauthenticated POST with `Content-Length: 5000000000` → rejected before any multi-GiB allocation; response `413`. Peak allocation bounded by the cap, not the declared length.
- POST with no/invalid bearer token → `401`; with a valid non-Administrator token → `403`; **the body is not read** in either case (the handler answers from headers).
- Valid Administrator token, legitimately large archive under `--backup-max-bytes` → restore proceeds as today.
- Gateway started with `--tls-cert`/`--tls-key` → backup listener is TLS (option a) / refuses plaintext (option b) / warns loudly (option c). No cleartext admin token when TLS was requested.
- `main.rs:138` comment accurately describes an authenticated HTTP channel.

### Test requirements

- Add tests to the gateway crate's existing server/backup test module. Bind test listeners to **`127.0.0.1:0`** and read the assigned port back — no hardcoded ports.
  - **Body cap:** a raw TCP client sends valid request line + headers with an oversized `Content-Length` and no auth; assert the connection gets a `413` (or is closed with the over-limit signal) and — the load-bearing assertion — that the server did not allocate the declared size. Practically: drive `read_http_request`/its variant directly with a large `Content-Length` and a tiny actual body and assert it returns the over-limit error rather than looping to allocate. This test **must fail against the pre-fix code** (which would try to allocate / block reading the full body).
  - **Header-first auth:** a request with a bad bearer token and a non-trivial `Content-Length` gets `401`/`403` without the body being read — assert by sending headers, then *not* sending the body, and confirming the auth rejection arrives (the pre-fix code blocks waiting for the body, so this also fails without the fix).
  - **TLS wiring:** a structural test that `serve_backup_http` accepts and uses a `TlsAcceptor` when provided (construct with a self-signed cert as the WS TLS tests do, if such a fixture exists; otherwise assert the acceptor is threaded through). Note in the Codex log that end-to-end TLS on the backup channel is a maintainer manual-smoke item per `mqtt-tls-in-ci.md`.
- No `sleep`/`setTimeout`/wall-clock waits; use deterministic reads/channels. Full Rust matrix clean.

### Acceptance criteria

- [ ] `read_http_request` (backup path) enforces a configurable max body size and never allocates the untrusted `Content-Length` up front; oversize → `413` before large allocation.
- [ ] `--backup-max-bytes` (or equivalent) flag exists with a sensible default that admits real project archives.
- [ ] Bearer + Administrator-role check runs on headers before the body is read; `401`/`403` answerable without consuming the body.
- [ ] `serve_backup_http` TLS-wraps its listener when the gateway is TLS-configured (or fails-closed / warns per the documented sub-decision); no cleartext admin token when TLS was requested.
- [ ] `main.rs:138` comment rewritten to describe the authenticated HTTP backup channel accurately.
- [ ] Body-cap and header-first-auth regression tests each fail against the pre-fix code; test listeners bind `127.0.0.1:0`.
- [ ] No `panic!`/`expect`/`unwrap` on the request path; oversize/parse failures are errors/responses, not panics.
- [ ] Full Rust matrix clean; TLS end-to-end noted as manual-smoke.

### Out of scope

- **Replacing the hand-rolled HTTP parser** with hyper/axum — harden in place.
- **Backup archive format / restore semantics** — BC owns the manifest ordering fix; this brief is transport + resource hardening.
- **Rate-limiting the backup endpoint** — CODEX-BG covers connection/attempt limits broadly; a backup-specific limiter can be a follow-up.
- **Streaming the archive to disk instead of buffering in memory** — a good v1.1 improvement, but the v1.0 bar is a hard cap, not streaming.
- **Real end-to-end TLS validation** — maintainer manual-smoke per `mqtt-tls-in-ci.md`.

### Risks / gotchas

- **The allocation is the vuln, not the read.** Even a chunked read loop is dangerous if it pre-sizes a `Vec` to `content_length`. Cap *before* sizing anything to the declared length. `Vec::with_capacity(content_length)` is as bad as `vec![0u8; content_length]`.
- **Header-first auth means restructuring the read.** Today `read_http_request` returns a fully-populated `HttpRequest` (headers + body) and the handler authenticates after. Split it: read+parse headers, let the handler authenticate, then read the capped body only if authorized. Keep the split minimal; don't rewrite the parser.
- **Chunked transfer encoding.** The current parser only honors `Content-Length`. If it doesn't support `Transfer-Encoding: chunked`, that's fine (reject/ignore), but make sure an attacker can't bypass the cap via a header the parser mis-handles. Reject requests you can't bound.
- **TLS config reuse.** Build one `ServerConfig`/`TlsAcceptor` in `tls_config` and share the `Arc` with both listeners. A second, subtly-different TLS config is a divergence bug waiting to happen.
- **Fail-closed vs warn.** If the operator passed `--tls-cert`/`--tls-key`, they asked for encryption everywhere. Serving the backup channel in cleartext then is a silent downgrade. Prefer TLS-wrap or fail-closed; a warning-only path is the weakest option and should be a documented fallback, not the default.
- **`127.0.0.1:0` in tests.** Per CLAUDE.md, no hardcoded ports — bind to port 0 and read `local_addr()` back. The existing gateway tests already do this; copy the neighbor.
- **Honesty in the log.** State plainly that end-to-end TLS on the backup channel was verified structurally (acceptor wired) but not exercised against a real TLS client in CI, and is a manual-smoke gate.

## Codex log

## Claude review

## Verdict
