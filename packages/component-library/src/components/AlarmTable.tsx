import { useEffect, useMemo, useState } from "react";
import type React from "react";
import type { AlarmEvent, ComponentDefinition } from "../types";
import { baseFont, designerStyle } from "./shared";

export type AlarmTableProps = {
  projectId: string;
  title: string;
  priorityBand: "all" | "critical" | "warning" | "info";
  stateFilter: "active" | "all";
  clearRetentionMs: number;
  maxEvents: number;
  events?: AlarmEvent[];
  onAck?: (alarmId: string, note?: string | null) => void;
};

type SortKey = "priority" | "state" | "time";

const STATE_RANK: Record<AlarmEvent["state"], number> = {
  active: 0,
  acked: 1,
  cleared: 2,
  clear: 3,
};

/** Live operator alarm table with priority/state filtering and ack actions. */
export const AlarmTable: ComponentDefinition<AlarmTableProps> = {
  kind: "AlarmTable",
  defaultProps: {
    projectId: "phase1-demo",
    title: "Alarms",
    priorityBand: "all",
    stateFilter: "active",
    clearRetentionMs: 5_000,
    maxEvents: 500,
  },
  propsSchema: {
    projectId: { type: "string", label: "Project", default: "phase1-demo" },
    title: { type: "string", label: "Title", default: "Alarms" },
    priorityBand: {
      type: "select",
      label: "Priority band",
      options: ["all", "critical", "warning", "info"],
      default: "all",
    },
    stateFilter: {
      type: "select",
      label: "State filter",
      options: ["active", "all"],
      default: "active",
    },
    clearRetentionMs: { type: "number", label: "Clear retention ms", default: 5_000 },
    maxEvents: { type: "number", label: "Max events", default: 500 },
    events: { type: "string", label: "Events" },
    onAck: { type: "string", label: "Ack callback" },
  },
  bindableProps: [],
  Render({ props, context }) {
    const projectId =
      props.projectId ||
      (context.mode === "runtime" ? context.projectId : undefined) ||
      "phase1-demo";
    const [events, setEvents] = useState<AlarmEvent[]>(() => props.events ?? []);
    const [priorityBand, setPriorityBand] = useState(props.priorityBand);
    const [stateFilter, setStateFilter] = useState(props.stateFilter);
    const [sortKey, setSortKey] = useState<SortKey>("priority");
    const [selectedId, setSelectedId] = useState<string | null>(null);
    const [note, setNote] = useState("");
    const [pinned, setPinned] = useState<Set<string>>(() => new Set());

    useEffect(() => {
      setPriorityBand(props.priorityBand);
    }, [props.priorityBand]);

    useEffect(() => {
      setStateFilter(props.stateFilter);
    }, [props.stateFilter]);

    useEffect(() => {
      if (props.events) {
        setEvents(limitEvents(props.events, props.maxEvents));
      }
    }, [props.events, props.maxEvents]);

    useEffect(() => {
      if (context.mode !== "runtime" || !context.onSubscribeAlarms || props.events) {
        return undefined;
      }
      return context.onSubscribeAlarms(
        { projectId, priorityMin: 1, priorityMax: 5 },
        (event) => {
          setEvents((current) => mergeEvent(current, event, props.maxEvents));
        },
      );
    }, [context, projectId, props.events, props.maxEvents]);

    useEffect(() => {
      const retentionMs = Math.max(0, props.clearRetentionMs);
      const cleared = events.filter(
        (event) => event.state === "cleared" && !pinned.has(event.alarm_id),
      );
      if (retentionMs === 0 || cleared.length === 0) {
        return undefined;
      }
      const timer = setTimeout(() => {
        const cutoff = Date.now() - retentionMs;
        setEvents((current) =>
          current.filter(
            (event) =>
              event.state !== "cleared" ||
              pinned.has(event.alarm_id) ||
              event.transitioned_at_ms > cutoff,
          ),
        );
      }, retentionMs);
      return () => clearTimeout(timer);
    }, [events, pinned, props.clearRetentionMs]);

    const visible = useMemo(
      () =>
        events
          .filter((event) => inPriorityBand(event.priority, priorityBand))
          .filter((event) =>
            stateFilter === "active"
              ? event.state === "active" || event.state === "acked"
              : true,
          )
          .sort((left, right) => compareAlarm(left, right, sortKey)),
      [events, priorityBand, sortKey, stateFilter],
    );

    const selected = visible.find((event) => event.alarm_id === selectedId) ?? null;
    const onAck = props.onAck ?? (context.mode === "runtime" ? context.onAckAlarm : undefined);

    return (
      <section
        aria-label={props.title}
        style={{
          ...styles.shell,
          ...designerStyle(context.mode === "designer" ? context.isSelected : false),
        }}
      >
        <header style={styles.header}>
          <div>
            <h3 style={styles.title}>{props.title}</h3>
            <div style={styles.meta}>{visible.length} visible</div>
          </div>
          <div style={styles.filters} aria-label="Alarm filters">
            {(["all", "critical", "warning", "info"] as const).map((band) => (
              <button
                key={band}
                type="button"
                style={{
                  ...styles.chip,
                  ...(priorityBand === band ? styles.chipActive : null),
                }}
                onClick={() => setPriorityBand(band)}
              >
                {band}
              </button>
            ))}
            {(["active", "all"] as const).map((state) => (
              <button
                key={state}
                type="button"
                style={{
                  ...styles.chip,
                  ...(stateFilter === state ? styles.chipActive : null),
                }}
                onClick={() => setStateFilter(state)}
              >
                {state}
              </button>
            ))}
          </div>
        </header>

        <div style={styles.sortBar}>
          {(["priority", "state", "time"] as const).map((key) => (
            <button
              key={key}
              type="button"
              style={{
                ...styles.sortButton,
                ...(sortKey === key ? styles.sortActive : null),
              }}
              onClick={() => setSortKey(key)}
            >
              Sort: {key}
            </button>
          ))}
        </div>

        <div role="table" style={styles.table}>
          <div role="row" style={{ ...styles.row, ...styles.headRow }}>
            <span>Pri</span>
            <span>Alarm</span>
            <span>Value</span>
            <span>State</span>
            <span>Time</span>
          </div>
          {visible.map((event) => (
            <button
              key={event.alarm_id}
              type="button"
              role="row"
              onClick={() => setSelectedId(event.alarm_id)}
              style={{
                ...styles.row,
                ...(event.state === "cleared" ? styles.cleared : null),
                ...(selectedId === event.alarm_id ? styles.selected : null),
              }}
            >
              <span style={priorityStyle(event.priority)}>{event.priority}</span>
              <span>
                <strong>{event.label}</strong>
                <small style={styles.path}>{event.tag_path}</small>
              </span>
              <span style={styles.value}>
                {formatValue(event.value)}
                {event.quality !== "good" ? (
                  <span title={`Quality: ${event.quality}`} style={styles.quality}>
                    ?
                  </span>
                ) : null}
              </span>
              <span>{event.state}</span>
              <span>{formatTime(event.transitioned_at_ms)}</span>
            </button>
          ))}
        </div>

        {visible.length === 0 ? <div style={styles.empty}>No matching alarms.</div> : null}

        {selected ? (
          <form
            style={styles.ackPanel}
            onSubmit={(event) => {
              event.preventDefault();
              onAck?.(selected.alarm_id, note || null);
              setNote("");
            }}
          >
            <div>
              <strong>{selected.label}</strong>
              <div style={styles.meta}>{selected.message}</div>
            </div>
            <input
              aria-label="Ack note"
              placeholder="Ack note"
              value={note}
              onChange={(event) => setNote(event.currentTarget.value)}
              style={styles.note}
            />
            <button type="submit" disabled={!onAck || selected.state === "cleared"} style={styles.ack}>
              Ack
            </button>
            <button
              type="button"
              style={styles.secondary}
              onClick={() =>
                setPinned((current) => {
                  const next = new Set(current);
                  if (next.has(selected.alarm_id)) {
                    next.delete(selected.alarm_id);
                  } else {
                    next.add(selected.alarm_id);
                  }
                  return next;
                })
              }
            >
              {pinned.has(selected.alarm_id) ? "Unpin" : "Pin"}
            </button>
          </form>
        ) : null}
      </section>
    );
  },
};

function mergeEvent(
  current: AlarmEvent[],
  event: AlarmEvent,
  maxEvents: number,
): AlarmEvent[] {
  return limitEvents(
    [event, ...current.filter((item) => item.alarm_id !== event.alarm_id)],
    maxEvents,
  );
}

function limitEvents(events: AlarmEvent[], maxEvents: number): AlarmEvent[] {
  return events.slice(0, Math.max(1, maxEvents || 500));
}

function compareAlarm(left: AlarmEvent, right: AlarmEvent, key: SortKey): number {
  switch (key) {
    case "state":
      return STATE_RANK[left.state] - STATE_RANK[right.state] || compareAlarm(left, right, "priority");
    case "time":
      return right.transitioned_at_ms - left.transitioned_at_ms;
    case "priority":
    default:
      return left.priority - right.priority || right.transitioned_at_ms - left.transitioned_at_ms;
  }
}

function inPriorityBand(priority: number, band: AlarmTableProps["priorityBand"]): boolean {
  switch (band) {
    case "critical":
      return priority <= 2;
    case "warning":
      return priority === 3;
    case "info":
      return priority >= 4;
    default:
      return true;
  }
}

function priorityStyle(priority: number): React.CSSProperties {
  const color = priority <= 2 ? "#991b1b" : priority === 3 ? "#92400e" : "#1f4e79";
  const background = priority <= 2 ? "#fee2e2" : priority === 3 ? "#fef3c7" : "#dbeafe";
  return {
    display: "inline-flex",
    justifyContent: "center",
    width: 24,
    padding: "2px 0",
    borderRadius: 999,
    background,
    color,
    fontWeight: 700,
  };
}

function formatValue(value: AlarmEvent["value"]): string {
  return String(value.value);
}

function formatTime(ts: number): string {
  if (ts <= 0) {
    return "--";
  }
  return new Date(ts).toLocaleTimeString();
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
  header: {
    display: "grid",
    gridTemplateColumns: "minmax(160px, 1fr) auto",
    gap: 12,
    alignItems: "start",
  },
  title: {
    margin: 0,
    fontSize: 16,
  },
  meta: {
    marginTop: 3,
    color: "#697586",
    fontSize: 12,
  },
  filters: {
    display: "flex",
    flexWrap: "wrap" as const,
    gap: 6,
    justifyContent: "flex-end",
  },
  chip: {
    minHeight: 28,
    padding: "4px 8px",
    borderStyle: "solid",
    borderWidth: 1,
    borderColor: "#cbd2d9",
    borderRadius: 999,
    background: "#ffffff",
    color: "#52606d",
    font: "inherit",
    fontSize: 12,
  },
  chipActive: {
    borderColor: "#1f4e79",
    background: "#e0f2fe",
    color: "#1f4e79",
  },
  sortBar: {
    display: "flex",
    gap: 6,
    flexWrap: "wrap" as const,
  },
  sortButton: {
    minHeight: 28,
    padding: "4px 8px",
    borderStyle: "solid",
    borderWidth: 1,
    borderColor: "#d9e2ec",
    borderRadius: 6,
    background: "#ffffff",
    color: "#52606d",
    fontSize: 12,
  },
  sortActive: {
    borderColor: "#1f4e79",
    color: "#1f4e79",
  },
  table: {
    display: "grid",
    overflowX: "auto" as const,
  },
  row: {
    display: "grid",
    gridTemplateColumns: "42px minmax(180px, 1.4fr) minmax(90px, 0.8fr) 84px 92px",
    gap: 10,
    alignItems: "center",
    minWidth: 620,
    padding: "8px 10px",
    border: 0,
    borderTop: "1px solid #eef2f7",
    background: "#ffffff",
    color: "inherit",
    textAlign: "left" as const,
    font: "inherit",
    fontSize: 13,
  },
  headRow: {
    borderTop: 0,
    color: "#52606d",
    fontSize: 12,
    fontWeight: 700,
    textTransform: "uppercase" as const,
  },
  selected: {
    background: "#eef6ff",
  },
  cleared: {
    opacity: 0.58,
  },
  path: {
    display: "block",
    marginTop: 2,
    color: "#697586",
    fontSize: 11,
  },
  value: {
    display: "inline-flex",
    gap: 6,
    alignItems: "center",
  },
  quality: {
    display: "inline-flex",
    justifyContent: "center",
    width: 18,
    height: 18,
    borderRadius: 999,
    background: "#fef3c7",
    color: "#92400e",
    fontWeight: 700,
  },
  empty: {
    padding: 18,
    border: "1px dashed #cbd2d9",
    borderRadius: 6,
    color: "#697586",
    textAlign: "center" as const,
  },
  ackPanel: {
    display: "grid",
    gridTemplateColumns: "minmax(180px, 1fr) minmax(160px, 260px) auto auto",
    gap: 8,
    alignItems: "center",
    padding: 10,
    border: "1px solid #d9e2ec",
    borderRadius: 6,
    background: "#f8fafc",
  },
  note: {
    minHeight: 32,
    border: "1px solid #9aa5b1",
    borderRadius: 6,
    padding: "4px 8px",
  },
  ack: {
    minHeight: 32,
    padding: "4px 10px",
    border: "1px solid #1f4e79",
    borderRadius: 6,
    background: "#1f4e79",
    color: "#ffffff",
    fontWeight: 700,
  },
  secondary: {
    minHeight: 32,
    padding: "4px 10px",
    border: "1px solid #cbd2d9",
    borderRadius: 6,
    background: "#ffffff",
  },
};
