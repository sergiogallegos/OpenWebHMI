# OpenWebHMI Designer

Form-based Phase 2 designer for authoring view JSON through the gateway. This is intentionally not a canvas editor; visual canvas authoring is deferred.

## Prerequisites

- Node 20+
- pnpm 9+
- Rust stable from `rust-toolchain.toml`
- Tauri v2 platform prerequisites for your OS: <https://v2.tauri.app/start/prerequisites/>
- Python 3.11+ on `PATH` for project scripts (`OPENWEBHMI_PYTHON` can point at a specific interpreter). `numpy` and `pandas` are recommended optional packages for future scripting work.

## Run

Start the gateway and runtime from the repo root, then run:

```sh
pnpm --filter @openwebhmi/designer dev
```

For the desktop shell:

```sh
pnpm --filter @openwebhmi/designer tauri dev
```

## Manual Smoke

1. Start `examples/sim-rockwell` and the gateway with `examples/projects/phase1-demo/project.toml`.
2. Start `apps/runtime-web` on `http://localhost:5173`.
3. Open the designer and connect to `ws://localhost:8080`.
4. Open the `home` view.
5. Add a `Container` child, then add a `Label` and `ValueDisplay` under it.
6. Select the label, set text to `Pressure`, and verify save status returns to `saved`.
7. Select the value display, choose `rockwell-1/Pressure` in the tag browser, and bind its `value` prop.
8. Confirm the runtime preview iframe shows the live pressure value.
9. Edit the label text and confirm the runtime preview hot-reloads.
10. Edit the `Setpoint` NumericInput in the preview and confirm the value loops through `tag.write`.
11. Open `Alarms` in the project explorer and define `rockwell-1/Pressure > 200` with priority `2`.
12. Add an `AlarmTable` component to the `home` view, force the simulator pressure high, confirm the table shows the active alarm, ack it, then clear the pressure and confirm the row clears.
13. Add a `Trend` component with `windowSeconds=60` and `tagPaths` set to `rockwell-1/Pressure, rockwell-1/Counter`.
14. Confirm the runtime preview shows two SVG trend lines and appends live samples as the simulator updates.
15. Confirm the bundled `derived-setpoint` script writes `rockwell-1/Setpoint` to half of `rockwell-1/Pressure` whenever pressure is above `100`.
