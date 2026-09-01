# OpenWebHMI license policy

Effective beginning with the first transition commit identified in
[`docs/license-transition.md`](docs/license-transition.md).

This policy explains which canonical license applies to each first-party part of
the repository. The license texts control if this summary conflicts with them.
It is general project information, not individualized legal advice.

## Software

| Repository surface | License |
|---|---|
| Gateway, supporting Rust crates, drivers, driver API, Designer, browser runtime, component library, simulators, examples, first-party modules, scripts, build tooling, and website source | [`AGPL-3.0-only`](LICENSES/AGPL-3.0-only.txt) |
| `crates/protocol/**` | [`MPL-2.0`](LICENSES/MPL-2.0.txt) |
| `packages/protocol-ts/**` | [`MPL-2.0`](LICENSES/MPL-2.0.txt) |
| Third-party dependencies or materials | Their original licenses and notices |

The root [`LICENSE`](LICENSE) is the unmodified GNU AGPLv3 text and is the
default for first-party software when no narrower rule above applies. The two
protocol directories contain the unmodified MPL 2.0 text and package metadata
declaring `MPL-2.0`.

The separately published `rust-ethernet-ip` dependency remains MIT-licensed. Its
license is not changed by OpenWebHMI's license choice.

## Documentation, media, marks, and user content

- First-party documentation and tutorials are available under CC BY 4.0 as
  described in [`CONTENT-LICENSE.md`](CONTENT-LICENSE.md).
- OpenWebHMI names, logos, and brand assets are governed by
  [`TRADEMARKS.md`](TRADEMARKS.md), not by the software licenses.
- OpenWebHMI does not claim ownership of an operator's plant project, tag
  database, historian/process data, PLC program, drawing, graphic, or other
  independently supplied content. Copying OpenWebHMI software into those items
  can still create obligations under the applicable software license.
- Third-party material retains its original terms even when stored beside
  first-party material.

## Practical points

- Commercial use, paid services, internal plant use, modification, and
  redistribution are not prohibited. Users and distributors must comply with
  the applicable license.
- AGPL section 13 applies when users interact over a network with a modified
  covered version. The canonical AGPL text, not this summary, defines the duty.
- MPL is file-level copyleft. Modified MPL-covered files remain MPL-covered when
  distributed; the canonical MPL text defines the exact requirements.
- No alternative commercial OpenWebHMI license is currently offered.
- Releases at or before the last MIT revision remain available under their
  existing MIT grant. That grant is not revoked.

Questions may be opened at
<https://github.com/sergiogallegos/OpenWebHMI/issues>. Questions about a specific
deployment or legal obligation require the recipient's own qualified adviser.
