import type { ComponentDefinition } from "../types";
import { badQualityStyle, baseFont, chartPalette, designerStyle } from "./shared";

export type ChartDatum = { label: string; value: number; color?: string };

export type BarChartProps = {
  data: ChartDatum[];
  orientation: "vertical" | "horizontal";
  min: number;
  max?: number;
  showValues: boolean;
};

const WIDTH = 360;
const HEIGHT = 220;
const PAD = 34;

/** SVG categorical bar chart for snapshot comparisons. */
export const BarChart: ComponentDefinition<BarChartProps> = {
  kind: "BarChart",
  defaultProps: {
    data: [
      { label: "A", value: 35 },
      { label: "B", value: 70 },
      { label: "C", value: 52 },
    ],
    orientation: "vertical",
    min: 0,
    showValues: true,
  },
  propsSchema: {
    data: { type: "objectList", label: "Data", default: [{ label: "A", value: 35 }, { label: "B", value: 70 }] },
    orientation: { type: "select", label: "Orientation", options: ["vertical", "horizontal"], default: "vertical" },
    min: { type: "number", label: "Minimum", default: 0 },
    max: { type: "number", label: "Maximum" },
    showValues: { type: "boolean", label: "Show values", default: true },
  },
  bindableProps: ["data"],
  Render({ props, bindings, context }) {
    const bound = bindings.data;
    const data = dataFromBound(bound?.value) ?? normalizeData(props.data);
    const max = props.max ?? Math.max(...data.map((item) => item.value), props.min + 1);
    const range = Math.max(1, max - props.min);
    const bad = bound !== undefined && bound.quality !== "good";

    return (
      <section
        aria-label="Bar chart"
        style={{
          ...styles.shell,
          ...badQualityStyle(bound),
          ...designerStyle(context.mode === "designer" ? context.isSelected : false),
        }}
      >
        {data.length === 0 ? <div style={styles.empty}>No data</div> : (
          <svg viewBox={`0 0 ${WIDTH} ${HEIGHT}`} role="img" aria-label="Bar chart graphic" style={styles.svg}>
            <line x1={PAD} x2={WIDTH - 12} y1={HEIGHT - PAD} y2={HEIGHT - PAD} stroke="#9aa5b1" />
            <line x1={PAD} x2={PAD} y1={12} y2={HEIGHT - PAD} stroke="#9aa5b1" />
            {props.orientation === "vertical"
              ? data.map((item, index) => {
                  const width = (WIDTH - PAD - 24) / data.length - 8;
                  const height = ((item.value - props.min) / range) * (HEIGHT - PAD - 20);
                  const x = PAD + 8 + index * (width + 8);
                  const y = HEIGHT - PAD - height;
                  return (
                    <g key={item.label}>
                      <rect x={x} y={y} width={width} height={height} fill={bad ? "#9aa5b1" : item.color ?? chartPalette[index % chartPalette.length]} stroke={bad ? "#697586" : undefined} strokeDasharray={bad ? "4 3" : undefined} />
                      <text x={x + width / 2} y={HEIGHT - 12} textAnchor="middle" fontSize="11" fill="#52606d">{item.label}</text>
                      {props.showValues ? <text x={x + width / 2} y={y - 4} textAnchor="middle" fontSize="11" fill="#1f2933">{item.value}</text> : null}
                    </g>
                  );
                })
              : data.map((item, index) => {
                  const height = (HEIGHT - PAD - 20) / data.length - 8;
                  const width = ((item.value - props.min) / range) * (WIDTH - PAD - 60);
                  const y = 16 + index * (height + 8);
                  return (
                    <g key={item.label}>
                      <text x="4" y={y + height / 2 + 4} fontSize="11" fill="#52606d">{item.label}</text>
                      <rect x={PAD} y={y} width={width} height={height} fill={bad ? "#9aa5b1" : item.color ?? chartPalette[index % chartPalette.length]} stroke={bad ? "#697586" : undefined} strokeDasharray={bad ? "4 3" : undefined} />
                      {props.showValues ? <text x={PAD + width + 4} y={y + height / 2 + 4} fontSize="11" fill="#1f2933">{item.value}</text> : null}
                    </g>
                  );
                })}
          </svg>
        )}
      </section>
    );
  },
};

export function normalizeData(data: ChartDatum[] | undefined): ChartDatum[] {
  return Array.isArray(data)
    ? data.filter((item) => typeof item.label === "string" && Number.isFinite(item.value))
    : [];
}

export function dataFromBound(value: unknown): ChartDatum[] | null {
  if (!value || typeof value !== "object" || !("type" in value)) return null;
  const tag = value as { type: string; value: unknown };
  if (tag.type !== "string" || typeof tag.value !== "string") return null;
  try {
    return normalizeData(JSON.parse(tag.value));
  } catch {
    return null;
  }
}

const styles = {
  shell: { minWidth: 260, padding: 8, border: "1px solid #cbd2d9", borderRadius: 8, background: "#ffffff", fontFamily: baseFont },
  svg: { width: "100%", height: "auto", display: "block" },
  empty: { color: "#697586", fontSize: 13 },
};
