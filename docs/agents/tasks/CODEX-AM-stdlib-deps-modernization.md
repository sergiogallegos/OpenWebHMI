---
id: CODEX-AM
title: Stdlib + deps modernization — thiserror = "2", single format-capture, missing-docs lint consistency
owner: codex
phase: 4
status: open
created: 2026-05-05
last-update: 2026-05-05 claude
---

# CODEX-AM — Stdlib + deps modernization (Tier 4 from review pass 2)

## Brief

> The last and smallest task in the post-AG quality sweep. Three narrow, mechanical, pre-v1.0 cleanups: (1) bump `thiserror` from 1.x to 2.x at the workspace level and absorb whatever build fallout appears; (2) fix the single `format!("{}", x)` non-capture site identified in review pass 2 at `crates/driver-opcua/src/driver.rs:236`; (3) add `#![deny(missing_docs)]` to the three driver crates that lack it (`driver-mqtt`, `driver-opcua`, `driver-ads`) and write doc comments for any pub items that this exposes. **Scope-locked.** No broader format-capture style sweep, no `Arc::clone(&x)` cosmetic rewrites, no reopening AL's ProjectStore identifier boundary, no other dep bumps, no exhaustive doc-coverage push beyond the three named crates. This brief exists specifically because each item is too small to deserve its own task and too useful to defer.

### Goal

After this lands:

- `Cargo.toml:42` reads `thiserror = "2"`. The workspace builds, tests pass, clippy `-D warnings` is clean. Any error-derive site that breaks on the bump (likely a `#[from]` annotation that's now too loose) gets a minimal targeted fix.
- The single `format!("{}", item.item_to_monitor().node_id)` at `driver-opcua/src/driver.rs:236` reads `format!("{node_id}")` after destructuring or a `let` binding. No other `format!`/`println!`/`tracing::*!` invocations are touched.
- `crates/driver-mqtt/src/lib.rs`, `crates/driver-opcua/src/lib.rs`, `crates/driver-ads/src/lib.rs` carry `#![deny(missing_docs)]` at the crate root. Every existing pub item in those three crates has a doc comment; the lint compiles clean.

### Context to read first

- **Original review pass 2 finding** (in `docs/agents/tasks/CODEX-AJ-async-hygiene.md`'s "Tier 5 minor cleanups" of the original review note): thiserror version pin, opcua format site, missing-docs lint consistency on the three driver crates.
- **CODEX-AG verdict** (`docs/agents/tasks/CODEX-AG-toolchain-edition-2024.md`): documents that the workspace deliberately holds `=`-pinned versions for `tokio-modbus`, `async-opcua`, `rumqttc`, `rumqttd`, `prost`, `jsonpath-rust`, `ads`. **Those pins are intentional and remain.** thiserror is not on that list — it's a major-version bump opportunity.
- **`Cargo.toml:42`** — `thiserror = "1"` workspace-level dep. All 13 crates inherit via `.workspace = true`, so the bump is one site.
- **`crates/driver-opcua/src/driver.rs:236`** — `let address = TagAddress::new(format!("{}", item.item_to_monitor().node_id));`. The line moved from :241 (original review) to :236 after AL's TagPath migration touched surrounding lines.
- **The three driver crates' lib.rs**:
  - `crates/driver-mqtt/src/lib.rs` — has `//! MQTT and Sparkplug B driver for OpenWebHMI.`, no lint attr.
  - `crates/driver-opcua/src/lib.rs` — has `//! OPC UA client driver for OpenWebHMI.`, no lint attr.
  - `crates/driver-ads/src/lib.rs` — has a fuller `//!` block; no lint attr.
- **Doc-lint reference** — every other driver crate (`driver-rockwell`, `driver-modbus`, `driver-api`) carries `#![deny(missing_docs)]`. That's the bar; match it.

### Files to modify

**Item 1 — `thiserror = "2"`:**

- `Cargo.toml:42` — change `thiserror = "1"` to `thiserror = "2"`.
- `Cargo.lock` will update; commit it.
- Run `cargo build --workspace --all-features --locked` and absorb any compile errors. The most common 2.x breakage is **stricter `#[from]` matching**: where a 1.x derive accepted a `From` impl with implicit coercion (e.g., `String` ⇄ `&str`), 2.x requires the source type to match the variant payload exactly. Where this fires, the minimal fix is an explicit `From` impl on the error type or a manual `match`/`map_err` at the call site.
- thiserror 2.x also tightens `#[error(transparent)]`; if a variant uses `transparent` with a non-error inner type, it'll fail. Should not affect us but verify.
- **Do not** rewrite error types beyond what's required to compile. Don't introduce new variants, don't restructure existing ones.

**Item 2 — Single format-capture cleanup:**

- `crates/driver-opcua/src/driver.rs:236` — change `format!("{}", item.item_to_monitor().node_id)` to use capture syntax. The cleanest shape:

  ```rust
  let node_id = item.item_to_monitor().node_id;
  let address = TagAddress::new(format!("{node_id}"));
  ```

  Or inline if `node_id` has a `Display` impl that produces the desired output:

  ```rust
  let address = TagAddress::new(format!("{}", item.item_to_monitor().node_id));
  // → use to_string() if simpler:
  let address = TagAddress::new(item.item_to_monitor().node_id.to_string());
  ```

  Pick whichever reads cleaner; the brief doesn't prescribe. **No other `format!` rewrites.**

**Item 3 — `#![deny(missing_docs)]` on three driver crates:**

- `crates/driver-mqtt/src/lib.rs` — add `#![deny(missing_docs)]` after the crate-level `//!` doc.
- `crates/driver-opcua/src/lib.rs` — same.
- `crates/driver-ads/src/lib.rs` — same.
- For each crate, run `cargo doc -p <crate> --no-deps` and fix every `missing_docs` warning. The expected fix per warning is a one-line `///` doc comment on the offending `pub` item. Where the pub item is internal-feeling (e.g., a re-export of a third-party type), prefer adding a one-liner `/// Re-exported from `foo` for convenience.` rather than removing the re-export.
- **Do not** restructure modules, change visibility, or remove pub items to avoid writing doc comments. Write the doc comments.
- **Do not** propagate `#![deny(missing_docs)]` to other crates that lack it. This brief is the three named crates only.

### Behavior

No behavior changes anywhere. This is a build-clean / lint-clean / dep-version task only.

### Test requirements

- **Existing matrix green**: `cargo fmt --all --check`, `cargo clippy --workspace --all-targets --all-features --locked -- -D warnings`, `cargo test --workspace --all-features --locked`, `cargo doc --workspace --no-deps`, `pnpm -r typecheck`, `pnpm -r test` — three consecutive runs each for `cargo test`.
- **No new tests** added. Each item is verified by the existing build + lint + doc tooling.
- **Cargo.lock review**: confirm only thiserror's entry (and any direct dependents) updated. If unrelated deps shifted versions, investigate before committing — that signals a wider lockfile drift than this brief authorizes.

### Acceptance criteria

- [ ] `Cargo.toml:42` reads `thiserror = "2"` (and any companion `thiserror = ` line in member crates' Cargo.toml continues to use `.workspace = true`).
- [ ] `cargo build --workspace --all-features --locked` succeeds.
- [ ] `cargo clippy --workspace --all-targets --all-features --locked -- -D warnings` clean.
- [ ] `cargo test --workspace --all-features --locked` green on three consecutive runs.
- [ ] `crates/driver-opcua/src/driver.rs:236` no longer contains `format!("{}", ...)`. The replacement reads cleanly.
- [ ] `crates/driver-mqtt/src/lib.rs`, `crates/driver-opcua/src/lib.rs`, `crates/driver-ads/src/lib.rs` carry `#![deny(missing_docs)]`.
- [ ] `cargo doc --workspace --no-deps` succeeds with zero new warnings.
- [ ] `Cargo.lock` diff is bounded (only thiserror + transitive crates that depend on it).
- [ ] `pnpm -r typecheck` + `pnpm -r test` green (regression sanity).

### Out of scope (explicit)

- **No broader format-capture sweep.** Other `format!("{}", x)` / `tracing::info!("{}", x)` sites stay as-is. Only the one named site at `driver-opcua/src/driver.rs:236` changes. If the workspace grep finds other candidates, they're tracked silently as v1.1 polish; this brief does not touch them.
- **No `Arc::clone(&x)` cosmetic rewrites.** ~321 `.clone()` sites in the workspace; explicit-refcount-intent is purely cosmetic. Skip entirely.
- **No reopening of AL's ProjectStore identifier boundary.** That's tracked as a v1.1 architectural follow-up in `wiki/architecture/api-surface-stability.md`.
- **No other dependency bumps.** The `=`-pinned versions (`tokio-modbus`, `async-opcua`, `rumqttc`, `rumqttd`, `prost`, `jsonpath-rust`, `ads`) stay at their pinned versions — they're load-bearing for compatibility. Other workspace deps stay at their current major versions.
- **No `#![deny(missing_docs)]` propagation** beyond the three named crates. Other crates that lack it stay as they are.
- **No clippy-rule additions** (e.g., `#![warn(clippy::pedantic)]` or category lints). Stay on the existing `-D warnings` gate.
- **No newtype additions** (the v1.1 ProjectId/ScriptId/AlarmId/SessionId/UserId follow-ups from AL). Tracked under AL's wiki page.
- **No cancellation-pattern, async-runtime, or store-actor changes.** AJ + AK + AL covered those.
- **No README / architecture doc edits** unless directly required by the missing-docs lint (which only fires on Rust pub items, not markdown).

### Risks / gotchas

- **thiserror 2.x `#[from]` strictness.** The most likely break: a derived error variant `Variant(#[from] OtherError)` works in 1.x even if `OtherError` has a blanket `From<&str>` impl that lets users construct it from a `&str` literal — but in 2.x the derive checks the type signature exactly. Where this fires, the minimal fix is to write the `From` impl by hand (just `impl From<X> for MyError { fn from(x: X) -> Self { Self::Variant(x.into()) } }` if a wrapper type is needed) or to introduce an intermediate `let typed: OtherError = x.into();` at the call site. **Don't restructure the error enum to avoid the breakage.**
- **thiserror 2.x error chain semantics.** 2.x is more careful about chain duplication when both `#[from]` and `#[source]` are present. We don't (currently) have any error variants with both; if the bump exposes a previously-tolerated double-source, the fix is to drop one annotation. Should not fire.
- **Cargo.lock drift.** Bumping thiserror also bumps `thiserror-impl` (the proc-macro crate it depends on). That's expected. If `serde`, `serde_json`, `tokio`, `pyo3`, or any of the `=`-pinned drivers in the workspace shift versions in the lockfile, that's a red flag — investigate before committing. The bump should be: thiserror + thiserror-impl, and nothing else outside the immediate transitive closure.
- **Missing-docs cascade in driver-opcua.** `driver-opcua` re-exports types from `async-opcua` (e.g., `NodeIdForm`, possibly others). The `#![deny(missing_docs)]` lint will fire on every pub item — including re-exports we don't own. The fix is a `///` comment on the re-export line itself (not on the upstream type). Use brief, factual one-liners: `/// Re-exported from `async-opcua` for runtime address parsing.` or similar.
- **driver-ads conditional compilation.** `crates/driver-ads/src/lib.rs` has `#[cfg(windows)] mod twincat_router;`. The missing-docs lint runs per-target; on non-Windows hosts the conditional module isn't compiled and its pub items aren't checked. The Windows host where Codex runs will catch them. Do not skip the doc comments inside `twincat_router` because they're cfg-gated — write them anyway so the Windows build is clean.
- **Don't `cargo update`.** Use `cargo build --workspace --all-features --locked` after the Cargo.toml edit to force the lockfile through the existing pinned graph; Cargo will only update what's required by the thiserror bump.
- **One-PR shape.** All three items land together. They don't depend on each other but are similar enough in size and scope that splitting would add board-update overhead without value.

## Codex log

<!-- Codex appends status transitions and notes here. -->

## Claude review

## Verdict
