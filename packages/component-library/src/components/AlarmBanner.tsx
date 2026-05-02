import { useEffect, useMemo, useState } from "react";
import type { AlarmEvent, ComponentDefinition } from "../types";
import { baseFont, designerStyle } from "./shared";

export type AlarmBannerProps = {
  projectId: string;
  showZero: boolean;
  clickAction: "none" | "scroll-to-alarm-table";
  events?: AlarmEvent[];
};

type Band = "critical" | "warning" | "info";

/** Compact active-alarm count strip grouped by priority band. */
export const AlarmBanner: ComponentDefinition<AlarmBannerProps> = {
  kind: "AlarmBanner",
  defaultProps: {
    projectId: "",
    showZero: false,
    clickAction: "none",
  },
  propsSchema: {
    projectId: { type: "string", label: "Project", default: "" },
    showZero: { type: "boolean", label: "Show zero", default: false },
    clickAction: {
      type: "select",
      label: "Click action",
      options: ["none", "scroll-to-alarm-table"],
      default: "none",
    },
    events: { type: "objectList", label: "Events", default: [] },
  },
  bindableProps: [],
  Render({ props, context }) {
    const projectId = props.projectId || (context.mode === "runtime" ? context.projectId : "") || "phase1-demo";
    const [events, setEvents] = useState<AlarmEvent[]>(props.events ?? []);

    useEffect(() => {
      if (props.events) {
        setEvents(props.events);
      }
    }, [props.events]);

    useEffect(() => {
      if (context.mode !== "runtime" || !context.onSubscribeAlarms || props.events) {
        return undefined;
      }
      return context.onSubscribeAlarms(
        { projectId, priorityMin: 1, priorityMax: 5 },
        (event) => setEvents((current) => mergeEvent(current, event)),
      );
    }, [context, projectId, props.events]);

    const counts = useMemo(() => countBands(events), [events]);
    const bands = (["critical", "warning", "info"] as const).filter(
      (band) => props.showZero || counts[band] > 0,
    );

    return (
      <section
        aria-label="Alarm banner"
        style={{
          ...styles.shell,
          ...designerStyle(context.mode === "designer" ? context.isSelected : false),
        }}
      >
        {bands.length === 0 ? <span style={styles.empty}>No active alarms</span> : null}
        {bands.map((band) => (
          <button
            key={band}
            type="button"
            aria-label={`${band} alarms`}
            onClick={() => {
              if (props.clickAction === "scroll-to-alarm-table") {
                document.querySelector("[aria-label='Alarms']")?.scrollIntoView();
              }
            }}
            style={{ ...styles.pill, ...bandStyle(band) }}
          >
            <span>{labelForBand(band)}</span>
            <strong>{counts[band]}</strong>
          </button>
        ))}
      </section>
    );
  },
};

function mergeEvent(current: AlarmEvent[], event: AlarmEvent): AlarmEvent[] {
  return [event, ...current.filter((item) => item.alarm_id !== event.alarm_id)];
}

function countBands(events: AlarmEvent[]): Record<Band, number> {
  const counts = { critical: 0, warning: 0, info: 0 };
  for (const event of events) {
    if (event.state !== "active" && event.state !== "acked") {
      continue;
    }
    counts[bandForPriority(event.priority)] += 1;
  }
  return counts;
}

function bandForPriority(priority: number): Band {
  if (priority <= 2) {
    return "critical";
  }
  if (priority === 3) {
    return "warning";
  }
  return "info";
}

function labelForBand(band: Band): string {
  switch (band) {
    case "critical":
      return "Critical";
    case "warning":
      return "Warning";
    case "info":
      return "Info";
  }
}

function bandStyle(band: Band) {
  switch (band) {
    case "critical":
      return { color: "#7f1d1d", background: "#fee2e2", borderColor: "#fecaca" };
    case "warning":
      return { color: "#78350f", background: "#fef3c7", borderColor: "#fde68a" };
    case "info":
      return { color: "#1e3a8a", background: "#dbeafe", borderColor: "#bfdbfe" };
  }
}

const styles = {
  shell: {
    display: "flex",
    flexWrap: "wrap" as const,
    gap: 8,
    minHeight: 38,
    alignItems: "center",
    padding: 8,
    border: "1px solid #cbd2d9",
    borderRadius: 8,
    background: "#ffffff",
    fontFamily: baseFont,
  },
  pill: {
    display: "inline-flex",
    gap: 8,
    alignItems: "center",
    minHeight: 30,
    padding: "5px 10px",
    border: "1px solid",
    borderRadius: 999,
    fontFamily: baseFont,
    cursor: "pointer",
  },
  empty: {
    color: "#697586",
    fontSize: 13,
  },
};
