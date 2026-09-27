# Historian Capacity And Retention

OpenWebHMI's current historian is a single SQLite database managed by `crates/historian`.
Disk use, backup time, query latency, durability and retention are the practical limits.
The accepted [engine and capacity plan](planning/engine-and-capacity.md) defines a
30-day selected-history target at 2,500 admitted samples/s for the Medium profile,
independent of its 50,000 live tags. This is not a validated capability of the
current per-sample transaction implementation.

## Schema

The live schema is in `crates/historian/src/store.rs`: `tag_dictionary(tag_id, tag_path)` plus `tag_history(tag_id, ts_ms, value, quality)` with primary key `(tag_id, ts_ms)`.

Each sample stores `tag_id`, `ts_ms`, a JSON `TagValue` string, and a quality string. A realistic sizing estimate is:

```text
bytes_per_sample = 8 tag_id + 8 ts_ms + value_json_bytes + quality_bytes + 16..32 SQLite btree overhead
```

Measure this overhead on the target storage layout and value distribution. The
capacity plan uses an illustrative 64–128 bytes/sample range; strings and JSON may
exceed it. No compression savings are assumed.

## Capacity

Use the [30-day sizing table](planning/engine-and-capacity.md#thirty-day-history-sizing)
for Edge, Standard and Medium workloads. At 2,500 samples/s, 30 days holds
6.48 billion samples: approximately 415–829 GB before operational reserves.
Recording every one of 50,000 tags at 1 Hz is a different workload: approximately
8.29–16.59 TB before reserves. GB/TB are decimal, not GiB/TiB.

Formula:

```text
samples = tag_count * samples_per_second * seconds
size_bytes = samples * estimated_bytes_per_sample
```

## Write Rate

Current implementation writes one sample per SQLite transaction through `HistorianStore::write_sample()`. That favors simple correctness over bulk ingest throughput.

CODEX-DT owns the reproducible baseline/capacity harness. CODEX-DS owns batched
ingestion, durable admission, bounded backlog, disk-pressure handling and retention
policy; CODEX-CG owns SQL-side bounded/downsampled reads. Measure the actual tag
mix, disk, filesystem, month-sized dataset and backup cadence before promising a
retention window. Backend substitution is an explicit evidence-based decision,
not part of the current implementation.

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
