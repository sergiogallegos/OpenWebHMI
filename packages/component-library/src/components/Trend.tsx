import { useEffect, useMemo, useRef, useState } from "react";
import type { HistoryPoint, Quality, TagValue } from "@openwebhmi/protocol";
import type { BoundValue, ComponentDefinition } from "../types";
import { baseFont, designerStyle } from "./shared";

export type TrendProps = {
  tagPaths: string[];
  windowSeconds: number;
  maxPoints: number;
  yMin?: number;
  yMax?: number;
  showLegend: boolean;
};

type ChartPoint = {
  tsMs: number;
  value: number;
  quality: Quality;
};

type PenSeries = {
  path: string;
  points: ChartPoint[];
};

const COLORS = ["#2563eb", "#dc2626", "#059669", "#7c3aed", "#d97706", "#0891b2"];
const WIDTH = 720;
const HEIGHT = 260;
const PADDING = { top: 18, right: 18, bottom: 28, left: 48 };

/** Pure-SVG multi-pen trend with historical prime and live append. */
export const Trend: ComponentDefinition<TrendProps> = {
  kind: "Trend",
  defaultProps: {
    tagPaths: ["rockwell-1/Pressure"],
    windowSeconds: 300,
    maxPoints: 600,
    showLegend: true,
  },
  propsSchema: {
    tagPaths: {
      type: "stringList",
      label: "Tag paths",
      default: ["rockwell-1/Pressure"],
    },
    windowSeconds: { type: "number", label: "Window seconds", default: 300 },
    maxPoints: { type: "number", label: "Max points", default: 600 },
    yMin: { type: "number", label: "Y min" },
    yMax: { type: "number", label: "Y max" },
    showLegend: { type: "boolean", label: "Show legend", default: true },
  },
  bindableProps: ["tagPaths"],
  Render({ props, bindings, context }) {
    const tagPaths = normalizeTagPaths(props.tagPaths, bindings.tagPaths);
    const [series, setSeries] = useState<PenSeries[]>(() =>
      tagPaths.map((path) => ({ path, points: [] })),
    );
    const [loading, setLoading] = useState(true);
    const [hidden, setHidden] = useState<Set<string>>(() => new Set());
    const lastLiveTs = useRef(new Map<string, number>());
    const readHistory = context.mode === "runtime" ? context.onReadHistory : undefined;
    const liveValues = context.mode === "runtime" ? context.liveValues : undefined;

    useEffect(() => {
      setSeries(tagPaths.map((path) => ({ path, points: [] })));
      if (!readHistory || tagPaths.length === 0) {
        setLoading(false);
        return undefined;
      }

      let cancelled = false;
      setLoading(true);
      const tEndMs = Date.now();
      const tStartMs = tEndMs - Math.max(1, props.windowSeconds) * 1_000;
      Promise.all(
        tagPaths.map(async (path) => ({
          path,
          points: historyToPoints(
            await readHistory({
              tagPath: path,
              tStartMs,
              tEndMs,
              aggregation: "raw",
              maxPoints: props.maxPoints,
            }),
          ),
        })),
      )
        .then((next) => {
          if (!cancelled) {
            setSeries(next);
            setLoading(false);
          }
        })
        .catch(() => {
          if (!cancelled) {
            setSeries(tagPaths.map((path) => ({ path, points: [] })));
            setLoading(false);
          }
        });

      return () => {
        cancelled = true;
      };
    }, [
      readHistory,
      props.maxPoints,
      props.windowSeconds,
      tagPaths.join("\n"),
    ]);

    useEffect(() => {
      if (!liveValues) {
        return;
      }
      const cutoff = Date.now() - Math.max(1, props.windowSeconds) * 1_000;
      let changed = false;
      const nextSeries = series.map((pen) => {
        const live = liveValues[pen.path];
        const point = liveToPoint(live);
        if (!point || lastLiveTs.current.get(pen.path) === point.tsMs) {
          return pen;
        }
        lastLiveTs.current.set(pen.path, point.tsMs);
        changed = true;
        return {
          ...pen,
          points: limitPoints(
            [...pen.points, point].filter((item) => item.tsMs >= cutoff),
            props.maxPoints,
          ),
        };
      });
      if (changed) {
        setSeries(nextSeries);
      }
    }, [liveValues, props.maxPoints, props.windowSeconds, series]);

    const visible = series.filter((pen) => !hidden.has(pen.path));
    const domain = useMemo(
      () => computeDomain(visible, props.yMin, props.yMax, props.windowSeconds),
      [visible, props.yMin, props.yMax, props.windowSeconds],
    );
    const paths = useMemo(
      () =>
        visible.map((pen, index) => ({
          path: pen.path,
          color: COLORS[index % COLORS.length],
          good: pathForQuality(pen.points, domain, true),
          bad: pathForQuality(pen.points, domain, false),
        })),
      [domain, visible],
    );

    return (
      <section
        aria-label="Trend"
        style={{
          ...styles.shell,
          ...designerStyle(context.mode === "designer" ? context.isSelected : false),
        }}
      >
        {loading ? <div role="status" style={styles.loading}>loading...</div> : null}
        <svg viewBox={`0 0 ${WIDTH} ${HEIGHT}`} role="img" aria-label="Trend chart" style={styles.svg}>
          <rect x="0" y="0" width={WIDTH} height={HEIGHT} fill="#ffffff" />
          {gridLines(domain).map((line) => (
            <g key={line.y}>
              <line x1={PADDING.left} x2={WIDTH - PADDING.right} y1={line.y} y2={line.y} stroke="#e5e7eb" />
              <text x="8" y={line.y + 4} fill="#52606d" fontSize="11">
                {line.label}
              </text>
            </g>
          ))}
          <line x1={PADDING.left} x2={WIDTH - PADDING.right} y1={HEIGHT - PADDING.bottom} y2={HEIGHT - PADDING.bottom} stroke="#9aa5b1" />
          <line x1={PADDING.left} x2={PADDING.left} y1={PADDING.top} y2={HEIGHT - PADDING.bottom} stroke="#9aa5b1" />
          <text x={PADDING.left} y={HEIGHT - 8} fill="#52606d" fontSize="11">
            {formatAxisTime(domain.xMin)}
          </text>
          <text x={WIDTH - PADDING.right - 58} y={HEIGHT - 8} fill="#52606d" fontSize="11">
            {formatAxisTime(domain.xMax)}
          </text>
          {paths.map((path) => (
            <g key={path.path}>
              <path d={path.good} fill="none" stroke={path.color} strokeWidth="2" />
              <path d={path.bad} fill="none" stroke={path.color} strokeWidth="2" strokeDasharray="5 4" />
            </g>
          ))}
        </svg>
        {props.showLegend ? (
          <div style={styles.legend}>
            {series.map((pen, index) => {
              const last = pen.points[pen.points.length - 1];
              return (
                <button
                  key={pen.path}
                  type="button"
                  onClick={() => setHidden(toggleHidden(hidden, pen.path))}
                  style={{
                    ...styles.legendItem,
                    opacity: hidden.has(pen.path) ? 0.45 : 1,
                  }}
                >
                  <span style={{ ...styles.swatch, background: COLORS[index % COLORS.length] }} />
                  <span>{pen.path}</span>
                  <strong>{last ? formatNumber(last.value) : "--"}</strong>
                </button>
              );
            })}
          </div>
        ) : null}
      </section>
    );
  },
};

function normalizeTagPaths(
  propPaths: string[] | undefined,
  binding: BoundValue | undefined,
): string[] {
  if (binding?.value.type === "string") {
    return splitPaths(binding.value.value);
  }
  return (propPaths ?? []).filter((path) => path.trim().length > 0);
}

function splitPaths(value: string): string[] {
  return value
    .split(",")
    .map((path) => path.trim())
    .filter(Boolean);
}

function historyToPoints(points: HistoryPoint[]): ChartPoint[] {
  return points
    .map((point) => ({ tsMs: point.ts_ms, value: numericValue(point.value), quality: point.quality }))
    .filter((point) => Number.isFinite(point.value));
}

function liveToPoint(live: BoundValue | undefined): ChartPoint | null {
  if (!live) {
    return null;
  }
  const value = numericValue(live.value);
  return Number.isFinite(value)
    ? { tsMs: live.ts, value, quality: live.quality }
    : null;
}

function numericValue(value: TagValue): number {
  return typeof value.value === "number" ? value.value : Number.NaN;
}

function limitPoints(points: ChartPoint[], maxPoints: number): ChartPoint[] {
  return points.slice(-Math.max(1, maxPoints || 600));
}

function computeDomain(
  series: PenSeries[],
  yMin: number | undefined,
  yMax: number | undefined,
  windowSeconds: number,
) {
  const all = series.flatMap((pen) => pen.points);
  const now = Date.now();
  const xMax = all.length > 0 ? Math.max(...all.map((point) => point.tsMs), now) : now;
  const xMin = xMax - Math.max(1, windowSeconds) * 1_000;
  const values = all.map((point) => point.value);
  const min = yMin ?? (values.length ? Math.min(...values) : 0);
  const max = yMax ?? (values.length ? Math.max(...values) : 1);
  const pad = min === max ? 1 : (max - min) * 0.08;
  return { xMin, xMax, yMin: min - pad, yMax: max + pad };
}

function pathForQuality(
  points: ChartPoint[],
  domain: ReturnType<typeof computeDomain>,
  good: boolean,
): string {
  return points
    .filter((point) => (good ? point.quality === "good" : point.quality !== "good"))
    .map((point, index) => `${index === 0 ? "M" : "L"} ${x(point.tsMs, domain)} ${y(point.value, domain)}`)
    .join(" ");
}

function x(tsMs: number, domain: ReturnType<typeof computeDomain>): number {
  const width = WIDTH - PADDING.left - PADDING.right;
  return PADDING.left + ((tsMs - domain.xMin) / (domain.xMax - domain.xMin || 1)) * width;
}

function y(value: number, domain: ReturnType<typeof computeDomain>): number {
  const height = HEIGHT - PADDING.top - PADDING.bottom;
  return PADDING.top + (1 - (value - domain.yMin) / (domain.yMax - domain.yMin || 1)) * height;
}

function gridLines(domain: ReturnType<typeof computeDomain>) {
  return [0, 0.25, 0.5, 0.75, 1].map((ratio) => {
    const value = domain.yMax - (domain.yMax - domain.yMin) * ratio;
    return {
      y: PADDING.top + (HEIGHT - PADDING.top - PADDING.bottom) * ratio,
      label: formatNumber(value),
    };
  });
}

function toggleHidden(hidden: Set<string>, path: string): Set<string> {
  const next = new Set(hidden);
  if (next.has(path)) {
    next.delete(path);
  } else {
    next.add(path);
  }
  return next;
}

function formatNumber(value: number): string {
  return Math.abs(value) >= 100 ? value.toFixed(0) : value.toFixed(2);
}

function formatAxisTime(tsMs: number): string {
  return new Date(tsMs).toLocaleTimeString([], { minute: "2-digit", second: "2-digit" });
}

const styles = {
  shell: {
    display: "grid",
    gap: 10,
    minWidth: 0,
    padding: 12,
    border: "1px solid #cbd2d9",
    borderRadius: 8,
    background: "#ffffff",
    color: "#1f2933",
    fontFamily: baseFont,
  },
  loading: {
    color: "#697586",
    fontSize: 13,
  },
  svg: {
    width: "100%",
    minHeight: 180,
    border: "1px solid #e5e7eb",
    borderRadius: 6,
  },
  legend: {
    display: "flex",
    flexWrap: "wrap" as const,
    gap: 8,
  },
  legendItem: {
    display: "inline-flex",
    alignItems: "center",
    gap: 6,
    minHeight: 30,
    padding: "4px 8px",
    border: "1px solid #d9e2ec",
    borderRadius: 6,
    background: "#ffffff",
    color: "#1f2933",
    font: "inherit",
    fontSize: 12,
  },
  swatch: {
    width: 12,
    height: 12,
    borderRadius: 3,
  },
};
