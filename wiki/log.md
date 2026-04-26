# Wiki Activity Log

Append-only. One line per durable wiki change. Newest at bottom. Format:

```
YYYY-MM-DD  <author>  <page-or-area>  <one-line summary>
```

Use `-` for "no specific page" entries (e.g. project-wide events, repo bootstrap).

---

2026-04-26  sergiogallegos  -                                              Repo re-architectured. Legacy Go/JS scaffolding removed. New stack: Rust gateway + Tauri designer + React web runtime + Python scripting (PyO3, in worker subprocesses). Architecture v0.1, roadmap, feature matrix, contributing guide, AGENTS rules drafted.
2026-04-26  sergiogallegos  drivers/rust-ethernet-ip-integration.md        Seed page created. driver-rockwell will wrap the rust-ethernet-ip crate (>=0.7) as a versioned crates.io dependency (not submodule); upgrades are a Cargo.toml version bump.
2026-04-26  sergiogallegos  -                                              Phase 1 exit criterion changed to simulator-based validation (no physical PLC available at this time). Real-hardware validation gate added pre-1.0; documented in docs/roadmap.md.
2026-04-26  sergiogallegos  -                                              Review-pass corrections applied: (1) README + contributing flag pre-Phase-0 empty-workspace state; (2) roadmap opener and cross-cutting principle #1 reconciled with simulator-first plan; (3) Phase 2/3 exit criteria updated from "bench" to "simulator-backed"; (4) v1 second driver committed to OPC UA, Modbus TCP cleanly post-1.0 in feature-matrix and roadmap Phase 4; (5) architecture §4.4 driver-containment guarantee narrowed (Rust-panic only; native faults / FFI / unsafe UB / panic=abort terminate the gateway, OS-level supervision required); (6) wiki/drivers/rust-ethernet-ip-integration.md Evidence section rewritten with versioned URLs (v0.7.0 tag), upstream commit hash 592bfa716309e3388cf8143c4095622d6302a7f6, and explicit "verified by OpenWebHMI" status table.
2026-04-26  sergiogallegos  -                                              Second-pass corrections: lingering driver-must-not-crash language fixed in architecture §4.1 Crash boundary and the Failure modes table (driver panic row split into Rust-unwind vs native-fault rows), plus contributing.md driver quality bar item rewritten. .obsidian/ added to .gitignore (personal Obsidian workspace state, not committed).
