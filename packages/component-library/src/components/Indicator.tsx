import type { ComponentDefinition } from "../types";
import { badQualityStyle, baseFont, designerStyle, tagValueToText } from "./shared";

export type IndicatorProps = {
  state?: boolean | string;
  onColor: string;
  offColor: string;
  mapping: Record<string, string>;
};

/** Status badge that maps boolean or string tag state into a color-coded indicator. */
export const Indicator: ComponentDefinition<IndicatorProps> = {
  kind: "Indicator",
  defaultProps: {
    state: false,
    onColor: "#16a34a",
    offColor: "#9aa5b1",
    mapping: {},
  },
  propsSchema: {
    state: { type: "string", label: "State" },
    onColor: { type: "color", label: "On color", default: "#16a34a" },
    offColor: { type: "color", label: "Off color", default: "#9aa5b1" },
    mapping: { type: "string", label: "String color mapping", default: "{}" },
  },
  bindableProps: ["state"],
  Render({ props, bindings, context }) {
    const bound = bindings.state;
    const state = bound ? bound.value.value : props.state;
    const stateText = bound ? tagValueToText(bound.value) : String(state ?? false);
    const color = colorForState(state, props);

    return (
      <span
        aria-label={`Indicator ${stateText}`}
        role="status"
        tabIndex={0}
        style={{
          display: "inline-flex",
          alignItems: "center",
          gap: 8,
          minHeight: 28,
          padding: "4px 8px",
          border: "1px solid #cbd2d9",
          borderRadius: 999,
          background: "#ffffff",
          color: "#1f2933",
          fontFamily: baseFont,
          ...badQualityStyle(bound),
          ...designerStyle(context.mode === "designer" ? context.isSelected : false),
        }}
      >
        <span
          aria-hidden="true"
          style={{
            width: 12,
            height: 12,
            borderRadius: "50%",
            background: bound && bound.quality !== "good" ? "#d64545" : color,
            boxShadow: `0 0 0 2px ${color}22`,
          }}
        />
        <span>{bound && bound.quality !== "good" ? "?" : stateText}</span>
      </span>
    );
  },
};

function colorForState(state: unknown, props: IndicatorProps): string {
  if (typeof state === "boolean") {
    return state ? props.onColor : props.offColor;
  }

  if (typeof state === "string" && props.mapping[state]) {
    return props.mapping[state];
  }

  return state ? props.onColor : props.offColor;
}
