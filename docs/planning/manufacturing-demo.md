# Manufacturing Starter and Demo Plan

> Status: planned after the v1.0 hardening gate. The initial template is useful
> before OEE, SPC, energy, or traceability exist; later modules expand the same
> fictional line.

## Product-ready release requirement

OpenWebHMI is not considered product-ready without a polished demo bundled in
the standard distribution. The demo is installed locally by default, requires
no separate download or internet connection, and is offered from the first-run
and new-project experience. Users may delete or ignore it, but they must not
have to discover, install, or configure an optional package before seeing a
working OpenWebHMI system.

The release artifact must therefore contain at least the initial useful slice
defined below. More advanced manufacturing modules may expand the demo after
release, but the lack of OEE, SPC, energy, or traceability cannot justify
shipping an empty first-run experience.

## Goals

The built-in demo is simultaneously a first-run experience, cloneable project
template, documentation sample, integration-test fixture, screenshot source,
and tutorial environment. It requires no PLC or internet connection and never
silently replaces a failed real source with simulation.

## Two first-class data modes

The same bundled demo project supports two explicit modes:

1. **Offline Simulation** — the safe first-run default. It requires no PLC,
   engineering software, network access, or internet connection and continuously
   supplies deterministic fictional operating data.
2. **Rockwell Demo PLC** — an opt-in real-hardware profile for a supported
   CompactLogix or ControlLogix controller. It uses OpenWebHMI's real
   EtherNet/IP driver and the same logical demo tags, screens, alarms, facts,
   and reports as simulation.

Mode selection is explicit, persistent, role-gated, and always visible in the
runtime and Designer. A real PLC disconnect, bad tag, or configuration error
produces bad/stale/disconnected quality with unavailable values. It never starts
the simulator or substitutes simulated values automatically.

The Rockwell package contains only fictional, redistributable demo material:

- a documented generic controller interface and logical tag map;
- a versioned controller project/export or reproducible import instructions,
  subject to license review;
- supported controller/firmware matrix and download/commissioning runbook;
- configurable route/address with no shipped plant IP or credentials;
- read and command allowlists, PLC permissives, watchdog behavior, and audit;
- hardware validation evidence including reconnect and long-duration operation.

Simulation and Rockwell adapters must emit equivalent logical behavior so a
user can learn offline, then point the cloned project at a real demo PLC without
rebuilding the HMI screens. Protocol-specific addresses stay in the connection
profile, never in views or manufacturing modules.

Future project choices:

- Blank Project
- Basic Machine HMI
- Manufacturing Machine
- Production/OEE
- Full Manufacturing Demo

All are templates/module bundles, not licensing editions.

## Fictional line

The full demo uses five simple stations:

```text
Load -> Assemble -> Inspect -> Test -> Unload
```

Each station is an internal station of one demo line machine for the first
slice. A later expansion can demonstrate multiple physical machines without
changing the equipment/fact contracts.

| Station | Normal behavior | Demonstrated abnormal behavior |
|---|---|---|
| Load | Introduces a fictional part and model | Starved, operator wait, sensor bad quality |
| Assemble | Cycle/torque measurement | Tool fault, slow cycle, maintenance wear |
| Inspect | Dimension/vision result | Drift, rejects, uncertain measurement |
| Test | Pressure/leak result | Failure, blocked downstream, retest |
| Unload | Completes boundary count | Blocked, count reset/recovery scenario |

The data uses generic names, fictional values, and generated identifiers. It
contains no reference-project branding, tags, addresses, alarm wording, process
drawings, or production values.

## Deterministic simulator

The simulator uses a seed and controllable logical clock. A scenario reset
reproduces the same events. It emits through the same tag/quality/fact contracts
used by modules and supports:

- normal production with bounded cycle variation;
- machine fault and simultaneous internal faults;
- blocked and starved flow;
- operator wait;
- model change and mismatch;
- reject and measurement drift;
- stale, bad, and disconnected sources;
- alarm flood/chatter scenario;
- target and policy change;
- maintenance degradation;
- energy variation by machine state;
- archive/reset and replay in later slices.

Every simulated value/fact carries simulated origin. Runtime and Designer show
an unmistakable simulation banner. Switching a real source to bad quality does
not activate simulation.

## Initial useful slice

The first built-in template should depend only on stable v1 platform services
plus the template/scenario work:

- Home/machine overview with mode, state, quality, count, and active alarm;
- simple Auto/Manual simulation controls through the command gateway;
- alarms and acknowledgment;
- live/historical trend;
- device/quality diagnostics;
- role examples for Administrator, Engineer, Operator, and Viewer;
- tutorial notes and reset-to-known-state action.

This slice is valuable before manufacturing analytics ship.

## Manufacturing expansion

After equipment/fact/production modules:

- interactive line layout and station drill-down;
- target versus actual, hourly output, good/reject, throughput, cycle metrics;
- machine-state timeline and quality gaps;
- top faults, Pareto, occurrence, duration, MTBF/MTTR;
- reproducible CSV/PDF shift report.

After downtime/OEE:

- versioned reason classification and supervisor correction;
- planned/external treatment and recalculation indicator;
- A/P/Q/OEE with inputs, completeness, policy version, and time filters;
- top losses and model/run analysis windows.

Later optional expansions:

- part/process table with fictional torque/dimension/leak measurements;
- statistically correct SPC example;
- power/state/part normalization and configurable demo CO2e factor;
- maintenance counters, PM due, documents, and transparent health rules;
- traceability only after the part identity/operation model exists.

## Screen catalog

| Screen | Initial | Expansion |
|---|:-:|---|
| Home/machine overview | Yes | Line layout, model, targets, OEE |
| Machine control | Simulation only | Permissives, command result/actual |
| Alarms | Yes | Shelving and analytics |
| Trends | Yes | Event overlays and replay |
| Engineering | Yes | Module/device health and versions |
| Production | Later | Counts, cycle distribution, targets |
| State/downtime | Later | Timeline, reasons, corrections, Pareto |
| Fault analytics | Later | Pareto, MTBF/MTTR, quality gaps |
| OEE | Later | A/P/Q, policy/completeness, rollups |
| Reports | Later | PDF/CSV with version metadata |
| Part/SPC/Energy/Maintenance | Optional | Added only with their modules |

## Template distribution

A template is a versioned project archive with:

- project schema and required OpenWebHMI version;
- required and optional module ids/versions;
- fictional assets, views, tags, alarms, scripts, roles, and scenarios;
- migration path and compatibility validation;
- a new-project transform that regenerates project/component identifiers;
- tests proving creation, import/export, startup, and deterministic scenario output.

The Designer offers `Create from template`, previews requirements, creates a
new independent project, and reports missing/incompatible modules before making
changes. Users can remove optional modules without corrupting the base project.

## Tutorial path

The same project supports incremental tutorials:

1. Create/open the demo and understand simulation quality.
2. Browse tags and bind a value.
3. Build an Auto screen and guarded simulated command.
4. Configure alarms and trends.
5. Add production and state timeline.
6. Add downtime classification and fault Pareto.
7. Configure OEE inputs and inspect completeness.
8. Add line-layout navigation.
9. Add optional SPC, energy, maintenance, or part data.
10. Configure users/roles, export reports, and deploy the gateway.

## Marketing and public-content source

The bundled demo is the canonical source for public OpenWebHMI visuals and
stories. Website screenshots, documentation images, release posts, social
posts, conference material, and YouTube videos should use named deterministic
demo scenes rather than one-off mockups or customer projects.

The demo therefore provides a small capture catalog such as:

- `overview-normal` — healthy line and production summary;
- `fault-response` — active alarm, affected station, and drill-down;
- `quality-degraded` — stale/bad values shown without fabricated defaults;
- `production-loss` — state timeline, target gap, and attributed loss;
- `oee-explained` — A/P/Q inputs, completeness, and policy version when available;
- `designer-build` — the same project open in the Designer;
- `mobile-overview` — responsive runtime view when that surface is supported.

Each scene fixes the seed, logical time, viewport, locale, theme, user role, and
scenario step. Capture tooling may automate screenshots and short browser
recordings, but the resulting media is generated from the real shipped runtime
and demo project. Marketing must not use hidden mock-only screens that users
cannot reproduce.

Capture metadata also records `Offline Simulation` or `Rockwell Demo PLC` as the
source. Most repeatable public captures use simulation; hardware-specific videos
may use the real PLC profile and must identify it accurately.

The content kit includes:

- approved screenshot list and capture instructions;
- YouTube episode/run-of-show outlines tied to tutorial checkpoints;
- short feature-story prompts for release notes and posts;
- alt text and captions;
- version metadata so outdated visuals can be regenerated;
- privacy/branding review proving all content is fictional and redistributable.

### First public demo video: Linux + Rockwell + offline simulation

The preferred first public product video uses an Omarchy Linux workstation as
the initial showcase configuration. It demonstrates the complete OpenWebHMI
path on an open-source desktop operating system:

```text
Omarchy Linux
  -> OpenWebHMI Gateway
  -> OpenWebHMI Designer
  -> browser HMI runtime
  -> real Rockwell demo PLC
```

The same video also proves the bundled project runs without PLC hardware by
opening the explicit Offline Simulation profile. Suggested sequence:

1. launch the standard OpenWebHMI distribution on the Omarchy workstation;
2. open the bundled demo offline and show deterministic live production data;
3. show the project in the Linux Designer;
4. explicitly select the configured Rockwell Demo PLC profile;
5. show real PLC state/count/alarm/trend updates and guarded command/actual state;
6. demonstrate visible bad/stale/disconnected quality during a controlled
   disconnect, with no automatic simulation fallback;
7. explicitly return to Offline Simulation and show the project operating again.

The capture records the Omarchy version, kernel, desktop/session, OpenWebHMI
build, browser, Rockwell controller/firmware, and network topology used. Omarchy
is the first showcase configuration, not a core dependency, exclusive supported
distribution, or implied partnership. Product support remains Linux, macOS, and
Windows, with broader Linux CI/release evidence owned separately.

## Acceptance gates

- The standard product distribution bundles the demo; no network download,
  plugin installation, or external repository is required.
- First run offers `Open Demo` and `Create from template` directly.
- The same screens run against explicit Offline Simulation and Rockwell Demo PLC
  profiles without protocol-specific bindings leaking into views.
- A failed Rockwell connection remains visibly failed and never falls back to
  simulation automatically.
- Named deterministic capture scenes reproduce website, documentation, post,
  and YouTube visuals from the shipped runtime.
- The first public video proves Gateway, Designer, and runtime on the recorded
  Omarchy Linux configuration with both real Rockwell and offline profiles.
- No reference-project identifiers or proprietary assets.
- Linux, macOS, and Windows Designer creation/import tests.
- Gateway/runtime scenario tests on all supported platforms.
- Deterministic scenarios contain no sleeps or wall-clock waits.
- Tests distinguish simulation from failed real data.
- Each module expansion can be disabled while the base template still opens.
- Screens and reports expose quality/completeness instead of plausible defaults.
