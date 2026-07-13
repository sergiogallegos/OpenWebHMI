---
id: CODEX-BH
title: Audit hash-chain tamper-evidence — HMAC keyed digest + external anchor option
owner: codex
phase: 4
status: open
created: 2026-07-12
last-update: 2026-07-12 claude [Fable 5]
---

# CODEX-BH — Audit hash-chain tamper-evidence: HMAC keyed digest (Tier-1 SECURITY)

## Brief

> **v1.0 blocker (elevated for CFR21 procurement).** The audit log's hash chain is an *unkeyed* SHA-256 over public row fields, stored in the same SQLite file it's meant to protect. Anyone who can write the DB file can edit or delete a row and recompute the entire chain; `verify_chain` and the `audit-verify` binary then report OK. Re-key the chain with an HMAC whose key is held **outside** the database (provisioned in the same shape as the JWT secret from CODEX-BD), optionally anchor the head hash to an append-only external sink periodically, and provide a one-way re-keying migration for existing unkeyed chains (mirror the CODEX-AN migration pattern). Preserve the `CanonicalAuditRow` serialization discipline and `#[non_exhaustive]` on `AuditEvent`. The CFR21 coverage map stays honest — "engineering coverage, not legal certification."

### Goal

The audit chain digest is `HMAC-SHA256(key, ...)` with a key provisioned from outside the SQLite file (env/flag, like the JWT secret). An attacker who can write the DB but does not hold the key can no longer forge a chain that passes verification: a row edited directly in the DB fails HMAC verification even after the attacker recomputes every stored hash. Existing unkeyed chains migrate one-way to the keyed scheme via a documented migration (recorded as an audit event, like CODEX-AN's migration). A regression test demonstrates that a tampered row which passes the *old* unkeyed check now *fails* HMAC verification.

### Context to read first

- `crates/audit-log/src/store.rs:379-397` — `compute_hash`. Builds a `CanonicalAuditRow` (368-377), then `Sha256::new()` + `update(prev_hash)` + `update(serde_json::to_vec(&canonical))`. Unkeyed: every input (prev_hash, row fields) is either public or reconstructable, so anyone can recompute the whole chain. This is the function to convert to HMAC.
- `crates/audit-log/src/store.rs:368-377` — `CanonicalAuditRow` (`#[serde]` struct). The canonical serialization discipline (stable field order, `prev_hash` encoding) must be preserved exactly so migrated chains and new rows agree; only the *keying* changes, not the canonical byte layout of the row.
- `crates/audit-log/src/store.rs:115` (`AuditLog::verify_chain`) and `:442` (`fn verify_chain(conn)`) — the verification path that must use the HMAC key. Both the library API and the `audit-verify` binary depend on it.
- `crates/audit-log/src/bin/audit-verify.rs` — the standalone verifier. It must obtain the HMAC key (same provisioning shape as the gateway) to verify; without the key it can confirm chain *linkage* but must clearly report it cannot attest *authenticity*. Make that distinction explicit in its output.
- `crates/audit-log/src/event.rs:33-34` — `#[non_exhaustive] pub enum AuditEvent`. Preserve this. The migration completion should be recorded as an `AuditEvent` variant (CODEX-AN added a `MigrationCompleted`-style variant — reuse or extend, consistent with `#[non_exhaustive]`).
- CODEX-AN (`docs/agents/tasks/CODEX-AN-audit-log-cfr21-hash-chain.md`) — the CFR21 hash-chain work and its schema migration pattern (records a migration event, one-way, documented). Mirror that migration shape; do not invent a new one.
- CODEX-BD (`docs/agents/tasks/CODEX-BD-jwt-secret-boot-refusal.md`) — the JWT secret provisioning shape (env/flag, refuse-to-boot-without). Provision the audit HMAC key the same way for consistency; coordinate so the two secrets are distinct keys with parallel provisioning ergonomics.
- The CFR21 coverage-map doc (search `wiki/` / `docs/` for the CFR21 / 21 CFR Part 11 coverage map, likely produced by CODEX-AN) — keep its "engineering coverage, not legal certification" framing; update the tamper-evidence row to reflect keyed HMAC + optional anchoring.
- [`docs/agents/notes/toolchain-drift.md`](../notes/toolchain-drift.md) — for the `hmac`/`sha2` crate availability question (check the lockfile before adding a dep).

### Files to create / modify

1. **Modify** `crates/audit-log/src/store.rs`:
   - Convert `compute_hash` to `HMAC-SHA256(key, prev_hash || canonical_bytes)`. Thread an HMAC key into `AuditLog` (store it in the handle, provisioned at construction). Keep `CanonicalAuditRow` and its serialization byte-for-byte identical; only the digest keying changes.
   - Update `verify_chain` (both 115 and 442) to recompute with the key. A row whose stored hash doesn't match the HMAC recomputation fails verification.
   - Optional periodic **external anchor**: expose a way to emit the current head hash to an append-only sink (e.g. an append-only file or a callback the gateway can wire to syslog). Keep it optional and minimal for v1.0 — the required deliverable is the HMAC; the anchor is a documented, testable extra point.
2. **Modify** `crates/audit-log/src/bin/audit-verify.rs`: accept the HMAC key (env/flag, mirroring provisioning). With the key: full authenticity verification. Without the key: report linkage-only and state clearly it cannot attest authenticity (don't print a bare "OK" that implies more than it verified — honesty discipline).
3. **Migration** (mirror CODEX-AN):
   - A one-way migration that re-keys an existing unkeyed chain to HMAC under the provisioned key, recorded as an audit migration event. Document that this is a trust-boundary transition: rows written before keying were never authenticated, so the migration attests "chain re-keyed as of <id/timestamp>," not "historical rows proven untampered." State this plainly in the migration event/docs.
   - Provide the migration path the same way AN did (schema/version bump + a migrate step run at open). Existing DBs must not hard-fail; they migrate forward.
4. **Modify** the CFR21 coverage-map doc: update the tamper-evidence entry to describe keyed HMAC + optional anchoring; retain "engineering coverage, not legal certification."
5. **Provisioning** (`crates/gateway/src/main.rs`): supply the audit HMAC key via env/flag (e.g. `--audit-hmac-key` / `OPENWEBHMI_AUDIT_HMAC_KEY`) in the same shape as BD's JWT secret. Coordinate the "refuse to boot without a key" posture with BD — the audit key should be at least as strict, since a missing/weak key defeats the whole point. If BD lands first, reuse its provisioning helper shape.
6. **Do NOT**:
   - Change `CanonicalAuditRow`'s field set or order, or the `prev_hash` encoding.
   - Remove `#[non_exhaustive]` from `AuditEvent`.
   - Claim legal CFR21 compliance anywhere.

### Behavior

- New rows are chained with `HMAC-SHA256(key, prev_hash || canonical)`. `verify_chain` with the correct key passes for an untampered chain.
- An attacker edits a row directly in SQLite and recomputes every stored hash with unkeyed SHA-256 (the old scheme) → `verify_chain` with the key **fails** (the recomputed hashes don't match the HMAC).
- `audit-verify` with the key attests authenticity; without the key it reports linkage-only and explicitly does not claim authenticity.
- Opening an existing unkeyed DB triggers the one-way re-key migration, recorded as an audit event; subsequent verification uses HMAC.
- The optional anchor, when enabled, appends the head hash to the configured sink.

### Test requirements

- Extend `crates/audit-log/tests/` — the existing hash-chain test file (`hash_chain.rs`) is the home; don't fragment.
  - **Tamper-evidence (the load-bearing test):** build a chain, then simulate an attacker: mutate a row's payload and recompute the *entire* chain using the **old unkeyed** SHA-256 scheme (so it would pass the pre-fix `verify_chain`). Assert the new HMAC `verify_chain` **fails** on that forged chain. This test must fail against the pre-fix (unkeyed) code — i.e. under unkeyed hashing the forged chain verifies OK; under HMAC it must not.
  - **Round-trip:** an untampered HMAC chain verifies OK with the key.
  - **Wrong key:** verifying an HMAC chain with a different key fails (proves the key is load-bearing, not decorative).
  - **Migration:** open a DB with a pre-existing unkeyed chain, run the migration, confirm a migration audit event is recorded and subsequent verification uses HMAC. Reuse/extend the CODEX-AN migration test rather than creating a new file.
  - **audit-verify without key:** confirm it does not print a bare authenticity "OK."
- No `sleep`/wall-clock. Use `tempdir` DBs (no hardcoded paths). Full Rust matrix clean (`cargo doc` too — `#![deny(missing_docs)]` is the crate bar; new public items need docs).

### Acceptance criteria

- [ ] `compute_hash` uses `HMAC-SHA256` with an externally-provisioned key; `CanonicalAuditRow` serialization is byte-for-byte unchanged.
- [ ] `verify_chain` (library API + `fn verify_chain`) recomputes with the key; a directly-edited row fails verification even after full unkeyed recomputation.
- [ ] The tamper-evidence regression test forges a chain that passes the old unkeyed check and fails the new HMAC check; it fails against pre-fix code.
- [ ] Wrong-key verification fails (key is load-bearing).
- [ ] One-way re-keying migration for existing unkeyed chains, recorded as an audit event, mirroring CODEX-AN; existing DBs migrate forward without hard-failing.
- [ ] `audit-verify` obtains the key (env/flag); without it, reports linkage-only and does not claim authenticity.
- [ ] Audit HMAC key provisioned in the same shape as BD's JWT secret (env/flag, refuse-to-boot-without or at least as strict); keys are distinct.
- [ ] `#[non_exhaustive]` on `AuditEvent` preserved; `CanonicalAuditRow` field set/order preserved.
- [ ] CFR21 coverage-map tamper-evidence entry updated; "engineering coverage, not legal certification" retained.
- [ ] No `panic!`/`expect`/`unwrap` on reachable paths; full Rust matrix clean including `cargo doc`; no new `#[allow]` (use `#[expect]` + reason).

### Out of scope

- **Asymmetric signing / external notary / blockchain anchoring.** HMAC + an optional append-only local anchor is the v1.0 bar; a signed-by-HSM notary is post-1.0.
- **Encrypting the audit DB at rest.** Tamper-evidence, not confidentiality, is this brief's target.
- **Legal CFR21 certification.** The doc stays "engineering coverage, not legal certification" — no compliance claims.
- **Backfilling authenticity for pre-migration rows.** They were never authenticated; the migration attests the re-key point forward, honestly documented — it does not retroactively prove old rows untampered.
- **Changing `CanonicalAuditRow` or `AuditEvent` shape** beyond adding/reusing a migration event variant.
- **Key rotation for the audit HMAC key.** Single active key for v1.0; rotation is a follow-up (and interacts with historical verification — note it as a known limitation).

### Risks / gotchas

- **Key rotation breaks historical verification — call it out, don't solve it.** With a single HMAC key, rotating the key invalidates verification of everything signed under the old key. v1.0 uses one key; document that rotation is unsupported and would require multi-key verification (a v1.1 concern). Don't half-build rotation.
- **The migration's honest claim.** Re-keying an unkeyed chain cannot prove the old rows weren't already tampered before migration — an attacker could have altered them under the unkeyed scheme. The migration attests "chain re-keyed and authenticated from here forward," not "history proven clean." Per CLAUDE.md honesty discipline, the migration event and docs must say exactly that. Overstating it is the trap.
- **Key must live outside the DB.** If the key ends up in the same SQLite file (or a sibling the same attacker can write), the fix is defeated. Provision it like the JWT secret — env/flag, operator-held. Coordinate with BD so both secrets share provisioning ergonomics but are distinct keys.
- **`hmac` crate availability.** Check the lockfile before adding a dependency — `sha2` is already present (used by `compute_hash`); the `hmac` crate (RustCrypto) pairs with it and may already be transitively available. If a new direct dep is needed, add only `hmac` at a pinned version and note it; don't `cargo update`.
- **`CanonicalAuditRow` must not drift.** The whole scheme depends on stable canonical bytes. Change only the keying of the digest, not the row's serialized form. A subtle field-order or encoding change silently invalidates every existing hash and makes the migration a no-op that hides breakage.
- **`audit-verify` honesty.** Without the key it can still check that each row's `prev_hash` links to the previous row's `hash` (structural linkage) but cannot attest the hashes are authentic. It must not print a bare "OK" — say "linkage OK; authenticity not verified (no key)." A verifier that overclaims is worse than none for a compliance context.
- **`cargo doc` / `#![deny(missing_docs)]`.** The audit-log crate denies missing docs. Any new public key-provisioning type/method needs a doc comment or the doc build fails.
- **Coordinate merge order with BD.** If BD lands first, reuse its secret-provisioning helper; if BH lands first, note the shape so BD matches. Either way the two provisioning surfaces should look alike to the operator.

## Codex log

## Claude review

## Verdict
