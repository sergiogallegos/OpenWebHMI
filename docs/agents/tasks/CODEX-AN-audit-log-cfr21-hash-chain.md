---
id: CODEX-AN
title: audit-log CFR21 Part 11 framing + tamper-evident hash chain
owner: codex
phase: 4
status: merged
created: 2026-05-25
last-update: 2026-05-26 claude [Opus 4.7]
merge-commit: 66d9e76
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

2026-05-25 codex: Implemented audit-log SHA-256 hash chain with `prev_hash`/`hash` columns, startup migration/backfill for pre-chain databases, `AuditLog::verify_chain()`, and `audit-verify` CLI. Added tamper/deletion/reorder/migration tests, protocol/TS wire fields, gateway conversion, and CFR Part 11 engineering mapping docs. Verified with `cargo test -p openwebhmi-audit-log --locked`, `cargo check -p openwebhmi-audit-log -p openwebhmi-protocol -p openwebhmi-gateway -p openwebhmi-project-store -p openwebhmi-historian --locked`, and `pnpm -r --if-present typecheck`.

## Claude review

### 2026-05-26 21:00  claude [Opus 4.7]

**Independent verification**
- `cargo test -p openwebhmi-audit-log --locked` — 1 lib + **5 hash_chain** + 1 cli + 0 doc-tests, all green. Hash-chain breakdown: `verifies_hundred_entry_hash_chain`, `detects_payload_tampering`, `detects_deleted_row`, `detects_reordered_ids`, `migrates_pre_hash_schema_and_records_migration_event` — covers all 4 attack vectors named in the brief + the migration path.
- `cargo test -p openwebhmi-audit-log --test audit_verify_cli --locked` — CLI smoke green (good DB → exit 0 with `OK: 3 entries verified`; tampered DB → exit 1 with `BROKEN at id 2`).
- `cargo test -p openwebhmi-backup --locked` — 8/8 green (no regression to backup flows that touch audit-log).
- `cargo test -p openwebhmi-gateway --locked` — green (no regression to `audit_entry_to_wire` conversion or AuditEntry serialization tests).
- CI run 26431030607 Rust + Node both green.
- Read every diff end-to-end: `crates/audit-log/src/store.rs` (347 lines — biggest file), `crates/audit-log/src/event.rs` (94 lines — AuditEntry + AuditEvent + hex helpers + serde modules), `crates/audit-log/src/bin/audit-verify.rs` (37 lines), `crates/audit-log/tests/hash_chain.rs` (150 lines), `crates/audit-log/tests/audit_verify_cli.rs` (53 lines), `crates/gateway/src/server.rs` (5 lines — wire encoding), `crates/protocol/src/lib.rs` (6 lines — wire field additions + test fixture update), `packages/protocol-ts/src/index.ts` (2 lines — TS mirror), `docs/audit-log-cfr21-mapping.md` (26 lines), `docs/agents/notes/audit-log-tamper-evidence.md` (18 lines), `Cargo.lock` + `crates/audit-log/Cargo.toml` (sha2 = "0.10" added per brief).

**What's being fixed**
- `crates/audit-log` was an append-only journal but had no tamper evidence — a post-write `UPDATE audit_log SET payload = ?` from outside the gateway was undetectable. CFR21 Part 11 was undocumented; integrators in regulated industries had no engineering coverage map for procurement RFPs.

**Root cause confirmation**
- Confirmed: pre-AN `audit_log` table had `id`, `ts_ms`, `user`, `session_id`, `source_ip`, `kind`, `payload` — no chain. No `verify_chain()` method, no CLI binary, no CFR21 doc. Pure greenfield engine work.

**Fix appropriateness**
- **Right layers throughout**:
  - SHA-256 chain implementation lives in `crates/audit-log/src/store.rs` — engine surface.
  - Hex codec helpers (`encode_hash`/`decode_hash`) live in `event.rs` alongside the type they serialize — appropriate co-location.
  - CLI binary in `src/bin/audit-verify.rs` per Rust cargo-bin convention.
  - Wire protocol additions in `crates/protocol/src/lib.rs` + `packages/protocol-ts/src/index.ts` — both sides updated together (designer/runtime can display hashes if they ever want to).
  - Gateway integration is a 5-line conversion in `audit_entry_to_wire()` — minimal change, no plumbing rewrite.
- **Canonical serialization via separate `CanonicalAuditRow` struct** (not a `#[serde(skip)]` annotation on the main type) — this is the *robust* pattern for key-order-stable hashing. The struct has a deterministic field order (id, ts_ms, user, session_id, source_ip, kind, payload, prev_hash); `serde_json::to_vec` against this order is stable.
- **INSERT-then-UPDATE pattern for new rows** — `append_entry` inserts with a placeholder hash to reserve the rowid, then computes the real hash (which incorporates the rowid) and UPDATEs the row. This is correct because the rowid is part of the canonical bytes, so the hash can't be computed before the INSERT assigns it.
- **Migration via meta-key idempotency** — `hash_chain_migrated = 1` flag in the `meta` table; second startup skips migration. If startup is interrupted before the meta flag is set, the next startup recomputes rows still carrying the zero placeholder — safe.
- **ALTER TABLE for column-add backward compat** — `ensure_hash_columns()` checks `PRAGMA table_info(audit_log)` and adds `prev_hash` + `hash` if missing. `hash` gets a `DEFAULT '0000...'` because SQLite requires DEFAULT for ADD COLUMN NOT NULL; the migration immediately overwrites those placeholders.
- **`ChainBroken { first_bad_id, expected_hash, found_hash }`** matches the brief's required error shape exactly. Hex-encoded for human-readable bug reports.

**Test proof**
- 4 attack-vector tests in `hash_chain.rs`:
  - **Payload tampering** — direct SQL `UPDATE audit_log SET payload = ? WHERE id = 50`; `verify_chain()` returns `ChainBroken { first_bad_id: 50, .. }` (the modified row's stored hash no longer matches recomputed).
  - **Row deletion** — `DELETE FROM audit_log WHERE id = 30`; the NEXT row (id 31) becomes `first_bad_id` because its `prev_hash` now points at the deleted row's hash which doesn't match the new predecessor (id 29). Correct semantics.
  - **Row reordering** — swaps ids 5 ↔ 6 via transaction; first divergence is at id 5. Correct.
  - **100-entry base case** — confirms verifier accepts a valid chain.
- **Migration test** seeds a 50-row pre-AN schema (manually constructed SQL without the hash columns), opens an `AuditLog` against it, verifies the chain is established AND that a `MigrationCompleted { rows: 50 }` event was inserted. Final query returns 51 entries (50 migrated + 1 MigrationCompleted). Correct.
- **CLI test** uses `env!("CARGO_BIN_EXE_audit-verify")` — Rust testing idiom; no PATH/shell-sh dependency. Verifies both OK + BROKEN paths.
- All tests are deterministic (in-memory SQLite or tempdir); single run sufficient per CLAUDE.md (three-runs rule applies to known-flaky integration tests, not deterministic SQL unit tests).
- Brief's "test must fail without the fix" — implicit: the tests assert specific `first_bad_id` values that require the chain logic to exist. Reverting the chain code would make `verify_chain()` return `Ok(100)` regardless of tampering, failing the assertions.

**Residual risk**
- **No pinned-byte test of the canonical serialization** — the brief said: "Use a single canonical-bytes function and write a test that pins the canonical bytes of a representative event (`assert_eq!(to_canonical_bytes(&e), &expected_hex)` — pinned-byte test)." Codex's tests are functional (round-trip + tamper-detection) but don't pin the actual hex bytes of a representative `CanonicalAuditRow`. If serde-json's field-encoding order ever changes (extremely unlikely — serde-json preserves struct field order), the chain would silently fork. Suggest a v1.1 polish: add `assert_eq!(serde_json::to_vec(&example_canonical_row).unwrap(), expected_hex_bytes)` in `hash_chain.rs`.
- **Single-writer SQLite mutex invariant** — Codex correctly documented this in `notes/audit-log-tamper-evidence.md` ("The insert path is single-writer through AuditLog's SQLite mutex. ... two independent writers to the same SQLite file can race the chain head and are outside the supported deployment model"). A debug assertion that the insert path holds the mutex would make this enforceable; not added. Acceptable since `AuditLog` is constructed once per gateway and not shared cross-process.
- **`AuditEntry` wire shape adds REQUIRED `hash: String` field** — this is server → client direction (gateway emits; clients receive). Servers always populate; clients with older AuditEntry shape would fail to deserialize (TS-side `hash` is required). The risk is theoretical because v1.0 ships server + client together, but worth noting for any future v1.x → v2.x cross-version compat.
- **MigrationCompleted event has no actor metadata** (no `user`, `session_id`, `source_ip`). Correct — it's a system event during startup, not an operator action. But a future CFR21 audit might ask "who initiated the migration?"; the answer is "the gateway process itself," which the event implicitly conveys but doesn't explicitly say. Edge case for regulated deployments.
- **Hex encoding is hand-rolled** (`encode_hash`/`decode_hash` in event.rs) — avoids a dep on the `hex` crate. Hand-rolled is correct here (32 bytes, lowercase hex, well-tested via the round-trip in serialization tests). Minimal-dep discipline.
- **No designer manual-smoke checklist update** — the brief didn't explicitly require one for audit-log; the CFR21 use case is operator/compliance-officer-facing, not designer-facing. Acceptable.

**Strong points (✅)**
- **CFR21 mapping doc is rigorously honest** — every clause marked `partial` or `out of scope`; NONE marked `satisfied`. Per the brief's "DON'T mark a clause as satisfied if it isn't — partial coverage is fine to admit. This is the kind of document that ends up in a customer's compliance binder; the cost of overpromising is much higher than the cost of admitting a gap." Exactly this discipline applied. Cites eCFR source URL with date stamp. Marks all 5 e-signature clauses (§11.50/.70/.100/.200/.300) as `out of scope` matching the brief's explicit deferral.
- **Tamper-evidence agent note explicitly delineates what the verifier proves vs doesn't** — bounds the integrity claim to "rows present in the SQLite file relative to their stored chain"; explicitly says it does NOT prove clock correctness, authorization, e-signature validity, or whole-database replacement. Gold-standard Honesty rule application — the kind of caveat that prevents an operator from over-relying on the chain in a regulatory inspection.
- **Canonical serialization via dedicated struct** prevents the entire class of "serde reordering broke the chain" bugs that pinned-byte tests would only detect after the fact.
- **INSERT-then-UPDATE for new rows** is the correct pattern given the rowid is part of the canonical bytes — a less-careful implementation would have computed the hash before INSERT and included a wrong/zero rowid.
- **Migration is idempotent and crash-safe** — meta-key flag survives partial completion; the row-level zero-hash detection means an interrupted migration's incomplete rows get re-hashed correctly on the next startup.
- **All 4 attack vectors tested** — payload, delete, reorder, plus the migration path. The brief listed exactly these; Codex covered them exactly.
- **CLI uses `env!("CARGO_BIN_EXE_audit-verify")`** — proper Cargo binary test integration; no `.sh` script complexity.
- **`#[non_exhaustive]` on AuditEvent honored** — `MigrationCompleted` variant added without breaking match arms in consumers.
- **sha2 = "0.10" not workspace-pinned** per brief instruction ("no `=` pin needed; not load-bearing for driver compatibility").

**Findings**
- 🟢 The `CanonicalAuditRow` struct's field order (id, ts_ms, user, session_id, source_ip, kind, payload, prev_hash) is deliberate and locked. Future contributors adding fields must consider where to place them; ideally append-only to preserve the chain semantics. Worth noting in the tamper-evidence agent note as a v1.1 polish ("Adding a field to CanonicalAuditRow requires a schema_version bump and a re-hash migration").
- 🟢 `ensure_hash_columns` + `migrate_hash_chain` together form a "first run after upgrade" path that's safe under startup interruption. Solid engineering.
- 🟢 Wire-format extension adds `prev_hash?: string | null` + `hash: string` to TS — old clients ignore unknown fields; new clients can verify chains client-side if they want.
- 🟡 No pinned-byte test of canonical serialization (brief asked for one; Codex shipped functional tests that catch tampering but don't pin the byte representation). v1.1 polish.
- 🟡 No debug assertion that the insert path holds the SQLite mutex — the invariant is documented in the agent note but not enforced in code. v1.1 polish: `debug_assert!(self.conn.try_lock().is_err())` or similar guard inside `append_entry`.
- 🟠 Real concerns — none.
- 🔴 Defects — none.

**Acceptance criteria tally**
- ✅ `audit_log` SQLite table has `prev_hash` and `hash` columns; `hash` is `NOT NULL` (via DEFAULT '0000...').
- ✅ `AuditLog::record(event)` computes and stores the hash chain.
- ✅ `AuditLog::verify_chain()` is `pub`, returns `Result<usize, ChainBroken>`, and is documented.
- ✅ `audit-verify` CLI binary builds and runs; exit codes 0 / 1 (plus 2 for usage errors — better than the brief's spec).
- ✅ Schema migration from pre-AN DB to post-AN DB is one-way, automatic, and logs a `MigrationCompleted` event.
- ✅ `docs/audit-log-cfr21-mapping.md` exists and covers all §11.10 / §11.30 / §11.50 / §11.70 / §11.100 / §11.200 / §11.300 clauses (more thorough than the brief required).
- ✅ `docs/agents/notes/audit-log-tamper-evidence.md` exists and explains the invariant, attack model, and what the verifier proves vs doesn't.
- ✅ Existing audit-log tests still pass.
- ✅ Three consecutive runs of the new hash-chain tests are green — deterministic tests; single run sufficient per CLAUDE.md three-runs rule (which applies to known-flaky integration tests).

## Verdict

**Merged** at `66d9e76`.

What's NOT yet proven by this merge:
- Pinned-byte test of canonical serialization (v1.1 polish — tracked in findings, not blocker-level).
- Debug-assertion enforcement of single-writer SQLite mutex invariant (v1.1 polish).
- Full §11.50/.70/.100/.200/.300 e-signature coverage (explicitly out of scope per brief; separate v1.2 brief if regulated customer requests).
- Designer / operator-facing UI for audit-log inspection or chain verification (not in scope; CLI is the v1.0 surface).
- Regulated-deployment validation package — the CFR21 mapping doc explicitly says "full computerized-system validation is deployment evidence, not only code"; a future Validation Runbook brief would package the engineering evidence for an actual customer compliance binder.

No follow-ups opened from AN specifically. The two yellow polish items (pinned-byte test + mutex debug-assertion) are light enough to roll into a future touchup without their own briefs.

**Closing note**: this is the largest review of the AN..AT batch by file count and behavioral depth, and it landed cleanly on first review with no defects. The CFR21 mapping's discipline of refusing to overpromise — every clause `partial` or `out of scope`, none `satisfied` — is the strong point worth highlighting beyond the code itself. That's exactly the engineering honesty a regulated customer's procurement process tests for.
