---
id: CODEX-BW
title: Tag-engine scaling — sharded locking, slot GC, update coalescing for thousands of sub-second tags
owner: codex
phase: 4
status: open
created: 2026-07-12
last-update: 2026-07-12 claude [Fable 5]
---

# CODEX-BW — Tag-engine scaling (sharded locking, slot GC, update coalescing)

## Brief

> The tag engine and its gateway fan-out don't scale to the performance-baseline target (thousands of tags at sub-second update rates). Three structural bottlenecks compound: every `publish` takes the single global `RwLock` in write mode so all drivers serialize on one lock; subscribe-created slots are never reclaimed so garbage/renamed tag paths leak the map (and its 1024-cap broadcast channels) forever; and the gateway spawns one forwarder task plus one broadcast receiver per `(connection × subscribed tag)` with no update batching (50 clients × 2000 tags ≈ 100k tasks, one WS frame per update). Shard the tag map, garbage-collect dead slots, and multiplex + coalesce per-connection updates. Structural, MEDIUM — it gates the performance-baseline milestone. Pairs conceptually with CODEX-BX (gateway service-context refactor) but is independently shippable; land either first.

### Goal

Publishes from concurrent drivers no longer serialize on one global lock. Never-published, zero-receiver tag slots are reclaimed instead of accumulating for the life of the process. The gateway serves each connection's tag subscriptions from a single multiplexed task that batches updates on a flush interval rather than a task-and-frame per tag. The engine sustains thousands of tags at sub-second rates without unbounded task or memory growth, and the poisoned-lock panic policy is self-documenting in code per CLAUDE.md.

### Context to read first

- `crates/tag-engine/src/lib.rs` — the whole file is small (~248 lines). The load-bearing sites:
  - `publish` (lines 71–83) — takes `self.inner.write()` at **line 79** for every publish; `entry(path).or_insert_with(make_empty_slot)` creates a slot on demand.
  - `subscribe` (lines 89–94) — takes `self.inner.write()` at **line 91** and `entry().or_insert_with` creates a slot for **any** path, published or not. Nothing ever removes slots.
  - `get` (lines 97–101, read lock at **line 99**) and `len` (lines 104–106, read lock at **line 105**).
  - `TagSlot` (lines 46–49) holds `last: Option<TagSnapshot>` and a `broadcast::Sender` with `BROADCAST_CAPACITY = 1024` (line 44). Each leaked slot pins a 1024-slot channel.
  - Module docs (lines 6–19) define the subscribe-then-read + report-by-exception semantics that must be preserved.
- `crates/gateway/src/server.rs`:
  - `TagSubscribe` handler (lines 699–719) — for each path, primes with `store.get` then `spawn_forwarder(path, store, out_tx)` at **line 717**, one task per path.
  - `spawn_forwarder` (lines 1936–1952) — one `tokio::spawn` per subscribed tag, each holding a `broadcast::Receiver` and emitting one `try_send_message` per update.
  - `OUTBOUND_CAPACITY = 256` (line 41) — the per-connection outbound `mpsc` bound; batching must respect this backpressure.
- `docs/architecture.md` §4.2 — the tag-store role as single source of truth for live values.
- `docs/agents/notes/toolchain-drift.md` — environment-mismatch discipline for the throughput assertions.
- CLAUDE.md "Rust code quality" — the poisoned-lock exemption for `tag-engine` is meant to be self-evidencing in code; the four `expect("tag store rwlock poisoned")` sites currently carry no invariant comment.

### Files to create / modify

1. **Modify** `crates/tag-engine/src/lib.rs`:
   - **Sharded / per-slot locking.** Replace the single `Arc<RwLock<HashMap<TagPath, TagSlot>>>` with a structure that lets concurrent publishes to *different* paths proceed without a global write lock. Two acceptable shapes — pick one and state why in the Codex log:
     - **(a) Sharding**: partition paths across N shards (e.g. `Box<[RwLock<HashMap<...>>]>`, shard index = stable hash of `TagPath` mod N), so a publish only write-locks its shard.
     - **(b) `dashmap`**: a concurrent map keyed by `TagPath`. If chosen, add `dashmap` to the workspace deps (single new dep; keep the `Cargo.lock` diff bounded to `dashmap` + its direct transitives per CLAUDE.md "Cargo.lock diff is bounded").
     Preserve the public API exactly (`new`, `publish`, `subscribe`, `get`, `len`, `is_empty`, `Clone`, `Default`) and the subscribe-then-read / no-replay-on-subscribe semantics documented in the module header.
   - **Slot GC.** Add a method (e.g. `gc(&self) -> usize` returning the count reclaimed) that removes slots which have both `last.is_none()` (never published) **and** `tx.receiver_count() == 0` (no live subscribers). Use `broadcast::Sender::receiver_count`. Do **not** reclaim a slot that has a `last` snapshot — that would drop a legitimately-published value; GC targets only speculative/garbage subscribe paths and dead subscriptions. Document the invariant (which slots are safe to drop and why) in a `// WHY` comment.
   - **Poisoned-lock invariant comment.** At each `expect("tag store rwlock poisoned")` site (currently lines 79, 91, 99, 105 — the count/locations change with the sharding rewrite), add the one-line invariant comment CLAUDE.md requires: a lock poisoned only if a prior holder panicked while mutating, which is non-recoverable state, so panicking is the intended policy. Keep the exemption self-evidencing in code, not just in CLAUDE.md.
2. **Modify** `crates/gateway/src/server.rs`:
   - Replace the task-per-tag fan-out with **one multiplexed subscription task per connection** that owns all of that connection's tag receivers and coalesces updates on a flush interval before emitting. Concretely: collect per-tag `broadcast::Receiver`s into one task (e.g. `select!`/`StreamMap` over the receivers, or a single merged stream), accumulate the latest snapshot per path within a flush window (default flush interval a small const, e.g. `TAG_FLUSH_INTERVAL_MS`, chosen for sub-second responsiveness — state the value and rationale in the Codex log), and emit either a batched `tag.update` set or one message per changed path per flush. Last-write-wins coalescing per path within a window matches the existing report-by-exception broadcast contract (module docs, lines 6–19) — a burst of N updates to one tag collapses to its latest.
   - Respect `OUTBOUND_CAPACITY` backpressure — batching must not bypass the bounded `mpsc`.
   - Subscribe/unsubscribe must add/remove a path from the connection's multiplex set without tearing down the whole task. Preserve the `store.get`-prime-on-subscribe behavior (line 713) so a newly-subscribed tag still gets its current value.
   - If coalescing needs a batched wire message, prefer reusing the existing `tag.update` shape emitted per changed path over inventing a new protocol message — **do not** change `crates/protocol` in this task unless a batch message is unavoidable, and if it is, stop and note it (it would pull in a protocol-ts sync obligation this brief doesn't budget for).
   - The engine-level `gc` is only useful if something calls it; wire a periodic GC (a low-frequency interval task spawned in `run`, or a hook on unsubscribe) so dead slots are actually reclaimed. State the chosen trigger in the Codex log.

### Behavior

- Concurrent drivers publishing to distinct paths do not block each other on a single global lock.
- A subscribe to a path that is never published, once its receiver is dropped, leaves no permanent slot after GC runs.
- A burst of rapid publishes to one tag within a flush window is delivered to a client as the latest value (coalesced), not N separate frames.
- `subscribe`/`get`/`publish`/`len`/`is_empty` observable semantics are unchanged for existing callers; all pre-existing `tag-engine` tests still pass unmodified.
- Sub-second update latency is preserved end-to-end — coalescing trades intermediate frames for the latest value, it does not add perceptible lag beyond one flush interval.

### Test requirements

- **Slot GC reclaims dead subscriptions** (add to `crates/tag-engine/src/lib.rs` `#[cfg(test)] mod tests`): subscribe to a path, drop the receiver, call `gc`, assert the slot count dropped and `gc` returned the reclaimed count. Assert GC does **not** reclaim a slot that has a published `last` value even with zero receivers. This test must fail against the pre-fix code (there is no `gc` and slots never shrink) — demonstrate that by construction.
- **Publish throughput does not globally serialize** (add to the same test module): spawn concurrent publishers to distinct paths and assert all values land via `get`; where practical, assert progress is possible while another shard/path is held (deterministic, no wall-clock throughput assertion — use a barrier/channel to prove two publishes to different paths can both make progress without one holding the other's lock). No `sleep()`; use deterministic synchronization (channels, `tokio::time::pause()` + `advance()`).
- **Batching coalesces a burst** (gateway-side test, add to the closest existing gateway test file — do not fragment): bind `127.0.0.1:0`, connect a client, subscribe a tag, publish a rapid burst within one flush window, and assert the client observes the latest value with fewer frames than publishes. Use `tokio::time::pause()`/`advance()` to drive the flush interval deterministically — no real-clock waits, no hardcoded ports.
- Full Rust matrix clean: `cargo build --workspace --all-features --locked`, `cargo clippy --workspace --all-targets --all-features --locked -- -D warnings`, `cargo test --workspace --all-features --locked`, `cargo doc --workspace --no-deps`.
- Any known-flaky gateway integration test that touches subscriptions: three consecutive green runs.

### Acceptance criteria

- [ ] `publish` to distinct paths no longer contends on a single global write lock (sharded map or `dashmap`); rationale for the choice recorded in the Codex log.
- [ ] `TagStore` public API (`new`, `publish`, `subscribe`, `get`, `len`, `is_empty`, `Clone`, `Default`) and subscribe-then-read / no-replay semantics unchanged; all pre-existing `tag-engine` tests pass unmodified.
- [ ] Slot GC reclaims only never-published, zero-receiver slots (via `receiver_count`), never a slot with a live `last` snapshot; GC is actually invoked (periodic task or unsubscribe hook), trigger stated in the log.
- [ ] Gateway serves each connection's tag subscriptions from one multiplexed task with per-flush-interval coalescing, not one task per tag; flush interval value + rationale in the log.
- [ ] `store.get`-prime-on-subscribe and `OUTBOUND_CAPACITY` backpressure preserved.
- [ ] The four (or post-rewrite equivalent) poisoned-lock `expect` sites carry the one-line invariant comment per CLAUDE.md.
- [ ] GC-reclaims test and coalescing test both fail against pre-fix code; throughput/GC tests use deterministic sync (no `sleep`); gateway tests bind `127.0.0.1:0`.
- [ ] Full Rust matrix clean (build + clippy + test + doc); `Cargo.lock` diff bounded (only `dashmap` + direct transitives if that path is chosen).

### Out of scope

- **Changing the wire protocol** (`crates/protocol` / `packages/protocol-ts`). If a batched `tag.update` message is truly unavoidable, stop and flag it — the protocol sync is a separate obligation.
- **The gateway service-context / singleton refactor** — that is CODEX-BX. This task touches `spawn_forwarder` / `TagSubscribe` fan-out only, not the `OnceLock` service globals.
- **Persisting or historizing tags** — the historian path is untouched.
- **Tuning `BROADCAST_CAPACITY`** or changing the report-by-exception drop-on-lag contract. GC is about slot lifetime, not channel depth.
- **A real hardware throughput benchmark.** The performance-baseline milestone owns the measured numbers; this task removes the structural blockers and proves them with deterministic tests.

### Risks / gotchas

- **`dashmap` vs sharding trade-off.** `dashmap` is a proven concurrent map but adds a dependency and its own internal sharding/locking semantics; a hand-rolled shard array is dependency-free but more code. Either is acceptable — the ask is that publishes to distinct paths don't globally serialize. Do **not** reach for a lock-free exotic; match the codebase's std-first bias unless `dashmap` clearly wins.
- **GC races a concurrent subscribe.** A slot can be created by `subscribe` between the `receiver_count()==0` check and the removal. Ensure GC and subscribe can't drop a slot a subscriber just took a receiver on — hold the relevant shard lock across the check+remove, or re-check under the lock. This is the classic check-then-act hazard; get it right or the GC test will be flaky under load.
- **Coalescing must not drop the *latest* value.** Last-write-wins per path within a window is correct; an off-by-one that drops the final update of a burst is a silent data bug. The burst test must assert the *final* value is delivered, not merely "fewer frames".
- **Flush interval vs sub-second target.** Too long a flush adds latency; too short defeats coalescing. Pick a small default (tens of ms range) and justify it; make it a named const so it's tunable, not a magic number.
- **Subscribe/unsubscribe mid-flight on the multiplexed task.** Adding/removing a receiver from a running `select!`/`StreamMap` set is the fiddly part — a `StreamMap` keyed by path is the clean shape. Don't rebuild the whole task on every subscribe (that reintroduces churn).
- **Preserve prime-on-subscribe ordering.** The current code sends the primed snapshot *before* spawning the forwarder (lines 713–717). Under multiplexing, ensure the primed value isn't lost or duplicated relative to the first coalesced flush.
- **Honesty (CLAUDE.md).** State exactly which map strategy was chosen and why, the flush interval and its rationale, the GC trigger, and whether the throughput test proves non-serialization deterministically or only smoke-checks it. Don't claim "scales to thousands" without saying what was actually measured vs asserted.

## Codex log

## Claude review

## Verdict
