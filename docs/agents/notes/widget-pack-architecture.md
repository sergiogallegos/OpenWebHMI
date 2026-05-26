# Widget Pack Architecture

Widget packs are additive maps from component type id to `ComponentDefinition`. The default pack remains the source of truth for prop schemas, bindable props, and default props. Alternate packs must preserve that API and only change rendering.

Runtime rendering calls `getComponentDefinition(typeId, packId)`. Missing pack entries fall back to the default component and emit one warning per `packId:typeId`, which keeps partial packs usable while making fallback visible during development.

Project pack selection is stored on the theme artifact as `theme.pack`. This keeps the setting project-local and lets the Theme Editor own the visual system switch without changing existing view schemas.

The Material demo pack consumes the same CSS variables produced by the Theme Editor, including `--primary-color`, `--surface`, `--text-primary`, `--text-secondary`, `--error`, and `--border-radius`. It also documents MD-style token names in `packs/material/tokens.css`, but the runtime source of truth remains the OpenWebHMI theme variables.
