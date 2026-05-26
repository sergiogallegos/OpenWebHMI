# Audit Log Tamper Evidence

`crates/audit-log` stores a SHA-256 hash chain across `audit_log` rows. Each row stores `prev_hash` and `hash`; `hash` is computed from the previous hash bytes plus canonical row bytes containing `id`, timestamp, actor metadata, event kind, payload JSON string, and `prev_hash`. The first row uses 32 zero bytes as its previous hash input.

The insert path is single-writer through `AuditLog`'s SQLite mutex. It inserts a placeholder hash to reserve the SQLite id, computes the final hash while still holding the mutex, then updates the row before publishing the entry to subscribers. This is load-bearing: two independent writers to the same SQLite file can race the chain head and are outside the supported deployment model.

On startup, existing pre-hash-chain databases are migrated once. `init_schema()` adds `prev_hash` and `hash` when missing, backfills rows in `id` order, writes `meta.hash_chain_migrated = 1`, and appends a `MigrationCompleted { rows }` audit event linked to the migrated tail. The migration is one-way. If startup stops before the meta marker is written, the next startup recomputes rows still carrying the zero placeholder.

`AuditLog::verify_chain()` reads rows in `id` order and recomputes every link. A changed payload, changed actor/timestamp/kind, deleted row, or reordered id causes the first downstream mismatch to return `ChainBroken { first_bad_id, expected_hash, found_hash }`. The `audit-verify` CLI wraps this as:

```text
OK: <n> entries verified
BROKEN at id <n>: expected <hex>, found <hex>
```

The verifier proves integrity of the rows present in the SQLite file relative to their stored chain. It does not prove that the original system clock was correct, that the event should have been authorized, that an operator's electronic signature met Part 11 requirements, or that an attacker did not replace the entire database with a different internally consistent database. Those require deployment controls such as trusted time, backups, access control, and future e-signature work.

Row retention is an operational consideration. Deleting old audit rows is detectable because the next retained row's `prev_hash` no longer matches the verifier's expected chain head. Regulated deployments should either retain the full audit DB for the required period or archive a complete DB plus its terminal hash before pruning.
