import type { ComponentDefinition } from "../types";
import { badQualityStyle, baseFont, clampNumber, designerStyle, tagValueToNumber } from "./shared";

export type ProgressBarProps = {
  value?: number;
  min: number;
  max: number;
  orientation: "horizontal" | "vertical";
  showLabel: boolean;
  fillColor: string;
};

/** Horizontal or vertical numeric fill bar. */
export const ProgressBar: ComponentDefinition<ProgressBarProps> = {
  kind: "ProgressBar",
  defaultProps: {
    value: 50,
    min: 0,
    max: 100,
    orientation: "horizontal",
    showLabel: true,
    fillColor: "#1f4e79",
  },
  propsSchema: {
    value: { type: "number", label: "Value", default: 50 },
    min: { type: "number", label: "Minimum", default: 0 },
    max: { type: "number", label: "Maximum", default: 100 },
    orientation: {
      type: "select",
      label: "Orientation",
      options: ["horizontal", "vertical"],
      default: "horizontal",
    },
    showLabel: { type: "boolean", label: "Show label", default: true },
    fillColor: { type: "color", label: "Fill color", default: "#1f4e79" },
  },
  bindableProps: ["value"],
  Render({ props, bindings, context }) {
    const bound = bindings.value;
    const max = props.max > props.min ? props.max : props.min + 1;
    const value = clampNumber(tagValueToNumber(bound?.value) ?? props.value ?? props.min, props.min, max);
    const pct = ((value - props.min) / (max - props.min)) * 100;
    const bad = bound !== undefined && bound.quality !== "good";
    const vertical = props.orientation === "vertical";

    return (
      <div
        aria-label="Progress bar"
        style={{
          ...styles.shell,
          ...(vertical ? styles.vertical : null),
          ...badQualityStyle(bound),
          ...designerStyle(context.mode === "designer" ? context.isSelected : false),
        }}
      >
        <div style={{ ...styles.track, ...(vertical ? styles.trackVertical : null), borderStyle: bad ? "dashed" : "solid" }}>
          <div
            data-testid="progress-fill"
            style={{
              ...styles.fill,
              background: bad ? "#9aa5b1" : props.fillColor,
              ...(vertical ? { height: `${pct}%`, width: "100%", bottom: 0 } : { width: `${pct}%`, height: "100%" }),
            }}
          />
        </div>
        {props.showLabel ? <span style={styles.label}>{Math.round(pct)}%</span> : null}
      </div>
    );
  },
};

const styles = {
  shell: {
    display: "grid",
    gridTemplateColumns: "1fr auto",
    gap: 8,
    alignItems: "center",
    minWidth: 160,
    fontFamily: baseFont,
  },
  vertical: {
    gridTemplateColumns: "auto",
    gridTemplateRows: "1fr auto",
    justifyItems: "center",
    minHeight: 180,
  },
  track: {
    position: "relative" as const,
    height: 22,
    overflow: "hidden",
    border: "1px solid #9aa5b1",
    borderRadius: 6,
    background: "#eef2f7",
  },
  trackVertical: {
    width: 36,
    height: 150,
  },
  fill: {
    position: "absolute" as const,
    left: 0,
  },
  label: {
    color: "#1f2933",
    fontSize: 13,
    fontVariantNumeric: "tabular-nums" as const,
  },
};
