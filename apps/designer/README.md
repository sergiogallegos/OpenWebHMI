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
16. Open `Scripts`, select `derived-setpoint`, and change the multiplier from `0.5` to `0.6`.
17. Confirm `Setpoint` tracks roughly `0.6 * Pressure` within the next second while pressure remains above `100`.
18. Introduce a typo such as `sytem.tag.read`, save, and confirm the traceback appears in Recent events.
19. Click the traceback row and confirm the editor opens the offending line, then fix the typo and confirm the script resumes.
20. Start `examples/sim-modbus` with `cargo run -p sim-modbus -- --bind 127.0.0.1:5502`.
21. Configure a Modbus TCP driver pointed at `127.0.0.1:5502`, subscribe to `1/holding/100:2:f32` and `1/coils/0`, and confirm the runtime shows `12.5` plus a `true` coil value.
22. Start `examples/sim-opcua` with `cargo run -p sim-opcua -- --bind 127.0.0.1:4855`.
23. Configure an OPC UA driver pointed at `opc.tcp://127.0.0.1:4855/`, subscribe to the simulator `Pressure` node using the namespace index reported by the simulator/tests, and confirm live updates plus a write round-trip in the runtime.
24. Start `examples/sim-mqtt` with `cargo run -p sim-mqtt -- --bind 127.0.0.1:1883`.
25. Configure an MQTT driver pointed at `127.0.0.1:1883`, subscribe to `factory/line1/temperature` and `spB/v1.0/group/DDATA/edge/device/Pressure`, and confirm generic plus Sparkplug B values update in the runtime.
