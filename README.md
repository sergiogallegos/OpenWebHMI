<p align="center">
  <picture>
    <source media="(prefers-color-scheme: dark)" srcset="brand/openwebhmi-mark-white.svg">
    <img src="brand/openwebhmi-mark-blue.svg" alt="OpenWebHMI" width="96" />
  </picture>
</p>

<h1 align="center">OpenWebHMI</h1>

<p align="center">
  <strong>Open-source, web-first SCADA / HMI platform</strong> for small and mid-size industrial systems.<br />
  Five v1 drivers: <strong>Rockwell EtherNet/IP</strong>, <strong>OPC UA</strong>, <strong>Modbus TCP/RTU</strong>, <strong>MQTT</strong> (incl. Sparkplug B), and <strong>Beckhoff TwinCAT (ADS)</strong>. MIT-licensed, community-extensible.
</p>

<p align="center">
  <a href="./LICENSE"><img alt="License: MIT" src="https://img.shields.io/badge/License-MIT-yellow.svg"></a>
  <a href="./docs/roadmap.md"><img alt="Status: pre-alpha" src="https://img.shields.io/badge/Status-pre--alpha-red.svg"></a>
</p>

---

## What this is

OpenWebHMI is an **open-source alternative to Inductive Automation Ignition** (with Rockwell FactoryTalk Optix as a secondary reference). Self-hosted, gateway-centric, web-first runtime, cross-platform desktop designer. Built for plant-floor SCADA and HMI on **small and mid-size industrial systems**.

A single **Rust gateway** owns the project, the tags, the drivers, the historian, the alarm engine, the scripting host, and authentication. Any number of **HMI runtime clients** (web browsers) and **designer clients** (Tauri desktop on Win + Mac) connect to it. Plant-floor connectivity ships **five v1 drivers** spanning the dominant protocol families: Rockwell EtherNet/IP (CompactLogix, ControlLogix) via the [`rust-ethernet-ip`](https://github.com/sergiogallegos/rust-ethernet-ip) crate, OPC UA for vendor-neutral integration (Siemens, Schneider, Yokogawa, and many others expose OPC UA endpoints natively), Modbus TCP/RTU for serial and TCP devices, MQTT with Sparkplug B for IIoT broker patterns, and **Beckhoff TwinCAT via ADS** for symbol-based PLC, NC, and I/O access.

### v1.0 target scope

- **Single gateway** per deployment (no clustering / federation).
- **≤ 10,000 live tags** under one gateway.
- **≤ 50 concurrent runtime clients** per gateway.
- **Five drivers shipped**: Rockwell EtherNet/IP, OPC UA, Modbus TCP/RTU, MQTT (incl. Sparkplug B), Beckhoff TwinCAT (ADS).
- **Web-only runtime** (browser); Tauri desktop runtime is post-1.0.
- **Linux + macOS + Windows** for the gateway; Raspberry Pi 4 / 5 edge deployment is documented in [`docs/deployment/raspberry-pi.md`](docs/deployment/raspberry-pi.md). Designer support remains desktop-focused.

These bounds are *the* design constraint. If anything in this repo implies bigger numbers, it's wrong and should be fixed. Larger deployments are a year-2+ conversation, not a v1.0 promise.

> **Status: pre-alpha.** The architecture, roadmap, and feature scope are committed. Code is being written. Don't deploy this anywhere that matters yet.

## Why another one

Industrial automation has been locked behind closed source and per-server licenses for decades. Tools like Ignition and Optix are excellent — and tens of thousands of dollars per gateway. There is no equivalent open-source platform with comparable reach: most open-source HMIs are either toy-scale, abandoned, or single-language stacks that don't address the SCADA + HMI surface.

OpenWebHMI's goal is a credible open-source platform a small or mid-size plant could actually run. AI-assisted development makes the timeline plausible in a way it wasn't five years ago.

## Stack

- **Gateway**: Rust + Tokio (single binary, embedded SQLite)
- **Drivers**: Rust, in-process plugin model. Five v1 drivers: Rockwell EtherNet/IP (via `rust-ethernet-ip`), OPC UA (via `async-opcua`), Modbus TCP/RTU (via `tokio-modbus`), MQTT incl. Sparkplug B (via `rumqttc` + `prost`), and Beckhoff TwinCAT ADS (via `ads`).
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
| [`AGENTS.md`](AGENTS.md) | Codebase-wide code, test, and dependency rules for any agent (Codex, Claude Code) — auto-loaded |
| [`VISION.md`](VISION.md) | What OpenWebHMI is, non-goals, and what we won't merge |
| [`wiki/AGENTS.md`](wiki/AGENTS.md) | Engineering-wiki governance — layered docs, source authority, page format |
| [`docs/agents/`](docs/agents/) | Cross-LLM collaboration protocol — Claude designs/reviews, Codex develops/debugs, all hand-offs durable in markdown |
| [`wiki/`](wiki/) | Synthesized engineering knowledge — vendor quirks, validation results, decision rationale |
| [`apps/website/`](apps/website/) | Public marketing + docs site (Astro). Hosted at [`openwebhmi.com`](https://openwebhmi.com) (DNS via Cloudflare) |

## Roadmap headline

- **Phase 0** — Foundations: gateway boots, simulated tags flow to a browser. *(~1 month)*
- **Phase 1** — Vertical slice: Rockwell driver wired end-to-end, validated against an EtherNet/IP simulator (no physical PLC available yet). *(~2 months)*
- **Phase 2** — Designer MVP: visual drag/drop authoring, 10 standard components, hot reload. *(~3 months)*
- **Phase 3** — Core SCADA: alarms, historian + trends, auth + roles, Python scripting. *(~3 months)*
- **Phase 4** — 1.0: four new drivers (OPC UA, Modbus TCP/RTU, MQTT/Sparkplug B, Beckhoff ADS) alongside Rockwell, plugin SDK, 25 components, real-hardware validation gate, public release. *(~3 months)*
- **Phase 5+** — MES (recipes, OEE, traceability), redundancy, mobile, more drivers.

Full detail: [`docs/roadmap.md`](docs/roadmap.md).

## Targeted parity (highlights)

What OpenWebHMI 1.0 aims to ship that Ignition users would recognize:

- Tag providers, OPC tags, memory tags, expression tags, UDTs
- Drag/drop visual designer (Tauri, Win + Mac)
- Web HMI runtime with live tag binding, project themes, navigation. See [`docs/theme-editor.md`](docs/theme-editor.md).
- Raspberry Pi 4 / 5 gateway deployment guide with systemd unit and cross-compile notes. See [`docs/deployment/raspberry-pi.md`](docs/deployment/raspberry-pi.md).
- Single-widget export/import for moving configured components between projects. See [`docs/widget-export-import.md`](docs/widget-export-import.md).
- Opt-in Material Design demo widget pack. See [`docs/widget-packs.md`](docs/widget-packs.md).
- Alarm engine with state machine, journal, ack workflow
- Tag historian with trend rendering, aggregations, and a 281 TB SQLite file-format ceiling. See [`docs/historian.md`](docs/historian.md).
- Python scripting with `system.tag`, `system.alarm`, `system.db`, `system.http` libraries
- Role-based auth, per-view ACLs, TLS
- Project export / import as a portable archive
- Plugin SDK for community drivers and components

What we explicitly *don't* try to be:

- A hard-real-time control platform.
- A safety-rated (IEC 61508 / SIL) system.
- A replacement for vendor-specific PLC engineering tooling.
- An MES platform — recipes, OEE, batch, and traceability are post-1.0 (year-2+) and explicitly out of scope for the headline. Planning notes only in [`docs/roadmap.md`](docs/roadmap.md) Phase 5+.
- An enterprise-scale (>10K tag, >50 client, multi-gateway) system in v1.

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
