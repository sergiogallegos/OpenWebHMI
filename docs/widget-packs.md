# Widget Packs

OpenWebHMI ships the default v1 component pack plus an opt-in Material Design demo pack.

## Switching Packs

Open the designer Theme module and set `Pack` to `Material Design`. The selection is stored in the project theme artifact as `pack: "material"`. Projects without a pack, or with `pack: null`, continue to render with the default pack.

The runtime reads the theme on load and resolves components through the selected pack. If the selected pack does not implement a component type, the runtime falls back to the default component and logs one warning per missing type.

## Material Demo Coverage

The Material pack currently styles 8 of the 25 v1 components:

- `Button`
- `ToggleSwitch`
- `NumericInput`
- `Slider`
- `Card`
- `Dropdown`
- `Gauge`
- `Modal`

The remaining components intentionally fall back to the default pack. This is a demo-scope architecture proof, not a complete Material widget library.

## Contract

Packs share the same prop API. A `Button` in the Material pack accepts the same props, bindings, and default values as a `Button` in the default pack. Pack-specific behavior belongs in shared CSS variables and pack styling, not in new widget props.
