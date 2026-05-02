import { useEffect, useRef, useState } from "react";
import type { ComponentDefinition } from "../types";
import {
  badQualityStyle,
  baseFont,
  clampNumber,
  designerStyle,
  tagValueFromNumber,
  tagValueToNumber,
} from "./shared";

export type SliderProps = {
  value?: number;
  tagPath?: string;
  min: number;
  max: number;
  step: number;
  commitMode: "release" | "live";
  units: string;
};

/** Operator slider that writes numeric values on release or live drag. */
export const Slider: ComponentDefinition<SliderProps> = {
  kind: "Slider",
  defaultProps: {
    value: 50,
    tagPath: "",
    min: 0,
    max: 100,
    step: 1,
    commitMode: "release",
    units: "",
  },
  propsSchema: {
    value: { type: "number", label: "Value", default: 50 },
    tagPath: { type: "string", label: "Write tag path", default: "" },
    min: { type: "number", label: "Minimum", default: 0 },
    max: { type: "number", label: "Maximum", default: 100 },
    step: { type: "number", label: "Step", default: 1 },
    commitMode: {
      type: "select",
      label: "Commit mode",
      options: ["release", "live"],
      default: "release",
    },
    units: { type: "string", label: "Units", default: "" },
  },
  bindableProps: ["value"],
  Render({ props, bindings, context }) {
    const bound = bindings.value;
    const max = props.max > props.min ? props.max : props.min + 1;
    const sourceValue = clampNumber(tagValueToNumber(bound?.value) ?? props.value ?? props.min, props.min, max);
    const [draft, setDraft] = useState(sourceValue);
    const lastLiveWrite = useRef(0);
    const disabled = bound !== undefined && bound.quality !== "good";

    useEffect(() => {
      setDraft(sourceValue);
    }, [sourceValue]);

    const commit = (next: number) => {
      if (context.mode !== "runtime" || disabled) {
        return;
      }
      context.onWriteTag(
        props.tagPath || "value",
        tagValueFromNumber(clampNumber(next, props.min, max)),
      );
    };

    return (
      <label
        aria-label="Slider"
        style={{
          ...styles.shell,
          ...badQualityStyle(bound),
          ...designerStyle(context.mode === "designer" ? context.isSelected : false),
        }}
      >
        <input
          aria-label="Slider input"
          type="range"
          min={props.min}
          max={max}
          step={props.step}
          value={draft}
          disabled={disabled}
          onChange={(event) => {
            const next = Number(event.currentTarget.value);
            setDraft(next);
            if (props.commitMode === "live") {
              const now = Date.now();
              if (now - lastLiveWrite.current >= 33) {
                lastLiveWrite.current = now;
                commit(next);
              }
            }
          }}
          onMouseUp={() => commit(draft)}
          onTouchEnd={() => commit(draft)}
          style={styles.input}
        />
        <span style={styles.value}>{formatNumber(draft)}{props.units ? ` ${props.units}` : ""}</span>
      </label>
    );
  },
};

function formatNumber(value: number) {
  return Number.isInteger(value) ? String(value) : value.toFixed(2);
}

const styles = {
  shell: {
    display: "grid",
    gridTemplateColumns: "minmax(120px, 1fr) auto",
    gap: 10,
    alignItems: "center",
    minWidth: 220,
    padding: 8,
    border: "1px solid #cbd2d9",
    borderRadius: 8,
    background: "#ffffff",
    fontFamily: baseFont,
  },
  input: {
    width: "100%",
  },
  value: {
    minWidth: 58,
    textAlign: "right" as const,
    color: "#1f2933",
    fontVariantNumeric: "tabular-nums" as const,
  },
};
