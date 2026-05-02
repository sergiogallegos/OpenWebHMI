import type { TagValue } from "@openwebhmi/protocol";
import type { ComponentDefinition } from "../types";
import {
  badQualityStyle,
  baseFont,
  designerStyle,
  tagValueToPrimitive,
} from "./shared";

export type ButtonProps = {
  target?: string;
  label: string;
  writeValue: TagValue | TagValue[];
  style: "primary" | "secondary" | "danger";
  confirmPrompt?: string;
};

/** Click-to-write operator button. */
export const Button: ComponentDefinition<ButtonProps> = {
  kind: "Button",
  defaultProps: {
    target: "",
    label: "Press",
    writeValue: { type: "bool", value: true },
    style: "primary",
  },
  propsSchema: {
    target: { type: "string", label: "Target tag", default: "" },
    label: { type: "string", label: "Label", default: "Press" },
    writeValue: { type: "objectList", label: "Write value", default: [{ type: "bool", value: true }] },
    style: {
      type: "select",
      label: "Style",
      options: ["primary", "secondary", "danger"],
      default: "primary",
    },
    confirmPrompt: { type: "string", label: "Confirm prompt" },
  },
  bindableProps: ["target"],
  Render({ props, bindings, context }) {
    const bound = bindings.target;
    const boundTarget = tagValueToPrimitive(bound?.value);
    const target = typeof boundTarget === "string" ? boundTarget : props.target ?? "";
    const disabled = !target || (bound !== undefined && bound.quality !== "good") || context.mode === "designer";
    const writeValue = normalizeWriteValue(props.writeValue);

    return (
      <button
        aria-label="Button"
        type="button"
        disabled={disabled}
        onClick={() => {
          if (context.mode !== "runtime" || disabled) {
            return;
          }
          if (props.confirmPrompt && !window.confirm(props.confirmPrompt)) {
            return;
          }
          context.onWriteTag(target, writeValue);
        }}
        style={{
          ...styles.button,
          ...buttonStyle(props.style),
          opacity: disabled ? 0.55 : 1,
          ...badQualityStyle(bound),
          ...designerStyle(context.mode === "designer" ? context.isSelected : false),
        }}
      >
        {props.label}
      </button>
    );
  },
};

function normalizeWriteValue(value: TagValue | TagValue[]): TagValue {
  return Array.isArray(value) ? value[0] ?? { type: "bool", value: true } : value;
}

function buttonStyle(style: ButtonProps["style"]) {
  switch (style) {
    case "danger":
      return { background: "#dc2626", borderColor: "#b91c1c", color: "#ffffff" };
    case "secondary":
      return { background: "#ffffff", borderColor: "#9aa5b1", color: "#1f2933" };
    default:
      return { background: "#1f4e79", borderColor: "#163a5a", color: "#ffffff" };
  }
}

const styles = {
  button: {
    minHeight: 36,
    minWidth: 96,
    padding: "8px 14px",
    border: "1px solid",
    borderRadius: 6,
    fontFamily: baseFont,
    fontWeight: 700,
    cursor: "pointer",
  },
};
