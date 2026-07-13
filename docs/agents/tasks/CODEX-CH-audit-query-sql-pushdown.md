---
id: CODEX-CH
title: Audit query SQL pushdown — filter/paginate in SQL using the existing indexes
owner: codex
phase: 4
status: open
created: 2026-07-12
last-update: 2026-07-12 claude [Fable 5]
---

# CODEX-CH — Audit query SQL pushdown

## Brief

> `AuditLog::query` calls `read_all` (SELECT every row, deserialize each JSON payload, sort in SQL by `ts`) then filters and paginates in Rust. The `idx_audit_ts_user` / `idx_audit_kind` indexes are never used by a `WHERE` clause. With multi-day retention the table grows to millions of rows, and a single admin opening the audit viewer deserializes the entire table into memory per request **while holding the connection mutex** — blocking all appends for the duration. Ignition's audit log and FactoryTalk's diagnostic log both page server-side. Push the `from`/`to`/`user`/`kinds` filters and `LIMIT`/`OFFSET` into the SQL `WHERE` + `LIMIT` so the indexes are used and only the requested page is materialized. The query is read-only; the hash-chain integrity and canonical serialization must not be disturbed.

### Goal

`AuditLog::query` builds a parameterized `WHERE` from the `AuditQuery` filters plus `LIMIT`/`OFFSET`, so a page request against a million-row table materializes only that page (and one count query for the total), and the `idx_audit_ts_user` / `idx_audit_kind` indexes are exercised. The returned `(Vec<AuditEntry>, total)` is identical to the current Rust-side filter on the same data. The connection mutex is held only for the bounded page query, not a full-table scan-and-deserialize.

### Context to read first

- `crates/audit-log/src/store.rs:96-106` — `AuditLog::query`: `read_all` → `filter_entries` → `total = filtered.len()` → skip/take. This is the method to rework.
- `crates/audit-log/src/store.rs:243-282` — `read_all`: `SELECT ... FROM audit_log ORDER BY ts_ms DESC, id DESC` (no `WHERE`), deserializing payload + hashes per row into `AuditEntry`. The page query reuses this row→`AuditEntry` mapping but with a `WHERE` and `LIMIT`/`OFFSET`.
- `crates/audit-log/src/store.rs:474-493` — `filter_entries`: the Rust-side predicate to translate to SQL — `from_ts_ms` (`ts_ms >= from`), `to_ts_ms` (`ts_ms <= to`), `user` (equality, and the entry's user must be present), `kinds` (empty = all, else `kind IN (..)`). Preserve these semantics exactly, including the `user`-present requirement (an entry with `user = NULL` does not match a `user` filter).
- `crates/audit-log/src/store.rs:138-158` — schema + indexes: `idx_audit_ts_user ON audit_log(ts_ms, user)`, `idx_audit_kind ON audit_log(kind, ts_ms)`. The `WHERE` + `ORDER BY ts_ms DESC` should let the planner use these. Note the stored column is `kind` (the `kind_name()` string), and `filter_entries` matches on `entry.kind.kind_name()` — so the SQL `kind IN (..)` filter compares against the same stored strings. Confirm `AuditEntry.kind` deserialization vs the `kind` column semantics before writing the `IN` clause.
- `crates/audit-log/src/store.rs:17` — `const MAX_LIMIT: usize = 1000`; `query` clamps `limit` to it today. Preserve the clamp in the SQL `LIMIT`.
- `crates/protocol/src/lib.rs:145-162` — `AuditQuery` (`#[non_exhaustive]`). `kinds` has `#[serde(default)]`; **`limit` and `offset` do not** — a client must currently send both or deserialization fails. Add `#[serde(default)]` to `limit` and `offset` (defaulting to `0`) so clients needn't always send both. Keep the Rust↔TS mirror in sync (`packages/protocol-ts/src/index.ts` — make the fields optional there to match).
- `crates/gateway/src/server.rs:1279` — `spawn_blocking(move || audit_log.query(&stored_query))`. No handler change needed; confirm the signature holds.
- The hash chain: `read_all` maps `prev_hash`/`hash` into the entry but the chain is *verified* separately (`verify_chain`, `store.rs:442-472`). `query` is read-only and does not recompute hashes — keep it that way; do not touch `compute_hash` / `CanonicalAuditRow` / append paths.

### Files to create / modify

1. **Modify** `crates/audit-log/src/store.rs`:
   - Rewrite `query` to build a parameterized `WHERE` from the present filters (`from_ts_ms`, `to_ts_ms`, `user`, `kinds`) and a `LIMIT ?/OFFSET ?`. Preserve `ORDER BY ts_ms DESC, id DESC`. Reuse the `read_all` row→`AuditEntry` closure (factor it into a shared helper rather than duplicating the deserialization).
   - Build the `kinds IN (?, ?, ...)` clause with the right number of placeholders (dynamic parameter count); bind each kind string. Empty `kinds` → omit the clause entirely (matches "empty = all").
   - Translate the `user` filter faithfully: `user = ?` (which in SQL already excludes `NULL` users, matching the `is_some_and` semantics — confirm this equivalence and note it).
   - Compute `total` with a `SELECT COUNT(*)` using the **same** `WHERE` (without `LIMIT`/`OFFSET`), so `total` still reflects all matching rows, not just the page. Bind the same filter params.
   - Clamp `limit` to `MAX_LIMIT` in the SQL `LIMIT` as today. Clamp `offset` sensibly (a huge offset returns an empty page, as skip/take does now).
   - Keep the connection lock held only across the two bounded queries (count + page), not a full-table deserialize.
   - No `unwrap`/`expect`/`panic`; propagate errors via the existing `anyhow::Result` path.
2. **Modify** `crates/protocol/src/lib.rs` — add `#[serde(default)]` to `AuditQuery.limit` and `AuditQuery.offset`.
3. **Modify** `packages/protocol-ts/src/index.ts` — make the corresponding TS `limit`/`offset` fields optional to match, and update the protocol-ts test if it asserts the shape.

### Behavior

- A filtered query returns exactly the same rows and `total` as the current `read_all` + `filter_entries` + skip/take on the same fixture, for every combination of `from`/`to`/`user`/`kinds`/`limit`/`offset`.
- Against a large table, only the page (≤ `limit`, ≤ `MAX_LIMIT`) plus one count query are materialized; the full table is never deserialized into memory in one request.
- The `idx_audit_ts_user` / `idx_audit_kind` indexes are used by the planner for time/kind-filtered queries.
- `total` still counts all matching rows regardless of `limit`/`offset`.
- Omitting `limit`/`offset` in the wire JSON deserializes to `0`/`0` (default) rather than erroring.
- Hash-chain verification (`verify_chain`) is untouched and still passes.

### Test requirements

- Add to the existing audit-log test file(s); do not fragment.
- **Filter-parity test**: on a fixture spanning multiple users, kinds, and timestamps, assert the new SQL `query` returns identical `(entries, total)` to a reference computed the old way (`read_all` + the `filter_entries` predicate + skip/take) for a matrix of filter combinations — including `user`-filter-excludes-NULL-user, empty `kinds` = all, `from`/`to` bounds inclusive, and pagination edges (`offset` past the end → empty page, `total` unchanged).
- **Materialization-bound test**: prove a filtered/paginated query does not deserialize the whole table. Options: `EXPLAIN QUERY PLAN` asserting index usage (`USING INDEX idx_audit_ts_user` / `idx_audit_kind`) and no full-table scan feeding Rust-side filtering; and/or instrument the row→entry mapping to count constructed `AuditEntry`s and assert it is `<= limit` for a table with far more matching rows. Output-length assertion alone is insufficient.
- **`total` correctness**: with `limit` smaller than the match count, assert `total` equals the full match count, not the page length.
- **serde default**: deserialize an `AuditQuery` JSON omitting `limit`/`offset`; assert it succeeds with `0`/`0`. Add/extend a protocol wire test.
- **Hash chain intact**: after querying, `verify_chain` still returns Ok with the full row count (query must not mutate anything).
- Deterministic; no sleeps. The filter-parity test must fail if a `WHERE` clause is dropped or a bound flipped (mutate one predicate to confirm; note in Codex log).
- Full matrix: `cargo test -p openwebhmi-audit-log`, workspace clippy `-D warnings`, `cargo fmt --check`, `cargo doc -p openwebhmi-audit-log --no-deps`, `pnpm -r typecheck`, `pnpm -r test` (protocol-ts shape change).

### Acceptance criteria

- [ ] `query` pushes `from`/`to`/`user`/`kinds` into a parameterized SQL `WHERE` and pagination into `LIMIT`/`OFFSET`; `read_all` is no longer used by the query path (or is retained only where still needed, e.g. verification).
- [ ] Returned `(entries, total)` is identical to the old Rust-side filter for every tested filter/pagination combination, including `user`-excludes-NULL and empty-`kinds`-means-all.
- [ ] `total` reflects all matching rows independent of `limit`/`offset`.
- [ ] `idx_audit_ts_user` / `idx_audit_kind` are exercised (EXPLAIN QUERY PLAN probe); a large-table query materializes only the page + count.
- [ ] `limit` clamped to `MAX_LIMIT` in SQL; `AuditQuery.limit`/`offset` gain `#[serde(default)]`; TS mirror made optional and its test updated.
- [ ] Row→`AuditEntry` deserialization is shared (not duplicated) between the page query and any retained reader.
- [ ] Hash-chain canonical serialization / `compute_hash` / append paths untouched; `verify_chain` still passes post-query.
- [ ] Query stays inside `spawn_blocking`; lock held only for the bounded count + page queries; no `unwrap`/`expect`/`panic`.
- [ ] Full matrix (Rust + pnpm) clean; Codex log records the mutation check confirming filter-parity coverage.

### Out of scope

- Keyset / cursor pagination (the brief keeps `LIMIT`/`OFFSET`; deep-offset optimization is a later brief if it becomes a hotspot).
- Adding new audit indexes or changing the schema beyond using the existing indexes.
- Any change to append, hashing, `verify_chain`, backup/restore, or retention pruning.
- Full-text / payload-content search over the JSON payload (the filters stay `ts`/`user`/`kind`).
- Reworking `MAX_LIMIT` or the audit viewer UI.

### Risks / gotchas

- **`user = ?` NULL semantics.** SQL `user = 'alice'` already excludes rows where `user IS NULL`, which matches `filter_entries`' `entry.user.as_ref().is_some_and(..)`. Confirm this equivalence explicitly in a test (a NULL-user entry must not match a `user` filter) — it's the subtle one.
- **Dynamic `IN` placeholder count.** `kinds IN (?, ?, ...)` needs exactly as many placeholders as kinds, all bound in order. Build the clause and the param vector together; an off-by-one placeholder mismatch is a runtime SQL error. Empty `kinds` must omit the clause, not emit `IN ()` (invalid SQL).
- **`total` needs the same WHERE, no LIMIT.** A `COUNT(*)` over the filtered set — bind the same filter params but not the pagination ones. Getting `total` from the page length silently breaks the viewer's pager (regresses to the current-but-correct behavior's contract).
- **Don't disturb the hash chain.** `query` is read-only. It must not recompute or rewrite hashes, and must not alter the canonical row serialization. Keep `compute_hash`/`CanonicalAuditRow`/append entirely out of the diff. `verify_chain` passing post-query is the guard.
- **`ORDER BY ts_ms DESC, id DESC` must be preserved** so pagination is stable and matches the current output ordering. The `idx_audit_ts_user` index leads with `ts_ms`, so the planner can serve the order — confirm with EXPLAIN QUERY PLAN.
- **serde default breadth.** Only add `#[serde(default)]` to `limit`/`offset` per this brief; do not alter the other `AuditQuery` fields' serde behavior. Keep the TS side in lockstep and update the protocol-ts wire test, or `pnpm -r test` fails.
- **Parameter binding limits.** SQLite caps bound parameters (default 999+); a pathological `kinds` list is far below that, but build the query so an empty or single-kind list is handled without a special-case bug.
- **`spawn_blocking` discipline (CODEX-AJ).** All SQLite work stays synchronous inside the blocking context; the mutex must not be held across an `.await`.

## Codex log

## Claude review

## Verdict
