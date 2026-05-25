---
id: CODEX-AN
title: audit-log CFR21 Part 11 framing + tamper-evident hash chain
owner: codex
phase: 4
status: open
created: 2026-05-25
last-update: 2026-05-25 claude [Opus 4.7]
---

# CODEX-AN — audit-log CFR21 Part 11 framing + tamper-evident hash chain

## Brief

> Add a hash-chain `prev_hash`/`hash` to every audit entry so tampering with any earlier event invalidates all subsequent hashes; ship a verification CLI; document which FDA 21 CFR Part 11 clauses the current `crates/audit-log` satisfies, which are partially satisfied, and which need a separate brief. This unlocks pharma / medical-device / food-safety procurement conversations that currently fail at the audit-log line of the RFP.

### Goal

`crates/audit-log` becomes a **tamper-evident** audit journal: any post-write modification, deletion, or row insertion is detectable by recomputing the chain. A maintainer or auditor can run `cargo run --bin audit-verify -- <path-to-db>` and get a clean pass or a precise breakage report (which `id` first diverged).

This is a v1.1 brief — not a v1.0 closeout gate — but lands cleanly on the existing CODEX-AE infrastructure without architectural change.

### Context to read first

- `crates/audit-log/src/{lib,event,query,store}.rs` — current API surface.
- `docs/agents/tasks/CODEX-AE-audit-log.md` — original brief + verdict for the audit-log crate.
- The `AuditEvent` enum in `event.rs` is `#[non_exhaustive]` (AL sweep), so adding a `Tampered { id, expected_hash, found_hash }` variant for the verifier output is safe.

### Files to create / modify

- **Modify** `crates/audit-log/src/store.rs` — add `prev_hash` and `hash` columns to the `audit_log` table; recompute `hash` on insert; add a `verify_chain()` method.
- **Modify** `crates/audit-log/src/event.rs` — the `AuditEntry` struct gains `prev_hash: Option<[u8; 32]>` and `hash: [u8; 32]` fields. Use SHA-256.
- **Create** `crates/audit-log/src/bin/audit-verify.rs` — CLI binary: `audit-verify <path>` reads the DB and prints `OK: <n> entries verified` or `BROKEN at id <n>: expected <hex>, found <hex>` with exit code 1.
- **Create** `docs/audit-log-cfr21-mapping.md` — table mapping each Part 11 §11.10(a)–(k), §11.30, §11.50, §11.70, §11.100, §11.200, §11.300 clause to one of: `satisfied` (with file:line), `partial` (with gap named), `out of scope` (with separate-brief pointer), or `not applicable`.
- **Create** `docs/agents/notes/audit-log-tamper-evidence.md` — agent note explaining the hash-chain invariant, what happens on schema migration, what happens on a row-deletion attack, what the verifier proves vs doesn't.
- **Modify** `crates/audit-log/Cargo.toml` — add `sha2 = "0.10"` (no `=` pin needed; not load-bearing for driver compatibility).

### Behavior

- **Insert path**: `AuditLog::record(event)` computes `hash = sha256(prev_hash || serialized_canonical_entry)`. `serialized_canonical_entry` is a deterministic byte representation of all fields except `hash` itself — use `serde_json` with `serde_json::to_vec` and key-sorted output (or define a small `to_canonical_bytes()` method to avoid serde-key-order surprises).
- **First entry**: `prev_hash` is `None`; hash input is `[0u8; 32] || canonical_entry`.
- **Schema migration**: existing rows without `prev_hash` / `hash` are migrated **once** at startup — compute hashes in `id` order, link them into a chain, log a single `MigrationCompleted { rows: n }` audit event whose `prev_hash` is the last migrated row's `hash`. The migration is not reversible; document this in the agent note.
- **Verification**: `verify_chain()` reads rows in `id` order, recomputes each hash, and returns `Ok(count)` or `Err(ChainBroken { first_bad_id, expected, found })`.
- **Subscription path**: emitted `AuditEntry` over the wire protocol now includes `prev_hash` and `hash` in the JSON shape. Bump the protocol message version? **No** — `AuditEntry` is `#[non_exhaustive]` per CODEX-AL, and the wire format adds two optional/required fields without breaking deserialization for older clients (they ignore unknown fields).

### Test requirements

- Unit test in `crates/audit-log/tests/hash_chain.rs`:
  - Insert 100 events; `verify_chain()` returns `Ok(100)`.
  - Tamper with row 50's `event` JSON in the DB directly (raw SQL update); `verify_chain()` returns `Err(ChainBroken { first_bad_id: 50, .. })`.
  - Delete row 30 from the DB; `verify_chain()` returns `Err(..)` (row 31's `prev_hash` no longer matches).
  - Reorder rows (`id 5` ↔ `id 6` via SQL); `verify_chain()` catches it.
- Migration test:
  - Open a DB at the pre-CODEX-AN schema, write 50 events, then upgrade — verify the chain is established, `verify_chain()` passes, and a `MigrationCompleted` event was inserted.
- CLI test in `crates/audit-log/tests/audit_verify_cli.sh`:
  - `audit-verify <good-db>` → exit 0, prints `OK`.
  - `audit-verify <tampered-db>` → exit 1, prints the broken id.
- The "test must fail without the fix" discipline applies — write the tamper detection tests **first**, run against pre-AN code (no hash column), confirm they fail with a "missing column" error or similar. Document that step in the Codex log.

### Acceptance criteria

- [ ] `audit_log` SQLite table has `prev_hash` and `hash` columns; `hash` is `NOT NULL`.
- [ ] `AuditLog::record(event)` computes and stores the hash chain.
- [ ] `AuditLog::verify_chain()` is `pub`, returns `Result<usize, ChainBroken>`, and is documented.
- [ ] `audit-verify` CLI binary builds and runs; exit codes 0 / 1 as specified.
- [ ] Schema migration from pre-AN DB to post-AN DB is one-way, automatic, and logs a `MigrationCompleted` event.
- [ ] `docs/audit-log-cfr21-mapping.md` exists and covers all §11.10 / §11.30 / §11.50 / §11.70 clauses.
- [ ] `docs/agents/notes/audit-log-tamper-evidence.md` exists and explains the invariant, attack model, and what the verifier proves.
- [ ] Existing audit-log tests still pass.
- [ ] Three consecutive runs of the new hash-chain tests are green (no flakiness from SQLite write ordering).

### Out of scope

- **Electronic signatures (§11.50, §11.70, §11.100, §11.200).** That's a separate v1.2 brief — would require re-authentication on critical actions, a signature-record table, and a UI surface.
- **Event-class opt-in granularity UI.** A designer panel to enable/disable audit categories per regulated-system policy is a separate v1.1 brief; this brief covers the *engine* side only.
- **Backfilling hashes on log rotation / archive.** Add to the agent note as a "known operational consideration" but don't implement; the AF/AI backup/restore path is the natural place for that.
- **Performance benchmarking.** SHA-256 per insert is sub-microsecond; don't add benches unless the audit-log insert path was previously benchmarked.

### Risks / gotchas

- **Canonical serialization is load-bearing.** Two different byte representations of the "same" event produce different hashes and break the chain. Use a single canonical-bytes function and write a test that pins the canonical bytes of a representative event (`assert_eq!(to_canonical_bytes(&e), &expected_hex)` — pinned-byte test).
- **SQLite WAL mode and the chain.** If two writers race, the `prev_hash` they each read may be stale. The audit-log crate is currently single-writer (per CODEX-AE); document this invariant explicitly in the agent note and add a debug-assertion that the insert path holds a `&mut` self or an exclusive lock.
- **Migration must be idempotent on second run.** If startup is interrupted mid-migration, the second startup should detect partial state and resume. Easiest: store `migration_completed_at` in the `meta` table; skip migration on second run if present.
- **`docs/audit-log-cfr21-mapping.md` honesty.** Don't mark a clause as `satisfied` if it isn't — partial coverage is fine to admit. This is the kind of document that ends up in a customer's compliance binder; the cost of overpromising is much higher than the cost of admitting a gap.
- **Wire-format expansion.** Confirm via `pnpm -r typecheck` that the TS side's `AuditEntry` regenerates cleanly with the two new fields. The ts-rs annotation must preserve `Option<[u8; 32]>` → `string | null` (hex-encoded) — pick a serde representation that gives a stable string in JSON, not a `[byte, byte, ...]` array.

## Codex log

## Claude review

## Verdict
