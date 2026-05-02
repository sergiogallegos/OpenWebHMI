import type { ComponentDefinition } from "../types";
import { badQualityStyle, baseFont, designerStyle } from "./shared";

export type ToggleSwitchProps = {
  value?: boolean;
  tagPath?: string;
  onLabel: string;
  offLabel: string;
  onColor: string;
  offColor: string;
};

/** Boolean write-back pill switch. */
export const ToggleSwitch: ComponentDefinition<ToggleSwitchProps> = {
  kind: "ToggleSwitch",
  defaultProps: {
    value: false,
    tagPath: "",
    onLabel: "On",
    offLabel: "Off",
    onColor: "#16a34a",
    offColor: "#9aa5b1",
  },
  propsSchema: {
    value: { type: "boolean", label: "Value", default: false },
    tagPath: { type: "string", label: "Write tag path", default: "" },
    onLabel: { type: "string", label: "On label", default: "On" },
    offLabel: { type: "string", label: "Off label", default: "Off" },
    onColor: { type: "color", label: "On color", default: "#16a34a" },
    offColor: { type: "color", label: "Off color", default: "#9aa5b1" },
  },
  bindableProps: ["value"],
  Render({ props, bindings, context }) {
    const bound = bindings.value;
    const value = bound?.value.type === "bool" ? bound.value.value : props.value ?? false;
    const disabled = bound !== undefined && bound.quality !== "good";

    return (
      <button
        aria-label="Toggle switch"
        type="button"
        disabled={disabled}
        onClick={() => {
          if (context.mode === "runtime" && !disabled) {
            context.onWriteTag(props.tagPath || "value", { type: "bool", value: !value });
          }
        }}
        style={{
          ...styles.button,
          background: value ? props.onColor : props.offColor,
          opacity: disabled ? 0.55 : 1,
          ...badQualityStyle(bound),
          ...designerStyle(context.mode === "designer" ? context.isSelected : false),
        }}
      >
        <span style={{ ...styles.knob, transform: value ? "translateX(26px)" : "translateX(0)" }} />
        <span style={styles.label}>{value ? props.onLabel : props.offLabel}</span>
      </button>
    );
  },
};

const styles = {
  button: {
    position: "relative" as const,
    display: "inline-grid",
    alignItems: "center",
    minWidth: 86,
    height: 34,
    padding: "0 12px 0 42px",
    border: "1px solid transparent",
    borderRadius: 999,
    color: "#ffffff",
    fontFamily: baseFont,
    fontWeight: 700,
    cursor: "pointer",
  },
  knob: {
    position: "absolute" as const,
    left: 4,
    width: 26,
    height: 26,
    borderRadius: "50%",
    background: "#ffffff",
    boxShadow: "0 1px 3px rgba(0,0,0,0.25)",
  },
  label: {
    justifySelf: "end",
    whiteSpace: "nowrap" as const,
  },
};
