import { useEffect, useMemo, useState } from "react";
import type { DesignerClient, DesignerScriptEvent } from "../lib/designerClient";

export type ScriptErrorPaneProps = {
  client: DesignerClient;
  projectId: string;
  selectedScriptId: string | null;
  onOpenScript: (scriptId: string, line?: number) => void;
};

type BufferedEvent = DesignerScriptEvent & { received_at: number };

/** Recent script event stream for the selected project. */
export function ScriptErrorPane({
  client,
  projectId,
  selectedScriptId,
  onOpenScript,
}: ScriptErrorPaneProps) {
  const [events, setEvents] = useState<BufferedEvent[]>([]);
  const [showAll, setShowAll] = useState(false);

  useEffect(() => {
    return client.subscribeScriptEvents(projectId, (event) => {
      setEvents((current) =>
        trimEvents([...current, { ...event, received_at: Date.now() }]),
      );
    });
  }, [client, projectId]);

  const visible = useMemo(
    () =>
      events.filter(
        (event) =>
          (showAll || event.event_kind === "error") &&
          (!selectedScriptId || event.script_id === selectedScriptId),
      ),
    [events, selectedScriptId, showAll],
  );

  return (
    <section style={styles.panel} aria-label="Recent script events">
      <header style={styles.header}>
        <div>
          <h2 style={styles.heading}>Recent events</h2>
          <div style={styles.meta}>{visible.length} shown</div>
        </div>
        <label style={styles.toggle}>
          <input
            type="checkbox"
            checked={showAll}
            onChange={(event) => setShowAll(event.currentTarget.checked)}
          />
          All
        </label>
      </header>
      <div style={styles.list}>
        {visible.length === 0 ? (
          <div style={styles.empty}>No script errors.</div>
        ) : (
          visible.map((event, index) => (
            <button
              key={`${event.script_id}-${event.received_at}-${index}`}
              type="button"
              style={styles.row}
              onClick={() => onOpenScript(event.script_id, lineFromMessage(event.message))}
            >
              <span style={styles.time}>
                {new Date(event.received_at).toLocaleTimeString()}
              </span>
              <span style={styles.kind}>{event.event_kind}</span>
              <span style={styles.script}>{event.script_id}</span>
              <span style={styles.message}>
                {firstLine(event.message ?? event.status ?? "")}
              </span>
            </button>
          ))
        )}
      </div>
    </section>
  );
}

function trimEvents(events: BufferedEvent[]): BufferedEvent[] {
  const byScript = new Map<string, BufferedEvent[]>();
  for (const event of events) {
    const list = byScript.get(event.script_id) ?? [];
    list.push(event);
    byScript.set(event.script_id, list.slice(-20));
  }
  return [...byScript.values()].flat().sort((a, b) => a.received_at - b.received_at).slice(-200);
}

function firstLine(message: string): string {
  return message.split(/\r?\n/).find(Boolean) ?? "";
}

export function lineFromMessage(message?: string | null): number | undefined {
  const match = message?.match(/line\s+(\d+)/i);
  return match ? Number(match[1]) : undefined;
}

const styles = {
  panel: {
    borderTop: "1px solid #d9e2ec",
    background: "#ffffff",
  },
  header: {
    display: "flex",
    justifyContent: "space-between",
    gap: 12,
    alignItems: "center",
    padding: "10px 12px",
  },
  heading: {
    margin: 0,
    fontSize: 15,
  },
  meta: {
    marginTop: 2,
    color: "#697586",
    fontSize: 12,
  },
  toggle: {
    display: "flex",
    gap: 6,
    alignItems: "center",
    fontSize: 12,
  },
  list: {
    maxHeight: 180,
    overflow: "auto",
    display: "grid",
    gap: 4,
    padding: "0 12px 12px",
  },
  row: {
    display: "grid",
    gridTemplateColumns: "76px 52px 120px minmax(0, 1fr)",
    gap: 8,
    alignItems: "center",
    minHeight: 30,
    border: "1px solid #d9e2ec",
    borderRadius: 6,
    background: "#ffffff",
    color: "#1f2933",
    textAlign: "left" as const,
    font: "inherit",
    fontSize: 12,
  },
  time: {
    color: "#697586",
  },
  kind: {
    fontWeight: 700,
  },
  script: {
    fontFamily: "ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace",
  },
  message: {
    overflow: "hidden",
    textOverflow: "ellipsis",
    whiteSpace: "nowrap" as const,
  },
  empty: {
    color: "#697586",
    fontSize: 13,
    padding: 8,
  },
};
