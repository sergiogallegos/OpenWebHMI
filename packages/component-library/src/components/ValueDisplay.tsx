import type { TagValue } from "@openwebhmi/protocol";
import type { ComponentDefinition } from "../types";
import { badQualityStyle, baseFont, designerStyle } from "./shared";

export type ValueDisplayProps = {
  value?: TagValue;
  format: "number" | "string" | "auto";
  decimals: number;
  unit: string;
};

/** Read-only live value display that preserves visible quality and timestamp context. */
export const ValueDisplay: ComponentDefinition<ValueDisplayProps> = {
  kind: "ValueDisplay",
  defaultProps: {
    format: "auto",
    decimals: 2,
    unit: "",
  },
  propsSchema: {
    value: { type: "string", label: "Value" },
    format: {
      type: "select",
      label: "Format",
      options: ["auto", "number", "string"],
      default: "auto",
    },
    decimals: { type: "number", label: "Decimals", default: 2 },
    unit: { type: "string", label: "Unit", default: "" },
  },
  bindableProps: ["value"],
  Render({ props, bindings, context }) {
    const bound = bindings.value;
    const value = bound?.value ?? props.value;
    const rendered = formatValue(value, props.format, props.decimals);

    return (
      <output
        aria-label="Value display"
        title={bound ? `${bound.quality} @ ${new Date(bound.ts).toISOString()}` : "No binding"}
        style={{
          display: "inline-flex",
          alignItems: "baseline",
          gap: 6,
          minWidth: 80,
          padding: "6px 8px",
          border: "1px solid #cbd2d9",
          borderRadius: 6,
          background: "#ffffff",
          color: "#1f2933",
          fontFamily: baseFont,
          fontVariantNumeric: "tabular-nums",
          ...badQualityStyle(bound),
          ...designerStyle(context.mode === "designer" ? context.isSelected : false),
        }}
      >
        <span>{rendered}</span>
        {props.unit ? <span style={{ color: "#52606d", fontSize: 12 }}>{props.unit}</span> : null}
      </output>
    );
  },
};

function formatValue(
  value: TagValue | undefined,
  format: ValueDisplayProps["format"],
  decimals: number,
): string {
  if (value === undefined) {
    return "--";
  }

  if (format === "string") {
    return String(value.value);
  }

  if (format === "number" || value.type === "int" || value.type === "real") {
    const numeric = Number(value.value);
    return Number.isFinite(numeric) ? numeric.toFixed(decimals) : String(value.value);
  }

  return String(value.value);
}
