# OpenWebHMI — Design-System Roadmap

> Post-1.0 roadmap for the **UI/UX/component-library** layer. Sibling to [`roadmap.md`](roadmap.md), which covers the v1.0 platform itself. This document captures the work that turns OpenWebHMI from "a working open-source SCADA platform" into "the open-source SCADA platform with the best out-of-the-box UI." None of the items here block v1.0; all are post-tag.

The framing — and most of the specific patterns — is derived from established industrial-HMI design practice in process automation: award-winning packaging, extrusion, dosing, pharma-fill, and operating-theatre HMIs that have shipped over the past decade against Siemens Unified, COPA-DATA zenon, and similar runtimes. The patterns repeat across that body of work; this roadmap captures them. The relevant standards anchors are catalogued in [`docs/design/principles.md`](design/principles.md).

This is a **plan, not a contract.** Order reflects what unlocks the next item, not perceived completion-shape. Items are sized in T-shirt units (XS = ~½ day, S = 1–2 days, M = ~1 week, L = 2–3 weeks, XL = month-plus) for relative ordering, not commitments.

---

## Strategic framing

OpenWebHMI's v1.0 positioning is on the **technology axis**: an open-source, web-native, Rust-gateway alternative to Ignition / FactoryTalk Optix. The v1.0 component library (25 components after CODEX-AC) is a competitive baseline but not yet a differentiator.

The complementary positioning we don't yet occupy is the **quality axis**: "modern, web-native, industrial-grade UX out of the box, not a 1995 control-room aesthetic." OEMs pay industrial-design studios real money for exactly that quality on top of Siemens Unified and zenon today. Closing the gap requires roughly twelve specific items, sequenced below.

**What this roadmap is not.** It is not an attempt to compete with a design studio's hand-crafted, OEM-specific delivery. It is the platform substrate that lets an in-house team or a contracted studio deliver against OpenWebHMI as efficiently as they currently deliver against Siemens Unified or COPA-DATA zenon. The studio is the customer. We provide the rails.

---

## v1.1 — Foundations and credibility (target: ~6 weeks after 1.0)

**Theme: ship the design-system surface area that an OEM purchasing committee expects.** None of these items individually shifts the product's center of gravity; together they signal "this is a real platform, not a hobby project."

### 1. Atomic-Design re-export structure for `packages/component-library` — XS

The component-library currently exports 25 components as a flat list (`packages/component-library/src/index.ts`, `registry.ts`, `components/`). Atomic Design (atoms → molecules → organisms → templates → pages) is the most-cited organizing methodology across modern industrial UI work — adopting the same structure communicates the methodology without changing any component's behavior.

**What changes:** add `atoms/`, `molecules/`, `organisms/`, `templates/` re-export barrels. Existing imports continue to work via the flat surface. Documentation references the new taxonomy.

**Why it matters:** clarifies the contribution path for community-built components ("where does my new gauge belong?") and aligns vocabulary with the methodology most frontend teams already know.

**Acceptance:** `packages/component-library/src/atoms/index.ts` etc. exist and re-export the matching components; existing flat exports unchanged for back-compat; one paragraph in `packages/component-library/README.md` explains the taxonomy.

### 2. Commentable alarms in `AlarmTable` — XS

`crates/alarm-engine` already supports `note: Option<String>` on the ack path (CODEX-Q). The `AlarmTable` component does not surface notes as an editable column. Award-winning HMIs in the cutting / packaging space have made commentable alarms a recognized pattern: alarms become shift-handover artifacts that inherit context from the previous operator, not just notifications. This is a small UI change against an existing crate.

**What changes:** `AlarmTable` gains a notes column (read-only by default; editable when the user has the `Operator` role). The ack flow opens an inline editor; the note round-trips through the existing `system.alarm.ack` path.

**Why it matters:** turns alarms from "notifications" into "shift-handover artifacts" — directly matching the operator workflow that the high-end industrial UX studios productize.

**Acceptance:** ack flow accepts a note; the note persists across reconnect; the column hides when the bound view's `permissions` exclude operators.

### 3. `docs/design/hci-principles.md` — XS

ISO 9241-210 (Human-Centered Design Process) and ISA-101.01 (HMIs for process automation) are the standards every serious industrial-UX practitioner cites. EEMUA 191 / ISA-18.2 are the alarm-management equivalents. We have no documented HCD posture. A single page that names which standards we align to, what accessibility commitments the runtime makes, and how operators-under-stress shape component defaults gives the platform credibility on a dimension OEM buyers check.

**What changes:** new file at `docs/design/hci-principles.md` (~400 words; the public-facing condensation of [`docs/design/principles.md`](design/principles.md), the working reference). References ISO 9241-210, ISA-101.01, EEMUA 191 / ISA-18.2, WCAG 2.2 AA. Linked from `README.md` and `wiki/index.md`.

**Why it matters:** OEM purchasing committees will ask. Having a one-page answer beats not having one.

**Acceptance:** doc exists; cross-linked from README.

### 4. Color-and-form encoding at theme-token layer — S

An award-winning packaging-machine HMI introduced a "color and form guide system" that encodes machine-parameter identity in **both color and shape** so operators with poor color vision still parse the screen. Industrial workforces skew older; gloves, eye fatigue, low-contrast lighting all compound. Today our components encode state (good/bad/uncertain quality, alarm severity, run/stop) in color alone — a WCAG 2.2 SC 1.4.1 ("Use of Color") miss.

**What changes:** add a `--state-shape-*` family to the theme tokens (e.g., `--state-shape-good: circle`, `--state-shape-bad: triangle`, `--state-shape-uncertain: square`). Components that render state indicators (`Trend` quality marks, `AlarmBanner`, `ValueDisplay` quality badge, `MultiState`) gain a small icon overlay driven by the shape token. A theme flag toggles shape-encoding on/off.

**Why it matters:** WCAG 2.2 SC 1.4.1 ("Use of Color") compliance for free, plus a visible accessibility commitment we can put on the marketing page.

**Acceptance:** four affected components honor the shape tokens; `colorblind-encoding.test.tsx` asserts that two distinguishable states render distinguishable shapes when run through a deuteranopia color-matrix simulation; toggle works in the designer's theme editor.

### 5. Motion tokens + `prefers-reduced-motion` — S

Animations across the 25 components are written ad-hoc. Microinteractions — subtle animations that confirm user actions in real time — are a recognized industrial-HMI feedback principle, especially for touchscreen-only operator panels where the tactile click of a physical button is absent. Lifting motion to the theme system gives OEM-themable timing and a coherent feel for free, plus baseline reduced-motion compliance.

**What changes:** introduce `--motion-duration-fast`, `--motion-duration-base`, `--motion-duration-emphasis`, `--motion-easing-standard`, `--motion-easing-enter`, `--motion-easing-exit` tokens. Sweep all 25 components to use the tokens. Honor `@media (prefers-reduced-motion: reduce)` by collapsing all motion tokens to `0ms` at the runtime root.

**Why it matters:** consistency, OEM customization, and reduced-motion accessibility — all from a single token addition.

**Acceptance:** no component contains hardcoded `transition: ... <ms>`; reduced-motion media query collapses motion across the library; one storybook page documents the tokens.

### 6. Storybook docs site at `design.openwebhmi.org` — M

Mature industrial-platform design systems publish a browseable, public docs site (the IIoT-platform reference systems shipped by every major vendor follow this pattern). We have no equivalent. Storybook + MDX is the standard stack; the existing `packages/component-library` already has unit tests, so component-by-component stories are mostly mechanical.

**What changes:** add Storybook to `packages/component-library` (`pnpm dlx storybook init`), one MDX story per component, GitHub Pages or Cloudflare Pages deploy from `main`. Stories include a token-reference page, a color-and-form encoding demo (item #4), and a motion-tokens demo (item #5).

**Why it matters:** credibility (an OEM can browse the components before committing), self-documentation (we don't have to maintain the docs separately from the code), and a demo surface for the patterns introduced in v1.1.

**Acceptance:** deployed site at `design.openwebhmi.org` (or subdomain on the existing site); CI publishes on push to `main`; every component in `registry.ts` has at least one story.

---

## v1.2 — Industrial-HMI differentiation (target: ~3 months after 1.0)

**Theme: ship the patterns the contemporary industrial-HMI canon repeatedly uses that we currently lack.** These items shift the runtime from "competent web app" toward "operators recognize this as an HMI."

### 7. Style Dictionary-format theme import in the designer — M

Today the designer has a Theme system but no published "import a brand book → re-skin the runtime" path. Industrial-design studios productize this exact workflow as a service line — they hand the OEM a "shared construction kit" that strengthens the OEM's brand. [Style Dictionary](https://amzn.github.io/style-dictionary/) is the de-facto JSON token format; supporting it lets OEMs ship a single tokens.json and have OpenWebHMI consume it.

**What changes:** designer accepts a `tokens.json` file matching Style Dictionary's schema; tokens are written into the project's existing theme system. A reference brand book ships in `examples/branding/acme-tokens.json` so contributors can see the format. Runtime maps tokens to CSS variables on mount.

**Why it matters:** turns "OEM branding" into a workflow with a well-known schema rather than a custom format. Aligns with how design studios already deliver to clients.

**Acceptance:** `pnpm test` covers token import → CSS-var emission round-trip; an example brand applied via the designer visibly re-skins the runtime; documentation references the Style Dictionary spec.

### 8. `Drawer` / `Inspector` panel component — S

An iF-Award-winning extrusion-system HMI established the "details on demand" navigation principle — extensive functions accessible without modal-pushing or page transitions. Our Modal + Tabs default to navigation away from context. A `Drawer` slides in from an edge, dismissible, doesn't obscure the underlying view; perfect for parameter-tuning, alarm-detail, or recipe-step inspection while the operator stays oriented to the machine view.

**What changes:** new `Drawer` component in `packages/component-library`. Props: `side` (left/right/top/bottom), `size`, `open`, `onClose`, `modal` (false by default — non-modal so the underlying view stays interactive). Accessible: focus trap optional, ESC-to-close, scrim optional.

**Why it matters:** unlocks the "details on demand" pattern across every existing organism (AlarmTable row → Drawer; Trend pen click → Drawer; DataGrid row → Drawer).

**Acceptance:** Storybook page demonstrates non-modal Drawer with a live-updating Trend behind it; keyboard accessibility verified; bundle size delta < 4 KB gzipped.

### 9. Distance-view mode — M

A recognized industrial-HMI mode lets operators "evaluate system status from several meters away" — a deliberate low-information / high-contrast / large-glyph mode the operator can switch into. ISA-101 calls this the **overview level** of the HMI hierarchy. This isn't responsive design (which targets device size); it's intent-based density. We have nothing equivalent.

**What changes:** add a `viewMode` runtime context with values `operate` (default) and `distance`. Components honor it: `Trend` thickens pens and increases axis-label size; `DataGrid` collapses to a few KPI tiles; `AlarmBanner` fills more of the screen and switches to a reduced palette; typography scales via a `--font-scale-distance` token. A toolbar control toggles mode at the runtime root.

**Why it matters:** matches a real operator workflow (walking up to / past a panel) that high-end industrial HMIs explicitly design for and we currently cannot. Differentiates against generic web frameworks.

**Acceptance:** every component in `registry.ts` either honors `viewMode` or explicitly opts out via documented exception; toggling at runtime takes effect without a re-render flash; storybook story for each affected component.

### 10. `Timeline` component — M

Modern industrial-HMI practice surfaces alarms, maintenance events, and batch transitions on an event-based timeline with calendar + table views. We have AlarmTable but no time-axis presentation. Time is the natural axis for shift-handover, post-incident review, and predictive-maintenance workflows — and it's load-bearing for the alarm-flood analysis EEMUA 191 / ISA-18.2 expect.

**What changes:** new `Timeline` component. Renders events on a horizontal time axis (zoomable: hour / shift / day / week). Calendar view shows the same data as a month grid. Filter + sort surface mirrors AlarmTable's. Bound to a generic `events` source — first-party bindings target the alarm journal and the audit log; user data sources work via the existing tag-binding model.

**Why it matters:** post-incident review and predictive maintenance are both pull-asks from operators today; the Timeline is the visual surface for both.

**Acceptance:** virtualized rendering handles 100k events without frame drop on a mid-tier laptop; calendar/table view toggle; one binding to the alarm journal, one to the audit log; designer property panel covers axis range and filter config.

---

## v2.0 — Headline differentiation (target: year 2)

**Theme: the patterns that make OpenWebHMI visually distinct from any general-purpose web app.** These are the items most likely to be cited in marketing — and the most expensive to do well. Both are intentionally large; they cannot ship as a side-effect of any other work.

### 11. `MachineSchematic` component + hotspot binding — L

The single highest-impact missing primitive. Award-winning HMIs across responsive packaging, foam-cutting, and extrusion machinery all use a stylized representation of the machine itself as the **primary navigation affordance** — operators tap a part of the diagram to drill into that subsystem. This is what visually separates "industrial HMI" from "web app." We have nothing equivalent.

**What changes:** new `MachineSchematic` component renders an SVG canvas with bound hotspots. Each hotspot maps to a region of the SVG and supports two binding shapes:
- **Tag-write hotspot:** clicking writes a value (start/stop a subsystem, mode select, setpoint).
- **Navigation hotspot:** clicking pushes a view (drill-down to subsystem detail).
The SVG asset is project-authored; the hotspot model is OpenWebHMI's. Designer mode renders hotspot rectangles for editing; runtime renders them invisible. Hotspots animate per bound state (pulse on alarm, color tint on running, etc.) via the existing tag-binding system.

**Why it matters:** converts the runtime from "web app rendering tags" into "industrial HMI rendering this machine." First-screen recognition for any operator. This single component is the load-bearing visual differentiator of v2.0.

**Acceptance:** a demo project ships with a stylized SVG of `examples/sim-rockwell` (mock conveyor / fill station / discharge), with hotspots wired to the existing tags; clicking the conveyor hotspot navigates to a Trend view; hotspot animation reflects tag state with no visible jank at 1 Hz update rate; designer property panel covers hotspot CRUD.

### 12. `@openwebhmi/symbols` — ISA-101 aligned industrial-symbol package — L

Closely tied to item #11 but separable. The high-end industrial-design studios build custom illustration systems per OEM ("self-explanatory illustrations and animations make highly complex technology understandable") and we don't compete with that bespoke layer. We can compete by shipping a high-quality default symbol set — tanks, pipes, valves, motors, conveyors, pumps — that animates with process state. ISA-101.01 defines a recommended industrial-symbol vocabulary; several open-license SVG libraries already implement it (P&ID-style symbols).

**What changes:** new `packages/symbols` workspace package. Each symbol is an SVG component with state-driven animation hooks (e.g., `<ValveSymbol open={...} flow={...} />`). Distributed separately from `component-library` so projects opt in. Documentation pages render every symbol with its bindable state matrix.

**Why it matters:** without this, every OpenWebHMI deployment authors its own symbol library. With it, a P&ID-style overview view is buildable in a day.

**Acceptance:** at least 30 symbols (reduced ISA-101 set); symbol-by-symbol Storybook docs; `examples/sim-rockwell` ships an example overview view that uses 5+ symbols; bundle size < 50 KB gzipped for the typical 10-symbol-per-view usage.

---

## Explicit non-goals

These come up in any "improve the UI" conversation. They are not on this roadmap, by design:

- **3D visualization.** Some award-winning HMIs (cutting machines, robotic cells) bundle real-time 3D process previews. That's an OEM-specific deliverable, not a platform primitive. We support it via `<iframe>` or component plugin, not as a core component.
- **Hand-crafted illustrations per OEM.** A studio's actual visual judgment comes from years of project portfolio. We don't compete with that taste; we ship the rails so a designer can deliver against us. The "one default theme that is opinionated and credible" emerges from the v1.1 work.
- **VR/AR runtime.** Spatial-computing HMI demonstrations exist on the marketing edge of the industry but are out of scope for the platform.
- **Industrial-design (physical hardware) integration.** Studios also design panels, enclosures, and consoles. We're software.
- **AI-assisted design tooling.** Design-system "AI readiness" (context-aware patterns, intelligent analytics, Figma plugins) is independently valuable but not part of this roadmap — it lives under `roadmap.md`'s "Phase 5+ — AI assistance" horizon.

---

## Sequencing rules

These hold across the items above:

1. **No item lands without ≥1 reference deployment** in `examples/`. The example is the proof. v1.1 items use `examples/sim-rockwell-project`; v1.2+ items get bespoke examples as needed.
2. **Bundle-size discipline.** The component library is the single hottest performance gate of the runtime. Every item lists a bundle-size budget in its acceptance criteria. v1.1 cumulative budget: +20 KB gzipped. v1.2 cumulative: +60 KB. v2.0 cumulative: +200 KB (the symbol package and SVG canvas are inherently larger).
3. **Theming first; component features second.** Every item that would naturally hardcode a value (color, motion duration, font size, density) goes through the theme-token layer before shipping. No component-internal constants for design values.
4. **Storybook before Codex brief.** From v1.1 #6 onward, every new component or token requires a Storybook story as part of the same PR. The story is the contract; the docs site is the deliverable.
5. **Don't sweep all components for every change.** Items that touch many components (#4, #5, #9) explicitly audit-and-apply rather than rewrite-everything; component opt-out is allowed when it's documented.

---

## Open questions

Items below are not blockers; they are decisions to settle as the roadmap reaches them.

- **Storybook deployment target.** GitHub Pages is free and lives in-repo; Cloudflare Pages has better MDX support and faster cold-start. v1.1 #6 picks one when it lands.
- **Style Dictionary version pin.** v4 is current; v3 has wider tooling support. v1.2 #7 picks based on what designer-side tooling actually exists at that point.
- **Symbol package licensing.** Several ISA-101-aligned symbol libraries exist under various licenses (CC-BY-SA, MIT, Apache-2). v2.0 #12 picks only after compatibility review against the AGPL product boundary and the separate content/marks policy, not on aesthetics alone.
- **MachineSchematic SVG authoring tool.** The schematic itself is project-authored. We can either rely on Inkscape / Figma export + a custom editor in the designer, or ship a minimal in-designer SVG editor. v2.0 #11 punts the choice to the spike phase.
- **Distance-view trigger.** Toolbar toggle is the obvious answer; ambient-light sensor (where available) and proximity sensor (where available) are also options. v1.2 #9 ships toolbar-toggle; sensor-based switching is post-v2.0.

---

## How this composes with `roadmap.md`

The phase-based plan in [`roadmap.md`](roadmap.md) is the v1.0 ladder; this document is the post-1.0 design layer. The two compose cleanly:

- **`roadmap.md` Phase 5+** sketches "Mobile / responsive runtime" and "AI assistance" as horizons. v1.2 #9 (distance view) and v2.0 #11 (MachineSchematic) are the concrete work that fleshes out the responsive horizon. The AI-assistance horizon is intentionally separate.
- **CODEX-AJ / -AK / -AL / -AM** (currently in flight or queued) are Rust-side modernization tasks that don't touch this layer at all. The two tracks run in parallel.
- **The pre-1.0 hardware-validation gate** in `roadmap.md` is unaffected by anything in this roadmap. None of these items become 1.0-blockers if scope tightens.

---

## Standards anchors

The patterns above are grounded in the standards canonical to industrial-HMI design. Detailed working notes — what each standard mandates, where it touches our component library, and the broader principles distilled from contemporary award-winning HMI work — live in [`docs/design/principles.md`](design/principles.md). The headline references:

- **ISO 9241-210** — Human-centred design for interactive systems (process anchor for items #3, #5, #6)
- **ISO 9241-110** — Dialogue principles (suitability for the task, conformity with user expectations, error tolerance)
- **ISA-101.01** — Human-machine interfaces for process automation systems (hierarchy levels, color philosophy; anchor for items #9, #11, #12)
- **ANSI/ISA-18.2 / EEMUA 191** — Alarm-management standards (anchor for items #2, #10)
- **ISA-88** — Batch control models (informs future recipe / batch components)
- **WCAG 2.2 AA** — Web accessibility (anchor for items #4, #5)
