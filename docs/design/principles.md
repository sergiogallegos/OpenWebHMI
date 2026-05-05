# OpenWebHMI — Design Principles & Standards Reference

> Working reference for the UI/UX/component-library layer. The standards we align to, the operator-workflow principles that shape component defaults, and the cross-cutting design patterns used by contemporary award-winning industrial HMIs. The companion roadmap that turns these principles into concrete deliverables is [`docs/design-system-roadmap.md`](../design-system-roadmap.md).

This is a **living reference**, not a prescription. When a new component or theme decision is being designed, this doc is the first thing to read. When a principle is amended, learned, or contradicted by experience, this doc is amended in place — not appended. Update with intent.

---

## Why this document exists

OpenWebHMI is positioning to compete with Ignition, FactoryTalk Optix, Siemens WinCC Unified, and COPA-DATA zenon on both the *technology axis* (open-source, web-native, modern stack) and the *quality axis* (modern, accessible, operator-first UX). The technology axis is delivered by the v1.0 platform. The quality axis depends on a small number of repeatable principles being followed across every component, theme decision, and screen template. This document is that principles list.

The principles below are not invented here. They are distilled from:

- The relevant **international and industry standards** (ISO, ISA, ANSI, EEMUA, WCAG)
- **Cognitive ergonomics literature** (Norman; Endsley situation-awareness; Hollifield "High Performance HMI Handbook"; the ASM Consortium's gray-scale layered design canon)
- **Modern industrial-HMI practice** as visible in the past decade of award-winning packaging, extrusion, dosing, fastening, foam-cutting, pharma-fill, baking, soldering, and operating-theatre HMIs delivered against Siemens Unified, COPA-DATA zenon, atvise, and adjacent runtimes

Where a principle has a standard behind it, the standard is named. Where a principle is practitioner consensus without a single standard, that's noted too.

---

## 1. Standards we align to

### ISO 9241-210 — Human-centred design for interactive systems
The process anchor. Mandates that every design decision is grounded in real user research, prototyped against real operator behaviour, and iterated based on usability testing. **For OpenWebHMI:** every new component proposal includes an "operator scenario" — the specific factory-floor situation it solves. No component lands in `registry.ts` without one.

### ISO 9241-110 — Dialogue principles
Seven dialogue principles every interactive system should satisfy: **suitability for the task, self-descriptiveness, conformity with user expectations, learnability, controllability, error tolerance, customisability**. **For OpenWebHMI:** these become a 7-point self-review checklist for every component PR. "Is this component error-tolerant in a realistic operator-mistake scenario?" is the question that catches the most regressions.

### ISA-101.01 — Human-Machine Interfaces for Process Automation Systems
The industrial-HMI bible. Defines:
- **HMI hierarchy levels** (1 = plant overview; 2 = process unit; 3 = process detail; 4 = diagnostic / support). Each level has a different density and information role.
- **Color philosophy.** Color is reserved for *abnormal* states. Normal-state graphics use a *gray-scale, low-saturation* palette (the ASM Consortium "high-performance HMI" tradition). When everything is colorful, alarms don't pop.
- **Symbol vocabulary** for P&ID-style schematics (tanks, pipes, valves, motors, conveyors).

**For OpenWebHMI:** the runtime ships with a "high-performance HMI" default theme that follows ISA-101 color philosophy out of the box. Components default to muted, high-contrast palettes; alarm states are the only thing that uses saturated red/yellow/orange. (See Theme defaults below.)

### ANSI/ISA-18.2 + EEMUA 191 — Alarm-management standards
Together these define the alarm philosophy every modern process plant follows: **every alarm must be actionable, every alarm needs a documented operator response, alarm rates per operator must be bounded** (EEMUA 191's target: ≤ 1 alarm per 10 minutes during steady state; ≤ 10 in the first 10 minutes of an upset). Alarm priorities (low / high / urgent) are colour- and tone-coded; no more than ~5% of alarms should ever be priority "high+".

**For OpenWebHMI:** alarm UI components (`AlarmBanner`, `AlarmTable`, future `Timeline`) default to ISA-18.2-compliant priority colors. Alarm rationalization tooling (rate metrics, flood detection) is roadmapped under the Timeline / alarm-analytics horizon.

### ISA-88 — Batch control models
Defines the procedural model for batch processes (recipes, unit procedures, operations, phases). **For OpenWebHMI:** future batch / recipe components follow ISA-88 vocabulary. Don't invent custom terminology; operators already know the standard one.

### WCAG 2.2 AA — Web Content Accessibility Guidelines
The web-accessibility floor. Most relevant SCs for industrial HMI:
- **1.4.1 Use of Color** — color cannot be the *only* state indicator (drives the color-and-form encoding work in v1.1)
- **1.4.3 Contrast (Minimum)** — 4.5:1 for body text, 3:1 for large text and UI components
- **1.4.11 Non-text Contrast** — 3:1 for UI control boundaries
- **2.1.1 Keyboard** — all functionality keyboard-reachable
- **2.5.5 Target Size** — 24×24 CSS px minimum (the v1.1 default is 44×44 to match touch and gloved-hand reality, which exceeds 2.5.5)
- **3.3.4 Error Prevention** — confirmation for destructive actions

### DIN EN 894 / IEC 60073 (peripheral but relevant)
European ergonomic standards for visual displays and the meaning of indicator colors (red = stop / fault; green = run / safe; yellow = caution; blue = mandatory action; white = informational). **For OpenWebHMI:** the default theme's status palette follows IEC 60073 semantics. OEMs can override per their brand book; the *defaults* match what European operators expect.

---

## 2. Information architecture principles

### IA-1. Match the UI topology to the plant topology
Operators navigate the HMI by mentally walking the plant. The screen hierarchy should mirror the physical layout: plant overview → unit → equipment → component → diagnostic. ISA-101's four-level hierarchy is the canonical structure. Avoid grouping by "what the developer found convenient" (e.g., separating motors and valves into their own screens rather than putting them on the unit screen they belong to).

### IA-2. Machine schematic as primary navigation
A stylized representation of the machine itself should be the primary navigation affordance on the unit-level screen. Operators tap a region to drill into that subsystem. This is what visually distinguishes "industrial HMI" from "generic web dashboard" — and what every award-winning packaging / extrusion / cutting-machine HMI of the past decade has converged on.

**Implementation note:** treat the schematic SVG as a project artifact (the OEM authors it); treat the hotspot model as a platform primitive (we ship the binding system).

### IA-3. Details on demand, not navigation away
Drill-downs into parameter detail, alarm context, or recipe steps should slide in over the current view (drawer / inspector pattern), not push to a new page. The operator stays oriented. Modal-style dialogs are reserved for *blocking* decisions (confirm a destructive action; resolve an authentication challenge) — not information disclosure.

### IA-4. Consistent screen frame across the runtime
Every screen has the same frame: status bar (top, fixed), navigation rail (side, fixed), alarm banner (top, fixed when any active alarm exists), main canvas (center). Operators learn the frame once and apply it everywhere. Components do not own the frame; the runtime layout does.

### IA-5. Breadcrumb at consistent location
Every screen below Level 1 shows a breadcrumb path back to the overview, in the same location, every time. Click any segment to navigate up.

---

## 3. Visual language principles

### VL-1. ISA-101 color philosophy: color reserved for abnormal
Normal process state uses muted gray-scale graphics with high-contrast text. Color is reserved for: alarms, mode indicators, operator-actionable items. When 80% of the screen is gray and one valve glows red, the operator's eye finds the valve instantly. When everything is colorful, nothing is.

**Theme defaults for OpenWebHMI:**
- Background: `--surface-base` (off-white in light mode, near-black in dark mode)
- Primary text: `--ink-primary` (high-contrast neutral)
- Process graphics (running state): `--ink-tertiary` / `--surface-elevated` (low-saturation gray)
- Status colors used *only* by alarm/quality/mode indicators

### VL-2. Typography: legibility under stress
Body text minimum 14 px at 1× density; numeric process values minimum 18 px (operators read them from across a console). Tabular numerals required (`font-feature-settings: "tnum"`) so digits don't dance when values change. Avoid italic for any process-relevant text.

**Default font stack:** system-UI sans-serif. Do *not* ship a brand font as the default; OEMs that need their brand font can override via the theme. Default has to load fast and render reliably on touchscreen panel hardware.

### VL-3. Density: comfortable by default; compact and distance modes available
Three density levels via theme token:
- **Compact** — control rooms with multiple panels per operator; max info density.
- **Comfortable** (default) — standard operator panels; balance density and target size.
- **Distance** — readable from several meters away; large glyphs, low information.

Density is a runtime mode; the operator selects it. Components honor it via tokens, not media queries.

### VL-4. Color and form encoding (WCAG 1.4.1)
Every state distinction uses **at least two encoding channels** — color *and* shape, or color *and* position, or color *and* icon. Color-blind operators (8% of men) and operators in low-contrast lighting still parse the screen. This is non-negotiable in any new component design.

### VL-5. Iconography: consistent, sparing, recognizable
One icon library across the runtime. Icons are functional, not decorative. Default to widely recognized industrial conventions (the IEC 60417 graphical symbols, where applicable). Custom icons per OEM project layer on top of the system library.

### VL-6. Motion: confirm causation, never decorate
Motion is reserved for confirming that an operator action caused an effect: a button press is acknowledged with a 150 ms response, a drawer slides open at 250 ms, an alarm banner appears at 400 ms with attention-grabbing easing. Motion that is *purely decorative* (parallax, particle effects, etc.) has no place here.

`prefers-reduced-motion: reduce` is honored at the runtime root: every motion token collapses to `0ms`.

---

## 4. Status, feedback, and alarm principles

### AL-1. Every alarm is actionable
Per ISA-18.2 / EEMUA 191: an alarm exists if and only if the operator can do something about it. Status indicators ("the valve is closed") are *not alarms*. Alarms cost attention; spending them on non-actionable events is alarm-flooding.

### AL-2. Alarm priority is encoded in three channels
Color (red/orange/yellow), shape (circle/triangle/square or icon variant), and audio cue. Operators can identify priority across the room without reading text.

### AL-3. Acknowledge → silence → return-to-normal as distinct states
The alarm UI represents four states: **active+unacked**, **active+acked**, **cleared+unacked** (return-to-normal but not yet acknowledged), **cleared+acked** (resolved). Each has a distinct visual treatment. The transition workflow is: alarm fires (active+unacked) → operator silences and acknowledges (active+acked) → process recovers (cleared+unacked) → operator confirms (cleared+acked, removed from list). All four states are visible on the timeline.

### AL-4. Commentable alarms for shift handover
Operators can attach a free-text note to an alarm at acknowledgement time. The next shift inherits the note. This is the single highest-return change to alarm UX: it converts alarms from "interruption notifications" to "shift-handover artifacts." (`crates/alarm-engine` already supports `note: Option<String>`; the AlarmTable component just needs to surface it.)

### AL-5. Quality indicators on every value display
Every numeric or text value display shows quality state: good (no marker), uncertain (small marker), bad (struck-through value, distinct background). Quality is *always* visible — operators must never trust a stale value because the UI happily showed it.

### AL-6. Microinteractions for action confirmation
Touchscreen panels lack the tactile click of a hardware button. Replace it with a 150 ms visual response on tap: button color shift, ripple, or scale change. The response confirms the *touch was registered*; the longer-running effect (write-to-PLC, navigate, etc.) is confirmed separately when complete.

### AL-7. Setpoint vs process value visual differentiation
When a numeric input shows a setpoint and the actual reading side-by-side, they are visually distinct (color, position, label). Operators routinely confuse them in poorly designed HMIs and write the wrong value.

---

## 5. Accessibility & ergonomics

### EA-1. Touch targets ≥ 44 × 44 CSS px (gloved hands)
Industrial operators frequently wear gloves; the bare-hand 44 px Apple HIG minimum is inadequate. Default to 48 × 48; key controls (e-stop equivalents, mode select) get 60+. WCAG 2.2 SC 2.5.5 mandates 24 px floor; we exceed it.

### EA-2. No multi-touch gestures for primary controls
Pinch-zoom, two-finger rotate, swipe-back are unreliable with gloves. Primary controls (start, stop, mode select, alarm acknowledge) must be tap-only. Pan/zoom on schematics is fine because the alternative is a fixed view; just provide buttons too.

### EA-3. Keyboard navigation is mandatory
Every interactive component is reachable and operable via keyboard. WCAG 2.1.1. Industrial workstations frequently have keyboards mounted next to touchscreens; operators alternate.

### EA-4. Focus visible at 3:1 contrast minimum
Every focusable element shows a visible focus ring at WCAG 2.2 SC 1.4.11 contrast (3:1 against the adjacent surface). Browser-default ring is too subtle; use a 2 px solid ring at the primary brand color.

### EA-5. Contrast: 4.5:1 body, 3:1 UI components, 7:1 in distance mode
Distance mode (VL-3) raises body-text contrast to 7:1 (WCAG AAA) automatically. This is an opt-in mode, not a global default — but the support has to be there.

### EA-6. Screen-reader labels on every interactive component
Industrial HMIs are not commonly used by visually impaired operators in production, but: (a) supervisor / engineering tooling on the same component library *is* used by people with vision differences; (b) screen-reader labels double as automation hooks for end-to-end testing. There is no operational reason to skip this work.

### EA-7. Reduced-motion compliance is a hard requirement
`prefers-reduced-motion` is honored without exception. Some operators have vestibular disorders; some panels run at low refresh rates where motion blurs. Motion tokens collapse to `0` at the runtime root when the media query is set.

---

## 6. Operator workflow principles

### OW-1. Operator role determines visible surface
The same project, viewed by an Operator, Supervisor, Engineer, and Administrator, shows progressively more controls. Roles gate visibility, not just write-permission. An Operator who can't see the engineering surface can't be confused by it.

### OW-2. Mode awareness is always on screen
Run / stop / hold / fault / maintenance — whichever mode-set applies to the equipment, the current mode is shown in the same location on every screen, in the same shape and color. Operators never have to drill in to find out what state the machine is in.

### OW-3. Cognitive load: 7 ± 2 items per attentional surface
George Miller's bound, applied: an attentional surface (the screen, an inspector panel, a modal) has at most 7 ± 2 distinct interactive elements competing for the operator's attention at once. Information density itself can be higher (a trend can show 10 pens), but interactive controls are the gated count.

### OW-4. Primary path: 0 documentation reads
The primary operator path (start the machine, monitor it, acknowledge alarms, change a setpoint) must be discoverable without reading documentation. Self-explanatory illustrations, microinteractions, and consistent affordances do this work. If new operators consistently need to be shown the workflow, the workflow is broken.

### OW-5. Confirm destructive, don't confirm reversible
Stopping production, deleting a recipe, force-overriding a safety interlock: confirm. Acknowledging an alarm, changing a comfortable-setpoint, navigating between screens: never confirm. Confirmation fatigue makes operators tap-through dangerous prompts.

### OW-6. Errors recover gracefully, never lose work
A network drop during a setpoint write should: (a) preserve the operator's entered value; (b) show a clear retry affordance; (c) not blame the operator. Industrial reality: networks drop, PLCs reset, gateways restart. The UI degrades visibly, not catastrophically.

### OW-7. Shift handover lives in the UI
Alarms accumulate notes (AL-4). The current shift's actions are visible on a timeline (the v1.2 Timeline component). The operator coming on shift can answer "what happened in the last 8 hours?" from the runtime alone. Shift logs are not a separate spreadsheet.

---

## 7. Theming & branding

### TB-1. Tokens, not constants
Every design value (color, spacing, typography size, motion duration, density factor, status palette) lives in the theme-token layer. Components read from tokens. No component-internal constants for design values. This is the contract that makes OEM theming a workflow rather than a fork.

### TB-2. Theme tokens follow Style Dictionary structure
The theme file format aligns with [Style Dictionary](https://amzn.github.io/style-dictionary/) JSON conventions: `color.background.base`, `motion.duration.fast`, etc. Designers use familiar tooling; the runtime imports tokens directly.

### TB-3. Default theme is opinionated, ISA-101-aligned, AAA-capable
Out of the box, the runtime ships a single high-quality default theme that follows ISA-101 color philosophy, exceeds WCAG AA on all defaults, and supports light + dark + distance modes. OEMs override on top; new users don't have to design a theme to get a credible runtime.

### TB-4. Brand-book import is a one-step workflow
An OEM ships `tokens.json`. The designer imports it. The runtime re-skins. No code changes, no rebuild. This is the workflow industrial-design studios deliver against today; we match it.

### TB-5. Component appearance is theme-driven, behavior is component-driven
Tokens never reach into component logic. A motion token controls *duration*, never *whether motion happens*. A color token controls *which red is used for alarms*, never *whether alarms are red*. The boundary keeps themes interchangeable.

---

## 8. Documentation & governance

### DG-1. Atomic Design as the organizing structure
Components are organized: **atoms** (Button, Spinner, Icon, Badge), **molecules** (NumericInput-with-label, Slider-with-readback, AlarmRow), **organisms** (AlarmTable, Trend, DataGrid, MachineSchematic), **templates** (DashboardLayout, MachineDetailLayout), **pages** (project-authored, not platform-shipped). The taxonomy clarifies where new contributions belong.

### DG-2. Every component has a Storybook story
The Storybook story is the contract. It documents the component's props, states, and intended use. New PRs without a story do not land. The Storybook deployment is the public face of the design system.

### DG-3. Every component has an "operator scenario" in its docs
Per ISO 9241-210: the doc page for each component opens with a one-paragraph scenario describing the factory-floor situation it solves. Removes ambiguity about intent.

### DG-4. Bundle-size budget per phase
The component library is the single hottest performance gate of the runtime. Bundle-size budgets are stated per roadmap phase. New components count against the phase budget at PR time. (See `docs/design-system-roadmap.md`'s sequencing rules.)

### DG-5. No silent breaking changes
Component prop renames, default-value changes, and behavior changes are SemVer-versioned. The component-library publishes its own version line independent of the gateway crate's version.

### DG-6. Public design-system docs site
The Storybook deployment is published to a public URL (target: `design.openwebhmi.org`). OEMs evaluating the platform browse it before committing. This is what every mature industrial-platform design system does.

---

## 9. Differentiation matrix — where the ROI is

Not every principle here drives equal differentiation against Ignition / Optix / WinCC / zenon. Ranked by the impact-per-effort estimate:

| Rank | Principle / pattern | Why it differentiates | Status today | Roadmap item |
|---|---|---|---|---|
| 1 | **Machine schematic as primary navigation (IA-2)** | Single highest-impact missing primitive; converts "web app" → "industrial HMI" on first sight | Not implemented | v2.0 #11 |
| 2 | **ISA-101 color philosophy default theme (VL-1)** | Out-of-box theme is more credible than commercial defaults | Partial (theme tokens exist; not ISA-101-aligned) | v1.1 #4, v1.2 #7 |
| 3 | **Color and form encoding (VL-4 / EA accessibility)** | WCAG 1.4.1 compliance + serves real ageing operator workforce | Color-only today | v1.1 #4 |
| 4 | **Distance-view mode (VL-3 / IA hierarchy Level 1)** | Matches a real walking-up-to-the-panel workflow no general framework offers | Not implemented | v1.2 #9 |
| 5 | **Commentable alarms (AL-4)** | Tiny effort, large operator-workflow win; backend already supports it | UI not surfaced | v1.1 #2 |
| 6 | **Style Dictionary brand-book import (TB-4)** | Turns "OEM theming" from "fork the codebase" into a workflow | Project-local theme only | v1.2 #7 |
| 7 | **Public Storybook docs site (DG-6)** | Credibility for OEM purchasing committees | Not deployed | v1.1 #6 |
| 8 | **Drawer / inspector pattern (IA-3)** | Unlocks "details on demand" navigation across every existing component | Modal-only | v1.2 #8 |
| 9 | **Motion tokens + reduced-motion (VL-6 / EA-7)** | Coherent feel + accessibility floor | Ad-hoc per component | v1.1 #5 |
| 10 | **Symbol package (P&ID-aligned, ISA-101 vocabulary)** | A P&ID-style overview is buildable in a day | No symbols ship | v2.0 #12 |
| 11 | **Event timeline component (AL-3 / OW-7)** | Shift-handover + alarm-flood analysis surface | No time-axis component | v1.2 #10 |
| 12 | **Atomic Design re-export structure (DG-1)** | Cosmetic, but signals professional design-system practice | Flat export list | v1.1 #1 |

Items 1–4 are the **load-bearing differentiators**; items 5–9 are **credibility-building**; items 10–12 are **polish that signals taste**. None of them require the platform to be re-architected; all of them slot into the existing component-library, theme-token, and view-schema layers.

---

## 10. What this document is not

- **Not a style guide.** A style guide names specific colors, fonts, and spacing. This document names the *principles* that shape those choices. The style guide is the default theme's `tokens.json` and the Storybook docs site (both v1.1 deliverables).
- **Not a component spec.** Component-level specs live in each component's Storybook MDX page. This document is the *cross-cutting* layer those specs all conform to.
- **Not a research paper.** Where literature exists (Norman, Endsley, Hollifield, the ASM Consortium handbook, the Tognazzini and Nielsen heuristics), it informs principles here without citation overhead. Practitioners can find the references in any industrial-HMI textbook.
- **Not OEM-specific.** OEM brand books layer on top of this baseline. Where a brand book contradicts a principle here, the brand book wins for that OEM's deployment — but the principle is what the platform default obeys.

---

## 11. Further reading (uncited but informing)

- **ISA-101.01-2015** — *Human Machine Interfaces for Process Automation Systems* (the standard)
- **ISA-18.2-2016** — *Management of Alarm Systems for the Process Industries* (the alarm standard)
- **EEMUA 191** — *Alarm Systems: A Guide to Design, Management and Procurement* (the practical alarm bible)
- **ISO 9241-210:2019** — *Human-centred design for interactive systems*
- **ISO 9241-110:2020** — *Dialogue principles*
- **ANSI/ISA-88.01** — *Batch Control Part 1: Models and Terminology*
- **WCAG 2.2** (W3C Recommendation, October 2023) — web accessibility floor
- **NUREG-0700 Rev. 3** (US NRC) — *Human-System Interface Design Review Guidelines* (nuclear-grade, broadly applicable)
- **High Performance HMI Handbook** — Hollifield, Oliver, Nimmo, Habibi (2008)
- **Designing for Situation Awareness** — Mica Endsley (2003)
- **The Design of Everyday Things** — Donald Norman (revised 2013)
- **The Visual Display of Quantitative Information** — Edward Tufte (2001) — for trend / data-density work
- **ASM Consortium Effective Operator Display Design Guidelines** — gray-scale, layered HMI design canon
