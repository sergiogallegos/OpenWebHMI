# crates/AGENTS.md

Rust-specific rules for the `crates/` workspace. Loaded automatically by Codex and Claude Code when work is under `crates/`. The root `AGENTS.md` applies in addition; this file is the Rust-scoped tightening.

## Toolchain

Pinned by `rust-toolchain.toml`: Rust 1.95.0, edition 2024. Do not bump in a non-toolchain PR. Edition migrations are their own task (precedent: CODEX-AG).

## Production-code prohibitions

Production = anything reachable from a non-test, non-startup, non-`main` path. The following do not land in production code:

- `panic!`, `unwrap()`, `unreachable!()`, `expect()`.
- `.unwrap_or_default()` on types where "default" is silently wrong (e.g. integer zero for a missing tag value).
- `todo!()`, `unimplemented!()` — these are for in-flight work, not committed code.
- Calls that allocate, lock, or hit syscalls inside the hot `TagStore::publish` path. The path is busy; profile before adding work to it.

Documented exception form:

```rust
// invariant: rwlock poisoning means a writer panicked while holding the lock;
// the only recoverable response is to fail the whole tag-engine task.
let guard = self.tags.read().expect("rwlock poisoned");
```

`tag-engine`'s `expect("rwlock poisoned")` is the existing precedent. New exceptions get the same shape.

Startup config validation, test code, and `fn main` are exempt — those are allowed to panic on unrecoverable invariants because there's no caller to surface the error to.

## Syntax & idioms

- **`#[expect(...)]` over `#[allow(...)]` for clippy lints.** When touching code that has an `#[allow]`, convert it.
- **Let chains over nested `if let`.** Edition 2024 syntax.
  ```rust
  // good
  if let Some(driver) = registry.get(id) && driver.is_ready() { ... }
  // avoid
  if let Some(driver) = registry.get(id) {
      if driver.is_ready() { ... }
  }
  ```
- **Top-level imports only.** No `use foo::Bar;` inside function bodies, except to break a cyclic-import that can't be resolved structurally. Rare; gets a `// cyclic-import workaround` comment.
- **Full variable names.** `version` not `ver`; `tag_path` not `tp`; `historian` not `hist`.
- **`thiserror = "2"`** for error types; manual `impl Error` only when there's a stated reason.

## Async hygiene

- **Blocking work inside async tasks goes through `spawn_blocking`.** SQLite calls, filesystem walks, CPU-bound work. Three identical wraps in `gateway/src/server.rs` show the shape; match it.
- **`JoinHandle` ≠ cancellation handle.** Dropping a `JoinHandle` does not cancel its task. Store an `AbortHandle` separately when you need cancel-on-drop semantics. See CODEX-AK for the pattern sweep.
- **Coordinated shutdown** uses the gateway's supervisor pattern (graceful drain, then abort). Don't roll your own `tokio::select! { _ = ctrl_c => ... }` in a subsystem — wire into the existing supervisor.
- **Cancel-safety is a documented contract.** `async fn`s that are awaited inside `tokio::select!` arms must be cancel-safe; if they aren't, document it on the function with a `// not cancel-safe: ...` comment.

## Crate docs

- **`#![deny(missing_docs)]` is the bar for every crate.** New crates land with this attribute from day one.
- Module-level docs (`//!`) explain *what this module is for and what it owns*. Item-level docs (`///`) explain *how to use the item correctly*, including invariants the caller must uphold.

## Dependencies

- **`=`-pinned workspace versions stay pinned**: `tokio-modbus`, `async-opcua`, `rumqttc`, `rumqttd`, `prost`, `jsonpath-rust`, `ads`. These are load-bearing for driver-side compatibility.
- **`cargo update --precise <crate>@<version>`** for targeted bumps. Never `cargo update` with no args.
- **`Cargo.lock` diff bound**: bumped crate + proc-macro counterpart + direct transitives. Wider = investigate first.

## Tests

- Place tests in `crates/<crate>/tests/<area>.rs` (integration) or `#[cfg(test)] mod tests` at the bottom of the source file (unit). Extend existing files; don't fragment.
- Use `tokio::time::pause()` + `advance()` for time-dependent tests. No `tokio::time::sleep` in tests.
- `127.0.0.1:0` for ports. Read the assigned port back from the listener.
- Regression tests are validated by running them against the pre-fix code at least once; the test must fail without the fix.

## Driver-specific

- Each driver crate lives at `crates/driver-<protocol>/`.
- Drivers implement the `driver-api::Driver` trait.
- A driver speaks the *real* wire protocol against the *real* upstream crate. JSON-line frames to a fake simulator are not a driver — see the CODEX-Z v1 rejection.
- Each driver has a paired simulator under `examples/sim-<protocol>/` for CI integration tests.
- Each driver has a wiki entry under `wiki/drivers/<protocol>.md` that's honest about what's been proven (sim CI, manual smoke, hardware validation).

## See also

- `AGENTS.md` (root) — codebase-wide rules.
- `VISION.md` — won't-merge policy.
- `docs/architecture.md` — system topology.
