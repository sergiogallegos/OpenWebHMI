import { useEffect, useState } from "react";
import type { ComponentDefinition } from "../types";
import {
  badQualityStyle,
  baseFont,
  designerStyle,
  tagValueFromNumber,
  tagValueToNumber,
} from "./shared";

export type NumericInputProps = {
  value?: number;
  tagPath?: string;
  min?: number;
  max?: number;
  step: number;
  disabled: boolean;
};

/** Operator-editable numeric input that writes a validated value on Enter or blur. */
export const NumericInput: ComponentDefinition<NumericInputProps> = {
  kind: "NumericInput",
  defaultProps: {
    value: 0,
    tagPath: "",
    step: 1,
    disabled: false,
  },
  propsSchema: {
    value: { type: "number", label: "Value", default: 0 },
    tagPath: { type: "string", label: "Write tag path", default: "" },
    min: { type: "number", label: "Minimum" },
    max: { type: "number", label: "Maximum" },
    step: { type: "number", label: "Step", default: 1 },
    disabled: { type: "boolean", label: "Disabled", default: false },
  },
  bindableProps: ["value"],
  Render({ props, bindings, context }) {
    const bound = bindings.value;
    const numericValue = tagValueToNumber(bound?.value) ?? props.value ?? 0;
    const [draft, setDraft] = useState(String(numericValue));

    useEffect(() => {
      setDraft(String(numericValue));
    }, [numericValue]);

    const commit = () => {
      if (context.mode !== "runtime" || props.disabled) {
        return;
      }

      const parsed = Number(draft);
      if (!Number.isFinite(parsed)) {
        setDraft(String(numericValue));
        return;
      }

      const clamped = clamp(parsed, props.min, props.max);
      const path = props.tagPath || "value";
      context.onWriteTag(path, tagValueFromNumber(clamped));
      setDraft(String(clamped));
    };

    return (
      <input
        aria-label="Numeric input"
        disabled={props.disabled}
        min={props.min}
        max={props.max}
        step={props.step}
        type="number"
        value={draft}
        onBlur={commit}
        onChange={(event) => setDraft(event.currentTarget.value)}
        onKeyDown={(event) => {
          if (event.key === "Enter") {
            commit();
          }
        }}
        style={{
          width: 120,
          minHeight: 34,
          boxSizing: "border-box",
          padding: "6px 8px",
          border: "1px solid #9aa5b1",
          borderRadius: 6,
          background: props.disabled ? "#f5f7fa" : "#ffffff",
          color: "#1f2933",
          fontFamily: baseFont,
          fontVariantNumeric: "tabular-nums",
          ...badQualityStyle(bound),
          ...designerStyle(context.mode === "designer" ? context.isSelected : false),
        }}
      />
    );
  },
};

function clamp(value: number, min: number | undefined, max: number | undefined): number {
  if (min !== undefined && value < min) {
    return min;
  }
  if (max !== undefined && value > max) {
    return max;
  }
  return value;
}
