# MIT to AGPL/MPL transition

OpenWebHMI changed the license for new project revisions on 2026-09-01.

- **Last MIT revision:** [`a8cbdbc`](https://github.com/sergiogallegos/OpenWebHMI/commit/a8cbdbc)
- **First AGPL/MPL revision:** `FIRST_AGPL_COMMIT` (backfilled immediately after the transition commit)
- **Product core:** `AGPL-3.0-only`
- **Rust and TypeScript wire-protocol packages:** `MPL-2.0`
- **Alternative commercial license:** none currently offered

The prior MIT license remains valid for every copy received under it. This
transition does not revoke, narrow, or retroactively replace those permissions.
Newer revisions are offered under the path-specific terms in
[`LICENSE-POLICY.md`](../LICENSE-POLICY.md).

## Why this split?

The AGPL keeps modifications to the network-served SCADA/HMI product available
under the circumstances defined in the license, including its section 13 network
interaction rule. MPL keeps the shared wire-protocol files reusable through a
narrow file-level copyleft boundary.

Both licenses permit commercial use. OpenWebHMI remains self-hostable,
air-gap-capable, modifiable, redistributable, and free of activation or
per-server fees. Compliance obligations still apply; “commercial use permitted”
does not mean “without conditions.”

## Source for a build

Official builds display their version and Git commit in their Legal/Source
surface. Corresponding source for an official commit is available at:

`https://github.com/sergiogallegos/OpenWebHMI/tree/<full-commit>`

A distributor of a modified build must provide the corresponding source for the
version it distributes or runs as required by the applicable license. Pointing
only to the upstream project is not sufficient when it does not match the
modified build.

## User projects and branding

OpenWebHMI does not claim ownership of independently created plant projects,
process data, PLC programs, or supplied graphics. Incorporating covered source
code is a separate issue governed by the software licenses.

The `OpenWebHMI™` name and logos are unregistered marks and are governed by
[`TRADEMARKS.md`](../TRADEMARKS.md). The registration symbol `®` is not used.

## Review record

The maintainer performed a good-faith project review using the canonical GNU
AGPLv3 and Mozilla MPL 2.0 texts, Mozilla's MPL FAQ, SPDX identifiers, Harmony CLA
materials, and USPTO trademark guidance. No outside lawyer reviewed this
transition because paid legal review was not affordable at launch. That fact is
disclosed to avoid implying professional legal approval. The canonical license
texts control, and recipients should obtain their own advice for specific facts.
