import type { ComponentDefinition } from "../types";
import { badQualityStyle, baseFont, clampNumber, designerStyle, tagValueToNumber } from "./shared";

export type GaugeProps = {
  value?: number;
  min: number;
  max: number;
  units: string;
  lowWarn?: number;
  highWarn?: number;
  lowAlarm?: number;
  highAlarm?: number;
};

const START = -225;
const SWEEP = 270;

/** Circular SVG gauge for a numeric process value. */
export const Gauge: ComponentDefinition<GaugeProps> = {
  kind: "Gauge",
  defaultProps: {
    value: 50,
    min: 0,
    max: 100,
    units: "",
  },
  propsSchema: {
    value: { type: "number", label: "Value", default: 50 },
    min: { type: "number", label: "Minimum", default: 0 },
    max: { type: "number", label: "Maximum", default: 100 },
    units: { type: "string", label: "Units", default: "" },
    lowWarn: { type: "number", label: "Low warn" },
    highWarn: { type: "number", label: "High warn" },
    lowAlarm: { type: "number", label: "Low alarm" },
    highAlarm: { type: "number", label: "High alarm" },
  },
  bindableProps: ["value"],
  Render({ props, bindings, context }) {
    const bound = bindings.value;
    const min = props.min;
    const max = props.max > min ? props.max : min + 1;
    const value = clampNumber(tagValueToNumber(bound?.value) ?? props.value ?? min, min, max);
    const percent = (value - min) / (max - min);
    const angle = START + percent * SWEEP;
    const needle = polar(100, 100, 58, angle);
    const bad = bound !== undefined && bound.quality !== "good";

    return (
      <section
        aria-label="Gauge"
        style={{
          ...styles.shell,
          ...badQualityStyle(bound),
          ...designerStyle(context.mode === "designer" ? context.isSelected : false),
        }}
      >
        <svg viewBox="0 0 200 150" role="img" aria-label="Gauge chart" style={styles.svg}>
          <path d={arcPath(100, 100, 70, START, START + SWEEP)} fill="none" stroke="#d1d5db" strokeWidth="14" strokeLinecap="round" />
          {segments(props, min, max).map((segment) => (
            <path
              key={`${segment.color}-${segment.start}-${segment.end}`}
              d={arcPath(100, 100, 70, valueAngle(segment.start, min, max), valueAngle(segment.end, min, max))}
              fill="none"
              stroke={segment.color}
              strokeWidth="14"
              strokeLinecap="round"
            />
          ))}
          <line
            x1="100"
            y1="100"
            x2={needle.x}
            y2={needle.y}
            stroke={bad ? "#7b8794" : "#1f2933"}
            strokeWidth="4"
            strokeDasharray={bad ? "5 4" : undefined}
            strokeLinecap="round"
          />
          <circle cx="100" cy="100" r="6" fill={bad ? "#7b8794" : "#1f2933"} />
          <text x="100" y="128" textAnchor="middle" fill="#1f2933" fontSize="20" fontWeight="700">
            {formatNumber(value)}
          </text>
          <text x="100" y="143" textAnchor="middle" fill="#697586" fontSize="12">
            {props.units}
          </text>
        </svg>
      </section>
    );
  },
};

function segments(props: GaugeProps, min: number, max: number) {
  const result = [{ start: min, end: max, color: "#16a34a" }];
  for (const [threshold, low] of [
    [props.lowWarn, true],
    [props.highWarn, false],
  ] as const) {
    if (threshold !== undefined) {
      result.push(low ? { start: min, end: threshold, color: "#f59e0b" } : { start: threshold, end: max, color: "#f59e0b" });
    }
  }
  for (const [threshold, low] of [
    [props.lowAlarm, true],
    [props.highAlarm, false],
  ] as const) {
    if (threshold !== undefined) {
      result.push(low ? { start: min, end: threshold, color: "#dc2626" } : { start: threshold, end: max, color: "#dc2626" });
    }
  }
  return result
    .map((segment) => ({
      ...segment,
      start: clampNumber(segment.start, min, max),
      end: clampNumber(segment.end, min, max),
    }))
    .filter((segment) => segment.end > segment.start);
}

function valueAngle(value: number, min: number, max: number) {
  return START + ((value - min) / (max - min)) * SWEEP;
}

function arcPath(cx: number, cy: number, radius: number, startAngle: number, endAngle: number) {
  const start = polar(cx, cy, radius, startAngle);
  const end = polar(cx, cy, radius, endAngle);
  const largeArc = endAngle - startAngle <= 180 ? 0 : 1;
  return `M ${start.x} ${start.y} A ${radius} ${radius} 0 ${largeArc} 1 ${end.x} ${end.y}`;
}

function polar(cx: number, cy: number, radius: number, angle: number) {
  const radians = (angle * Math.PI) / 180;
  return {
    x: cx + radius * Math.cos(radians),
    y: cy + radius * Math.sin(radians),
  };
}

function formatNumber(value: number) {
  return Number.isInteger(value) ? String(value) : value.toFixed(1);
}

const styles = {
  shell: {
    width: "100%",
    minWidth: 180,
    padding: 8,
    boxSizing: "border-box" as const,
    border: "1px solid #cbd2d9",
    borderRadius: 8,
    background: "#ffffff",
    fontFamily: baseFont,
  },
  svg: {
    display: "block",
    width: "100%",
    height: "auto",
  },
};
