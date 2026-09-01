# OpenWebHMI brand assets

Logo marks and favicons for the project.

## Files

| File | Use |
|---|---|
| [`openwebhmi-mark-blue.svg`](openwebhmi-mark-blue.svg) | Primary mark on light backgrounds (web pages, README). Color: `#0066ff`. |
| [`openwebhmi-mark-black.svg`](openwebhmi-mark-black.svg) | Primary mark on light backgrounds when monochrome is required (printed docs, watermarks). Color: `#1d1d1f`. |
| [`openwebhmi-mark-white.svg`](openwebhmi-mark-white.svg) | Primary mark on dark backgrounds. Used by the README's `prefers-color-scheme: dark` source. |
| [`openwebhmi-favicon-blue.svg`](openwebhmi-favicon-blue.svg) | Favicon for `apps/runtime-web` and the future docs site. Slightly thicker strokes and a square center for crispness at small sizes. |
| [`openwebhmi-favicon-black.svg`](openwebhmi-favicon-black.svg) | Favicon, monochrome variant. |

## Design intent

- **Concept**: two opposing brackets framing a centered glyph — "the platform that frames live plant data".
- **Bracket strokes**: 4px (mark) / 6px (favicon), butt caps, mitered joins, stroke-aligned for crisp rendering.
- **Center glyph**: a circle in the mark; a square in the favicon (the favicon prioritizes pixel snap at 16/32/48px sizes).
- **Primary color**: `#0066ff`. Background-aware via `<picture>` + `prefers-color-scheme` in markdown that supports it.
- **Canvas**: 64×64 viewBox; scales cleanly to any size.

## Usage

In Markdown that supports HTML (GitHub README, docs site):

```html
<p align="center">
  <picture>
    <source media="(prefers-color-scheme: dark)" srcset="brand/openwebhmi-mark-white.svg">
    <img src="brand/openwebhmi-mark-blue.svg" alt="OpenWebHMI" width="96" />
  </picture>
</p>
```

In HTML (designer, runtime-web):

```html
<link rel="icon" type="image/svg+xml" href="/brand/openwebhmi-favicon-blue.svg" />
```

## Copyright and trademark terms

These files are OpenWebHMI brand assets, not generally licensed as software.
The `OpenWebHMI™` name and logos are unregistered marks claimed by Sergio
Gallegos. Truthful reference, community discussion, and non-misleading
compatibility uses are allowed under [`TRADEMARKS.md`](../TRADEMARKS.md).

Forks may exercise the software rights in [`LICENSE-POLICY.md`](../LICENSE-POLICY.md)
but must use distinct branding unless separate permission applies. Do not imply
official status, certification, sponsorship, partnership, or endorsement.
