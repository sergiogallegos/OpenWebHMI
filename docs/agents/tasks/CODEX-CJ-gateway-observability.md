---
id: CODEX-CJ
title: Gateway observability — /health + /metrics endpoints and optional structured logging
owner: codex
phase: 4
status: open
created: 2026-07-12
last-update: 2026-07-12 claude [Fable 5]
---

# CODEX-CJ — Gateway observability

## Brief

> The gateway's only observability today is `tracing` to stderr in human-readable format (`init_tracing` in `crates/gateway/src/main.rs:598-605`). There is no HTTP liveness/readiness endpoint (architecture.md §6 leans on liveness probes that have nothing to probe but an open TCP socket), no metrics endpoint, and no machine-parseable log option. For a 24/7 plant deployment that is a real operational gap. Add a lightweight `/health` + `/readyz` endpoint and a Prometheus/OpenMetrics `/metrics` endpoint, plus an opt-in JSON log format selectable by flag/env. Keep it minimal and dependency-light — **reuse the gateway's existing hand-rolled HTTP side-channel pattern**, do not pull a web framework, and respect the `=`-pin discipline for any new dependency.

### Goal

An operator can point a liveness probe at `GET /health` (process up → 200), a readiness probe at `GET /readyz` (drivers connected / stores open → 200; otherwise 503), and a Prometheus scraper at `GET /metrics` (OpenMetrics text with basic gateway counters/gauges). A `--log-format json` flag (and matching env var) switches `tracing` output to one JSON object per line for log shippers. Human format stays the default; no behavior changes for existing deployments that don't opt in.

### Context to read first

- `crates/gateway/src/main.rs:27-66` — the `clap` `Args` struct. New flags (`--log-format`, an observability bind address) land here following the existing `#[arg(long, ...)]` shape (see `--backup-bind`, `--log-level`).
- `crates/gateway/src/main.rs:598-605` — `init_tracing`: `tracing_subscriber::fmt().with_env_filter(filter)`. The JSON option adds a `.json()` layer path here. Note it uses `try_init` and swallows the double-init error — preserve that.
- `crates/gateway/src/main.rs:139-151` — how the backup HTTP side-channel is wired: bind an address from an `Option<SocketAddr>` arg, spawn `server::serve_backup_http(listener, ...)` as a cancellable task, warn-and-continue on bind failure. The observability endpoint follows this exact pattern.
- `crates/gateway/src/server.rs:212-242` — `serve_backup_http`: an accept loop over a `TcpListener` that dispatches to `handle_backup_http`, which matches on HTTP method and calls the `read_http_request` / `write_http_response` helpers (`server.rs:1536-1610` and nearby). **This is the hand-rolled HTTP toolkit to reuse** — the observability handler is a sibling of `handle_backup_http`, not a new framework.
- `crates/gateway/src/server.rs:75` and the audit/driver wiring — how process-wide handles (audit log, alarm journal) are threaded to handlers via `set_default_*`. The readiness check needs read access to driver connection state and store handles; find the least-invasive existing handle to observe.
- `docs/architecture.md` §6 (failure modes, ~350-357) and §4 (component list) — what "ready" should mean (drivers connected, project store open, tag engine live). Keep `/readyz` semantics consistent with the documented failure modes.
- [`VISION.md`](../../../VISION.md) §"Privacy & operator trust" (~42-44): **no telemetry by default, no third-party calls during startup or normal operation.** `/metrics` is pull-only (a scraper connects to the gateway); the gateway must never push metrics anywhere. The observability endpoint is opt-in via an explicit bind flag, off by default, and binds `127.0.0.1` unless configured otherwise.
- [`CLAUDE.md`](../../../CLAUDE.md) §"Dependency management" and §"Rust code quality" — `=`-pin discipline, no `panic!`/`unwrap`/`expect` in production paths, bounded `Cargo.lock` diff, `#![deny(missing_docs)]`.
- CODEX-B (`docs/agents/tasks/CODEX-B-gateway.md`) and the backup-channel task history — the established shape for adding a gateway HTTP surface.

### Files to create / modify

1. **`crates/gateway/src/main.rs`**
   - Add `--log-format` (`human` | `json`, default `human`, `env = "OPENWEBHMI_LOG_FORMAT"`) to `Args`.
   - Add an observability bind flag (e.g. `--observability-bind: Option<SocketAddr>`, mirroring `--backup-bind`; off when absent). Decide whether `/health`, `/readyz`, `/metrics` share the observability listener or ride the existing backup listener — **share one dedicated observability listener** so the backup channel's auth semantics don't leak onto unauthenticated probe endpoints. State the choice in the Codex log.
   - Extend `init_tracing` to take the log-format and build either the current human `fmt()` layer or a `fmt().json()` layer. Keep `EnvFilter` behavior and the `try_init` swallow.
   - Wire the observability listener with the same bind-and-spawn-cancellable pattern as the backup channel (~139-151), warn-and-continue on bind failure.

2. **`crates/gateway/src/server.rs`** (or a new `crates/gateway/src/observability.rs` module if it keeps `server.rs` from bloating — Codex's call, note it)
   - `serve_observability_http(listener, <handles>)` — accept loop mirroring `serve_backup_http`.
   - A handler that routes `GET /health` → 200 (liveness: process is up), `GET /readyz` → 200 or 503 based on readiness (project store open, drivers in connected state, tag engine live), `GET /metrics` → 200 OpenMetrics text. Reuse `read_http_request` / `write_http_response`.
   - A minimal metrics surface: counters/gauges for at least active WebSocket connections, active subscriptions, tag updates (total or rate), per-driver connection state, and dropped/lagged messages. Prefer a small hand-maintained registry (atomics already tracked in the server, or a thin `AtomicU64` set) over pulling a metrics framework. If a metrics crate is genuinely warranted, propose it in the Codex log first with a license + `Cargo.lock`-diff justification — do not add one silently.

3. **Metric plumbing** — thread whatever counters don't already exist to where events happen (connection accept/drop, subscribe/unsubscribe, tag publish, driver state transitions). Use existing atomics/handles where they exist; add the minimum new ones. No `unwrap`/`expect` on the hot path.

4. **`docs/architecture.md`** — §6 (or a new short "Observability" subsection): document `/health`, `/readyz`, `/metrics`, the `--observability-bind` flag, and `--log-format json`. Keep it accurate to what shipped (per the CODEX-CI honesty discipline — do not document endpoints that weren't wired).

### Behavior

- Endpoints are **off by default**. With no `--observability-bind`, the gateway behaves exactly as today. This preserves the "no third-party calls / no telemetry by default" invariant and avoids surprising existing deployments.
- `GET /health` → `200` with a tiny body (`{"status":"ok"}` or `ok`) whenever the process is serving.
- `GET /readyz` → `200` when the project store is open, drivers are in their connected/ready state, and the tag engine is live; `503` with a body naming the not-ready subsystem otherwise. Distinguish "process up" (`/health`) from "fully wired" (`/readyz`).
- `GET /metrics` → `200`, `Content-Type: text/plain; version=0.0.4` (or OpenMetrics), body is parseable Prometheus exposition text with `# HELP`/`# TYPE` lines for each metric.
- Unknown paths/methods on the observability listener → `404`/`405` via `write_http_response`, matching the backup handler's error style.
- `--log-format json` (or `OPENWEBHMI_LOG_FORMAT=json`) → every `tracing` event is one JSON object per line; `human` (default) is unchanged.
- The observability listener binds `127.0.0.1` semantics by default (operator opts into a wider bind explicitly); no auth is required on the probe endpoints (they expose no secrets — keep it that way; `/metrics` must not leak tag values, tokens, or project contents).

### Test requirements

- Add tests to the closest existing gateway test module (the `#[cfg(test)]` block in `main.rs` / `server.rs` already exercises bind-to-`127.0.0.1:0` and reads the port back — follow that pattern; **no hardcoded ports**, bind `:0` and read the assigned port).
- `GET /health` returns `200`.
- `GET /readyz` returns `200` when a ready fixture is wired and `503` when a subsystem is forced not-ready (drive the not-ready branch deterministically, not via timing).
- `GET /metrics` returns `200` and the body parses as Prometheus text: at least one `# TYPE` line and one sample line; assert a specific counter name is present and increments after a simulated event (e.g. a connection or tag update) — the test must fail if the counter is never wired.
- A `--log-format json` unit test: build the JSON subscriber path and assert an emitted line is valid JSON with the expected fields. If capturing subscriber output is awkward, at minimum test the `LogFormat` parse (`"json"`/`"human"`) and that `init_tracing` accepts both without error.
- No `sleep`/wall-clock waits; use deterministic synchronization. Run the new tests three consecutive times to confirm determinism.
- Full matrix clean: `cargo build`, `cargo clippy --workspace --all-targets --all-features --locked -- -D warnings`, `cargo test --workspace --all-features --locked`, `cargo doc --workspace --no-deps`. `#![deny(missing_docs)]` — new public items get docs.

### Acceptance criteria

- [ ] `GET /health` → 200 (liveness).
- [ ] `GET /readyz` → 200 ready / 503 not-ready, distinguishing process-up from drivers-connected/stores-open, with the not-ready subsystem named in the body.
- [ ] `GET /metrics` → 200 OpenMetrics/Prometheus text with `# TYPE` lines and at least: active connections, active subscriptions, tag updates, per-driver connection state, dropped/lagged messages.
- [ ] `--log-format json` (+ `OPENWEBHMI_LOG_FORMAT`) emits one JSON object per log line; `human` remains the default and is unchanged.
- [ ] Observability endpoints are opt-in via a bind flag, off by default, and expose no secrets/tag values (VISION.md telemetry + operator-trust invariants respected).
- [ ] Endpoints reuse the existing hand-rolled `read_http_request`/`write_http_response` pattern; no web framework added. Any new dependency is justified in the Codex log with license + `Cargo.lock` diff and respects `=`-pin discipline.
- [ ] Tests cover health 200, readyz 200/503, metrics parseable + counter increments, and the JSON-log path — bound to `127.0.0.1:0`, deterministic, three clean runs.
- [ ] `docs/architecture.md` documents exactly what shipped (no over-claim).
- [ ] Full validation matrix clean; no `unwrap`/`expect`/`panic!` on production paths.

### Out of scope

- **A full metrics framework** (`metrics` + `metrics-exporter-prometheus`, OpenTelemetry, `prometheus` crate) unless justified in the Codex log first. Default is a hand-maintained atomic registry rendered to text — keep the dependency footprint near zero.
- **Distributed tracing / OTLP export / spans over the wire.** JSON-formatted `tracing` to stdout is the ceiling for this task; shipping to a collector is a later phase.
- **Auth on the probe endpoints.** They expose no secrets by design; adding auth is a separate decision. Just ensure nothing sensitive is emitted.
- **Grafana dashboards / alerting rules / a bundled Prometheus.** Out of scope; the endpoint is the contract, dashboards are downstream.
- **Reworking the backup HTTP side-channel** — reuse its helpers, don't refactor it.
- **Per-tag or per-client metric cardinality.** Keep metrics low-cardinality (aggregate counters/gauges), not one series per tag/client — high cardinality would blow past the ≤10k-tag/≤50-client envelope's memory budget.

### Risks / gotchas

- **Metric cardinality.** Per-driver state is fine (bounded, 5 drivers); per-tag or per-client series are not (could be 10k / 50). Keep labels bounded.
- **`/metrics` must not leak.** No tag values, no tokens, no project names in metric labels or the body. It's a numbers surface, not a data surface.
- **Readiness semantics vs. the failure-modes table.** architecture.md §6 says clients reconnect and resubscribe on gateway crash; `/readyz` should reflect the documented "ready" state (stores open, drivers connected), not a stricter invented one. When in doubt, match the doc.
- **`try_init` double-init.** `init_tracing` deliberately swallows the "already initialized" error (`let _ = ...try_init()`) so tests that call it repeatedly don't panic. Preserve that when adding the JSON branch — don't switch to `init()`/`.expect()`.
- **Off-by-default is load-bearing for VISION.md.** The gateway runs in air-gapped plants; anything that binds a network port or phones home by default is a bug. The observability listener stays opt-in and localhost-default.
- **Honesty in the docs.** Document only endpoints/flags that actually landed. If `/readyz` ships with a coarse readiness check (e.g. stores-open only, driver-state deferred), say so in both the doc and the Codex log — don't describe the aspirational version.
- **`Cargo.lock` diff bound.** If a metrics/serde-json-adjacent dep sneaks in, confirm the lockfile diff is bounded (crate + proc-macro + direct transitives). `serde_json` is already a workspace dep — prefer it over adding new serialization crates.

## Codex log

## Claude review

## Verdict
