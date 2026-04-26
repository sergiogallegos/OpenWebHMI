# OpenWebHMI

> **Open-source SCADA / HMI / MES platform.** Ignition-class capability, MIT-licensed, community-extensible.

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](./LICENSE)
[![Status: pre-alpha](https://img.shields.io/badge/Status-pre--alpha-red.svg)](./docs/roadmap.md)

---

## What this is

OpenWebHMI is an **open-source alternative to Inductive Automation Ignition** (with Rockwell FactoryTalk Optix as a secondary reference). Self-hosted, gateway-centric, web-first runtime, cross-platform desktop designer. Built for plant-floor SCADA and HMI today, with MES capabilities planned for year two.

A single **Rust gateway** owns the project, the tags, the drivers, the historian, the alarm engine, the scripting host, and authentication. Any number of **HMI runtime clients** (web browsers) and **designer clients** (Tauri desktop on Win + Mac) connect to it. Plant-floor connectivity ships with a Rockwell EtherNet/IP driver (CompactLogix, ControlLogix) backed by the [`rust-ethernet-ip`](https://github.com/sergiogallegos/rust-ethernet-ip) crate.

> **Status: pre-alpha.** The architecture, roadmap, and feature scope are committed. Code is being written. Don't deploy this anywhere that matters yet.

## Why another one

Industrial automation has been locked behind closed source and per-server licenses for decades. Tools like Ignition and Optix are excellent — and tens of thousands of dollars per gateway. There is no equivalent open-source platform with comparable reach: most open-source HMIs are either toy-scale, abandoned, or single-language stacks that don't address the SCADA + HMI + MES surface.

OpenWebHMI's goal is a credible open-source platform a small or mid-sized plant could actually run. AI-assisted development makes the timeline plausible in a way it wasn't five years ago.

## Stack

- **Gateway**: Rust + Tokio (single binary, embedded SQLite)
- **Drivers**: Rust, in-process plugin model (first driver wraps `rust-ethernet-ip`)
- **Designer / IDE**: Tauri (Rust shell) + React + TypeScript (Win + Mac)
- **HMI runtime**: React + TypeScript in the browser
- **Scripting**: Python 3.11+ via PyO3, run in worker subprocesses for crash isolation
- **Wire protocol**: JSON over WebSocket (single duplex stream per client)
- **Persistence**: SQLite for v1 (project store, historian, auth); pluggable backends post-1.0

Three languages total — Rust, TypeScript, Python. Deliberately *not* five.

### Stack at a glance vs the incumbents

| Layer | Ignition | Optix | **OpenWebHMI** |
|---|---|---|---|
| Gateway | Java (JVM) | C# / .NET | **Rust** (single static binary) |
| Designer | Java + Swing | Visual Studio (Win-only) | **Tauri + React/TS** (Win + Mac) |
| Web HMI | Perspective (React/TS over Java) | Optix WebPresentation | **React + TypeScript** |
| Scripting | Jython 2.7 (frozen 2010) | C# / JS | **CPython 3.11+** in worker subprocesses |
| Open source? | ❌ | ❌ | ✅ MIT |

The CPython 3 scripting layer is the headline differentiator: `numpy`, `pandas`, `scikit-learn`, `torch`, and LLM SDKs all run *natively in scripts* — Ignition's Jython 2.7 cannot. That makes in-platform predictive maintenance, anomaly detection, and AI-assisted authoring a normal feature, not an integration project. Full reasoning: [`docs/stack-rationale.md`](docs/stack-rationale.md).

## Quick reference

| Document | What it covers |
|---|---|
| [`docs/architecture.md`](docs/architecture.md) | System topology, every component, data flows, failure modes, security boundaries |
| [`docs/roadmap.md`](docs/roadmap.md) | Phase 0 → 1.0 plan with concrete exit criteria per phase |
| [`docs/feature-matrix.md`](docs/feature-matrix.md) | Side-by-side feature catalog vs Ignition and Optix; v1 / post-1.0 / not-planned markers |
| [`docs/stack-rationale.md`](docs/stack-rationale.md) | Why **Rust + Python + TypeScript** vs Ignition's Java/Jython or Optix's C# — and why the Python side unlocks AI/ML and predictive maintenance natively |
| [`docs/scale-estimates.md`](docs/scale-estimates.md) | Expected LOC per component for v1.0 (target floor: **2M+**), with the actual count tracked over time |
| [`docs/contributing.md`](docs/contributing.md) | How to add drivers, components, scripts; PR workflow; local dev setup |
| [`AGENTS.md`](AGENTS.md) | Rules for AI agents (and humans) maintaining the wiki and decision record |
| [`docs/agents/`](docs/agents/) | Cross-LLM collaboration protocol — Claude designs/reviews, Codex develops/debugs, all hand-offs durable in markdown |
| [`wiki/`](wiki/) | Synthesized engineering knowledge — vendor quirks, validation results, decision rationale |

## Roadmap headline

- **Phase 0** — Foundations: gateway boots, simulated tags flow to a browser. *(~1 month)*
- **Phase 1** — Vertical slice: Rockwell driver wired end-to-end, validated against an EtherNet/IP simulator (no physical PLC available yet). *(~2 months)*
- **Phase 2** — Designer MVP: visual drag/drop authoring, 10 standard components, hot reload. *(~3 months)*
- **Phase 3** — Core SCADA: alarms, historian + trends, auth + roles, Python scripting. *(~3 months)*
- **Phase 4** — 1.0: second driver, plugin SDK, 25+ components, real-hardware validation gate, public release. *(~3 months)*
- **Phase 5+** — MES (recipes, OEE, traceability), redundancy, mobile, more drivers.

Full detail: [`docs/roadmap.md`](docs/roadmap.md).

## Targeted parity (highlights)

What OpenWebHMI 1.0 aims to ship that Ignition users would recognize:

- Tag providers, OPC tags, memory tags, expression tags, UDTs
- Drag/drop visual designer (Tauri, Win + Mac)
- Web HMI runtime with live tag binding, themes, navigation
- Alarm engine with state machine, journal, ack workflow
- Tag historian with trend rendering and aggregations
- Python scripting with `system.tag`, `system.alarm`, `system.db`, `system.http` libraries
- Role-based auth, per-view ACLs, TLS
- Project export / import as a portable archive
- Plugin SDK for community drivers and components

What we explicitly *don't* try to be: a hard-real-time control platform, a safety-rated (SIL) system, or a replacement for vendor-specific engineering tooling.

## Building

> **Pre-Phase-0:** the Cargo workspace is currently empty (`members = []` in `Cargo.toml`), so `cargo build --workspace` and `cargo test --workspace` exit with "virtual workspace has no members" — that's expected. The full setup below becomes runnable once the first Phase 0 crate lands. `cargo metadata` and `pnpm install` work today.

```bash
git clone https://github.com/sergiogallegos/OpenWebHMI.git
cd OpenWebHMI
pnpm install
cargo build --workspace   # runnable from Phase 0 onwards
```

See [`docs/contributing.md`](docs/contributing.md) for the full local-dev runbook.

## Contributing

Contributions are welcome and expected — the project is structured around community-built drivers, components, and script libraries. Read [`docs/contributing.md`](docs/contributing.md) before opening a PR. For non-trivial changes, file an issue first so the design conversation happens before the code.

## Maintainers

Sergio Gallegos — repo owner — [sergiogallegos.net](https://sergiogallegos.net)

## License

MIT — see [`LICENSE`](LICENSE).
