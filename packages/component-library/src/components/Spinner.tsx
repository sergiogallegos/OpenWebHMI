import type { ComponentDefinition } from "../types";
import { badQualityStyle, baseFont, designerStyle } from "./shared";

export type SpinnerProps = {
  size: "sm" | "md" | "lg";
  loading: boolean;
  label: string;
};

/** Async loading state indicator. */
export const Spinner: ComponentDefinition<SpinnerProps> = {
  kind: "Spinner",
  defaultProps: { size: "md", loading: true, label: "Loading..." },
  propsSchema: {
    size: { type: "select", label: "Size", options: ["sm", "md", "lg"], default: "md" },
    loading: { type: "boolean", label: "Loading", default: true },
    label: { type: "string", label: "Label", default: "Loading..." },
  },
  bindableProps: ["loading"],
  Render({ props, bindings, context }) {
    const bound = bindings.loading;
    const loading = bound?.value.type === "bool" ? bound.value.value : props.loading;
    const px = props.size === "sm" ? 20 : props.size === "lg" ? 44 : 30;
    if (!loading) {
      return <span aria-label="Spinner hidden" />;
    }
    return (
      <div
        aria-label="Spinner"
        style={{
          ...styles.shell,
          ...badQualityStyle(bound),
          ...designerStyle(context.mode === "designer" ? context.isSelected : false),
        }}
      >
        <svg width={px} height={px} viewBox="0 0 50 50" role="img" aria-label="Loading spinner">
          <circle cx="25" cy="25" r="18" fill="none" stroke="#d1d5db" strokeWidth="6" />
          <g>
            <animateTransform attributeName="transform" type="rotate" from="0 25 25" to="360 25 25" dur="1s" repeatCount="indefinite" />
            <path d="M25 7 A18 18 0 0 1 43 25" fill="none" stroke="#1f4e79" strokeWidth="6" strokeLinecap="round" />
          </g>
        </svg>
        {props.label ? <span style={styles.label}>{props.label}</span> : null}
      </div>
    );
  },
};

const styles = {
  shell: { display: "inline-grid", justifyItems: "center", gap: 6, padding: 6, fontFamily: baseFont },
  label: { color: "#52606d", fontSize: 12 },
};
