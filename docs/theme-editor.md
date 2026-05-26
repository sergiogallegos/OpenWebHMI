# Theme Editor

The Designer Theme panel edits one project-level theme stored as the `theme` artifact. The runtime loads that artifact when a project view opens and injects CSS variables into a single `openwebhmi-project-theme` style block.

The editable variables are:

| Variable | Theme field |
|---|---|
| `--primary-color` | `primary_color` |
| `--secondary-color` | `secondary_color` |
| `--background` | `background` |
| `--surface` | `surface` |
| `--text-primary` | `text_primary` |
| `--text-secondary` | `text_secondary` |
| `--accent` | `accent` |
| `--error` | `error` |
| `--warning` | `warning` |
| `--font-family` | `font_family` |
| `--font-size-base` | `font_size_base` |
| `--spacing-unit` | `spacing_unit` |
| `--border-radius` | `border_radius` |

The artifact stores separate `light` and `dark` variable sets plus the active `mode`. The mode is also mirrored in `localStorage` as `openwebhmi.themeMode` so runtime reloads keep the operator's last choice.

Components should consume these variables with CSS fallbacks, for example:

```ts
background: "var(--primary-color, #1f4e79)"
```

Custom variables are a code-level extension for now: add the variable to `packages/protocol-ts/src/theme.ts`, add the matching Rust project-store type field, then consume it from component styles.
