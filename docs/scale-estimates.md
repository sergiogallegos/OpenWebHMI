# OpenWebHMI — Scale Estimates and Progress Tracking

> Floor target for **OpenWebHMI 1.0**: not less than **2,000,000 lines of code** across source, tests, and documentation. This document tracks expected ranges per component and the actual numbers as we progress.

---

## Why this document exists

An Ignition-class open-source SCADA/HMI/MES platform is — by reference points — a multi-million-LOC undertaking:

| Reference platform | Approx LOC | Stack |
|---|---|---|
| **Inductive Automation Ignition** | 2–3M (estimated, closed source) | Java + Jython 2.7 + Perspective JS/TS |
| **FactoryTalk Optix** | similar scale (closed source) | C++/Qt native platform + C#/.NET NetLogic + web technology |
| **VS Code** | ~2M | TypeScript + supporting |
| **OpenWebHMI 1.0 floor target** | **≥ 2M** | **Rust + Python (CPython 3.11+) + TypeScript** |
| OpenWebHMI year-2 stretch (incl. MES, mobile, redundancy) | 3–4M | same |

Tracking expected vs actual LOC at the component level lets us:

1. **Sanity-check progress.** If after 6 months we're at 50k LOC for the gateway alone, we're not building Ignition-class — we're building a toy.
2. **Spot scope drift early.** A component growing 2× past its max range without a deliberate decision means scope creep or unnecessary abstraction.
3. **Communicate honestly.** "We are 4% of the way to 1.0" is a real number, not a vibe.

LOC is a **flawed metric** — quality, not quantity, ships software. We use it as a coarse progress sanity check, never as a target or a budget. A change that *removes* 10k lines without losing capability is celebrated, not penalized.

## Stack comparison (why our LOC profile differs)

| Layer | Ignition | Optix | OpenWebHMI |
|---|---|---|---|
| Core / runtime | Java 17 / JVM | C++/Qt native platform + C#/.NET NetLogic | Rust |
| Designer / IDE | Java/Swing | C++/Qt + web technology; C#/.NET authoring | Tauri (Rust shell) + React/TS |
| Web HMI runtime | Perspective (React/TS over Java backend) | Web Presentation Engine (HTML5/browser) | React + TypeScript |
| Scripting | Jython 2.7.4 (Python 2.7 language level) | C#/.NET NetLogic | CPython 3.11+ worker subprocesses |
| Module / extension distribution | proprietary `.modl` | proprietary | crates.io / npm / PyPI |
| Deployment shape | JVM-based gateway and clients | native runtime for Windows/Linux and x86/ARM | Rust gateway + browser clients + CPython workers |

Three implications for our LOC profile:

- **Rust is denser than Java.** Equivalent functionality typically takes 30–50% fewer LOC in idiomatic Rust than in idiomatic Java (no boilerplate `getX()/setX()`, no `Optional<>` ceremony, exhaustive pattern matching). So matching Ignition feature-for-feature should land us in the ~1.5–2.5M LOC range, not 3M.
- **Tauri designer is much smaller than a Java/Swing designer.** Web tech (React + Monaco) is less verbose than Swing. Designer LOC budget is meaningfully lower.
- **CPython 3 vs Jython 2.7** — our scripting host manages standard Python worker processes rather than implementing another interpreter.

These are tailwinds, not excuses. The 2M floor still holds.

## Component breakdown

Ranges are wide because they reflect honest uncertainty, not false precision. **Expected** is the planning anchor.

### Source code (no tests, no docs)

| # | Component | Path | Language | Min | **Expected** | Max |
|---|---|---|---|---:|---:|---:|
| 1 | Wire protocol | `crates/protocol`, `packages/protocol-ts` | Rust + TS | 8k | **15k** | 25k |
| 2 | Tag engine | `crates/tag-engine` | Rust | 15k | **25k** | 40k |
| 3 | Gateway core | `crates/gateway` | Rust | 50k | **80k** | 120k |
| 4 | Driver API | `crates/driver-api` | Rust | 8k | **15k** | 25k |
| 5 | Driver — Rockwell EtherNet/IP | `crates/driver-rockwell` | Rust | 30k | **50k** | 80k |
| 6 | Driver — OPC UA (v1) | `crates/driver-opcua` | Rust | 60k | **100k** | 150k |
| 7 | Driver — Modbus TCP (post-1.0) | `crates/driver-modbus` | Rust | 20k | **30k** | 50k |
| 8 | Alarm engine | `crates/alarm-engine` | Rust | 25k | **40k** | 70k |
| 9 | Historian | `crates/historian` | Rust | 30k | **50k** | 80k |
| 10 | Project store | `crates/project-store` | Rust | 20k | **35k** | 60k |
| 11 | Auth + sessions | `crates/auth` | Rust | 15k | **25k** | 40k |
| 12 | Scripting host | `crates/scripting` (Rust host + CPython worker harness) | Rust + Py | 30k | **50k** | 80k |
| 13 | Designer / IDE | `apps/designer` (Tauri shell + React UI) | Rust + TS | 200k | **350k** | 500k |
| 14 | HMI runtime — web | `apps/runtime-web` | TS | 80k | **130k** | 200k |
| 15 | Component library | `packages/component-library` | TS | 50k | **90k** | 150k |
| 16 | Plugin SDK | `packages/sdk` + Rust crate templates + Python stubs | Rust + TS + Py | 8k | **15k** | 25k |
| 17 | Simulators / examples | `examples/sim-rockwell`, future `examples/sim-*` | Rust | 25k | **50k** | 80k |
| 18 | CI / scripts / tooling | `.github/`, `scripts/` | YAML + sh | 5k | **10k** | 20k |
| 19 | Installers | `installers/` (NSIS, pkg, deb/rpm, Docker, Helm) | various | 8k | **20k** | 40k |
| 20 | Website / docs site | `apps/website` (docs.openwebhmi.org) | TS / MDX | 15k | **30k** | 60k |
| | **Source subtotal** | | | **~700k** | **~1,210k** | **~1,895k** |

### Tests

Industry mid-range: tests run **1.2× to 1.8× the source LOC** for SCADA-grade quality. We pin **1.5×** as the planning anchor.

| Estimate | Source × 1.5 | Total tests |
|---|---:|---:|
| Min | 700k × 1.2 | 840k |
| **Expected** | **1,210k × 1.5** | **~1,815k** |
| Max | 1,895k × 1.8 | 3,411k |

### Documentation (Markdown only)

| Surface | Min | **Expected** | Max |
|---|---:|---:|---:|
| User docs (`docs/`) | 15k | **30k** | 60k |
| Engineering wiki (`wiki/`) | 20k | **50k** | 100k |
| Tutorials | 10k | **20k** | 50k |
| API reference (auto-generated counted separately, hand-written for high-level) | 5k | **20k** | 40k |
| **Docs subtotal** | **50k** | **~120k** | **~250k** |

### v1.0 grand total (source + tests + docs)

| | Min | **Expected** | Max |
|---|---:|---:|---:|
| Source | 700k | **1,210k** | 1,895k |
| Tests | 840k | **1,815k** | 3,411k |
| Docs | 50k | **120k** | 250k |
| **TOTAL v1.0** | **~1,590k** | **~3,145k** | **~5,556k** |

The **expected total of ~3.1M LOC is in line with Ignition's estimated scale**. The **floor of ~1.6M** is below our 2M target, which means Phases 0 → 4 should track to expected, not to min, to clear the bar.

### Year-2+ additions (Phase 5+)

| Component | Min | **Expected** | Max |
|---|---:|---:|---:|
| MES — recipes, OEE, batch, traceability | 200k | **350k** | 500k |
| Mobile / responsive layouts | 50k | **80k** | 130k |
| Redundancy / failover | 30k | **60k** | 100k |
| Additional drivers (Siemens S7, MQTT/Sparkplug, BACnet, ASCII) | 200k | **350k** | 500k |
| Designer enhancements (multi-developer, git integration) | 50k | **100k** | 200k |
| Tests for the above (1.5×) | — | **~1,400k** | — |
| Docs for the above | 30k | **60k** | 120k |
| **Year-2+ subtotal** | **~560k** | **~2,400k** | — |

**Full platform (1.0 + year-2 expected): ~5.5M LOC.** That's in the same league as Ignition + Sepasoft modules combined.

## Methodology — how we count

### Tool

[`tokei`](https://github.com/XAMPPRocky/tokei) is the canonical counter. Reproducible, fast, multi-language, ignores blanks/comments separately from code.

```bash
brew install tokei            # macOS
cargo install tokei           # any platform
```

### Command

The single command of record for "what's the project's LOC":

```bash
tokei \
  --exclude target \
  --exclude node_modules \
  --exclude dist \
  --exclude pnpm-lock.yaml \
  --exclude Cargo.lock \
  --exclude .obsidian
```

A wrapper script lives at [`scripts/loc-snapshot.sh`](../scripts/loc-snapshot.sh) so the count is one command.

### What counts

- **Code** (the column tokei labels `Code`) — actual non-blank, non-comment source lines.
- **Comments** are tracked but reported separately. We do not pad LOC with comments.
- **Markdown** is reported under "Comments" by tokei (intentional — Markdown is documentation, not code). For our totals: Markdown is documentation LOC, *not* source LOC.
- **Generated files** (`Cargo.lock`, `pnpm-lock.yaml`, `dist/`, `target/`) are excluded.
- **Vendored / third-party** is excluded.

### What does NOT count

- Lockfiles. They're machine-managed.
- Build artifacts (`dist/`, `target/`, `*.tsbuildinfo`).
- IDE/editor state (`.obsidian/`, `.vscode/`).
- The `wiki/` directory's `index.md` and `log.md` are counted as docs (they're authored content), not source.

## Tracking discipline

- **End of each phase**: capture a new snapshot; add a row to "Progress history" below.
- **End of each task that creates ≥10k LOC** (single component crossing the threshold): capture a snapshot.
- **Never edit prior history rows.** If a count was wrong, add a new row that supersedes it.
- **Snapshots commit as their own commit** with title `chore: LOC snapshot YYYY-MM-DD — Phase N` so `git log --grep="LOC snapshot"` is the audit trail.

## Phase milestones (expected LOC checkpoints)

These are *expected* numbers at the end of each phase, used to flag drift early. Significantly under = scope cut or speed problem. Significantly over = scope creep.

| Phase | End-of-phase target | Source | Tests | Docs | Total |
|---|---|---:|---:|---:|---:|
| 0 — Foundations | already shipped | ~3k | ~2k | ~10k | **~15k** |
| 1 — PLC vertical slice | end Phase 1 | ~80k | ~120k | ~30k | **~230k** |
| 2 — Designer MVP | end Phase 2 | ~250k | ~370k | ~50k | **~670k** |
| 3 — Core SCADA | end Phase 3 | ~600k | ~900k | ~80k | **~1,580k** |
| 4 — 1.0 release | end Phase 4 | ~1,210k | ~1,815k | ~120k | **~3,145k** |
| 5+ — MES + extras | year 2 | ~2,200k | ~3,300k | ~180k | **~5,680k** |

## Progress history

Append-only. Newest at bottom. One row per snapshot.

| Date | Phase | Commit | Source (Rust+TS+config) | Tests | Docs (Markdown) | Total | Notes |
|---|---|---|---:|---:|---:|---:|---|
| 2026-04-26 | Phase 0 complete | `aeb9c71` | 2,080 | (counted within source above) | 2,727 | **6,389** | Baseline. ~0.2% of v1.0 expected. |

> Definition for the table: "Source" = `tokei` Code column for non-Markdown languages, excluding lockfiles and build artifacts. "Tests" are not yet broken out separately because Phase 0 mixes test code into source crates; from Phase 1 onward, tests will be counted by file path patterns (`*/tests/*`, `*.test.ts`).

## What "actual" looks like today (2026-04-26)

Run `bash scripts/loc-snapshot.sh` to reproduce locally. Snapshot:

```
Source code (no Markdown):
  Rust:                1,272 LOC  (gateway, protocol, tag-engine)
  TypeScript + TSX:      621 LOC  (protocol-ts, runtime-web)
  TOML / JSON / HTML:    187 LOC  (configs)
  Subtotal:            2,080 LOC

Documentation:
  Markdown:            2,727 lines  (~1,994 content lines, ~733 blank)

Other:
  YAML (CI):               9 lines

Project total:         6,389 lines
```

**Where we stand: ~0.2% of the v1.0 expected total.** That is exactly where a Phase 0 baseline should be — proof that the workflow operates and the architecture is alive, not a substantial chunk of the platform.

## Caveats

- **Driver-rockwell wraps `rust-ethernet-ip` (external crate, ~13k+ LOC upstream).** The upstream LOC is *not counted* in our totals; we only count the wrapper. This is correct — we don't take credit for code we didn't write — but it means our driver LOC numbers underestimate the *capability* delivered per LOC.
- **Component library LOC depends heavily on whether we ship Storybook stories** for every component. The expected range assumes yes. Without Storybook, runtime LOC drops by ~30%.
- **Designer is the highest-uncertainty component.** Visual canvas + property panel + tag-binding picker + Monaco script editor + project explorer + alarm config + theme editor is genuinely a small IDE. The 200k–500k range is wide for that reason.

## Related documents

- [`docs/architecture.md`](architecture.md) — what the components are.
- [`docs/roadmap.md`](roadmap.md) — when each component lands.
- [`docs/feature-matrix.md`](feature-matrix.md) — the scope contract this LOC plan is sized for.
