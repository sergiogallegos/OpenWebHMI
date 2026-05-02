import type { ComponentDefinition } from "../types";
import { badQualityStyle, baseFont, designerStyle, tagValueToText } from "./shared";

export type StepItem = { id: string; label: string };

export type StepperProps = {
  steps: StepItem[];
  currentStep?: string;
  orientation: "horizontal" | "vertical";
  completedColor: string;
  currentColor: string;
  pendingColor: string;
};

/** Multi-step process indicator. */
export const Stepper: ComponentDefinition<StepperProps> = {
  kind: "Stepper",
  defaultProps: {
    steps: [
      { id: "fill", label: "Fill" },
      { id: "heat", label: "Heat" },
      { id: "hold", label: "Hold" },
    ],
    currentStep: "heat",
    orientation: "horizontal",
    completedColor: "#16a34a",
    currentColor: "#1f4e79",
    pendingColor: "#9aa5b1",
  },
  propsSchema: {
    steps: {
      type: "objectList",
      label: "Steps",
      default: [
        { id: "fill", label: "Fill" },
        { id: "heat", label: "Heat" },
        { id: "hold", label: "Hold" },
      ],
    },
    currentStep: { type: "string", label: "Current step", default: "heat" },
    orientation: {
      type: "select",
      label: "Orientation",
      options: ["horizontal", "vertical"],
      default: "horizontal",
    },
    completedColor: { type: "color", label: "Completed color", default: "#16a34a" },
    currentColor: { type: "color", label: "Current color", default: "#1f4e79" },
    pendingColor: { type: "color", label: "Pending color", default: "#9aa5b1" },
  },
  bindableProps: ["currentStep"],
  Render({ props, bindings, context }) {
    const bound = bindings.currentStep;
    const steps = normalizeSteps(props.steps);
    const current = tagValueToText(bound?.value) || props.currentStep || steps[0]?.id;
    const currentIndex = Math.max(0, steps.findIndex((step) => step.id === current));
    const vertical = props.orientation === "vertical";
    return (
      <ol
        aria-label="Stepper"
        style={{
          ...styles.list,
          ...(vertical ? styles.vertical : null),
          ...badQualityStyle(bound),
          ...designerStyle(context.mode === "designer" ? context.isSelected : false),
        }}
      >
        {steps.map((step, index) => {
          const color = index < currentIndex ? props.completedColor : index === currentIndex ? props.currentColor : props.pendingColor;
          return (
            <li key={step.id} style={styles.item}>
              <span data-testid={`step-${step.id}`} style={{ ...styles.marker, borderColor: color, background: index <= currentIndex ? color : "#ffffff", color: index <= currentIndex ? "#ffffff" : color }}>
                {index + 1}
              </span>
              <span style={{ color }}>{step.label}</span>
            </li>
          );
        })}
      </ol>
    );
  },
};

function normalizeSteps(steps: StepItem[] | undefined): StepItem[] {
  return Array.isArray(steps) ? steps.filter((step) => step.id && step.label) : [];
}

const styles = {
  list: { display: "flex", gap: 14, padding: 8, margin: 0, listStyle: "none", fontFamily: baseFont },
  vertical: { flexDirection: "column" as const },
  item: { display: "inline-flex", alignItems: "center", gap: 8 },
  marker: { display: "inline-grid", placeItems: "center", width: 26, height: 26, border: "2px solid", borderRadius: "50%", fontWeight: 700 },
};
