import type { ComponentDefinition } from "../types";
import { badQualityStyle, baseFont, designerStyle, tagValueToText } from "./shared";

export type LabelProps = {
  text: string;
  color: string;
  fontSize: number;
};

/** Static text label with optional tag-bound text for lightweight runtime annotations. */
export const Label: ComponentDefinition<LabelProps> = {
  kind: "Label",
  defaultProps: {
    text: "Label",
    color: "#1f2933",
    fontSize: 16,
  },
  propsSchema: {
    text: { type: "string", label: "Text", default: "Label" },
    color: { type: "color", label: "Color", default: "#1f2933" },
    fontSize: { type: "number", label: "Font size", default: 16 },
  },
  bindableProps: ["text"],
  Render({ props, bindings, context }) {
    const bound = bindings.text;
    const text = bound ? tagValueToText(bound.value) : props.text;

    return (
      <span
        aria-label="Label"
        style={{
          display: "inline-block",
          minHeight: props.fontSize,
          padding: "2px 4px",
          borderRadius: 4,
          fontFamily: baseFont,
          color: props.color,
          fontSize: props.fontSize,
          ...badQualityStyle(bound),
          ...designerStyle(context.mode === "designer" ? context.isSelected : false),
        }}
      >
        {text || "Label"}
      </span>
    );
  },
};
