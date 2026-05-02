import type { ComponentDefinition } from "../types";
import { badQualityStyle, baseFont, chartPalette, designerStyle } from "./shared";
import { dataFromBound, normalizeData, type ChartDatum } from "./BarChart";

export type PieChartProps = {
  data: ChartDatum[];
  donut: boolean;
  showLegend: boolean;
  showPercentages: boolean;
};

/** SVG categorical pie or donut chart. */
export const PieChart: ComponentDefinition<PieChartProps> = {
  kind: "PieChart",
  defaultProps: {
    data: [
      { label: "Run", value: 62 },
      { label: "Idle", value: 25 },
      { label: "Fault", value: 13 },
    ],
    donut: false,
    showLegend: true,
    showPercentages: true,
  },
  propsSchema: {
    data: { type: "objectList", label: "Data", default: [{ label: "Run", value: 62 }, { label: "Idle", value: 25 }] },
    donut: { type: "boolean", label: "Donut", default: false },
    showLegend: { type: "boolean", label: "Show legend", default: true },
    showPercentages: { type: "boolean", label: "Show percentages", default: true },
  },
  bindableProps: ["data"],
  Render({ props, bindings, context }) {
    const bound = bindings.data;
    const data = dataFromBound(bound?.value) ?? normalizeData(props.data);
    const total = data.reduce((sum, item) => sum + Math.max(0, item.value), 0);
    const bad = bound !== undefined && bound.quality !== "good";
    let cursor = -90;

    return (
      <section
        aria-label="Pie chart"
        style={{
          ...styles.shell,
          ...badQualityStyle(bound),
          ...designerStyle(context.mode === "designer" ? context.isSelected : false),
        }}
      >
        {total <= 0 ? <div style={styles.empty}>No data</div> : (
          <svg viewBox="0 0 260 180" role="img" aria-label="Pie chart graphic" style={styles.svg}>
            {data.map((item, index) => {
              const sweep = (Math.max(0, item.value) / total) * 360;
              const path = slicePath(90, 90, 70, cursor, cursor + sweep, props.donut ? 32 : 0);
              const labelAngle = cursor + sweep / 2;
              const label = polar(90, 90, 48, labelAngle);
              cursor += sweep;
              return (
                <g key={item.label}>
                  <path d={path} fill={bad ? "#9aa5b1" : item.color ?? chartPalette[index % chartPalette.length]} stroke="#ffffff" strokeWidth="2" strokeDasharray={bad ? "4 3" : undefined} />
                  {props.showPercentages && sweep > 22 ? <text x={label.x} y={label.y} textAnchor="middle" fontSize="11" fill="#ffffff">{Math.round((item.value / total) * 100)}%</text> : null}
                </g>
              );
            })}
            {props.donut ? <circle cx="90" cy="90" r="32" fill="#ffffff" /> : null}
            {props.showLegend ? data.map((item, index) => (
              <g key={item.label} transform={`translate(175 ${28 + index * 22})`}>
                <rect width="10" height="10" fill={item.color ?? chartPalette[index % chartPalette.length]} />
                <text x="16" y="10" fontSize="11" fill="#1f2933">{item.label}</text>
              </g>
            )) : null}
          </svg>
        )}
      </section>
    );
  },
};

function slicePath(cx: number, cy: number, radius: number, start: number, end: number, inner: number) {
  const a = polar(cx, cy, radius, start);
  const b = polar(cx, cy, radius, end);
  const large = end - start > 180 ? 1 : 0;
  if (inner <= 0) {
    return `M ${cx} ${cy} L ${a.x} ${a.y} A ${radius} ${radius} 0 ${large} 1 ${b.x} ${b.y} Z`;
  }
  const c = polar(cx, cy, inner, end);
  const d = polar(cx, cy, inner, start);
  return `M ${a.x} ${a.y} A ${radius} ${radius} 0 ${large} 1 ${b.x} ${b.y} L ${c.x} ${c.y} A ${inner} ${inner} 0 ${large} 0 ${d.x} ${d.y} Z`;
}

function polar(cx: number, cy: number, radius: number, angle: number) {
  const radians = (angle * Math.PI) / 180;
  return { x: cx + radius * Math.cos(radians), y: cy + radius * Math.sin(radians) };
}

const styles = {
  shell: { minWidth: 260, padding: 8, border: "1px solid #cbd2d9", borderRadius: 8, background: "#ffffff", fontFamily: baseFont },
  svg: { width: "100%", height: "auto", display: "block" },
  empty: { color: "#697586", fontSize: 13 },
};
