---
status: active
last-validated: 2026-04-26
---

# Project store on-disk format

## Summary

Phase 2 stores each OpenWebHMI project as a directory under the gateway project-store root. Files are the source of truth for artifact content; SQLite is only the metadata index for project version and last-modified time.

## Layout

```text
<store-root>/
├── _index.sqlite
└── <project-id>/
    ├── project.toml
    ├── views/
    │   └── <view-id>.json
    ├── tags/
    │   └── tags.json
    ├── alarms/
    ├── scripts/
    └── assets/
```

Phase 2 writes `project.toml`, `views/*.json`, and `tags/tags.json`. The other directories are created as stable placeholders for later phases.

## Versioning

The SQLite index contains one row per project: `id`, `version`, and `last_modified`. Every committed artifact save increments `version` once and broadcasts a `project.changed` event.

Artifact saves write a temporary sibling file first, then rename it into place. If the write fails before rename, the prior committed artifact remains intact and the version is not bumped.

## Compatibility

The Phase 1 demo project remains valid: inline `tags[]` in `project.toml` are accepted for backwards compatibility, while Phase 2 writes canonical tags to `tags/tags.json`.
