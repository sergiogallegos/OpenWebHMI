# OpenWebHMI — Stack Rationale

> Current direction: [web-first designer and dependency-light core](planning/2026-09-architecture-review.md). Named UI libraries below describe earlier options, not an approved dependency list or an inventory of installed packages.

> Why **Rust + Python (CPython 3.11+) + TypeScript** for an Ignition-class open-source SCADA/HMI platform — and why the combination matters.

This document is the substantive answer to "why not Java? why not C#? why not all-Rust?". It compares OpenWebHMI's stack against Ignition and Optix, lays out the reasoning per-language, and is honest about tradeoffs.

## Three platforms, three stacks

| Layer | **Inductive Automation Ignition** | **Rockwell FactoryTalk Optix** | **OpenWebHMI** |
|---|---|---|---|
| Core / runtime | **Java 17 / JVM** | **C++/Qt native platform + C#/.NET NetLogic**[^optix-stack] | **Rust** |
| Designer / IDE | **Java/Swing** | **C++/Qt + web technology; C#/.NET authoring**[^optix-stack] | **Browser-first React + TypeScript; optional Tauri shell** |
| Web HMI runtime | **Perspective** — React/TypeScript over Java backend | **Web Presentation Engine** — HTML5/browser | **React + TypeScript** |
| Desktop HMI runtime | **Vision** — Java/Swing | **Native Presentation Engine** — Qt-based | post-1.0 (Tauri reuse) |
| Scripting | **Jython 2.7.4** — Python 2.7 language level | **C#/.NET NetLogic** | **CPython 3.11+** in worker subprocesses |
| Module / extension format | proprietary `.modl` | proprietary | **crates.io / npm / PyPI** (no custom registry) |
| Deployment shape | JVM-based gateway and clients | native runtime for Windows/Linux and x86/ARM | **Rust gateway + browser clients + CPython workers** |
| Open source? | ❌ | ❌ | ✅ **AGPL core / MPL protocols** |

[^optix-stack]: FactoryTalk Optix is closed source. Rockwell documents [Qt in its native presentation engine](https://www.rockwellautomation.com/en-se/docs/factorytalk-optix/1-4-4/contents-ditamap/creating-projects/object-and-variable-reference/ftoptix-nativeui/datatypes/textrendertypeenum.html) and [C# NetLogic compiled into .NET assemblies](https://www.rockwellautomation.com/en-us/docs/factorytalk-optix/1-5-7/contents-ditamap/extending-projects/netlogic.html); [current ASEM/Rockwell roles](https://rockwellautomation.wd1.myworkdayjobs.com/en-US/External_Rockwell_Automation/job/Software-Engineer--C----Qt-_R26-1796) seek C++/Qt engineers for native and embedded industrial UI work. That supports the stack characterization, but the exact internal boundary between the designer, framework, and runtime is not public.

The three stacks reflect three different bets:

- **Ignition** uses a mature Java/JVM platform, a Swing designer, a React/TypeScript web runtime, and Jython for user scripting. Its integration story is strongest inside the Java ecosystem, while its Python language level remains 2.7.
- **Optix** combines a native, cross-platform industrial runtime with Qt-based presentation and a C#/.NET customization model. That is a capable embedded-to-edge design, but its implementation and extension boundary remain vendor-controlled.
- **OpenWebHMI** uses **Rust for the always-on systems boundary**, **TypeScript for the complete UI surface**, and **CPython for plant scripting and data work**. The AGPL product core and MPL protocol packages make those boundaries inspectable and changeable by the operator rather than only by the vendor.

## Why the combination is the vision

The advantage is not that Rust, TypeScript, or Python wins every category individually. The advantage is that each language owns the part of the system where its strengths are operationally relevant:

- **Rust protects the plant-facing core.** Drivers, tag processing, alarms, history, authentication, and client fan-out live in a memory-safe systems language without garbage-collector pauses. This is the smallest trusted core and the part that must remain available when a user script fails.
- **TypeScript unifies authoring and operation.** The designer, web runtime, component library, and protocol types use the browser ecosystem. Components and interaction models can be shared instead of maintaining separate desktop and web widget families.
- **CPython meets plant engineers where data work already happens.** Scripts use the current Python language and its packaging ecosystem. Worker subprocesses isolate interpreter and native-extension failures from the Rust gateway.
- **Open source turns technical choices into operator rights.** The AGPL/MPL split permits source review, internal use, air-gapped operation, independent security audits, modification, and redistribution under the applicable terms without a recurring runtime entitlement.

This separation also creates an understandable trust model: Rust is trusted platform code, TypeScript is distributed UI code, and Python is user-authored code behind a process boundary. The stack is therefore more than a list of popular languages; it expresses where failures are allowed, who can extend the system, and who ultimately controls a deployment.

## Why Rust (gateway, drivers, tag engine, designer shell)

The gateway runs 24/7, talks to PLCs, fans data out to many clients, and must not hiccup. Hard requirements:

- **Predictable latency** — no GC pauses. A 50ms GC stop in the middle of a tag-update fan-out is visible in the HMI.
- **Memory safety without runtime overhead** — no garbage collector, no JVM, no .NET CLR.
- **Async I/O within the v1 envelope** — driver poll loops and up to 50 runtime clients without a thread per connection.
- **FFI to C/C++ libraries** — most existing PLC protocol libraries (OpenENER, libplctag, open62541) are C; Rust integrates cleanly via `bindgen`.
- **Compact gateway deployment** — the gateway is a Rust binary with SQLite-backed state; Python is an explicit worker dependency rather than a managed runtime underneath the gateway.
- **Compile-time correctness** — data-type mismatches between PLC reads and the tag engine are caught at compile time, not at 2 AM in a plant.

What Rust gives us specifically over Java/C# for this domain:

| Concern | Rust outcome |
|---|---|
| Tag fan-out latency | No garbage collector in the gateway hot path; performance still requires measurement against the v1 envelope |
| Cold-start time | Native gateway startup without JVM or CLR initialization |
| Memory footprint | Explicit allocations and a bounded v1 target; final numbers remain benchmark-dependent |
| Plugin distribution | crates.io (versioned, semver-checked) vs proprietary `.modl` |
| Driver crate code reuse | We use upstream `rust-ethernet-ip` directly, no wrapper-of-wrapper |

Language choice does not establish lower memory, faster startup or fewer defects.
The [engine and capacity plan](planning/engine-and-capacity.md) requires measured
resource and latency evidence instead of source-line-count comparisons.

## Why TypeScript (designer UI, web runtime, component library, protocol package)

The UI is half the platform's surface. Both the designer (authoring HMIs) and the runtime (running them) need rich, interactive web tech.

- **Largest contributor pool of any UI ecosystem.** When we say "community-extensible component library", that's only true if community can actually contribute. React + TS is the most accessible UI stack today.
- **The right libraries already exist.** Monaco (the editor that powers VS Code) for our script editor. react-konva for the visual canvas. Recharts / D3 for trends. ag-grid for the alarm table. We don't reimplement; we compose.
- **Type-safe wire protocol**. The `@openwebhmi/protocol` package is hand-mirrored from the Rust `crates/protocol` types. TypeScript catches binding errors between gateway and UI at compile time.
- **Vite dev experience** — sub-second hot reload while authoring designer features. Compare: a Java/Swing designer cycle is "edit → build → restart" in tens of seconds.
- **Same components serve designer + runtime.** A `Gauge` component renders the same in the designer (with selection adornments) and in the runtime (live-bound to a tag). One implementation, two contexts. Java's Swing widget set in Ignition doesn't share with Perspective's React widgets — they have to maintain both.

## Why Python (scripting host) — and why this is the biggest differentiator

Modern CPython package compatibility is a useful differentiation; scripting API completeness and operational behavior still require validation.

Python is an optional application scripting layer for event handlers, transformations,
small workflows, PDF reports and CSV exports. It is not the primary mechanism for
simple UI behavior or a commitment to an embedded ML platform. CPython workers can
use compatible installed packages; package compatibility does not establish a
supported application workflow or resource-isolation guarantee.

The [project engineering plan](planning/project-engineering.md) defines bounded
worker jobs, explicit script tests, authenticated result downloads and managed
per-project environments. These remain implementation tasks. Environments are
prepared explicitly from versioned dependency metadata for each target; opening a
project never installs packages. Existing v1 code does not automatically provision
per-project virtual environments.

Bindings and built-in actions handle normal interface behavior. Custom components
use TypeScript through the paired Designer/runtime SDK. An optional terminal agent
edits the same text project and Python sources that the Designer opens; no built-in
AI provider, Python Designer-plugin system or mandatory model SDK is required.
Offline project schema validation and migration remain shared Rust services;
optional user-authored Python tools execute only when explicitly invoked.

Ignition documents Jython/Python 2.7 scripting; OpenWebHMI's CPython choice opens a
different package ecosystem. This is not a claim that competing products cannot
perform advanced logic or that OpenWebHMI has already shipped equivalent features.
See [Ignition scripting libraries](https://www.docs.inductiveautomation.com/docs/8.3/platform/scripting/python-scripting/libraries).

### What we give up by choosing Python over Jython/C#

Honest tradeoffs:

| Trade | Cost |
|---|---|
| **No first-class Java interop** | Ignition can call any Java library directly from Jython. We can't — interop is via Rust FFI or shelling out. For most modern needs, Python's own ecosystem covers it; for legacy-Java-shop integrations, this is friction. |
| **No first-class .NET interop** | Same shape, Optix-side. |
| **Subprocess overhead per script invocation** | IPC and scheduling overhead must be measured under the selected event rate; bounded worker pools do not eliminate that cost. |
| **Python packaging is genuinely complex** | `requirements.txt` + virtualenvs + native wheels for arm64 vs x86_64 is real work. Jython users don't deal with this. We accept the complexity for the ecosystem benefit. |

## What about all-Rust scripting?

Tempting and considered. Why we didn't:

- **Plant engineers don't write Rust.** Scripting is for plant engineers and integrators, not platform contributors. Python's audience overlap with industrial controls is enormous; Rust's is small.
- **Hot reload.** A Python script edit takes effect on next event. A Rust "script" requires `cargo build`, an artifact, and gateway reload. Authoring loop is 100× slower.
- **Library reach.** Rust's data/ML ecosystem (`linfa`, `polars`, `candle`) is excellent and growing, but ~5% the breadth of Python's. For this audience, that's the wrong direction.

The compromise: **system-level code (drivers, tag engine, gateway, alarm engine) is Rust; user-level scripting is Python.** Each language plays the role it's best at.

## What about all-TypeScript?

Also considered. Deno + TS server-side is a real option in 2026. Why not:

- **Latency / GC.** V8 has GC pauses; we'd land in the same hole as Java. Tag fan-out at scale needs predictable latency.
- **FFI to PLC libraries.** PLC vendor libraries are C/C++. Rust integrates cleanly; Node's FFI story is workable but rougher.
- **Single-binary deployment.** Bundling Node's runtime into a deployment unit is solvable but messy. Rust ships one file.

TypeScript stays where it shines: the UI surface.

## Summary: who does what

```
┌─────────────────────────────────────────────────────────────────────┐
│                        OpenWebHMI Stack Map                          │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│   Designer / IDE             Web HMI Runtime         Component lib   │
│   (React + TS,               (React + TS,            (React + TS)    │
│    Tauri shell in Rust)       browser)                               │
│            │                          │                              │
│            │  WebSocket               │  WebSocket                   │
│            └──────────────┬───────────┘                              │
│                           │                                          │
│  ┌────────────────────────┴─────────────────────────────┐            │
│  │              Gateway  (Rust, single binary)            │           │
│  │  ┌─────────────────────────────────────────────────┐  │           │
│  │  │  Tag engine · Drivers · Alarm engine · Historian │  │           │
│  │  │  Project store · Auth · WebSocket server         │  │           │
│  │  │     ⇅                                            │  │           │
│  │  │  Scripting host ─────────→ Python worker procs    │  │           │
│  │  │                            (numpy, pandas, sklearn,│ │           │
│  │  │                             torch, LLM clients)   │  │           │
│  │  └─────────────────────────────────────────────────┘  │           │
│  └─────────────────────────────────────────────────────┘            │
│                           │                                          │
│           ┌───────────────┴────────────┐                             │
│       PLCs (CompactLogix,        SQLite (project,                    │
│        ControlLogix, OPC UA…)    historian, auth)                    │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘

Rust       — systems, performance, safety, deployment      [gateway, drivers, tag engine, scripting host, designer shell]
TypeScript — UI, type-safe protocol, ecosystem            [designer UI, runtime, components, protocol-ts]
Python     — data, AI/ML, plant-engineer scripting        [user scripts, predictive maintenance, designer extensions, LLM bridges]
```

## Related documents

- [`docs/architecture.md`](architecture.md) — system design.
- [`docs/scale-estimates.md`](scale-estimates.md) — LOC ranges per component.
- [`docs/feature-matrix.md`](feature-matrix.md) — feature parity vs Ignition + Optix.
- [`docs/roadmap.md`](roadmap.md) — when each component lands.
