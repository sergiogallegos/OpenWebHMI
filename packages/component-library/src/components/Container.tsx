import type { ComponentDefinition } from "../types";
import { baseFont, designerStyle } from "./shared";

export type ContainerProps = {
  direction: "row" | "column";
  gap: number;
  padding: number;
  background: string;
};

/** Layout-only flex container whose child tree is supplied by the view renderer. */
export const Container: ComponentDefinition<ContainerProps> = {
  kind: "Container",
  defaultProps: {
    direction: "column",
    gap: 8,
    padding: 12,
    background: "#ffffff",
  },
  propsSchema: {
    direction: {
      type: "select",
      label: "Direction",
      options: ["row", "column"],
      default: "column",
    },
    gap: { type: "number", label: "Gap", default: 8 },
    padding: { type: "number", label: "Padding", default: 12 },
    background: { type: "color", label: "Background", default: "#ffffff" },
  },
  bindableProps: [],
  Render({ props, context, children }) {
    return (
      <div
        aria-label="Container"
        style={{
          display: "flex",
          flexDirection: props.direction,
          gap: props.gap,
          minWidth: 120,
          minHeight: 80,
          boxSizing: "border-box",
          padding: props.padding,
          border: "1px dashed #9aa5b1",
          borderRadius: 6,
          background: props.background,
          color: "#1f2933",
          fontFamily: baseFont,
          ...designerStyle(context.mode === "designer" ? context.isSelected : false),
        }}
      >
        {children}
      </div>
    );
  },
};
