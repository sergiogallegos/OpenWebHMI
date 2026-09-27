# Reusable engine and deployment capacity

Status: accepted direction, 2026-09-27. Implementation and performance remain
unproven until the gates below pass. This plan supersedes the earlier 10K-tag
ceiling and desktop-first designer assumptions. `VISION.md` owns product scope;
this document owns the workload definitions and engine delivery sequence.

## Product goals

1. Ship an embeddable Rust SCADA/HMI engine that other projects can consume without
   the OpenWebHMI gateway executable, network server, designer or renderer.
2. Make the OpenWebHMI gateway itself a consumer of that same engine. A second,
   minimal example application must prove reuse without copying internal code.
3. Deliver browser authoring and operation first. Future installable designers use
   the shared web interface plus narrow native host adapters, subject to measured
   responsiveness. Three separate native UI implementations are not committed.
4. Target 50,000 live tags and 50 concurrent runtime clients on a medium deployment,
   with 30 days of selected raw process history at a defined recording rate.
5. Preserve a separate low-resource headless profile. The 50K workload is not
   promised on a 1 GB computer, and the browser need not run on the gateway host.
6. Keep AGPL product/engine code and the existing MPL protocol policy; library
   reuse is a technical goal, not a new permissive or commercial license grant.

## External reference and interpretation

Inductive Automation's [sizing guide](https://inductiveautomation.com/resources/article/ignition-server-sizing-and-architecture-guide)
lists a medium example at 50,000 tags / 50 clients on four cores, 16 GB RAM and SSD.
Its historian sizing is separate: medium examples span 500–5,000 value changes/s,
with database hardware considered separately. Its small ARM example uses 500 tags,
two clients and 1 GB RAM. These are workload-dependent examples, not universal limits.

The [8.3 historian documentation](https://www.docs.inductiveautomation.com/docs/8.3/ignition-modules/tag-historian)
distinguishes Core Historian's disk/WAL path from SQL-provider store-and-forward.
The [8.3 upgrade guide](https://docs.inductiveautomation.com/docs/8.3/getting-started/installing-and-upgrading/ignition-8-upgrade-guide/81to83-upgrade-guide)
identifies QuestDB behind Core Historian. A vendor storage-engine benchmark is not
an end-to-end PLC-to-browser benchmark. OpenWebHMI does not claim matching throughput.

[Ghostty's architecture](https://ghostty.org/docs/about) separates its reusable
library from its GUI consumers. Its [1.3 release notes](https://ghostty.org/docs/install/release-notes/1-3-0)
describe extraction and independent library versioning work. Adopt the separation
and second-consumer proof, not Ghostty's language, rendering architecture or license.

## Workload profiles — acceptance targets, not measured support

All rows are provisional engineering budgets to test, not minimum hardware claims.
64-bit ARM64 and x86-64 are the intended architectures. Record exact CPU, storage,
OS, compiler, build configuration and enabled features in every benchmark result.

| Profile | Reference test host | Live tags | Accepted tag updates/s, sustained | Runtime clients | Selected history samples/s, sustained | Gateway RSS budget |
|---|---|---:|---:|---:|---:|---:|
| Edge | 2 low-power cores, 1 GB RAM, SSD | 500 | 100 | 2 | 10 | 256 MiB |
| Standard | 4 cores, 4 GB RAM, SSD | 10,000 | 2,000 | 20 | 500 | 512 MiB |
| Medium | 4 performance cores, 16 GB RAM, SSD/NVMe | 50,000 | 10,000 | 50 | 2,500 | 2 GiB |

RSS is the headless gateway process with the profile's selected driver(s), alarms,
authentication and history active. It excludes the OS, browser and optional Python
workers; report their memory separately and total deployment memory as well. Medium
uses a starting storage test fixture of 2 TB usable SSD, subject to measured growth.
Edge requires no Node, desktop toolkit or Python installation when scripting is off.
Neither the feature selection nor these budgets are implemented/proven yet.

- Live tags means resident configured tags with active acquisition or update sources,
  not 50K unused definitions. Acquisition rate, value-change rate, history admission
  rate and browser delivery rate are separate measurements. A 50K-tag source changing
  every five seconds produces 10K updates/s; the simulator must actually exercise it.
- Initial payload mix: 80% numeric, 15% boolean and 5% strings up to 64 UTF-8 bytes.
  Large strings, arrays and structures require separate payload-size limits and tests.
- Baseline clients have 100 / 500 / 1,000 active tag subscriptions per client for
  Edge / Standard / Medium. Test both overlapping and disjoint visible tag sets.
  Medium browser delivery budget is 50K tag-value deliveries/s aggregate after display
  coalescing; it is not 50 clients subscribing to every tag at the source rate.
- Exercise alarm evaluation on 10% of live tags, with a recorded transition workload,
  plus concurrent writes, login/reconnect, one bounded trend query and retention.
  Also run all-tag alarm evaluation as a stress case without conflating its result
  with the baseline. Python scripts get a separate bounded mixed-load profile.
- Medium burst target: 20K tag updates/s and 5K admitted history samples/s for 60 s;
  recover the backlog within 120 s without unbounded RAM, lost acknowledged samples,
  or blocking command handling. Queue/spool capacities must be explicit and measured.
- Initial latency budgets under baseline load: gateway accept-to-outbound-enqueue
  p95 <= 100 ms / p99 <= 250 ms; local-network tag-to-visible-screen p95 <= 250 ms /
  p99 <= 500 ms. Publish driver acquisition latency separately. UI coalescing must
  not coalesce alarm transitions or command results. No hard-real-time guarantee.
- CPU target is <=70% of total host capacity averaged over the steady-state interval;
  record core saturation, queue ages and p99 tails, not only averages.

These values can change only through a recorded scope/budget decision based on
evidence. Failing a budget must not be hidden by disabling durability, losing samples,
reducing the workload, or relabeling the same build as supporting 50K tags.

## Thirty-day history sizing

“One month of production” means **30 consecutive 24-hour days of configured raw
process history**, including value, quality and timestamp; not every live value at
every scan. Alarm/audit records have independent retention. Raw retention cannot
silently become aggregates, and capacity calculations cannot assume future compression.

```text
samples = admitted_samples_per_second × 2,592,000
base_bytes = samples × measured_bytes_per_sample
```

Illustrative planning range: 64–128 bytes per stored sample, including an assumed
row/index allowance. This is not a measurement of the existing SQLite schema.
JSON/string-heavy records may exceed it. GB/TB below are decimal.

| Recording workload | Samples / 30 days | Base data at 64–128 B/sample | Initial 2× disk allowance |
|---|---:|---:|---:|
| Edge: 10/s | 25,920,000 | 1.66–3.32 GB | 3.32–6.64 GB |
| Standard: 500/s | 1,296,000,000 | 82.94–165.89 GB | 165.89–331.78 GB |
| Medium: 2,500/s | 6,480,000,000 | 414.72–829.44 GB | 0.83–1.66 TB |
| Higher-rate study: 5,000/s | 12,960,000,000 | 0.83–1.66 TB | 1.66–3.32 TB |
| Every 50K tag at 1 Hz | 129,600,000,000 | 8.29–16.59 TB | 16.59–33.18 TB |

The 2× allowance is a preliminary reserve for WAL, maintenance and operational free
space, not a guarantee that compaction or backup fits. Measure peak disk use. A full
backup copy needs additional capacity or a separate destination. High-endurance SSDs,
power-loss behavior and restore time matter as much as the nominal file size.

Keep SQLite as the default v1 storage adapter. Introduce a narrow historian storage
interface now, while implementing batching, numeric storage evaluation, bounded SQL
queries/aggregation, pruning and crash recovery. Run the Medium gate on a real
30-day-sized dataset; a short ingest test does not prove month-sized query behavior.
Target an eight-pen, 2,000-point-per-pen 30-day trend response within 5 s on the Medium
reference host while ingestion continues, with warm/cold results reported separately.

If SQLite cannot satisfy durability, retention, bounded memory and query budgets,
record the result and open a backend decision before claiming Medium support. No
QuestDB, custom storage engine or mandatory external database is selected by this
plan. No automatic pruning is claimed today: existing primitives still need policy
wiring, operator controls, safe backups and disk-pressure behavior.

## Engine architecture and public contract

The following are target boundaries; names may be refined by their implementation
briefs. Current libraries are not already this independent.

```text
Rust host application ──> openwebhmi-engine <── gateway composition / CLI
                              │                        │
                         async services          Axum HTTP / WS adapter
                              │                        │
                         domain rules            browser / desktop clients
                              │
                    driver / storage / script ports
                              │
                      selected concrete adapters
```

- **Domain:** pure values/identifiers, quality, validation, alarm transitions and
  command outcomes. `std/core` only by default. No Tokio, serialization, clock,
  filesystem, database or transport requirements. `no_std` is not a v1 commitment.
- **Services:** narrowly enabled Tokio scheduling, bounded channels, subscriptions,
  cancellation and supervision. Tokio is not transitively dependency-free.
- **Adapters:** serde/JSON, SQLite, maintained driver libraries, Rustls/auth and
  optional CPython processes. HTTP uses Axum/Hyper rather than a custom parser.
  Do not replace mature security/protocol libraries to improve a dependency count.
- **Composition:** caller-provided configuration, paths, runtime ownership and
  adapter selection. No process globals, implicit listeners, process exits or
  forced telemetry. Engine shutdown returns only after a documented drain/abort
  policy completes. A host can supply its runtime without creating nested Tokio
  runtimes, and two instances must coexist without sharing state accidentally.
- **API:** typed errors, versioned public types, subscription snapshot/gap semantics,
  typed authorized commands, explicit write receipts/results and independent
  observed PLC state. Local embedding must not bypass authorization by accident:
  trusted low-level operations are distinguished from operator-facing commands.
- **Storage/driver contracts:** avoid exposing SQLite handles, HTTP types or React
  concepts through core APIs. Preserve existing public re-exports and wire shapes
  during extraction; document migrations and run old/new conformance fixtures.
- **Shared schema/licensing:** remove protocol's dependency on AGPL storage. Keep
  MPL wire DTOs self-contained; map to AGPL domain types in an adapter. Do not move
  AGPL source into an MPL directory or add license exceptions implicitly. Reuse of
  the engine remains under AGPL; the libghostty analogy does not change that policy.
- **Packaging:** first deliver Rust crates and an external-consumer example using
  only public APIs. Establish crate semver/MSRV and release compatibility checks.
  A C ABI, Python engine bindings, WASM engine and stable dynamic plugin ABI are
  deferred until a concrete consumer justifies their ownership/FFI burden.

## Minimal deployment and fast desktop design

Provide explicit build/deployment profiles, not one binary enabling every driver,
Python and GUI dependency. A minimal engine must build independently of the Cargo
workspace's desktop members. CI checks individual feature combinations, normal/build
dependency graphs, installed footprint and license/advisory reports. Test dependencies
are counted separately. Selecting an unavailable driver/service fails at startup.

Use `cargo tree` evidence to remove feature leakage (including workspace Tokio full
and OPC UA server features in client builds), unused dependencies and duplicate
versions where compatible. Keep reviewed pins. Build/test-only tooling need not be
installed at runtime. No new deployment profile is claimed until dependency checks
and a real low-resource hardware run pass.

Designer responsiveness is a release condition, not an assumption about webviews.
Use the same editor in browsers and optional Tauri apps. Isolate file dialogs and
other native capabilities behind small host adapters; preload only the active view,
virtualize large tag trees and defer Monaco until needed. Initial benchmark targets
on a recorded four-core/8 GB client: p95 input feedback <=100 ms, p95 drag frame
<=16.7 ms in a representative 500-component view, warm project open <=2 s, and cold
local application ready <=3 s. Record document/tag sizes, renderer/webview version,
network conditions, client memory and startup definitions. Runtime and Designer
processes are measured separately from the gateway. Revisit rendering choices if
measurements fail before considering a separate native UI rewrite.

## Implementation sequence and ownership

| Stage | Existing work retained | New task | Exit evidence |
|---|---|---|---|
| Baseline | DK compiler/build update | DL accepted scope/research/planning | Consistent docs, board, sources and explicit target labels |
| Measure current behavior | CJ observability | DT capacity harness | Versioned workload definitions, current numbers, correctness oracle |
| Secure operations | BC–BH, BR/BS/BT, BY/BZ | — | Pre-fix regression failures and green tests |
| Finish driver integration | BI/BJ, review BK, BL–BQ | — | Gateway-selected drivers, loss/reconnect/write/quality evidence |
| Extract foundation | BX service context, BV paths | DM domain/schema; DN engine API | No storage in core/protocol graph; independent second consumer |
| Reduce runtime footprint | CK build matrix | DO feature profiles | Independent minimal builds, dependency budget, real edge run |
| Replace transport safely | BE/BG/BU and CJ | DP Axum/Hyper | Old wire compatibility, limits/TLS/origin/security tests |
| Deliver browser authoring | CC integrity, CN platform proof | DQ browser deployment; DR draft/publish | Offline/browser matrix and atomic revision-checked publishing |
| Prove history capacity | CG SQL reads, CH audit queries | DS historian ingest/retention | Durable batches, disk-pressure/restart tests, 30-day dataset |
| Upgrade UI toolchain | Existing widget/runtime tests | DU frontend upgrades | One compatible group per checkpoint, no peer suppression |
| Reach target performance | BW tag scaling, CB rendering, CE/CF alarms | DT final evidence | Edge/Standard/Medium benchmarks and sustained mixed-load soak |
| Optional desktop | CN browser gates complete | DV desktop host/performance (Phase 5) | Three-platform packages and measured interaction budgets |

DT's harness work starts before optimizations; final certification depends on the
other stages. Existing security fixes are not blocked on architectural cleanup.
New tasks augment existing ownership rather than duplicating those implementations.
Future manufacturing tasks remain post-v1 consumers of these services.

Release gates: three-platform gateway validation, browser workflow matrix, offline
demo, restore drill, engine embedding example, real edge hardware, Medium load plus
30-day-sized history tests, and the existing 24-hour Rockwell PLC soak. A 72-hour
synthetic mixed-load soak must report memory growth, queue age, data gaps and latency.
Preseeded history is allowed for size/query tests but does not count as 30 days of
real-time durability evidence. Keep correctness tests deterministic; benchmark/soak
harnesses use explicitly measured time and are not flaky unit-test replacements.

## Status at acceptance

Implemented in the accompanying baseline: Rust 1.98.1, product TypeScript 7.0.2,
Node 24.21.0, browser-default designer build and removal of its unused Tauri JS API.
The website remains on TypeScript 5.9.3 pending its compatible framework upgrade.

Not yet implemented: extracted engine, minimal feature profiles, Axum migration,
safe publishing, history ingestion redesign, any 50K capacity certification or
desktop responsiveness proof. Existing security/driver gaps remain open. This
accepted plan and its task briefs authorize the implementation program; committing
the plan must not be described as shipping those capabilities.
