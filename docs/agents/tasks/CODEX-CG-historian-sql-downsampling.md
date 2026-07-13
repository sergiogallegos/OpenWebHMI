---
id: CODEX-CG
title: Historian read-path SQL downsampling — bucket server-side instead of loading all raw rows
owner: codex
phase: 4
status: open
created: 2026-07-12
last-update: 2026-07-12 claude [Fable 5]
---

# CODEX-CG — Historian read-path SQL downsampling

## Brief

> `HistorianStore::read` calls `read_raw_unbounded` (every point in `[t_start, t_end]`) and buckets them in memory via `aggregations::aggregate`; `max_points` limits only the **output**, not the rows scanned. A Trend requesting a month of a 1 Hz tag pulls ~2.6M rows into a `Vec<HistoryPoint>` before downsampling to 500 points. The call is correctly wrapped in `spawn_blocking` (`server.rs:853`) so it won't stall the runtime, but memory and latency scale with the query range, not the result size. Ignition's tag historian and FactoryTalk's trend engine both bucket server-side; OpenWebHMI must push the bucketing/aggregation into SQL with `GROUP BY` so only ~`max_points` rows are materialized. Aggregation semantics must stay byte-for-byte identical to the current in-memory path.

### Goal

`HistorianStore::read` computes time buckets and per-bucket aggregates (min/max/avg/first/last/count/sum) in SQL via `GROUP BY`, materializing on the order of `max_points` rows instead of every raw sample in range. The returned downsampled series is identical to what the current in-memory `aggregate` produces on the same data. `Raw` aggregation still returns raw samples (bounded appropriately). The `aggregations::aggregate` in-memory implementation is retained as the correctness oracle for tests (and may remain the code path for `Raw`).

### Context to read first

- `crates/historian/src/store.rs:74-92` — `HistorianStore::read`: `read_raw_unbounded` then `aggregate`. This is the method to rework.
- `crates/historian/src/store.rs:178-208` — `read_raw_unbounded`: the unbounded `SELECT ts_ms, value, quality ... WHERE tag_id = ? AND ts_ms BETWEEN ? AND ? ORDER BY ts_ms ASC`. The new path replaces this with a bucketed `GROUP BY` query for non-`Raw` aggregations.
- `crates/historian/src/store.rs:221-236` — the schema: `tag_history(tag_id, ts_ms, value TEXT, quality TEXT)` `WITHOUT ROWID`, PK `(tag_id, ts_ms)`. **Note `value` is stored as JSON text** (`serde_json::to_string(value)`), so numeric SQL aggregates (`AVG`, `MIN`, `MAX`, `SUM`) can't run directly on the column — see gotchas.
- `crates/historian/src/aggregations.rs:80-112` — `aggregate`: bucket width = `(t_end - t_start).max(1) / max_points`, index = `floor((ts - t_start) / width)`, clamped to `bucket_count - 1`; left-closed/right-open buckets, final bucket includes `t_end`; empty buckets are dropped; each emitted point's `ts_ms = t_start + index * width`. **The SQL bucketing must reproduce this exact bucket assignment and timestamp**, or downsampled series will differ.
- `crates/historian/src/aggregations.rs:114-150` — `aggregate_bucket`: `first`/`last` by bucket order, `count`, `avg`, `sum`, `min`, `max`; emitted `quality` is the **last** point's quality in the bucket. Lines 119-120 hold `bucket.first().expect("non-empty bucket")` / `bucket.last().expect(...)` on the production read path — guarded by the preceding `filter(|(_, bucket)| !bucket.is_empty())` at line 103, but still an `expect` reachable from non-test code. Restructure per CLAUDE.md: `if let Some(first) = bucket.first()` with the empty case returning early / skipped, or carry a one-line invariant comment explaining why the filter guarantees non-empty. Prefer eliminating the `expect`.
- `crates/gateway/src/server.rs:836-872` — the `history.read` handler; `historian.read(..)` at line 854 inside `spawn_blocking` at 853. No change needed here, but confirm the signature stays the same.
- [CODEX-AJ] (`docs/agents/tasks/`) — the `spawn_blocking` SQLite discipline. `read` must stay callable from the `spawn_blocking` context; do not move SQLite work onto the async runtime.

### Files to create / modify

1. **Modify** `crates/historian/src/store.rs`:
   - Add a bucketed read that computes `bucket_index = min((ts_ms - t_start) * bucket_count / span, bucket_count - 1)` in SQL (integer arithmetic matching the float `floor` result — verify the two agree on boundaries; integer `(ts - t_start) * bucket_count / span` floors identically for non-negative operands) and `GROUP BY bucket_index`, selecting the per-bucket aggregate.
   - Because `value` is JSON text, either (a) `json_extract(value, '$.value')` (SQLite's JSON1 — confirm it is compiled in for the bundled `rusqlite` build) to get a numeric column for `AVG`/`MIN`/`MAX`/`SUM`, or (b) `CAST` after stripping — JSON1 via `json_extract` is the clean route if available. For `first`/`last` per bucket, select the value at the min/max `ts_ms` within the bucket (window function or correlated subquery). Emit `quality` from the last point in the bucket (matches `aggregate_bucket`).
   - Emit each bucket's `ts_ms` as `t_start + bucket_index * width` where `width = span / bucket_count` — **reproduce the existing timestamp formula exactly**, including the `(index as f64 * width) as u64` truncation, so outputs match the oracle.
   - Keep `Raw` on the existing `read_raw_unbounded` path (bounded by `max_points` as today via `aggregate`'s `Raw` early-return at `aggregations.rs:87-89`), or apply a SQL `LIMIT` — but preserve current `Raw` semantics (`aggregate` currently returns the *first* `max_points` raw points).
   - Preserve `NonNumeric` error behavior: `avg`/`sum`/`min`/`max` on non-numeric values must still surface an error equivalent to `AggregationError::NonNumeric` (a numeric aggregate over a JSON string yields SQL NULL or 0 — detect and error rather than silently returning a wrong number).
2. **Modify** `crates/historian/src/aggregations.rs:119-120` — remove the two `expect("non-empty bucket")` calls (restructure to `if let`/early-continue or document the invariant per CLAUDE.md). The in-memory `aggregate` stays as the test oracle.
3. Keep the public `read` signature `(tag_path, t_start_ms, t_end_ms, aggregation, max_points) -> Vec<HistoryPoint>` unchanged.

### Behavior

- For a non-`Raw` aggregation over a large range, the SQL query materializes ~`max_points` rows (one per non-empty bucket), not every raw sample. Memory and latency scale with `max_points`, not range.
- The returned series equals `aggregate(read_raw_unbounded(..), ..)` on the same fixture — same bucket timestamps, same aggregate values, same emitted `quality`, same dropping of empty buckets, for every aggregation mode.
- `Raw` behaves exactly as today.
- Non-numeric values under a numeric aggregation error out equivalently to `NonNumeric` (no silent zero/NULL).
- `max_points == 0` and empty result sets behave as today (empty vec).

### Test requirements

- Add to the existing historian test file(s); do not fragment.
- **Oracle-equivalence test**: on a fixture with a few hundred samples of a known numeric pattern across a range, assert the new SQL path returns *exactly* the same `Vec<HistoryPoint>` as the retained in-memory `aggregate(read_raw_unbounded(..))` for every aggregation mode (avg/min/max/first/last/count/sum). This is the core correctness proof and must pass for all modes.
- **Row-scan bound**: prove the large-range query does not materialize all raw rows. Options: `EXPLAIN QUERY PLAN` asserting a `GROUP BY` / index usage rather than a full scan feeding Rust-side bucketing; or instrument the read to count rows returned from SQL and assert it is `<= max_points` (plus any per-bucket helper rows) for a range containing far more than `max_points` samples. A pure "result length" assertion is not sufficient — it must show the *scan/materialization* is bounded.
- **Boundary parity**: include samples exactly on bucket boundaries and at `t_end` to confirm the SQL integer bucket index matches the float `floor` assignment (this is the highest-risk divergence).
- **Non-numeric error parity**: a numeric aggregation over string values errors like the in-memory path.
- Deterministic; no sleeps. The oracle-equivalence test must fail if the SQL bucketing diverges (mutate one bucket-index expression to confirm the test catches it; note in Codex log).
- Full matrix: `cargo test -p openwebhmi-historian`, workspace clippy `-D warnings`, `cargo fmt --check`, `cargo doc -p openwebhmi-historian --no-deps`.

### Acceptance criteria

- [ ] `HistorianStore::read` buckets and aggregates in SQL (`GROUP BY`) for non-`Raw` modes; ~`max_points` rows materialized, not every raw sample.
- [ ] Output is bit-for-bit identical to the retained in-memory `aggregate` oracle across all aggregation modes on a fixture (bucket timestamps, values, quality, empty-bucket dropping).
- [ ] Bucket index + emitted `ts_ms` reproduce the existing float formula exactly, including boundary/`t_end` handling.
- [ ] `Raw` mode semantics unchanged.
- [ ] Non-numeric values under numeric aggregation error equivalently to `AggregationError::NonNumeric`.
- [ ] `aggregations.rs:119-120` no longer contains `expect("non-empty bucket")` on the production path (restructured or invariant-commented per CLAUDE.md).
- [ ] Row-scan-bound test proves materialization is bounded (query-plan or SQL-row-count probe), not just output length.
- [ ] Read stays inside `spawn_blocking`; `read` signature unchanged; no SQLite work on the async runtime.
- [ ] Full matrix clean; Codex log records the mutation check confirming the oracle test catches divergence.

### Out of scope

- Changing the on-disk `value` storage from JSON text to a typed/numeric column (a schema migration; would help SQL aggregation but is a separate, larger brief). Work with the JSON text column as-is via `json_extract` or equivalent.
- Continuous / rollup aggregate tables (pre-aggregated tiers) — a future scalability brief.
- Interpolation, gap-filling, or LTTB-style visual downsampling — the bucket aggregation semantics stay exactly as `aggregate` defines them.
- Touching `write_sample`, prune paths, or backup/restore/merge.
- Changing the `history.read` protocol message or the `server.rs` handler beyond confirming the signature holds.

### Risks / gotchas

- **`value` is JSON text, not a number.** `AVG(value)` on `"{\"type\":\"real\",\"value\":250.0}"` is meaningless. Use `json_extract(value, '$.value')` if SQLite JSON1 is available in the bundled build (`rusqlite` `bundled` feature ships JSON1 by default — **verify** in `crates/historian/Cargo.toml`), else parse a numeric substring. Confirm JSON1 availability before committing to that route; if absent, that's a stop-and-ask, not a silent workaround.
- **Integer vs float bucket index divergence.** The in-memory code does `floor((ts - t_start) / width)` in `f64`; SQL integer `(ts - t_start) * bucket_count / span` must produce the identical index at every boundary. They agree for non-negative operands, but the emitted `ts_ms = t_start + (index as f64 * width) as u64` truncation must also be reproduced. The boundary-parity test is the guard — do not skip it.
- **`first`/`last` per bucket need the value at min/max `ts_ms`**, not `MIN(value)`/`MAX(value)` (which would compare JSON strings lexically). Use a window function (`ROW_NUMBER() OVER (PARTITION BY bucket ORDER BY ts_ms)`) or correlated subquery. Confirm the SQLite version supports window functions (3.25+; the bundled version is far newer).
- **`quality` is the last point's quality.** `aggregate_bucket` emits `last.quality`. The SQL path must select quality from the max-`ts_ms` row in each bucket, not an arbitrary one.
- **Non-numeric silent failure.** A numeric SQL aggregate over a non-numeric `json_extract` yields NULL or 0, not an error. Detect the type mismatch and return a `NonNumeric`-equivalent error so behavior matches the in-memory oracle — a silently-wrong average is worse than the current slow-but-correct path.
- **Keep the in-memory `aggregate` as the oracle.** Deleting it removes the correctness reference and the `Raw` path. Retain it; the SQL path is an optimization that must match it.
- **`spawn_blocking` discipline (CODEX-AJ).** All SQLite work stays synchronous inside the blocking context; don't introduce async SQL or move the lock across an `.await`.

## Codex log

## Claude review

## Verdict
