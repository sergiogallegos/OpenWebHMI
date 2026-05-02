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
25. Configure an MQTT driver pointed at `127.0.0.1:1883`, subscribe to `factory/line1/temperature`, `factory/line1/count`, `factory/line1/json`, and `spB/v1.0/group/DDATA/edge/device/Pressure`, and confirm generic plus Sparkplug B values update in the runtime.
26. Reconfigure the same MQTT driver with `transport="websocket"` and `host="ws://127.0.0.1:1884/mqtt"` (the simulator logs the WebSocket address on start), then confirm `factory/line1/temperature` continues updating.
27. TLS smoke uses an external TLS broker such as Mosquitto because the embedded `rumqttd` fixture is kept plaintext for deterministic CI. Start Mosquitto with a server cert signed by a local CA, set `transport="tls"`, `host`/`port` to the TLS listener, and `ca_cert_path` to that CA PEM; confirm the driver connects without setting `tls_insecure`.
28. Add a `Gauge` bound to `rockwell-1/Pressure`, set units to `PSI`, set warn/alarm thresholds around the simulator's live range, and confirm the SVG needle and colored bands render in the runtime preview.
29. Add a `ProgressBar` bound to `rockwell-1/Pressure`, try both horizontal and vertical orientations, and confirm the fill percentage tracks the live pressure value.
30. Add a `Slider` bound to a writable numeric tag such as `rockwell-1/Setpoint`, keep `commitMode=release`, drag and release the thumb, and confirm the tag write loops through the gateway.
31. Add a `Dropdown` with Auto/Manual options, bind `value` to a string memory tag, change the selected option in runtime mode, and confirm the selected primitive is written.
32. Add a `ToggleSwitch` bound to a boolean tag, click it in runtime mode, and confirm it writes the inverse boolean while designer mode suppresses live writes.
33. Add a `Button`, bind `target` to a constant/tag path for a writable tag, configure a constant `writeValue`, click it in runtime mode, and confirm the write occurs only after any configured confirmation prompt.
34. Add a `MultiState` bound to a small string or boolean tag, configure at least two label/color states, and confirm unmatched values fall back to the default label.
35. Add an `AlarmBanner` above the existing `AlarmTable`, trigger pressure alarms, and confirm Critical/Warning/Info counts update from the same alarm subscription stream.
36. Add `Tabs` with two child components, switch tabs by click and keyboard arrows, and confirm only the active child content is visible.
37. Add a `Modal` with a child label, configure confirm/cancel tag paths, open it from a boolean binding, then confirm the buttons write `true` and Escape/click-outside cancel.
38. Add a `DataGrid` bound to a string tag containing JSON row data, click column headers to sort, and use Prev/Next pagination with a small `pageSize`.
39. Add `BarChart` and `PieChart` components using the same JSON categorical data, verify bar/legend labels render, then toggle PieChart donut mode.
40. Add `Card`, `Divider`, `Spinner`, and `Stepper` components to the `home` view, verify children render inside Card, Divider labels align, Spinner hides when loading is false, and Stepper highlights the bound current step.
