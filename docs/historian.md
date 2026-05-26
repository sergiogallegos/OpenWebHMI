# Historian Capacity And Retention

OpenWebHMI's historian is a single SQLite database managed by `crates/historian`. SQLite's published maximum database size is 281 TB, but that is a file-format ceiling, not an operational recommendation. For typical v1 deployments, disk, backup time, query latency, and retention policy are the practical limits.

Source for the SQLite ceiling: <https://www.sqlite.org/limits.html>.

## Schema

The live schema is in `crates/historian/src/store.rs`: `tag_dictionary(tag_id, tag_path)` plus `tag_history(tag_id, ts_ms, value, quality)` with primary key `(tag_id, ts_ms)`.

Each sample stores `tag_id`, `ts_ms`, a JSON `TagValue` string, and a quality string. A realistic sizing estimate is:

```text
bytes_per_sample = 8 tag_id + 8 ts_ms + value_json_bytes + quality_bytes + 16..32 SQLite btree overhead
```

For planning, use 64 bytes/sample for booleans and small integers, 72 bytes/sample for floating-point values, and 96 bytes/sample for short strings.

## Capacity

| Scenario | Samples | Estimate | Notes |
|---|---:|---:|---|
| 100 tags at 1 Hz for 90 days | 777,600,000 | ~52 GB | 72 bytes/sample floating-point planning value. |
| 1,000 tags at 1 Hz for 365 days | 31,536,000,000 | ~2.1 TB | Practical only with a storage and backup plan. |
| 10,000 tags at 100 ms for 30 days | 25,920,000,000 | ~1.7 TB | Beyond the v1 target envelope for most deployments. |
| 500 tags at 0.2 Hz for 365 days | 3,153,600,000 | ~212 GB | More realistic for slow process values. |

Formula:

```text
samples = tag_count * samples_per_second * seconds
size_bytes = samples * estimated_bytes_per_sample
```

## Write Rate

Current implementation writes one sample per SQLite transaction through `HistorianStore::write_sample()`. That favors simple correctness over bulk ingest throughput.

No committed benchmark harness existed before `CODEX-AQ`, and this sandbox blocked machine-spec introspection. Treat write-rate numbers as deployment-specific until a dedicated benchmark lands. On production hardware, measure with the actual tag mix, disk, filesystem, and backup cadence before promising a retention window.

## Retention

The store exposes two explicit pruning primitives:

- `HistorianStore::prune_older_than(tag_path, cutoff_ms)`
- `HistorianStore::prune_to_max_rows(tag_path, max_rows)`

They are opt-in engine primitives. Default behavior remains no automatic pruning. Runtime policy wiring in the recorder and a Designer retention UI are separate follow-ups; regulated deployments should configure pruning conservatively and keep retention age at least twice the backup interval.

## Backup

`HistorianStore::backup_to_path()` snapshots the live database using SQLite's online backup API. `HistorianStore::restore_from_path()` replaces a live store from a backup snapshot, and `merge_from_path()` appends non-duplicate samples from another historian DB.

If retention is enabled operationally, take backups before pruning can remove data that still falls inside the required record-retention period.

## File Location

The gateway owns the concrete historian file path when it creates the `HistorianStore`. For project backups, the historian SQLite file is exported as `historian.sqlite` inside the `.owhmi` archive. For local inspection, use SQLite tools against a stopped gateway or a copied online-backup snapshot, not the hot live file.
