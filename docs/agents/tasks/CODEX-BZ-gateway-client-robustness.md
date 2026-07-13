---
id: CODEX-BZ
title: Gateway client robustness — socket teardown, write/ack feedback, request timeouts, error correlation
owner: codex
phase: 4
status: open
created: 2026-07-12
last-update: 2026-07-12 claude [Fable 5]
---

# CODEX-BZ — Gateway client robustness (teardown, feedback, timeouts, error correlation)

## Brief

> The runtime gateway client (`apps/runtime-web/src/gatewayClient.ts`) has four reliability defects that surface exactly when the network is flaky — the moment an operator HMI most needs to be trustworthy. (a) `connect()` overwrites `this.ws` without closing the previous socket, so a reconnect while the old socket is still `CONNECTING` yields two live sockets both firing `resubscribeAll` and duplicate `tag.update` callbacks, and the zombie's `onclose` schedules yet another reconnect. (b) `sendRaw` silently no-ops when the socket isn't `OPEN`, so operator `writeTag` and `ackAlarm` are dropped during a reconnect with zero feedback. (c) `readHistory`/`readTheme` enqueue a pending promise then `send()` which no-ops when disconnected, leaking the map entry and dangling the promise forever. (d) any `error` server message rejects the *oldest* pending history **and** the oldest pending artifact read because `ServerMessage::Error` carries no request id. Fix teardown, add write/ack feedback, add request timeouts, and add a correlation id to the `Error` protocol message (Rust **and** protocol-ts, kept in sync). Also fix a wrong quality value and a hardcoded backoff constant. HIGH.

### Goal

A reconnect tears down the previous socket (close + detached handlers) before opening a new one, so there is never more than one live socket and no duplicate callbacks. A write or ack issued while disconnected either queues-and-flushes on reconnect or rejects with a surfaced error — never a silent drop. A history/theme request issued while disconnected rejects (or retries) on a timeout and cleans up its map entry — no dangling promise, no leak. An `error` server message is routed to the specific pending request it belongs to via a correlation id present in both the Rust and TypeScript protocol definitions. Reconnect marks bindings with a truthful quality, and the backoff uses the named constant.

### Context to read first

- `apps/runtime-web/src/gatewayClient.ts`:
  - `connect()` (lines 112–136) — sets `this.ws = ws` (line 123) without closing/detaching the prior socket; wires `onopen`→`resubscribeAll`, `onclose`→`scheduleReconnect`.
  - `sendRaw` (lines 385–390) — returns silently if `this.ws?.readyState !== OPEN`.
  - `writeTag` (lines 261–263) and `ackAlarm` (lines 304–306) — both route through `send`→`sendRaw`; silently dropped when not OPEN.
  - `readHistory` (lines 308–325) and `readTheme` (lines 327–339) — set a pending map entry then `send`; the send no-ops when disconnected, leaving the promise pending forever.
  - `handleMessage` `error` case (lines 423–428) → `rejectOldestHistory` (lines 507–518) — rejects the oldest `pendingHistory` **and** oldest `pendingArtifacts` entry blindly.
  - `resubscribeAll` (lines 520–538), `scheduleReconnect` (lines 540–555) — note the **hardcoded `8_000`** at line 548 that should be `MAX_BACKOFF_MS`.
  - `MAX_BACKOFF_MS` constant (line 70) and `INITIAL_BACKOFF_MS` (line 69), `OPEN = 1` (line 68).
  - `markBindingsBad` (lines 557–573) — sets `quality: "bad"` on reconnect.
  - `disconnect` (lines 341–347) — the existing clean-teardown reference (`close()` + null `this.ws`).
- `crates/protocol/src/lib.rs`:
  - `Quality` enum (lines 221–232): `Bad` is documented "driver disconnected" (line 227); `Uncertain` is "reconnect in progress" (line 229); `Stale` is "not refreshed within window" (line 231). Reconnect is *not* a driver disconnect — see below.
  - `ServerMessage::Error` (lines 538–544): `{ code: String, message: String }` — **no request id**. This is the correlation gap.
- `packages/protocol-ts/src/index.ts` — the TypeScript mirror of the protocol; the `error` `ServerMessage` variant must gain the same optional correlation field. Keep Rust↔TS in exact sync (no drift) — the CI validator and `pnpm typecheck` guard this.
- `docs/agents/notes/binding-write-asymmetry.md` — relevant to write-path behavior: input components already route writes through an explicit `tagPath`; this task is about the *transport* reliability of those writes (feedback on drop), not the binding path derivation. Don't conflate the two.
- CLAUDE.md frontend discipline: no `any`, remove listeners on teardown, deterministic tests (no `setTimeout`/wall-clock waits — use fake timers).

### Files to create / modify

1. **Fix socket teardown on reconnect** — `connect()` (lines 112–136): before assigning a new socket, tear down the previous one. Add a private `teardownSocket()` that, if `this.ws` exists, nulls its four handlers (`onopen`/`onmessage`/`onclose`/`onerror`) **first** (so the zombie's `onclose` can't schedule another reconnect) then calls `close()`. Call it at the top of `connect()` (and reuse it in `disconnect`). After this, there is exactly one live socket; no duplicate `resubscribeAll` / `tag.update`.
2. **Write/ack feedback instead of silent drop** — `sendRaw` (lines 385–390) currently no-ops when not OPEN. Choose and implement one coherent policy (state it in the log):
   - **Queue-and-flush**: buffer outbound messages while not OPEN and flush them in `onopen`/`resubscribeAll` order; **or**
   - **Reject-and-surface**: return a boolean / throw so `writeTag` and `ackAlarm` can report failure to the UI via the existing `errorCallbacks` channel.
   For operator control actions (`writeTag`, `ackAlarm`) a **silent** drop is the defect — the operator must learn the action didn't land. Prefer surfacing a failure (error callback) for control actions even if subscription messages are queued. Don't queue unboundedly; cap the buffer and surface overflow.
3. **Request timeouts for pending promises** — `readHistory` (308–325) and `readTheme` (327–339): each pending entry gets a timeout that, on expiry, rejects the promise **and** deletes its map entry (no leak). On reconnect, either replay the request or reject the pending ones with a clear error; don't leave them dangling. Use an injectable timeout (constant + optional override) so tests can drive it with fake timers deterministically. Clear the timeout on normal resolution (`dispatchHistoryResult` / `dispatchProjectArtifact`).
4. **Correlate `error` to its request** — add an optional correlation id to the `Error` message and route by it:
   - **Rust** `crates/protocol/src/lib.rs` `ServerMessage::Error` (538–544): add `#[serde(default, skip_serializing_if = "Option::is_none")] request_id: Option<String>`. Update the gateway `send_error` call sites to pass the originating `request_id` where one exists (history/artifact/audit request handlers); pass `None` for connection-level errors that have no request. Update the existing protocol round-trip test to cover the new field.
   - **protocol-ts** `packages/protocol-ts/src/index.ts`: add `request_id?: string | null` to the `error` `ServerMessage` variant, mirroring the Rust `Option<String>` exactly.
   - **Client** `rejectOldestHistory` (507–518): rename/replace with correlated routing — if the error carries a `request_id`, reject **that** specific `pendingHistory`/`pendingArtifacts` entry; only fall back to the oldest-heuristic when `request_id` is absent (connection-level error). Still fan out to `errorCallbacks` for UI display.
5. **Quality fix** — `markBindingsBad` (557–573): a reconnect-in-progress is **not** a driver disconnect. Per the `Quality` doc-comments (lib.rs 227–231), use `"uncertain"` (reconnect in progress) or `"stale"` (last-good, not refreshed) rather than `"bad"`. Pick one and justify (reconnect-in-progress maps most precisely to `"uncertain"`); rename the method to match its new meaning (e.g. `markBindingsUncertain`). Update its test.
6. **Backoff constant** — `scheduleReconnect` (line 548): replace the hardcoded `8_000` with `MAX_BACKOFF_MS` (line 70).

### Behavior

- Reconnecting while the previous socket is still `CONNECTING` leaves exactly one live socket; subscriptions are resubscribed once; each `tag.update` fires each callback once.
- A `writeTag`/`ackAlarm` issued while disconnected surfaces a failure to the UI (or is queued and flushed on reconnect per the chosen policy) — never silently dropped.
- A `readHistory`/`readTheme` issued while disconnected rejects on timeout and removes its pending-map entry; no promise dangles, no map leak.
- An `error` server message carrying a `request_id` rejects exactly the matching pending request; a request-less error still reaches `errorCallbacks`.
- On reconnect, bound tags read `uncertain` (not `bad`); the backoff ceiling uses `MAX_BACKOFF_MS`.

### Test requirements

Add to the existing `gatewayClient` test file (Vitest + a `WebSocketLike` fake — the client already accepts `webSocketImpl` for exactly this). Do not fragment; extend the existing suite. Use **fake timers** for all timeout/backoff assertions — no real `setTimeout` waits.

- **No zombie socket on reconnect**: open a socket, trigger a reconnect while it is still `CONNECTING`, and assert the previous socket's handlers were detached and `close()` was called, and that after both sockets settle `resubscribeAll` ran once and a single `tag.update` invokes each callback once (not twice). Must fail against pre-fix code (which leaves the old socket live).
- **Dropped control action surfaces an error**: with the socket not OPEN, call `writeTag`/`ackAlarm` and assert the failure is surfaced (error callback fired, or the promise/return signals failure) — not silently swallowed. Must fail against pre-fix `sendRaw` (which returns silently).
- **Pending request times out and cleans up**: call `readHistory` while disconnected, advance fake timers past the timeout, assert the promise rejects and the `pendingHistory` map no longer holds the entry. Must fail against pre-fix code (promise dangles forever).
- **Error correlates to the right request**: enqueue two pending requests, deliver an `error` carrying the second's `request_id`, assert the **second** rejects and the first stays pending (then resolves normally). Must fail against pre-fix `rejectOldestHistory` (which rejects the oldest).
- **Quality on reconnect**: assert bindings become `uncertain` (not `bad`) on reconnect.
- **Rust**: extend the protocol round-trip/serde test to cover `Error.request_id` present and absent (absent must still deserialize old-format messages — `#[serde(default)]`).
- `cargo test --workspace --all-features --locked`, `cargo clippy ... -D warnings`, `cargo fmt --check`, `pnpm -r typecheck`, `pnpm -r test` all clean.

### Acceptance criteria

- [ ] `connect()` tears down the previous socket (detach handlers **then** close) before opening a new one; a reconnect-while-connecting leaves exactly one live socket and no duplicate callbacks.
- [ ] `writeTag`/`ackAlarm` issued while disconnected surface a failure (or are queued-and-flushed per the stated policy) — never silently dropped; policy recorded in the log.
- [ ] `readHistory`/`readTheme` pending promises reject on timeout and delete their map entry; timeouts cleared on normal resolution; no leak.
- [ ] `ServerMessage::Error` gains `request_id: Option<String>` in **both** `crates/protocol` and `packages/protocol-ts` with no drift; `send_error` passes the originating id where one exists; client routes rejections by id and falls back to oldest only when absent.
- [ ] `markBindingsBad` reports `uncertain` (renamed accordingly) on reconnect, not `bad`; `8_000` replaced with `MAX_BACKOFF_MS`.
- [ ] All five listed client tests plus the Rust serde test pass and the four "must fail against pre-fix" tests demonstrably fail without the fix; all deterministic (fake timers, no wall-clock waits).
- [ ] Full matrix clean: `cargo test`/`clippy`/`fmt --check`, `pnpm -r typecheck`, `pnpm -r test`; no new `any`.

### Out of scope

- **Rewriting the reconnect/backoff strategy** (jitter, ceiling, state machine) beyond fixing the hardcoded constant — the existing backoff shape stays.
- **A general request/response correlation framework** for every message — this task adds `request_id` to `Error` and routes it; the broader request-id audit of all messages is a separate concern.
- **Server-side alarm-snapshot-on-resubscribe** — that reconciliation is CODEX-CA's territory.
- **Changing `history.read`/`project.read_artifact` request shapes** — they already carry `request_id`; this task consumes it, it doesn't redesign them.
- **Offline write persistence across a full disconnect** (a durable outbox) — the queue-and-flush option here is best-effort in-memory for a transient reconnect, not durable storage.
- **The binding write-path derivation** (`docs/agents/notes/binding-write-asymmetry.md`) — this task is transport reliability, not binding path resolution.

### Risks / gotchas

- **Detach handlers before `close()`, not after.** If you `close()` first, the zombie's `onclose` fires and schedules a reconnect before you null it. Order matters: null the four handlers, *then* `close()`. The reconnect-loop bug is exactly this ordering.
- **`Error.request_id` must be backward-compatible.** `#[serde(default, skip_serializing_if = "Option::is_none")]` so old-format `{code,message}` messages still deserialize and new messages without an id don't emit a null field. The serde test must cover the absent case — a required field here breaks every existing `send_error` call and wire compatibility.
- **Rust↔TS drift is a CI failure.** The protocol-ts change must exactly mirror `Option<String>` → `request_id?: string | null`. Update both in the same commit; run `pnpm -r typecheck` and the Rust serde test together.
- **Queue-vs-reject policy must be coherent for control actions.** Queuing a `writeTag` silently and flushing it seconds later on reconnect can be *worse* than a visible failure — the operator may have moved on, and a stale setpoint write lands unexpectedly. For control actions, prefer surfacing failure over silently deferring; if you queue, cap and expire the queue. State the reasoning.
- **`uncertain` vs `stale`.** Reconnect-in-progress maps to `uncertain` per the enum docs (lib.rs 229); `stale` is for a value past its refresh window. Don't pick `stale` for a reconnect. Renaming `markBindingsBad` ripples to its callers and test — update them.
- **Fake timers and promise microtasks.** Timeout tests need to interleave `vi.advanceTimersByTime` with `await`/flush of microtasks correctly, or the rejection won't have settled when you assert. Get the timer/microtask ordering right so the test isn't secretly racy.
- **Honesty (CLAUDE.md).** State the chosen drop policy, whether each "must fail against pre-fix" test was actually run against the old code to confirm it fails, and confirm the protocol change was verified in both languages (not just one side).

## Codex log

## Claude review

## Verdict
