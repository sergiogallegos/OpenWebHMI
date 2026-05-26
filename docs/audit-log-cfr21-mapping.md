# Audit Log 21 CFR Part 11 Mapping

This is an engineering coverage map, not a legal certification. It maps the current OpenWebHMI audit-log implementation to the current eCFR text for 21 CFR Part 11, displayed by eCFR as Title 21 up to date as of 2026-05-21. Source: <https://www.ecfr.gov/current/title-21/chapter-I/subchapter-A/part-11>.

| Clause | Status | OpenWebHMI coverage | Gap / follow-up |
|---|---|---|---|
| §11.10(a) validation and altered-record detection | partial | `crates/audit-log/src/store.rs` computes `prev_hash`/`hash` and `AuditLog::verify_chain()` detects altered, deleted, or reordered rows. Tests in `crates/audit-log/tests/hash_chain.rs` cover tamper cases. | Full computerized-system validation is deployment evidence, not only code. Requires validation package/runbook brief. |
| §11.10(b) accurate and complete copies | partial | `AuditLog::query()` returns structured entries; gateway audit query messages expose entries to clients. Hashes are included in wire entries. | Human-readable export/print package for inspections is not implemented. |
| §11.10(c) record protection and retrieval through retention | partial | SQLite persistence plus retention setting; hash verification proves stored rows have not changed since insertion. | Retention policy is operator-configured and not yet tied to regulated-record retention classes. |
| §11.10(d) authorized system access | partial | Gateway auth uses local users, roles, and JWT sessions; audit events include user/session/source metadata. | Role policy validation and account lifecycle procedures are deployment controls. |
| §11.10(e) secure, time-stamped audit trails | partial | Audit events are computer-generated, timestamped, append-only through `AuditLog::append_at()`, and chained with SHA-256. | System-clock trust and time-source configuration are not enforced by the audit-log crate. |
| §11.10(f) operational system checks | out of scope | Not an audit-log engine responsibility. | Needs workflow/recipe sequencing controls in the feature that owns the operation. |
| §11.10(g) authority checks | partial | Gateway handlers authorize actions before writes, project changes, user admin, and audit reads; denied/failed operations can be logged. | Critical-action reauthentication and e-signature authority checks are separate. |
| §11.10(h) device/input source checks | partial | Audit entries carry `source_ip` when the gateway captures a peer address. | Device identity, terminal validation, and trusted client registration are not implemented. |
| §11.10(i) training/experience | out of scope | No code artifact can prove personnel training. | Deployment SOP/training records required. |
| §11.10(j) written accountability policies | out of scope | Audit entries support accountability evidence. | Customer SOP/policy requirement; not implemented in product code. |
| §11.10(k)(1) documentation access/distribution controls | out of scope | Repository docs are versioned. | Controlled document management is outside v1 runtime. |
| §11.10(k)(2) documentation revision/change audit trail | partial | Project changes can emit audit events; source control tracks product docs. | In-product controlled-document workflow is not implemented. |
| §11.30 open-system controls | partial | Hash chain provides integrity verification for audit records at rest; TLS is supported by the gateway when configured. | Encryption/digital-signature standards for open transmission require deployment design and likely a separate hardening brief. |
| §11.50 signature manifestations | out of scope | No electronic-signature record model yet. | Separate e-signature brief: signer name, timestamp, and signature meaning on signed records. |
| §11.70 signature/record linking | out of scope | Hash chain links audit entries to each other, not electronic signatures to signed records. | Separate e-signature brief. |
| §11.100 general e-signature requirements | out of scope | Local user identity exists. | Legal certification, identity proofing, and unique e-signature lifecycle are outside this brief. |
| §11.200 e-signature components and controls | out of scope | Login uses username/password. | Critical signing ceremonies with two components are not implemented. |
| §11.300 identification-code/password controls | partial | Local users, password hashing, JWT sessions, and role checks exist. | Password aging, token/card device controls, urgent security reporting, and periodic device testing are not complete. |

The eCFR text states that §11.10 closed-system controls include validation, record copies, retrieval protection, access limits, secure time-stamped audit trails, authority checks, and documentation controls; §11.30 applies those controls plus open-system integrity/confidentiality measures; §§11.50 and 11.70 define signature manifestation/linking; §§11.100, 11.200, and 11.300 define e-signature and identification-code/password controls.
