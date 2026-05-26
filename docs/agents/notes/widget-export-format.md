# Widget Export Format

`schema_version: 1` stores one component subtree as JSON:

- `widget_type`: component-library registry key.
- `props`: opaque JSON copied from the view component.
- `bindings`: binding array copied with full tag path strings.
- `children`: nested exported child widgets.
- `exported_at`: Unix epoch milliseconds.
- `openwebhmi_version`: informational only; import gates on schema version, not product version.

Import regenerates component ids and places the widget in the current view. Unknown widget types and unsupported schema versions abort before state changes. Missing tag paths produce warnings because templates are often moved between projects before the destination tag namespace is complete.

Version 1 is frozen once v1.0 ships. Any incompatible shape change must increment `schema_version` and add a migration.
