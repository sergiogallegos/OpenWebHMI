---
id: CODEX-BX
title: Gateway service-context refactor — replace process-global OnceLock singletons, split handle_connection
owner: codex
phase: 4
status: open
created: 2026-07-12
last-update: 2026-07-12 claude [Fable 5]
---

# CODEX-BX — Gateway service-context refactor (kill the OnceLock singletons, split dispatch)

## Brief

> The gateway wires historian, alarm journal, alarm engine, script host, and audit log as process-wide set-once `OnceLock` singletons. That makes integration tests contaminate each other through global state, makes a project reload unable to swap services, makes two gateways in one process impossible, and forces the alarm engine into an `Arc<Mutex<AlarmEngineHandle>>` with `spawn_blocking` gymnastics around a std mutex. On top of that, `handle_connection` is a ~900-line message `match` inside a 2112-line file — past the point where dispatch should be split per domain. Replace the singletons with an injected services/context struct (the way `project_store` is already threaded through), split the dispatch into per-domain handlers, and make audit appends a single ordered appender task instead of an unbounded spawn-per-event. Behavior-preserving — no functional change; the win is testability, reload-ability, ordered audit, and review surface. Structural cleanup, MEDIUM. Pairs conceptually with CODEX-BW but is independently shippable.

### Goal

Historian, alarm journal, alarm engine, script host, and audit log are carried on an injected `GatewayServices` (constructed per `run`) and reach `handle_connection` through a per-connection context struct — no `static OnceLock`, no `set_default_*` process globals. A test can construct a gateway with its own service set without touching any global and without leaking state into the next test. `handle_connection`'s message dispatch is decomposed into per-domain handlers (auth / tags / views / project / history / alarms / audit / scripts). Audit events append through one bounded `mpsc` consumed by a single appender task, preserving hash-chain order under load. Full Rust matrix stays green; behavior is unchanged.

### Context to read first

- `crates/gateway/src/server.rs`:
  - Global singletons (lines 42–47): `DEFAULT_HISTORIAN`, `DEFAULT_ALARM_JOURNAL`, `DEFAULT_ALARM_ENGINE` (`OnceLock<Arc<Mutex<AlarmEngineHandle>>>`), `DEFAULT_SCRIPT_HOST`, `DEFAULT_AUDIT_LOG` (`BACKUP_DOWNLOADS` at line 47 is a related process-global map — fold it into the context in the same pass).
  - The `set_default_*` setters (lines 70–93) — the process-wide injection points to be removed.
  - Alarm-engine locking (lines 630–657) — the `Arc<Mutex<AlarmEngineHandle>>` `spawn_blocking(move || engine.lock() ...)` shape; the mutex is a `std::sync::Mutex` wrapped for use across `.await`.
  - `TagSubscribe` / `ScriptSubscribe` handlers etc. — representative arms of the giant `match`; note how `project_store` is *already* threaded as a parameter (the target pattern to extend).
  - `append_audit` (lines 1498–1517) — pulls `DEFAULT_AUDIT_LOG`, clones it, and fires `tokio::spawn` → `spawn_blocking(|| audit_log.append(...))` per event. No ordering guarantee across concurrent events for a hash-chained log; no backpressure.
  - `run` (starts ~line 96) — where services are currently set into the globals and where the injected `GatewayServices` should be constructed instead.
  - File is 2112 lines total; `handle_connection` is the dominant `match`.
- `crates/gateway/src/project.rs` — how `DriverHandles` / `project_store` are already passed as owned/injected state; mirror this ownership shape for `GatewayServices`.
- The gateway integration test file(s) under `crates/gateway/tests/` (e.g. `integration.rs`) — these currently call `set_default_alarm_engine` / `set_default_script_host` and thereby mutate the process globals, which is exactly the cross-test contamination to eliminate.
- CLAUDE.md "Rust code quality" — no `unwrap`/`panic`/`expect` on production paths; `#[expect]` over `#[allow]`; top-level imports only; the `spawn_blocking` neighbor-copy rule.
- `docs/agents/notes/toolchain-drift.md` — for verification-claim discipline.

### Files to create / modify

1. **Introduce a `GatewayServices` (owned, constructed in `run`) and a per-connection `ConnectionContext`.**
   - `GatewayServices` holds the historian, alarm journal, alarm engine, script host, audit-log append handle, and the backup-downloads map — as owned/`Arc` fields, not `OnceLock`. Cheap to clone (Arc-share the inner state, mirroring `TagStore`/`project_store`).
   - `ConnectionContext` bundles the per-connection state currently passed as a long parameter list (`store`, `project_store`, `auth`, `out_tx`, `peer_addr`, `services`, the subscription maps, …) so handlers take `&mut ctx` instead of a dozen args.
   - Thread `GatewayServices` from `run` → the accept loop → `handle_connection` → each per-domain handler, exactly as `project_store` is already threaded. Remove all five `set_default_*` functions and the `static OnceLock`s. Keep the public `run` signature source-compatible where callers exist; if `run` must take the services, update the call sites (`main`/bin and tests) in the same commit.
2. **Split `handle_connection`'s dispatch into per-domain handlers.** One function (or small module) per domain — auth, tags, views, project, history, alarms, audit, scripts. `handle_connection` becomes a thin loop that reads a `ClientMessage` and routes to the domain handler with `&mut ctx`. This is a mechanical extraction: **move**, don't rewrite, the arm bodies. Preserve every `authorize(...)` permission check, every `send_error` code string, and every early-`continue` exactly. No behavior change.
3. **Replace per-event audit spawning with one ordered appender task.**
   - On `GatewayServices` construction, spawn a single audit-appender task that owns the `AuditLog` and consumes a bounded `mpsc::Receiver<AuditAppendRequest>` (request = user, session_id, source_ip, event). It calls `audit_log.append(...)` (via `spawn_blocking` inside the appender if the append is blocking) **sequentially**, preserving hash-chain order. Store the `mpsc::Sender` on `GatewayServices`.
   - `append_audit` becomes a `ctx.services.audit_tx.try_send(request)` (or bounded `send` from an async context) — non-blocking, with a `warn!` on a full channel (backpressure surfaced, not silently dropped without a log). No more `tokio::spawn` per event.
   - Provide a clean shutdown/drain path: dropping the sender ends the appender loop after draining queued events (mirror the AJ/AK JoinHandle/AbortHandle discipline; document `// WHY` at the drain point).
4. **Simplify the alarm-engine access** now that it's injected. It may remain `Arc<Mutex<AlarmEngineHandle>>` if the handle genuinely requires blocking-locked access, but the `spawn_blocking(|| engine.lock())` gymnastics should be encapsulated behind one helper on `GatewayServices` (e.g. `services.ack_alarm(...)`) rather than open-coded in the handler. Do not change alarm semantics; just stop leaking the mutex-across-await shape into the dispatch body.

### Behavior

- Every existing gateway integration and unit test passes **unmodified** except the tests that previously reached for `set_default_*` globals — those now inject services through the new constructor. No wire-protocol change, no permission-check change, no `send_error` code change.
- Two `GatewayServices` can coexist in one process (no shared statics); a test constructs its own without contaminating another test.
- Audit events emitted concurrently from multiple connections append in a single, hash-chain-valid order; a burst does not corrupt the chain or silently drop without a log line.

### Test requirements

- **Injectable services without globals** (add to the closest existing gateway test file — do not fragment): construct a gateway/`GatewayServices` with a test-owned audit log (and/or alarm engine) and drive a flow that exercises it, asserting no `set_default_*` call is needed and that a second independently-constructed instance sees none of the first's state. This test must be impossible to write against the pre-fix code (the globals are set-once and shared) — note that in the log.
- **Audit appends in order** (add to the audit-related gateway test, or `crates/audit-log/tests/` if the appender lands there): enqueue N audit events concurrently from multiple tasks through the bounded channel and assert the stored chain is a valid hash chain in submission-consistent order (the appender serializes them). Use deterministic synchronization (a barrier/`oneshot` to release the producers, then `await` the appender drain) — no `sleep()`. This test must fail against the pre-fix `tokio::spawn`-per-event path (which has no ordering guarantee).
- **Behavior-preserving smoke**: the full pre-existing gateway suite green, three consecutive runs for any known-flaky integration test.
- Full Rust matrix clean: `cargo build --workspace --all-features --locked`, `cargo clippy --workspace --all-targets --all-features --locked -- -D warnings`, `cargo test --workspace --all-features --locked`, `cargo doc --workspace --no-deps`.

### Acceptance criteria

- [ ] All five `static OnceLock` service globals and their `set_default_*` setters removed (plus `BACKUP_DOWNLOADS` folded into the context); services carried on an injected `GatewayServices`.
- [ ] `handle_connection` dispatch split into per-domain handlers taking a `ConnectionContext`; arm bodies moved verbatim (permission checks, `send_error` codes, `continue`s preserved).
- [ ] Audit appends go through one bounded `mpsc` → single appender task; ordered, backpressured, drained on shutdown; full-channel case logs rather than silently dropping.
- [ ] Alarm-engine mutex-across-await access encapsulated behind a `GatewayServices` helper; no behavior change to ack/alarm flow.
- [ ] A test injects its own services with no global and no cross-test contamination; the audit-order test fails against pre-fix code.
- [ ] No `unwrap`/`panic`/`expect` added on production paths; `#[expect]` (not `#[allow]`) for any new lint suppression; top-level imports only.
- [ ] Full Rust matrix clean; wire protocol and permission semantics unchanged.

### Out of scope

- **Tag-engine sharding / slot GC / update coalescing** — that is CODEX-BW. This task does not touch `spawn_forwarder` or the tag fan-out.
- **Actually implementing project reload / hot-swap of services.** This task *unblocks* it by removing the singletons; wiring a live reload is a separate brief. Note in the log that the context now permits it.
- **Changing any wire message** (`crates/protocol` / `packages/protocol-ts`). Behavior-preserving means the bytes on the wire are identical.
- **Rewriting handler logic.** The dispatch split is a move-not-rewrite. Any behavior change discovered as "obviously better" gets a follow-up brief, not a silent ride-along.
- **Converting the alarm-engine std mutex to an async mutex** unless it falls out naturally; the ask is to stop leaking `spawn_blocking(|| lock())` into the dispatch body, not to re-architect the alarm engine.

### Risks / gotchas

- **This is a large mechanical diff — the risk is a silent behavior change during the move.** Extract arm bodies verbatim; resist "cleaning up" logic mid-move. Any real bug found gets a `// WHY` note and a log entry, not a quiet fix folded into the refactor.
- **`run` signature change ripples to call sites.** `main`/the bin entry and every test that starts a gateway must construct `GatewayServices`. Grep for `set_default_` and `::run(` and update all call sites in the same commit so nothing half-migrates.
- **Audit ordering vs the hash chain.** The whole point of the single appender is that a hash-chained log must append strictly serially. If two appends interleave, the chain breaks. The appender must be the *only* writer; assert that in the ordering test.
- **Bounded channel full = backpressure, not data loss.** A full audit channel means the appender is behind. Decide the policy (block the producer briefly vs `try_send` + `warn!` + drop) and state it; a hash-chained security log dropping events silently is a closeout blocker. Prefer surfacing the drop with a `warn!` and a metric-shaped log field.
- **Drain on shutdown.** Dropping the sender must let the appender finish queued events before the task ends (AK's `// Dropping a JoinHandle does not cancel its task` lore applies). Don't `abort()` the appender with events still queued.
- **`ConnectionContext` borrow friction.** Bundling `out_tx`, the subscription maps, and `&services` into one struct can fight the borrow checker when a handler needs `&mut` on the maps and `&` on services simultaneously. Split fields so disjoint borrows are possible (services behind `Arc`, maps owned by the context) — this is the usual "context struct" ergonomics work.
- **Independence from CODEX-BW.** Whichever of BW/BX merges second will rebase over the other's touches in `server.rs` (BW edits `spawn_forwarder`/`TagSubscribe`; BX moves those arms into a tags handler). Keep the diffs minimal and note the overlap so the second merge is clean.
- **Honesty (CLAUDE.md).** State whether the full pre-existing suite was actually run (not just compiled), which tests needed editing and why, and whether the audit-order test genuinely fails against pre-fix code (run it against the old path once to prove it).

## Codex log

## Claude review

## Verdict
