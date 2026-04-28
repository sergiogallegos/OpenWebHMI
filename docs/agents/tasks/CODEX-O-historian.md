---
id: CODEX-O
title: crates/historian — tag time-series storage + read API
owner: codex
phase: 3
status: submitted
created: 2026-04-27
last-update: 2026-04-27 18:31 codex
---

# CODEX-O — `crates/historian`

## Brief

### Goal

Persistent time-series storage for tag values. The gateway selectively logs configured tags into SQLite, and clients can query historical values + aggregations to render trends and alarm context. This is the foundation; the `Trend` component (CODEX-P) consumes its read API.

### Context to read first

- `docs/architecture.md` §4.6 (Historian).
- `docs/roadmap.md` Phase 3 — historian deliverables.
- `crates/tag-engine/src/lib.rs` — `TagSnapshot` and the broadcast pattern this crate subscribes to.
- `crates/project-store/src/types.rs` — extend with `HistoryConfig` per-tag.
- `crates/protocol/src/lib.rs` — `history.read` / `history.result` wire additions.

### Files to create / modify

- `crates/historian/Cargo.toml`
- `crates/historian/src/lib.rs` — re-exports.
- `crates/historian/src/store.rs` — SQLite schema + writer + reader.
- `crates/historian/src/recorder.rs` — subscribes to `TagStore`, writes logged tags.
- `crates/historian/src/aggregations.rs` — `raw, avg, min, max, count, first, last`.
- `crates/historian/tests/store.rs` — round-trip + aggregation tests.
- `crates/protocol/src/lib.rs` — add `ClientMessage::HistoryRead { tag_path, t_start_ms, t_end_ms, aggregation, max_points }` + `ServerMessage::HistoryResult { tag_path, points }`.
- `packages/protocol-ts/src/index.ts` — mirror.
- `crates/project-store/src/types.rs` — add `tags[].history?: { rate_ms?, deadband? }`.

Add `crates/historian` to workspace `members`.

### Schema (SQLite)

```sql
CREATE TABLE tag_history (
    tag_id INTEGER NOT NULL,
    ts_ms  INTEGER NOT NULL,
    value  TEXT    NOT NULL,            -- JSON-encoded TagValue
    quality TEXT   NOT NULL,            -- "good"|"bad"|"uncertain"|"stale"
    PRIMARY KEY (tag_id, ts_ms)
) WITHOUT ROWID;

CREATE TABLE tag_dictionary (
    tag_id   INTEGER PRIMARY KEY AUTOINCREMENT,
    tag_path TEXT UNIQUE NOT NULL
);
```

Tag paths are interned into integer ids to keep `tag_history` rows small. `value` stores the wire-form JSON of `TagValue` so future variants don't break the schema.

### Aggregations

For a query window, the historian returns **at most `max_points` rows**:

- `raw` — every recorded sample (capped at `max_points`).
- `avg`, `min`, `max`, `count`, `sum`, `first`, `last` — bucket the window into `max_points` equal-width buckets, return one row per bucket.

For non-numeric values (`Bool`, `String`), only `raw`, `first`, `last`, `count` are supported; numeric aggregations return `Err(DriverError::UnsupportedType)`.

### Recorder

A long-lived task that:
1. On gateway boot, loads logged-tag configs from the project store.
2. For each, calls `tag_store.subscribe(path)` and on each `TagSnapshot`:
   - Apply deadband filter: if numeric and `(new - last) < deadband`, skip.
   - Apply rate-limit: if `(now - last_logged_ts) < rate_ms`, skip.
   - Otherwise write to `tag_history`.
3. Hot-reload: on `project.changed` for `tags` artifact, diff the logged-tag set and adjust subscriptions.

### Wire protocol

```rust
#[serde(rename = "history.read")]
HistoryRead {
    tag_path: String,
    t_start_ms: u64,
    t_end_ms: u64,
    aggregation: String,             // "raw" | "avg" | "min" | "max" | "count" | "sum" | "first" | "last"
    max_points: u32,
    request_id: Option<String>,
}

#[serde(rename = "history.result")]
HistoryResult {
    request_id: Option<String>,
    tag_path: String,
    points: Vec<HistoryPoint>,       // { ts_ms, value, quality }
}
```

### Test requirements

- Round-trip: write samples, read raw, verify ordering + quality.
- Each aggregation produces correct results on a known dataset.
- Deadband: a stream of `1.0, 1.0001, 1.5` with deadband 0.1 records only `1.0` and `1.5`.
- Rate-limit: a 100Hz stream with rate_ms=1000 records ~1 sample/sec.
- Hot-reload: removing a tag from logged-set unsubscribes its recorder.
- Wire round-trip + literal form for `HistoryRead`.

### Acceptance criteria

- [ ] `cargo test -p openwebhmi-historian` green.
- [ ] `cargo test --workspace --all-features --locked` stays green.
- [ ] Phase 1 demo project extended with `[tags.history]` config for `rockwell-1/Pressure` (rate_ms=500); recorder writes samples; `history.read` returns them.
- [ ] All public items have rustdoc.

### Out of scope

- Pluggable backend (Timescale/Influx) — Phase 5+.
- Continuous downsampling tables — Phase 5+.
- Export to CSV/Parquet — post-1.0.
- Backfill from external sources — post-1.0.

### Risks / gotchas

- **`WITHOUT ROWID`** on `tag_history` matters for storage size; don't drop it without measuring.
- **Bucketing windows** require careful boundary handling — the last bucket may be partial; document the convention (left-closed, right-open).
- **JSON-encoded `value`** is wasteful but schema-stable. SQLite TEXT compression in v2 if it becomes a problem.
- **Don't subscribe to every tag in TagStore** — only logged ones. Default deadband=0, rate_ms=1000 if config omits them.

## Codex log

*(codex — append working notes here)*

### 2026-04-27 15:33  codex
Started. Taking CODEX-O first because it is independent and unlocks CODEX-P. Reading historian brief, tag-store broadcast semantics, project tag config shape, and gateway websocket routing before editing.

### 2026-04-27 16:24  codex
Submitted. Added `crates/historian` with SQLite schema, tag dictionary, raw reads, bucket aggregations, recorder tasks with rate/deadband filtering, and hot-reload add/remove support. Added `HistoryConfig` to project tags, `history.read` / `history.result` protocol in Rust and TS, gateway history.read routing, and Phase 1 demo history config for `rockwell-1/Pressure`. Verification: `cargo test -p openwebhmi-historian`; `cargo test --workspace --all-features --locked`; `cargo clippy --workspace --all-targets --all-features -- -D warnings`; `cargo fmt --all -- --check`; `pnpm -r typecheck`; `pnpm -r test`; `pnpm -r build`.

### 2026-04-27 18:31  codex
Submission tightened. Gateway now keeps the historian recorder attached to `ProjectStore::subscribe_changes`, filters the global change stream by `project_id`, and reloads configs on `ArtifactKind::Tags`. Recorder hot-reload now restarts an existing tag recorder when its rate/deadband changes, not only when paths are added or removed. Verification: `cargo test -p openwebhmi-historian`; `cargo check -p openwebhmi-gateway`; `cargo fmt --all -- --check`; `cargo clippy --workspace --all-targets --all-features -- -D warnings`; `cargo test --workspace --all-features --locked`.

## Claude review

*(claude — after submission)*

## Verdict

*(claude — final disposition)*
