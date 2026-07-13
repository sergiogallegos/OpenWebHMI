---
id: CODEX-BG
title: Auth + connection hardening — login rate-limit/lockout, connection caps, message/subscription limits
owner: codex
phase: 4
status: open
created: 2026-07-12
last-update: 2026-07-12 claude [Fable 5]
---

# CODEX-BG — Auth + connection hardening (Tier-1 SECURITY)

## Brief

> **v1.0 blocker.** The gateway has no abuse limits. `AuthLogin` allows unlimited attempts, each burning a bcrypt verify on the blocking pool — a cheap DoS amplifier and an unthrottled credential-stuffing surface. The accept loop holds unauthenticated connections open indefinitely with no per-IP throttle, no account lockout, no cap on concurrent connections (the README's 50-client envelope is unenforced), no per-connection subscription cap, and no maximum WebSocket message size. Add per-IP/username login rate-limiting with temporary lockout, a configurable max concurrent connections, a max WS message size, a per-connection subscription cap, and an unauthenticated-handshake idle timeout. **Deterministic tests only — no sleeps; drive lockout windows with `tokio::time::pause`/`advance`.**

### Goal

Repeated failed logins from an IP/username trigger a temporary lockout after N attempts, so bcrypt verifies can't be spammed. Concurrent connections are capped at a configurable limit; connections beyond it are refused. Each WS message is bounded by a configurable max size; oversize frames close the connection. Each connection is capped at a configurable number of active subscriptions. An unauthenticated connection that doesn't complete its handshake within a timeout is dropped. All limits are configurable via CLI flags with sensible defaults matching the README's documented envelope (≈50 clients).

### Context to read first

- `crates/gateway/src/server.rs:464-484` — `ClientMessage::AuthLogin`. Every attempt runs `users.authenticate(...)` on `spawn_blocking` (bcrypt). No counting, no delay, no lockout. An attacker loops this to (a) brute-force and (b) saturate the blocking pool.
- `crates/gateway/src/server.rs:279-307` — the accept loop. `listener.accept()` in an unbounded loop; every accepted stream spawns `handle_connection` with no admission control. `tasks.try_join_next()` reaps finished tasks but never caps concurrency. Unauthenticated peers stay connected until they disconnect themselves.
- The subscription handling in `handle_connection` (search `Subscribe`/`Unsubscribe` message handling and the per-connection subscription set) — to find where a per-connection subscription cap slots in.
- The WS read side (`tokio_tungstenite` / the message read loop in `handle_connection`) — to find where a max-message-size limit applies. `tungstenite` has a config for max message/frame size; prefer configuring the library limit over hand-rolling if available.
- `crates/gateway/src/main.rs` `Args` — where to add `--max-connections`, `--max-message-bytes`, `--max-subscriptions-per-connection`, `--login-max-attempts`, `--login-lockout-secs`, `--handshake-timeout-secs` (names illustrative; match existing clap style).
- `crates/auth/` — `UserStore::authenticate` and whether any attempt-tracking state exists (it doesn't today). The lockout state is gateway-side (per-IP/username), not necessarily persisted.
- `README.md` — the "50 concurrent runtime clients per gateway" envelope; `docs/architecture.md:§8` lists "> 50 concurrent runtime clients" as out of the v1.0 envelope. The connection cap enforces that stated bound.
- CLAUDE.md testing discipline — "No `sleep()`/`setTimeout()`/wall-clock waits in tests. Use `tokio::time::pause()` + `advance()`." Lockout-window tests are the canonical case for this.

### Files to create / modify

1. **Modify** `crates/gateway/src/server.rs`:
   - **Login rate-limit + lockout:** track failed-attempt counts keyed by IP and/or username with a time window. After `login_max_attempts` failures, reject further attempts for `login_lockout_secs` **without** running bcrypt (the point is to stop the amplification). Successful login clears the counter. Emit an `AuditEvent` for lockout so it's observable. Use a monotonic clock source that `tokio::time` can pause in tests (`tokio::time::Instant`), not `std::time::Instant`/`SystemTime`.
   - **Connection cap:** an admission counter (e.g. an `Arc<Semaphore>` or `AtomicUsize`) checked in the accept loop; when at `max_connections`, refuse the new connection (close promptly, optionally with a "server busy" close code) rather than spawning a handler. Release on connection end.
   - **Max message size:** configure the WS reader's max message/frame size (`tungstenite` `WebSocketConfig`) to `max_message_bytes`; an oversize frame closes the connection cleanly.
   - **Per-connection subscription cap:** track active subscriptions per connection; reject a `Subscribe` that would exceed `max_subscriptions_per_connection` with an error, don't silently drop.
   - **Handshake/idle timeout:** an unauthenticated connection that hasn't authenticated within `handshake_timeout_secs` is dropped. Use `tokio::time::timeout` around the pre-auth phase.
2. **Modify** `crates/gateway/src/main.rs`: add the CLI flags with defaults (`--max-connections` default ≈ 50 to match the envelope; message/subscription/lockout defaults chosen conservatively) and thread them into the server config struct.
3. **Do NOT**:
   - Persist lockout state to disk or add a distributed rate-limiter — in-memory per-gateway is the v1.0 scope (single-gateway per architecture §8).
   - Add a full IP-reputation / fail2ban system — a simple counting limiter with a lockout window is the bar.

### Behavior

- After `login_max_attempts` failed logins for an IP/username within the window, further attempts are rejected immediately (no bcrypt) until the lockout expires; a successful login before the threshold clears the count.
- Connection number `max_connections + 1` is refused promptly; after an existing connection closes, a new one is admitted.
- A WS message exceeding `max_message_bytes` closes the connection instead of being buffered unboundedly.
- A `Subscribe` that would exceed the per-connection subscription cap is rejected with an error; existing subscriptions are unaffected.
- An unauthenticated connection idle past the handshake timeout is dropped.
- All limits are configurable; defaults reflect the README's 50-client envelope.

### Test requirements

- Add tests to the gateway crate's existing server test module. **Bind test listeners to `127.0.0.1:0`** and read the port back.
  - **Lockout:** N failed logins → the (N+1)th is rejected *without* invoking `authenticate` (assert via a counter/mock or by timing-independent observation that bcrypt wasn't called). Advance `tokio::time` past the lockout window with `pause()`/`advance()` and confirm attempts are accepted again. **No `sleep`.** This test must fail against the pre-fix code (which never locks out).
  - **Connection cap:** open `max_connections` connections, assert the next is refused; close one, assert a new one is admitted.
  - **Max message size:** send an oversize frame, assert the connection closes and the server didn't buffer it unboundedly.
  - **Subscription cap:** subscribe up to the cap, assert the next `Subscribe` is rejected with an error, existing ones intact.
  - **Handshake timeout:** open a connection, never authenticate, advance time past the timeout with `pause()`/`advance()`, assert it's dropped.
- Every timing-dependent test uses `tokio::time::pause()` + `advance()`; zero wall-clock waits. Full Rust matrix clean.

### Acceptance criteria

- [ ] Failed logins are counted per IP/username; after `login_max_attempts` a lockout blocks further attempts (skipping bcrypt) for `login_lockout_secs`; success clears the counter; lockout emits an audit event.
- [ ] `--max-connections` caps concurrent connections (default ≈ 50); over-limit connections refused; slots released on close.
- [ ] `--max-message-bytes` bounds WS message size; oversize closes the connection.
- [ ] `--max-subscriptions-per-connection` caps subscriptions; over-limit `Subscribe` rejected with an error.
- [ ] `--handshake-timeout-secs` drops unauthenticated idle connections.
- [ ] All limits configurable via CLI with documented defaults matching the README envelope.
- [ ] Lockout + handshake-timeout tests use `tokio::time::pause`/`advance` (no `sleep`); test listeners bind `127.0.0.1:0`; the lockout test fails against pre-fix code.
- [ ] Lockout uses `tokio::time::Instant` (pausable), not `SystemTime`/`std::Instant`.
- [ ] No `panic!`/`expect`/`unwrap` on the connection/auth path; full Rust matrix clean; no new `#[allow]` (use `#[expect]` + reason).

### Out of scope

- **Persistent / distributed rate-limiting.** In-memory per-gateway only (single-gateway per architecture §8).
- **IP reputation / fail2ban / CAPTCHA.** A counting limiter with a lockout window is the v1.0 bar.
- **Global request-rate limiting** beyond login and connection admission (e.g. per-message-type QPS) — a possible v1.1 follow-up.
- **Changing the password hash (bcrypt) or auth algorithm** — separate concern.
- **The backup HTTP channel's body cap** — CODEX-BE owns that; this brief is the WS/login surface.

### Risks / gotchas

- **`tokio::time::Instant`, not `SystemTime`.** Lockout windows must be measured with a clock `tokio::time::pause()` can control, or the tests can't be deterministic without sleeping. Using `SystemTime`/`std::time::Instant` forces a wall-clock wait — which CLAUDE.md forbids. This is the single most common way this task goes wrong; get the clock source right first.
- **The lockout must skip bcrypt.** The DoS amplification is the bcrypt verify. If the limiter counts attempts but still runs `authenticate` before rejecting, it doesn't stop the amplification. Reject *before* `spawn_blocking(authenticate)`.
- **Keying the limiter.** Per-IP protects against distributed username spraying against one account; per-username protects one account across IPs. Do both (compound key or two counters) if practical; at minimum document which and why. Beware NAT: purely per-IP can lock out many legitimate users behind one NAT — pairing with per-username mitigates.
- **Connection-cap release on every exit path.** Whatever counts admissions must decrement on *every* way a connection ends (clean close, error, panic-in-task, shutdown). A `Semaphore` permit held by the connection task (released on drop) is the leak-proof shape; a manual `AtomicUsize` needs a guard type so it can't leak on an early return.
- **Message-size limit via the library.** `tungstenite`'s `WebSocketConfig` has `max_message_size`/`max_frame_size`. Prefer configuring those over hand-rolling a size check after buffering — the library enforces before buffering the whole message, which is the point.
- **Subscription cap error, not silent drop.** Rejecting a `Subscribe` over the cap must send an error the client can see, per the repo's honesty posture — a silently ignored subscription is a debugging nightmare.
- **Defaults match the stated envelope.** `--max-connections` default should reflect the README's 50-client bound so behavior matches documentation out of the box; note the number in the Codex log.
- **Handshake timeout vs slow legit clients.** Set the default generously enough that a real login round-trip (including a bcrypt verify under load) completes comfortably. Too tight and legitimate logins drop; document the default's rationale.

## Codex log

## Claude review

## Verdict
