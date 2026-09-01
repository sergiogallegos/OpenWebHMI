# Licensing transition audit

Status: CODEX-DG audit, activated by CODEX-DH on 2026-09-01. The effective terms
are the canonical texts and path map linked from the root `LICENSE-POLICY.md`.

## Recorded direction

- Future product code: `AGPL-3.0-only`.
- Rust and TypeScript wire-protocol packages only: `MPL-2.0`.
- Separately published Rust EtherNet/IP library: remains MIT.
- No commercial license is currently offered.
- Contributions: contributor-retains-copyright CLA with broad relicensing rights
  and a promise that accepted contributions remain available under the outbound
  open-source/content license in effect when submitted.
- Documentation/media and brand marks: separate terms, finalized before CODEX-DH.

## Repository authorship evidence

The local Git history reviewed on 2026-09-01 contains 160 authored commits across
three author strings:

| Commits | Author string |
|---:|---|
| 124 | `sergiogallegos <gallegoshinojosa@gmail.com>` |
| 32 | `Sergio Gallegos <gallegoshinojosa@gmail.com>` |
| 4 | `Sergio Gallegos <sergio@LAPTOP-FFSU8F5B>` |

No `Co-authored-by` or `Signed-off-by` trailer was found in the local commit
messages. These facts support, but do not prove, common ownership. Git metadata
does not establish whether work was created for an employer/client, copied from
another source, generated under additional terms, or submitted through a hosted
pull request with different ownership.

The GitHub pull-request history queried through the repository API on 2026-09-01
returned no open, closed, or merged pull requests. No additional hosted PR author
identity therefore needs reconciliation as of the audit date.

On 2026-09-01, Sergio Gallegos attested that the OpenWebHMI code, documentation,
and brand assets authored under the three Git identities above belong to him and
are not owned by an employer, client, school, or another entity, except for
explicitly identified third-party material and dependencies. This resolves the
same-author identity and employer-ownership gate; it does not convert third-party
or insufficiently proven material into first-party work.

## Current license mechanics

- Root `LICENSE` is MIT, copyright 2023 Sergio Gallegos.
- Rust crates inherit `license = "MIT"` from the workspace, including the future
  MPL protocol crate; there is currently no per-crate exception.
- The root npm manifest is MIT. Product package manifests do not consistently
  declare an explicit license boundary.
- README, VISION, feature matrix, stack rationale, design-system planning, website,
  and brand documentation contain explicit MIT claims.
- `brand/README.md` currently grants marks under MIT and permits fork/derivative
  use. That statement requires deliberate replacement alongside a trademark policy.
- No prior CLA, CLA acceptance record, root third-party notice inventory, or
  repository-wide license map was found.

## Proposed first-party path classification

The authoritative proposed matrix is in
`docs/planning/licensing-transition.md`. Its narrow exception is:

- `crates/protocol/**` — MPL-2.0;
- `packages/protocol-ts/**` — MPL-2.0;
- all other first-party software — AGPL-3.0-only unless the finalized content or
  brand policy applies.

`crates/driver-api` remains AGPL in the proposal. Moving it or a future module SDK
to MPL would create a wider proprietary-extension boundary and needs a separate
decision rather than being inferred from the word “protocol.”

## Third-party and special-material inventory

CODEX-DH must preserve original terms and notices for:

- all Cargo and npm dependencies;
- the external MIT Rust EtherNet/IP dependency;
- fonts, icons, screenshots, logos, and generated/bundled frontend assets;
- PLC/controller examples and vendor-derived artifacts;
- copied or adapted standards material;
- code generated from schemas or external tools where the generator or input
  imposes attribution or distribution conditions.

The tracked repository contains 447 files and no Git submodules as of the audit.
The tracked binary/visual asset set is limited to Designer icons, website favicon
and social image, and the files under `brand/`. These appear to be variants of the
OpenWebHMI mark but remain subject to maintainer ownership/provenance attestation.

### Resolved Sparkplug schema finding

`crates/driver-mqtt/proto/sparkplug_b.proto` says it is a minimal vendored subset
of the Eclipse Tahu Sparkplug B schema. Its message/field names and numbers track
the upstream `sparkplug_b.proto`, but the local file omits the upstream copyright,
`SPDX-License-Identifier: EPL-2.0`, and notice. The upstream file identifies
Cirrus Link Solutions and others as copyright holders and Eclipse Public License
2.0 as its license:

<https://github.com/eclipse-tahu/tahu/blob/master/sparkplug_b/sparkplug_b.proto>

CODEX-DJ established that the file had no compiler or runtime consumer: the crate
has no `build.rs`, `prost-build`, generated include, or module reference to it.
`crates/driver-mqtt/src/sparkplug.rs` already implements the required protobuf
wire fields as first-party Rust `prost` types against the public Sparkplug
specification. CODEX-DJ deleted the unused derived source rather than retaining,
misattributing, or purporting to relicense it. MQTT build/test verification is
recorded in that task. The EPL-derived file is therefore no longer part of the
planned AGPL distribution.

The merged CODEX-Y brief still names the original file because lifecycle briefs
are immutable historical records after work starts. It is not a current product
or architecture claim; CODEX-DJ's log records the superseding disposition.

### Same-owner adapted validator

`scripts/validate-agent-files` states that it was adapted from the separately
published MIT `rust-ethernet-ip` validator. The local Git history attributes the
OpenWebHMI adaptation and the upstream project to Sergio Gallegos, but the MIT
origin and notice must remain recorded; ownership attestation still applies.

The inbound dependency rule remains permissive-only unless separately reviewed.
Licensing OpenWebHMI under AGPL does not authorize changing dependency terms or
silently introducing additional copyleft dependencies.

## Contribution policy disposition

`CONTRIBUTOR_LICENSE_AGREEMENT.md` version 1.0 launches a manual CLA flow. The PR
template records affirmative acceptance. Until automation is selected, reviewers
must confirm that the checkbox is checked and that an authorized entity acceptance
is present when an employer or other entity owns relevant rights.

The CLA is adapted from the Harmony Contributor License Agreement model. It does
not assign copyright. It grants copyright and patent rights sufficient for future
relicensing while promising continued availability under the outbound license in
effect at submission. The maintainer accepted the electronic acceptance
mechanics under the risk disposition below.

## Separate-content disposition

Finalized by CODEX-DH:

- user-facing documentation and tutorials: CC-BY-4.0;
- reproducible screenshots and general marketing/demo media: CC-BY-4.0 unless an
  asset-specific notice says otherwise;
- executable demo code, scenario logic, and first-party example software: AGPL;
- wire protocol code: MPL;
- OpenWebHMI name and official logos: copyright retained and governed by a
  separate trademark policy, not licensed for misleading fork branding;
- third-party media: its original license and attribution.

## User-content disposition

The policy states that operators retain rights in projects, tag models,
historian/process data, PLC programs, drawings, recipes, scripts, and other content
they independently create or supply. It does not assert that every plugin or
script is legally independent of the AGPL product and grants no plugin or module
SDK exception.

## Maintainer risk disposition before activation

Paid outside legal review was not affordable at launch. On 2026-09-01 the
maintainer directed activation after a second good-faith review against the
canonical GNU AGPLv3 and Mozilla MPL 2.0 texts, Mozilla's MPL FAQ, SPDX,
Harmony CLA materials, and USPTO trademark guidance. This is an accepted project
risk and does not imply professional legal approval.

- The ownership attestation and file/provenance inventory are accepted.
- The last MIT revision is `a8cbdbc`; the first transition commit is backfilled
  in `docs/license-transition.md` immediately after it is created.
- Documentation/tutorial terms are finalized in `CONTENT-LICENSE.md`; media
  requires an explicit per-asset declaration.
- Third-party screenshot/font/icon/mark rights remain excluded unless inventoried.
- `brand/README.md` is governed by the root trademark policy.
- User content is not claimed; copied software remains governed by its license.
- No plugin or module-SDK exception is granted.
