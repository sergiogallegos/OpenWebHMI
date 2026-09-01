<p align="center">
  <picture>
    <source media="(prefers-color-scheme: dark)" srcset="brand/openwebhmi-mark-white.svg">
    <img src="brand/openwebhmi-mark-blue.svg" alt="OpenWebHMI" width="96" />
  </picture>
</p>

<h1 align="center">OpenWebHMI</h1>

<p align="center">
  <strong>Open-source, web-first SCADA / HMI platform</strong> for small and mid-size industrial systems.<br />
  Five v1 drivers: <strong>Rockwell EtherNet/IP</strong>, <strong>OPC UA</strong>, <strong>Modbus TCP/RTU</strong>, <strong>MQTT</strong> (incl. Sparkplug B), and <strong>Beckhoff TwinCAT (ADS)</strong>. AGPL core, MPL protocol packages.
</p>

<p align="center">
  <a href="./LICENSE-POLICY.md"><img alt="License: AGPL-3.0-only core and MPL-2.0 protocols" src="https://img.shields.io/badge/License-AGPL--3.0%20core%20%7C%20MPL--2.0%20protocols-blue.svg"></a>
  <a href="./docs/roadmap.md"><img alt="Status: pre-alpha" src="https://img.shields.io/badge/Status-pre--alpha-red.svg"></a>
</p>

---

## What this is

OpenWebHMI is an **open-source alternative to Inductive Automation Ignition** (with Rockwell FactoryTalk Optix as a secondary reference). Self-hosted, gateway-centric, web-first runtime, cross-platform desktop designer. Built for plant-floor SCADA and HMI on **small and mid-size industrial systems**.

A single **Rust gateway** owns the project, the tags, the drivers, the historian, the alarm engine, the scripting host, and authentication. Any number of **HMI runtime clients** (web browsers) and **designer clients** (Tauri desktop on Linux, macOS, and Windows) connect to it. Plant-floor connectivity ships **five v1 drivers** spanning the dominant protocol families: Rockwell EtherNet/IP (CompactLogix, ControlLogix) via the [`rust-ethernet-ip`](https://github.com/sergiogallegos/rust-ethernet-ip) crate, OPC UA for vendor-neutral integration (Siemens, Schneider, Yokogawa, and many others expose OPC UA endpoints natively), Modbus TCP/RTU for serial and TCP devices, MQTT with Sparkplug B for IIoT broker patterns, and **Beckhoff TwinCAT via ADS** for symbol-based PLC, NC, and I/O access.

### v1.0 target scope

- **Single gateway** per deployment (no clustering / federation).
- **≤ 10,000 live tags** under one gateway.
- **≤ 50 concurrent runtime clients** per gateway.
- **Five drivers shipped**: Rockwell EtherNet/IP, OPC UA, Modbus TCP/RTU, MQTT (incl. Sparkplug B), Beckhoff TwinCAT (ADS).
- **Web-only runtime** (browser); Tauri desktop runtime is post-1.0.
- **Linux + macOS + Windows** for both the gateway and desktop Designer; Raspberry Pi 4 / 5 edge deployment is documented in [`docs/deployment/raspberry-pi.md`](docs/deployment/raspberry-pi.md).

These bounds are *the* design constraint. If anything in this repo implies bigger numbers, it's wrong and should be fixed. Larger deployments are a year-2+ conversation, not a v1.0 promise.

> **Status: pre-alpha.** The architecture, roadmap, and feature scope are committed. Code is being written. Don't deploy this anywhere that matters yet.

## Why another one

Industrial automation has been locked behind closed source and per-server licenses for decades. Tools like Ignition and Optix are excellent — and tens of thousands of dollars per gateway. There is no equivalent open-source platform with comparable reach: most open-source HMIs are either toy-scale, abandoned, or single-language stacks that don't address the SCADA + HMI surface.

OpenWebHMI's goal is a credible open-source platform a small or mid-size plant could actually run. AI-assisted development makes the timeline plausible in a way it wasn't five years ago.

## Stack

- **Gateway**: Rust + Tokio (single binary, embedded SQLite)
- **Drivers**: Rust, in-process plugin model. Five v1 drivers: Rockwell EtherNet/IP (via `rust-ethernet-ip`), OPC UA (via `async-opcua`), Modbus TCP/RTU (via `tokio-modbus`), MQTT incl. Sparkplug B (via `rumqttc` + `prost`), and Beckhoff TwinCAT ADS (via `ads`).
- **Designer / IDE**: Tauri (Rust shell) + React + TypeScript (Linux + macOS + Windows)
- **HMI runtime**: React + TypeScript in the browser
- **Scripting**: CPython 3.11+ worker subprocesses over JSON-RPC for crash isolation
- **Wire protocol**: JSON over WebSocket (single duplex stream per client)
- **Persistence**: SQLite for v1 (project store, historian, auth); pluggable backends post-1.0

Three languages total — Rust, TypeScript, Python. Deliberately *not* five.

### Stack at a glance vs the incumbents

| Layer | Ignition | FactoryTalk Optix | **OpenWebHMI** |
|---|---|---|---|
| Core / runtime | Java 17 / JVM | C++/Qt native platform + C#/.NET NetLogic[^optix-stack] | **Rust** |
| Designer | Java/Swing | C++/Qt + web technology; C#/.NET authoring[^optix-stack] | **Tauri + React/TypeScript** |
| Web HMI | Perspective — React/TypeScript over Java | Web Presentation Engine — HTML5/browser | **React + TypeScript** |
| Scripting | Jython 2.7.4 — Python 2.7 language level | C#/.NET NetLogic | **CPython 3.11+** in worker subprocesses |
| Open source? | ❌ | ❌ | ✅ AGPL core / MPL protocols |

[^optix-stack]: Optix is closed source. C++/Qt is supported by [Rockwell's native-runtime documentation](https://www.rockwellautomation.com/en-se/docs/factorytalk-optix/1-4-4/contents-ditamap/creating-projects/object-and-variable-reference/ftoptix-nativeui/datatypes/textrendertypeenum.html) and [current ASEM/Rockwell engineering roles](https://rockwellautomation.wd1.myworkdayjobs.com/en-US/External_Rockwell_Automation/job/Software-Engineer--C----Qt-_R26-1796); [C#/.NET NetLogic](https://www.rockwellautomation.com/en-us/docs/factorytalk-optix/1-5-7/contents-ditamap/extending-projects/netlogic.html) is publicly documented. The exact designer/runtime implementation boundary is not public.

OpenWebHMI assigns one clear responsibility to each language: Rust owns the always-on gateway and protocol boundary, TypeScript owns the shared designer/runtime UI, and isolated CPython workers own plant scripting and the modern data/AI ecosystem. The AGPL product core and MPL protocol packages keep the system auditable, self-hostable, air-gap friendly, and extensible without a proprietary module or per-server licensing gate. This is not an argument that Java or C# are incapable; it is a deliberate alignment between each subsystem and the ecosystem best suited to it. Full reasoning: [`docs/stack-rationale.md`](docs/stack-rationale.md).

## Quick reference

| Document | What it covers |
|---|---|
| [`docs/architecture.md`](docs/architecture.md) | System topology, every component, data flows, failure modes, security boundaries |
| [`docs/roadmap.md`](docs/roadmap.md) | Phase 0 → 1.0 plan with concrete exit criteria per phase |
| [`docs/feature-matrix.md`](docs/feature-matrix.md) | Side-by-side feature catalog vs Ignition and Optix; v1 / post-1.0 / not-planned markers |
| [`docs/planning/manufacturing-platform.md`](docs/planning/manufacturing-platform.md) | Authoritative post-1.0 manufacturing capability, architecture, dependency, and sequencing plan |
| [`docs/planning/manufacturing-demo.md`](docs/planning/manufacturing-demo.md) | Built-in starter/template and deterministic manufacturing demo plan |
| [`docs/stack-rationale.md`](docs/stack-rationale.md) | Why **Rust + Python + TypeScript** vs Ignition's Java/Jython or Optix's C++/Qt/C# stack — and why the Python side unlocks AI/ML and predictive maintenance natively |
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
- **Phase 5+** — modular manufacturing foundation and optional modules, redundancy, responsive runtime, and more drivers. Manufacturing sequencing is detailed in [`docs/planning/manufacturing-platform.md`](docs/planning/manufacturing-platform.md).

Full detail: [`docs/roadmap.md`](docs/roadmap.md).

## Targeted parity (highlights)

What OpenWebHMI 1.0 aims to ship that Ignition users would recognize:

- Tag providers, OPC tags, memory tags, expression tags, UDTs
- Drag/drop visual designer (Tauri, Linux + macOS + Windows)
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

The product core is [`AGPL-3.0-only`](LICENSE); `crates/protocol` and
`packages/protocol-ts` are [`MPL-2.0`](LICENSES/MPL-2.0.txt). Documentation and
marks have separate terms. See [`LICENSE-POLICY.md`](LICENSE-POLICY.md) and the
[`MIT → AGPL/MPL transition notice`](docs/license-transition.md).

Commercial use is permitted under the applicable licenses. No alternative
commercial license, activation server, module fee, or per-server fee is offered.
Revisions through `a8cbdbc` retain their prior MIT grant.
