import type { ComponentDefinition } from "../types";
import { baseFont, designerStyle } from "./shared";

export type CardProps = {
  title: string;
  subtitle: string;
  headerStyle: "plain" | "primary" | "warning" | "danger";
};

/** Labeled child container for grouping HMI content. */
export const Card: ComponentDefinition<CardProps> = {
  kind: "Card",
  defaultProps: { title: "Card", subtitle: "", headerStyle: "plain" },
  propsSchema: {
    title: { type: "string", label: "Title", default: "Card" },
    subtitle: { type: "string", label: "Subtitle", default: "" },
    headerStyle: {
      type: "select",
      label: "Header style",
      options: ["plain", "primary", "warning", "danger"],
      default: "plain",
    },
  },
  bindableProps: [],
  Render({ props, context, children }) {
    const hasHeader = props.title || props.subtitle;
    return (
      <section
        aria-label="Card"
        style={{
          ...styles.shell,
          ...designerStyle(context.mode === "designer" ? context.isSelected : false),
        }}
      >
        {hasHeader ? (
          <header style={{ ...styles.header, ...headerStyle(props.headerStyle) }}>
            {props.title ? <strong>{props.title}</strong> : null}
            {props.subtitle ? <span style={styles.subtitle}>{props.subtitle}</span> : null}
          </header>
        ) : null}
        <div style={styles.body}>{children}</div>
      </section>
    );
  },
};

function headerStyle(style: CardProps["headerStyle"]) {
  switch (style) {
    case "primary":
      return { background: "#dbeafe", color: "#1e3a8a" };
    case "warning":
      return { background: "#fef3c7", color: "#78350f" };
    case "danger":
      return { background: "#fee2e2", color: "#7f1d1d" };
    default:
      return { background: "#f8fafc", color: "#1f2933" };
  }
}

const styles = {
  shell: {
    display: "grid",
    overflow: "hidden",
    minWidth: 180,
    border: "1px solid #cbd2d9",
    borderRadius: 8,
    background: "#ffffff",
    fontFamily: baseFont,
  },
  header: { display: "grid", gap: 2, padding: "10px 12px", borderBottom: "1px solid #e5e7eb" },
  subtitle: { fontSize: 12, opacity: 0.8 },
  body: { padding: 12 },
};
