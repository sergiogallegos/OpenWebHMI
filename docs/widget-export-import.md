# Widget Export And Import

Designer widgets can be moved between projects as human-readable `.owhmi-widget` JSON files.

## Export

Open a view in the designer and use `Export` on the widget card. The exported file contains the widget type, props, bindings, child widgets, export timestamp, and informational OpenWebHMI version. Project ids and the source view id are not exported.

## Import

Open the destination view and use `Import widget...` in the view editor header. The designer validates the file before modifying the view:

- Unknown widget type is a hard error and no widget is placed.
- Unsupported `schema_version` is a hard error.
- Tag bindings that do not exist in the destination project's tag namespace are warnings. The widget is still placed so the integrator can rebind it manually.

Imported widgets receive new component ids to avoid collisions when the same template is imported more than once.

## Format Contract

`schema_version: 1` is the v1.x format floor. OpenWebHMI v1.y imports v1.x widget exports where `y >= x`. Future incompatible changes must use a higher schema version and a migration path.

The format intentionally round-trips widget props, bindings, styles stored in props, and child widgets. It does not round-trip project UUIDs, parent-view ids, tag namespace state, or any runtime-only instance state.
