# OpenWebHMI — Stack Rationale

> Why **Rust + Python (CPython 3.11+) + TypeScript** for an Ignition-class open-source SCADA/HMI/MES platform — and what each language is doing here that the others can't.

This document is the substantive answer to "why not Java? why not C#? why not all-Rust?". It compares OpenWebHMI's stack against Ignition and Optix, lays out the reasoning per-language, and is honest about tradeoffs.

## Three platforms, three stacks

| Layer | **Inductive Automation Ignition** | **Rockwell FactoryTalk Optix** | **OpenWebHMI** |
|---|---|---|---|
| Gateway / runtime | **Java (JVM)** | **C# / .NET** | **Rust** (single static binary) |
| Designer / IDE | **Java + Swing** | **Visual Studio Studio** (Windows-only) | **Tauri (Rust shell) + React + TypeScript** (Win + Mac) |
| Web HMI runtime | **Perspective** — React-based JS/TS over Java backend | **Optix WebPresentation** | **React + TypeScript** |
| Desktop HMI runtime | **Vision** — Java/Swing | optional | post-1.0 (Tauri reuse) |
| Scripting | **Jython 2.7** (Python on JVM, frozen at 2015) | **C# (NetLogic)** + JavaScript | **CPython 3.11+** via PyO3, in worker subprocesses |
| Module / extension format | proprietary `.modl` | proprietary | **crates.io / npm / PyPI** (no custom registry) |
| Deployment unit | JVM bundle, ~300+ MB | .NET bundle | **single static Rust binary + SQLite, ~50 MB target** |
| Open source? | ❌ | ❌ | ✅ **MIT** |

The three stacks reflect three different bets:

- **Ignition** bet on Java's ubiquity in the enterprise circa 2003. The choice still works, but Jython 2.7 has aged out of the modern Python ecosystem and the JVM deployment footprint is heavy.
- **Optix** bet on .NET, on Visual Studio as the IDE, and on Microsoft's developer pipeline. Strong if you live in that world; closes off macOS/Linux for the Studio side.
- **OpenWebHMI** bets that **modern Rust eliminates the need for a managed runtime in 2026**, that **TypeScript is the universal UI layer**, and that **CPython is the universal data/AI/ML layer** — and combines all three rather than picking one.

## Why Rust (gateway, drivers, tag engine, designer shell)

The gateway runs 24/7, talks to PLCs, fans data out to many clients, and must not hiccup. Hard requirements:

- **Predictable latency** — no GC pauses. A 50ms GC stop in the middle of a tag-update fan-out is visible in the HMI.
- **Memory safety without runtime overhead** — no garbage collector, no JVM, no .NET CLR.
- **Async I/O at scale** — thousands of WebSocket clients + driver poll loops without thread-per-connection cost.
- **FFI to C/C++ libraries** — most existing PLC protocol libraries (OpenENER, libplctag, open62541) are C; Rust integrates cleanly via `bindgen`.
- **Single static binary deployment** — no "install JVM 17 first, then…". Production deploys are one binary + one SQLite file + one config file.
- **Compile-time correctness** — data-type mismatches between PLC reads and the tag engine are caught at compile time, not at 2 AM in a plant.

What Rust gives us specifically over Java/C# for this domain:

| Concern | Rust outcome |
|---|---|
| Tag fan-out latency to 1000 connected HMIs | Single-digit ms p99 (no GC pauses) |
| Cold-start time | < 1s vs JVM's 5–15s |
| Memory footprint | 50–100 MB target vs JVM's 300 MB+ baseline |
| Plugin distribution | crates.io (versioned, semver-checked) vs proprietary `.modl` |
| Driver crate code reuse | We use upstream `rust-ethernet-ip` directly, no wrapper-of-wrapper |

Rust **codes denser** than Java for equivalent functionality — typically 30–50% fewer lines, because there's no `getX()/setX()` boilerplate, no `Optional<>` ceremony, exhaustive pattern matching. This is partly why our [scale-estimates](scale-estimates.md) target ~3.1M LOC instead of Ignition's ~3M+.

## Why TypeScript (designer UI, web runtime, component library, protocol package)

The UI is half the platform's surface. Both the designer (authoring HMIs) and the runtime (running them) need rich, interactive web tech.

- **Largest contributor pool of any UI ecosystem.** When we say "community-extensible component library", that's only true if community can actually contribute. React + TS is the most accessible UI stack today.
- **The right libraries already exist.** Monaco (the editor that powers VS Code) for our script editor. react-konva for the visual canvas. Recharts / D3 for trends. ag-grid for the alarm table. We don't reimplement; we compose.
- **Type-safe wire protocol**. The `@openwebhmi/protocol` package is hand-mirrored from the Rust `crates/protocol` types. TypeScript catches binding errors between gateway and UI at compile time.
- **Vite dev experience** — sub-second hot reload while authoring designer features. Compare: a Java/Swing designer cycle is "edit → build → restart" in tens of seconds.
- **Same components serve designer + runtime.** A `Gauge` component renders the same in the designer (with selection adornments) and in the runtime (live-bound to a tag). One implementation, two contexts. Java's Swing widget set in Ignition doesn't share with Perspective's React widgets — they have to maintain both.

## Why Python (scripting host) — and why this is the biggest differentiator

This is where OpenWebHMI is meaningfully *better*, not just *equivalent*.

### What Ignition and Optix can do today

- **Ignition** ships **Jython 2.7** — Python implemented on the JVM, frozen at the 2.7 language version (released 2010, EOL'd by upstream Python in 2020). Calls any Java library; cannot meaningfully use modern Python packages.
- **Optix** uses **C# (NetLogic)** plus JavaScript. Powerful, but C# is not what plant data engineers reach for to do data work.

Neither has access to the modern Python data/AI/ML ecosystem **inside** the platform's scripting layer. Anything beyond basic logic gets pushed to an external service over REST.

### What OpenWebHMI does

We embed **CPython 3.11+** via [PyO3](https://pyo3.rs), running scripts in **worker subprocesses** (not in-process) for crash isolation. That single decision unlocks:

- **`import numpy`. `import pandas`. `import scikit-learn`. `import torch`. `import tensorflow`. `import xgboost`.** All of them. Native. No JVM bridge, no IPC fence, no "best-effort" port.
- **Real predictive maintenance in-platform.** Capture historian data → train a model in a Python script → deploy as a script that consumes live tag updates and writes derived prediction tags. The whole loop fits inside the gateway. With Ignition this is "stand up an external service, call it over REST, hope the latency is OK". With us it's:
  ```python
  # scripts/predictive/motor_health.py
  import joblib
  model = joblib.load("/projects/recipe-3/motor_model.pkl")
  
  def on_tag_change(tag):
      if tag.path == "rockwell-1/Motor1.Vibration":
          features = system.tag.read_history("rockwell-1/Motor1.*", lookback="10m")
          score = model.predict(features.to_numpy())[0]
          system.tag.write("derived/Motor1.HealthScore", float(score))
  ```
- **AI-assisted authoring in the designer.** The designer can call OpenAI, Anthropic, or a local model (Ollama, llama.cpp) from a Python plugin script. Auto-generate views from a tag list, suggest alarm conditions from natural language, propose tag bindings — all without leaving the designer. This is a serious feature differentiator: nobody else's plant-floor IDE has this.
- **Data science workflows in-platform.** Ad-hoc historian queries, anomaly detection on running data, time-series forecasting — Python is *the* language for this. Plant engineers already write Python notebooks; now they don't context-switch.
- **Modern Python packaging.** `pip install`, `uv`, `poetry`, `requirements.txt` — pick your tooling. A project bundles its `requirements.txt`; the gateway provisions a virtualenv per project. Versioning is solved.
- **Scripts crash-safely.** A numpy segfault, a torch CUDA error, an FFI abort in a C extension — none of it crashes the gateway, because scripts are subprocess-isolated. Ignition can't safely embed numpy because Jython doesn't support C extensions and the JVM share-everything model means a crash is fatal.

### Why Python *also* works as a designer extension language

The Tauri designer is Rust + TypeScript at its core, but plugins / extensions / migrators can be Python:

- **Project file linters and validators** — "warn if any tag has no description", "fail if any view has > 50 components" — trivial Python scripts living next to a project.
- **Schema migrators** — moving a project from `schema_version: 1` to `schema_version: 2` is a Python script that walks the JSON tree.
- **Code generators** — "generate scaffold views for every UDT in the tag list" — Python script in the designer.
- **AI-assisted authoring (again)** — designer plugins in Python can call LLMs to suggest scripts, fill in component bindings from tag names, refactor view hierarchies.
- **Faster contribution path than writing Tauri-side Rust.** A community contributor with Python skills can write a useful designer extension in an afternoon; doing it in Rust is a project.

### What we give up by choosing Python over Jython/C#

Honest tradeoffs:

| Trade | Cost |
|---|---|
| **No first-class Java interop** | Ignition can call any Java library directly from Jython. We can't — interop is via Rust FFI or shelling out. For most modern needs, Python's own ecosystem covers it; for legacy-Java-shop integrations, this is friction. |
| **No first-class .NET interop** | Same shape, Optix-side. |
| **Subprocess overhead per script invocation** | A few-ms cost vs in-process Jython. Negligible for tag-change handlers; visible if you fire 1000 tiny scripts/sec. We mitigate with a worker pool. |
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
│  │  │  Scripting host (PyO3) ─→ Python worker procs    │  │           │
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
