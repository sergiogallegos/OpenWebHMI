import type { ComponentDefinition } from "../types";
import {
  badQualityStyle,
  baseFont,
  designerStyle,
  tagValueToPrimitive,
} from "./shared";

export type MultiStateEntry = {
  value: string | number | boolean;
  label: string;
  color: string;
  bgColor: string;
};

export type MultiStateProps = {
  value?: string | number | boolean;
  states: MultiStateEntry[];
  defaultLabel: string;
  defaultColor: string;
};

/** Primitive value display with configurable state labels and colors. */
export const MultiState: ComponentDefinition<MultiStateProps> = {
  kind: "MultiState",
  defaultProps: {
    value: "run",
    states: [
      { value: "run", label: "Running", color: "#065f46", bgColor: "#d1fae5" },
      { value: "stop", label: "Stopped", color: "#7f1d1d", bgColor: "#fee2e2" },
    ],
    defaultLabel: "--",
    defaultColor: "#697586",
  },
  propsSchema: {
    value: { type: "string", label: "Value", default: "run" },
    states: {
      type: "objectList",
      label: "States",
      default: [
        { value: "run", label: "Running", color: "#065f46", bgColor: "#d1fae5" },
        { value: "stop", label: "Stopped", color: "#7f1d1d", bgColor: "#fee2e2" },
      ],
    },
    defaultLabel: { type: "string", label: "Default label", default: "--" },
    defaultColor: { type: "color", label: "Default color", default: "#697586" },
  },
  bindableProps: ["value"],
  Render({ props, bindings, context }) {
    const bound = bindings.value;
    const value = tagValueToPrimitive(bound?.value) ?? props.value;
    const states = normalizeStates(props.states);
    const match = states.find((state) => state.value === value);
    const bad = bound !== undefined && bound.quality !== "good";

    return (
      <div
        aria-label="MultiState"
        style={{
          ...styles.chip,
          color: match?.color ?? props.defaultColor,
          background: match?.bgColor ?? "#ffffff",
          borderStyle: bad ? "dashed" : "solid",
          ...badQualityStyle(bound),
          ...designerStyle(context.mode === "designer" ? context.isSelected : false),
        }}
      >
        {match?.label ?? props.defaultLabel}
      </div>
    );
  },
};

function normalizeStates(states: MultiStateEntry[] | undefined): MultiStateEntry[] {
  return Array.isArray(states)
    ? states.filter(
        (state) =>
          "value" in state &&
          typeof state.label === "string" &&
          typeof state.color === "string" &&
          typeof state.bgColor === "string",
      )
    : [];
}

const styles = {
  chip: {
    display: "inline-flex",
    alignItems: "center",
    justifyContent: "center",
    minHeight: 32,
    minWidth: 88,
    padding: "6px 10px",
    border: "1px solid #cbd2d9",
    borderRadius: 6,
    fontFamily: baseFont,
    fontWeight: 700,
  },
};
