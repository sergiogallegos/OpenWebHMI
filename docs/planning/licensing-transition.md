# OpenWebHMI licensing transition plan

Status: proposed; no license change is effective until the ownership gate,
scope matrix, license texts, manifests, public notices, and release evidence land
together in the transition task.

This plan is engineering and release guidance, not legal advice. An attorney
experienced in open-source licensing should review the final boundary, notices,
contribution terms, and trademark policy before the transition release.

## Decision

OpenWebHMI will move future product development from MIT to a strong-copyleft
model without introducing a commercial license at this time:

- `AGPL-3.0-only` for the Gateway, Designer, browser runtime, component library,
  first-party drivers, service crates, first-party modules, examples, and other
  product code unless an explicit exception applies;
- `MPL-2.0` for the shared wire-protocol packages only;
- the separately published Rust EtherNet/IP library remains under its existing
  MIT license and is consumed as a third-party permissive dependency;
- no paid, proprietary, or alternative commercial license is offered as part of
  this transition.

`AGPL-3.0-only`, rather than `AGPL-3.0-or-later`, makes the selected version
explicit. AGPL remains an OSI-approved open-source license and permits commercial
use. It does not prohibit competitors from distributing, hosting, supporting, or
selling the software under its terms; it requires covered source availability in
the circumstances defined by the license, including modified network services.

Previously published MIT revisions remain available under MIT. The transition
governs only the code version released under the new license and later work.

Authoritative license references:

- GNU AGPLv3: <https://www.gnu.org/licenses/agpl-3.0.html>
- Mozilla Public License 2.0: <https://www.mozilla.org/MPL/2.0/>
- MPL 2.0 FAQ: <https://www.mozilla.org/MPL/2.0/FAQ/>
- SPDX identifiers: <https://spdx.org/licenses/>

## Proposed scope matrix

| Surface | Proposed license | Rationale |
|---|---|---|
| `crates/gateway` | AGPL-3.0-only | Network-facing product core and AGPL source-offer boundary. |
| Gateway service crates | AGPL-3.0-only | First-party product implementation: tags, history, alarms, auth, scripting, storage, audit, and backup. |
| `crates/driver-*` and `crates/driver-api` | AGPL-3.0-only | In-process first-party product/plugin surface; no plugin exception is implied. |
| `apps/designer` | AGPL-3.0-only | Distributed desktop product. |
| `apps/runtime-web` | AGPL-3.0-only | Browser product served by the Gateway. |
| `packages/component-library` | AGPL-3.0-only | First-party HMI component implementation. |
| First-party manufacturing modules | AGPL-3.0-only | Default for future product modules. |
| `crates/protocol` | MPL-2.0 | Rust wire types can be integrated without applying AGPL to a separate consumer. |
| `packages/protocol-ts` | MPL-2.0 | TypeScript wire types mirror the Rust protocol boundary. |
| Simulators, examples, and bundled demo code | AGPL-3.0-only by default | Executable first-party examples remain reproducible and open; third-party assets retain their own terms. |
| `apps/website` source | AGPL-3.0-only by default | Repository code default; public prose/media policy remains a decision gate. |
| External Rust EtherNet/IP library | Existing MIT | Separate upstream work; retain its copyright/license notices. |
| Dependencies and vendored assets | Their original licenses | Never overwrite or imply ownership of third-party work. |

The MPL exception is deliberately narrow. Moving `driver-api`, component APIs,
or a future module SDK to MPL requires a separate architecture decision because
it materially changes what proprietary extensions can link to the product.

## Decisions required before implementation

### 1. Copyright and provenance gate

- Confirm that every first-party file being relicensed is owned by the
  relicensing copyright holder or carries permission sufficient to relicense.
- Review commit authors, pull requests, copied snippets, generated code,
  commissioned/employer work, fonts, icons, PLC artifacts, and media separately.
- Exclude third-party files from the new declaration and retain their notices.
- Record the last MIT commit and the first AGPL/MPL commit in a durable transition
  notice. Do not describe the earlier MIT grant as revoked.

### 2. Contribution policy

The maintainer selected a contributor-retains-copyright CLA based on the Harmony
license model. The implemented agreement uses the broad outbound option: the
project steward may use additional licenses in the future, including commercial
terms, but every accepted contribution remains available under the outbound
license governing the relevant project material when it was submitted.

No commercial license is created or offered by this decision. The CLA and its
electronic acceptance flow require attorney review before the project relies on
them for material external contributions.

### 3. Documentation, demo media, and brand policy

Software licenses are a poor fit for prose, screenshots, and trademarks. The
maintainer accepted separate terms in principle. Counsel should confirm the
working disposition before activation:

- documentation and tutorials: `CC-BY-4.0`;
- fictional demo data and redistributable media: an explicit Creative Commons
  policy chosen per asset type;
- OpenWebHMI name and logos: a trademark policy allowing truthful reference,
  community discussion, and compatible integrations without allowing a fork to
  imply endorsement or official status.

The current `brand/README.md` grants broad MIT reuse and must not be silently
rewritten without a deliberate transition/trademark decision.

### 4. User-owned project content

The public policy should make clear that the software license is not intended to
claim ownership of plant projects, tag databases, historian data, PLC programs,
graphics supplied by an operator, or other user content. Any explicit project-file,
script, or plugin exception must be drafted and reviewed rather than improvised in
website copy.

## Repository implementation

### License layout

- Put the full AGPLv3 text at the root as the default product-code license.
- Preserve the prior MIT text in a transition/history location with the exact
  last-MIT commit reference; do not present it as governing new product code.
- Put the full MPL 2.0 text and a local license notice in both protocol package
  boundaries, or use a central `LICENSES/` directory plus unambiguous package notices.
- Use exact SPDX expressions: `AGPL-3.0-only` and `MPL-2.0`.
- Add copyright/SPDX headers or REUSE-compatible annotations where automated
  classification cannot infer the correct license from the package boundary.
- Add a third-party notices artifact to release packages.

### Manifests and validation

- Change the Rust workspace default from MIT to AGPL and override
  `crates/protocol` to MPL.
- Change product npm package manifests to AGPL and `packages/protocol-ts` to MPL.
- Classify the root aggregator and website manifest explicitly.
- Keep the inbound dependency gate permissive (MIT, Apache-2.0, BSD, ISC, and
  reviewed compatible licenses). The project's own AGPL does not justify adding
  arbitrary copyleft dependencies, especially while future relicensing remains possible.
- Extend CI to verify package SPDX values, license-file presence, protocol
  overrides, generated artifact notices, and prohibited stale MIT claims.
- Audit Rust, npm, fonts, icons, example controller files, screenshots, and other
  bundled assets; dependency licenses do not change during the transition.

### AGPL network-source offer

- Add a visible Legal/Source entry in the browser runtime and Designer.
- Show license, exact OpenWebHMI version/commit, and a durable path to the
  corresponding source for that build.
- Ensure packaged/offline releases retain the notice and corresponding-source
  instructions without depending on analytics, activation, or startup internet access.
- Document the obligation for distributors and operators of modified network
  versions to update the offer to match their corresponding source.
- Do not claim that merely linking to the upstream repository satisfies every
  modified distributor's obligations.

## Documentation and website update

The transition release updates every public claim in the same coordinated window:

- `VISION.md`, `README.md`, `AGENTS.md`, `CLAUDE.md`, contribution guidance,
  architecture, roadmap, feature matrix, stack rationale, design-system roadmap,
  release/install documentation, and applicable task briefs;
- homepage badge/comparison/positioning, About license explanation, Download
  notices, documentation index, footer, social metadata, and website README;
- badges and package metadata must show the split accurately rather than reducing
  the repository to a misleading single-license label;
- explain in plain language that OpenWebHMI remains free for commercial plant use,
  self-hosted, air-gap capable, modifiable, and free of activation/per-server fees;
- explain that core modifications and modified network deployments carry AGPL
  obligations, while the protocol packages use MPL;
- state that no commercial license is currently offered;
- avoid legal promises about project files, plugins, scripts, or proprietary
  integrations until their boundary has been reviewed.

The website must link to the exact repository license policy rather than paraphrasing
the full legal terms. Marketing copy never replaces the license text.

## Release sequence

1. Freeze the candidate transition commit and complete the provenance inventory.
2. Obtain legal review of ownership, scope matrix, contribution policy, project
   content, plugin boundary, brand policy, and notices.
3. Record the last MIT revision and publish a migration/FAQ document.
4. Land license texts, package overrides, source-offer UI, contribution policy,
   repository docs, CI enforcement, and third-party notices atomically.
5. Build all release artifacts and verify that each contains the correct notices
   and corresponding-source information.
6. Publish the website update in the same release window.
7. Tag the first AGPL/MPL release and announce the exact cutoff without implying
   that older MIT copies changed retroactively.

## Acceptance gate

The transition is complete only when:

- ownership and third-party provenance are documented;
- every first-party path resolves deterministically to AGPL, MPL, or the approved
  documentation/brand license;
- manifests, source headers/annotations, binaries, installers, source archives,
  docs, and website agree;
- the runtime and Designer expose the exact license and source for their build;
- the last-MIT and first-AGPL/MPL revisions are recorded accurately;
- dependency and artifact license checks pass in CI;
- no public statement claims that AGPL prohibits commercial use, revokes prior
  MIT grants, forces publication of unrelated plant data, or offers a commercial
  license that does not exist.
