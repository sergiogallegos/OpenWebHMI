---
id: CODEX-DJ
title: Remove unused EPL-derived Sparkplug schema and reconcile provenance docs
owner: codex
phase: 4
status: merged
created: 2026-09-01
last-update: 2026-09-01 codex [gpt-5]
---

# CODEX-DJ — Remove unused EPL Sparkplug schema

## Brief

### Goal

Remove the unused Eclipse Tahu-derived `sparkplug_b.proto` file so the repository
does not redistribute EPL source without its notices and the future AGPL product
does not inherit an unnecessary EPL compatibility/provenance blocker.

### Evidence

- `crates/driver-mqtt` has no `build.rs`, `prost-build`, `include!`, or generated
  module consuming `proto/sparkplug_b.proto`.
- `src/sparkplug.rs` already owns the required first-party `prost::Message` and
  `prost::Oneof` Rust definitions and their field tags.
- the `.proto` file is therefore dead reference material, not part of compilation.

### Required behavior

- delete only the unused EPL-derived `.proto` file;
- retain the existing first-party Rust wire definitions and behavior unchanged;
- update roadmap/wiki/audit statements that incorrectly say the schema is vendored;
- record the public Sparkplug specification as the interoperability reference;
- do not copy additional Eclipse Tahu source or remove third-party notices from
  any material that remains in the repository.

### Tests and verification

- prove repository-wide that no compiled or documented path references the deleted file;
- run `cargo check -p openwebhmi-driver-mqtt --all-features --locked`;
- run the MQTT crate tests three consecutive times because this is a mechanical
  production-tree removal;
- run formatting and agent-file validation.

### Acceptance criteria

- [ ] No Eclipse Tahu-derived `.proto` source remains in the repository.
- [ ] MQTT/Sparkplug compilation and tests remain green without the file.
- [ ] Docs describe first-party Rust wire types rather than a vendored schema.
- [ ] CODEX-DG's Sparkplug provenance blocker is explicitly resolved.

### Out of scope

- Expanding Sparkplug support or fixing behavior tracked by CODEX-BQ.
- Claiming Sparkplug Compatible certification.
- Changing the current root MIT license or the planned AGPL/MPL boundary.

## Codex log

### 2026-09-01  codex [gpt-5]

Started after CODEX-DG found the provenance issue. Confirmed the file has no
build consumer and `src/sparkplug.rs` already provides the compiled wire types.

### 2026-09-01  codex [gpt-5]

Deleted the unused EPL-derived file and reconciled current roadmap, MQTT wiki,
and licensing-audit language with the existing first-party Rust `prost` wire
types. The merged CODEX-Y brief remains unchanged as historical task context;
no compiled path or current product/architecture document references the deleted
file. Verification: MQTT all-feature locked check passed; formatting passed;
agent validation passed with 113 task files; three consecutive all-feature locked
MQTT test runs passed (9 unit + 2 simulator integration each). The first sandboxed
test attempt could not bind an ephemeral port and did not exercise the integration
tests; the three counted runs used approved local bind access.

## Codex review

### 2026-09-01  codex [gpt-5]

**Independent verification**

- Full locked Rust workspace build, strict Clippy, tests, rustdoc, and format checks passed.
- All workspace TypeScript typechecks/tests passed; `scripts/validate-agent-files` and `git diff --check` passed.
- Repository-wide build failure is limited to the unrelated pre-existing Designer Tauri version mismatch.

**What's being fixed**

- Remove an unused Eclipse Tahu-derived EPL schema that unnecessarily complicated provenance and future license compatibility.

**Root cause confirmation**

- Confirmed: `crates/driver-mqtt` has no schema build consumer; `src/sparkplug.rs` already defines the active first-party `prost` wire types.

**Fix appropriateness**

- Deleting dead reference source is the narrowest correct fix and leaves runtime behavior unchanged.

**Test proof**

- MQTT all-feature check passed, three consecutive MQTT test runs passed, and the independent full workspace test run passed.

**Residual risk**

- Sparkplug behavioral completeness remains under its existing hardening tasks; no compatibility certification is claimed.

**Strong points (✅)**

- Removes the provenance blocker without adding copied upstream code or changing wire behavior.
- Current roadmap, wiki, and audit language now match the compiled implementation.

**Findings**

- 🟢 Historical merged briefs retain their original references as immutable context.
- 🟠 Real concerns — none.
- 🔴 Defects — none.

**Acceptance criteria tally**

- ✅ No Eclipse Tahu-derived `.proto` source remains in the repository.
- ✅ MQTT/Sparkplug compilation and tests remain green without the file.
- ✅ Docs describe first-party Rust wire types rather than a vendored schema.
- ✅ CODEX-DG's Sparkplug provenance blocker is explicitly resolved.

## Claude review

## Verdict

**Merged.** Merge commit: `53843e5`.
The unused EPL-derived schema is removed, active first-party wire types are
unchanged, and independent workspace verification passed.
