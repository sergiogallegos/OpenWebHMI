---
id: CODEX-BU
title: Gateway lifecycle hardening — non-fatal accept, reliable RPC replies, WS keepalive, backup-download TTL sweep
owner: codex
phase: 4
status: open
created: 2026-07-12
last-update: 2026-07-12 claude [Fable 5]
---

# CODEX-BU — Gateway lifecycle hardening

## Brief

> Four related connection-lifecycle robustness bugs in `crates/gateway/src`. **(a)** Every accept loop treats a transient accept error as fatal — `listener.accept().await.context("accept failed")?` (`server.rs:224`, `server.rs:280`, `main.rs:478`) — so a temporary `EMFILE`/`ENFILE` under fd exhaustion or `ECONNABORTED` permanently kills the listener and takes the gateway offline. **(b)** `try_send_message` (`server.rs:2077-2085`) pushes *every* outbound message — including RPC responses like `auth.result`, `history.result`, `project.save_result`, and error replies — through `try_send` on the 256-slot channel and silently drops on full (warn only); under a tag-update burst a client's request reply is dropped and it waits forever. **(c)** There is no server-initiated WS ping/keepalive, no idle timeout, and no handshake timeout (`server.rs:329-338`, `409-426`): a silently-dead peer (pulled cable) keeps its connection, per-tag forwarder tasks, and broadcast receivers alive until the OS TCP timeout, and `accept_hdr_async` with no deadline invites slowloris. **(d)** `BACKUP_DOWNLOADS` entries (`server.rs:47`, insert `1038-1049`, remove `1575-1581`) are only removed on an actual download; a token that expires unused leaks its entire archive `Vec<u8>` in a process-global map forever. Fix all four; they share the connection-lifecycle surface.

### Goal

The gateway survives transient accept failures without going offline; request/response replies are never silently dropped under load; dead peers and their per-connection tasks are reaped by a server keepalive/idle timeout; and expired-but-undownloaded backup tokens are swept so archives don't leak.

### Context to read first

- `crates/gateway/src/server.rs:224` and `:280` — the two `run_server` accept loops (WS listener and, second one, the backup HTTP listener path). Both `.context("accept failed")?`.
- `crates/gateway/src/main.rs:478` — the third accept loop (`.context("accept failed")?`).
- `crates/gateway/src/server.rs:2077-2085` — `try_send_message`: `try_send` + drop-on-`Full` (warn) + swallow-on-`Closed`. Callers span the file (grep `try_send_message` — ~40 sites); the RPC-reply sites include `auth.result`, `history.result`, `project.save_result`, `ProjectExportReady`, and the various `send_error`/`ServerMessage::Error` replies.
- `crates/gateway/src/server.rs:329-380` — the WS handshake (`accept_hdr_async`, no timeout) and the writer task that owns the split `sink`; the writer is where server-initiated pings would be emitted.
- `crates/gateway/src/server.rs:409-431` — the reader `select!` loop; idle timeout and pong tracking hook in here.
- `crates/gateway/src/server.rs:40-41` — `OutboundTx = mpsc::Sender<ServerMessage>`, `OUTBOUND_CAPACITY = 256`.
- `crates/gateway/src/server.rs:47` — `static BACKUP_DOWNLOADS`; `49` — `struct BackupDownload` (already has `expires_at: Instant` and `peer_ip`).
- `crates/gateway/src/server.rs:1036-1069` — token insert (sets `expires_at = now + 5 min`).
- `crates/gateway/src/server.rs:1575-1596` — download path: `downloads.remove(token)` then checks `expires_at <= Instant::now()` (returns 410). Expiry is enforced *only if the client shows up*; a no-show leaks forever.

### Files to create / modify

1. **Modify** the three accept loops (`server.rs:224`, `server.rs:280`, `main.rs:478`) — non-fatal accept:
   - Replace `?`-on-accept-error with log-and-continue plus a brief backoff (small bounded delay via `tokio::time::sleep` in the *production* loop is acceptable — this is not test code; the no-sleep rule is a test rule). Never return the listener on a transient accept error.
   - Distinguish fatal from transient if practical (e.g. a listener that is genuinely gone), but the safe default is: log the error, back off briefly, keep accepting.
   - Keep the loops cancel-safe with the existing `shutdown`/`CancellationToken` select arms.

2. **Modify** the outbound-send path (`server.rs`) — reliable lane for replies:
   - Introduce a reliable send for request/response and error replies: an `async` `send().await` on the 256-slot channel (awaits capacity instead of dropping), used by the RPC-reply and `send_error` sites. Keep `try_send_message` (drop-on-full) **only** for `tag.update` report-by-exception broadcasts, where dropping a stale sample under burst is correct.
   - Practical shape: add a `send_message(out_tx, msg).await` (reliable) alongside the existing `try_send_message` (lossy). Audit the ~40 call sites and route replies through the reliable one; leave tag-update fan-out on the lossy one. Handle `Closed` by breaking the connection loop, not by unwrapping.
   - The writer task and channel capacity stay; this is about which lane each message uses. Note: reliable `send().await` from within the reader loop can backpressure the reader if the writer stalls — that is acceptable and desirable (it applies flow control) as long as it composes with the shutdown select.

3. **Modify** the WS connection (`server.rs:329-431`) — keepalive + timeouts:
   - **Handshake timeout**: wrap `accept_hdr_async` in `tokio::time::timeout(HANDSHAKE_TIMEOUT, …)`; on elapse, drop the connection (slowloris guard).
   - **Server ping**: emit a periodic `Message::Ping` from the writer/reader machinery on a named interval (`PING_INTERVAL`).
   - **Idle timeout**: track the last inbound activity (any frame, including pong); if no activity within `IDLE_TIMEOUT` (a small multiple of `PING_INTERVAL`), close the connection and let the existing teardown abort the per-tag forwarder `AbortHandle`s (`subscriptions` et al., lines 402-406). Add the timeout as another `select!` arm in the reader loop.
   - Add named `const`s for the three durations. No magic numbers.

4. **Modify** the backup-download registry (`server.rs`) — TTL sweep:
   - Evict expired entries so an unused token's archive is reclaimed. Simplest correct approach: sweep expired entries on every insert (iterate and drop `expires_at <= now` before inserting the new token) — bounded, no extra task. Optionally also a periodic sweep task, but insert-time eviction alone stops the unbounded leak.
   - Keep the download-time `remove` + `expires_at` check unchanged.

5. **Modify** the `crates/gateway` tests (extend existing test modules; do not fragment):
   - Transient accept error does not kill the listener: inject a simulated transient accept failure and assert the loop keeps accepting a subsequent connection.
   - An RPC reply is not dropped under a full tag-update queue: saturate the outbound channel with tag updates, issue a request, assert the reply is eventually delivered (reliable lane) rather than dropped.
   - Expired backup token is evicted: insert a token, advance time past its TTL (`tokio::time::pause()` + `advance()`), trigger an insert (or sweep), assert the expired archive is gone from the map.
   - Dead peer is reaped: a peer that stops responding to pings is closed after `IDLE_TIMEOUT`, and its per-tag forwarder tasks are aborted — driven deterministically with `tokio::time` pause/advance, no wall-clock waits.

### Behavior

- Accept loops log-and-continue on transient errors; the gateway stays online through fd pressure.
- `auth.result` / `history.result` / `project.save_result` / error replies are delivered even under a tag-update burst.
- `tag.update` broadcasts still drop-on-full (report-by-exception semantics preserved).
- A pulled-cable peer is detected by ping/idle timeout and its resources reclaimed without waiting for OS TCP timeout; a slowloris handshake is dropped at `HANDSHAKE_TIMEOUT`.
- An unused backup token's archive is evicted at/after its TTL; no unbounded growth of `BACKUP_DOWNLOADS`.

### Test requirements

- **No `sleep()` / wall-clock waits in tests.** Use `tokio::time::pause()` + `advance()` for all timeout-driven tests (keepalive, idle, TTL). Drive the dead-peer scenario with a controllable duplex/stream, not real sockets where avoidable; if a listener is needed, bind `127.0.0.1:0` and read back the port.
- Each of the four fixes has a regression test that **fails against the pre-fix code** (accept-`?` kills the loop; reply dropped on full; token never evicted; dead peer never reaped). Confirm fail-before / pass-after for each; note in the Codex log.
- Full validation matrix clean (`cargo build`, `cargo clippy … -D warnings`, `cargo test --workspace --all-features --locked`, `cargo doc --workspace --no-deps`).

### Acceptance criteria

- [ ] All three accept loops (`server.rs:224`, `:280`, `main.rs:478`) log-and-continue with backoff on transient errors; none returns on a transient accept failure.
- [ ] A reliable reply lane (`send().await`) carries RPC responses and error replies; `tag.update` fan-out keeps drop-on-full. `Closed` handled without `unwrap`.
- [ ] WS handshake timeout, server ping interval, and idle timeout added as named consts; idle/dead peers are closed and their per-tag `AbortHandle`s aborted.
- [ ] `BACKUP_DOWNLOADS` sweeps expired entries (at minimum on insert); unused tokens no longer leak their archives.
- [ ] Four regression tests, each failing pre-fix and passing post-fix, all deterministic (no sleeps).
- [ ] No `unwrap`/`expect`/`panic!` on production paths; `#[expect]` over `#[allow]` for any new lint suppression; full matrix clean.

### Out of scope

- Rewriting the WS framing layer or swapping `tokio-tungstenite`.
- Backpressure redesign of the tag-update fan-out beyond routing replies to a reliable lane (the report-by-exception drop semantics stay).
- Making `BACKUP_DOWNLOADS` a non-global (dependency-injected) registry — a broader refactor; insert-time sweep is the bounded fix. Note the global as tech debt if worth a follow-up.
- Configurable timeout values via CLI — named consts are sufficient for this brief (CLI plumbing can be a later brief; coordinate with CODEX-BV's config surface if it lands first).
- The write-queue replay hazard (CODEX-BT) and the write-honesty work (CODEX-BS) — separate briefs.

### Risks / gotchas

- **Sleep is allowed in the production accept-backoff, forbidden in tests.** The no-`sleep` rule is a *test* discipline (flaky-test prevention). A bounded backoff in the real accept loop is correct engineering. Don't conflate the two.
- **Reliable send can backpressure the reader.** Routing replies through `send().await` means a stalled writer will pause the reader loop. That is intended flow control, but it must still compose with the `shutdown.cancelled()` select arm so a shutdown isn't blocked behind a wedged client. Verify the reader's select keeps the shutdown arm live.
- **Audit all ~40 `try_send_message` sites** before flipping them. Some are genuinely lossy-appropriate (tag updates, snapshots-by-exception). The rule: request/response and error replies → reliable; report-by-exception broadcasts → lossy. When unsure whether a site is a reply or a broadcast, treat a `request_id`-bearing message as a reply.
- **Ping/pong bookkeeping.** `tokio-tungstenite` auto-responds to inbound pings; for *server-initiated* liveness you must send pings and observe inbound pongs (or any inbound frame) as the liveness signal. Track last-activity in the reader loop; don't assume the library tracks it for you.
- **`BACKUP_DOWNLOADS` lock scope.** The insert-time sweep runs under the same `Mutex` lock as the insert — keep the critical section short (drop expired, insert) and don't hold the lock across an `.await`.
- **Determinism for the dead-peer test** is the hardest part — model the peer with an in-memory duplex stream you can stop feeding, then `tokio::time::advance` past `IDLE_TIMEOUT` and assert the connection task ends and the tag subscriptions are aborted. No real socket timeouts.
- **`if let` chains, top-level imports, full names, `#[expect]` over `#[allow]`** per CLAUDE.md Rust discipline.

## Codex log

## Claude review

## Verdict
