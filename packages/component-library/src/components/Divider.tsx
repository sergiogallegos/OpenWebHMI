import type { ComponentDefinition } from "../types";
import { baseFont, designerStyle } from "./shared";

export type DividerProps = {
  orientation: "horizontal" | "vertical";
  label: string;
  thickness: number;
};

/** Horizontal or vertical separator with optional label. */
export const Divider: ComponentDefinition<DividerProps> = {
  kind: "Divider",
  defaultProps: { orientation: "horizontal", label: "", thickness: 1 },
  propsSchema: {
    orientation: {
      type: "select",
      label: "Orientation",
      options: ["horizontal", "vertical"],
      default: "horizontal",
    },
    label: { type: "string", label: "Label", default: "" },
    thickness: { type: "number", label: "Thickness", default: 1 },
  },
  bindableProps: [],
  Render({ props, context }) {
    const vertical = props.orientation === "vertical";
    return (
      <div
        aria-label="Divider"
        style={{
          ...styles.shell,
          ...(vertical ? styles.vertical : null),
          ...designerStyle(context.mode === "designer" ? context.isSelected : false),
        }}
      >
        <span style={{ ...styles.line, ...(vertical ? { width: props.thickness } : { height: props.thickness }) }} />
        {props.label ? <span style={styles.label}>{props.label}</span> : null}
        <span style={{ ...styles.line, ...(vertical ? { width: props.thickness } : { height: props.thickness }) }} />
      </div>
    );
  },
};

const styles = {
  shell: {
    display: "grid",
    gridTemplateColumns: "1fr auto 1fr",
    alignItems: "center",
    gap: 8,
    minWidth: 120,
    fontFamily: baseFont,
  },
  vertical: {
    gridTemplateColumns: "auto",
    gridTemplateRows: "1fr auto 1fr",
    justifyItems: "center",
    minHeight: 120,
    minWidth: 24,
  },
  line: { display: "block", alignSelf: "stretch", background: "#cbd2d9" },
  label: { color: "#697586", fontSize: 12 },
};
