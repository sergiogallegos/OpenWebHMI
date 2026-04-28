import { useEffect, useMemo, useState } from "react";
import type { DesignerAlarm, DesignerAlarmCondition, DesignerTag } from "../lib/designerClient";

export type AlarmConfigProps = {
  alarms: DesignerAlarm[];
  tags: DesignerTag[];
  saving: boolean;
  onSave: (alarms: DesignerAlarm[]) => Promise<void> | void;
};

const CONDITION_LABELS: Record<DesignerAlarmCondition["kind"], string> = {
  high_limit: "HighLimit",
  low_limit: "LowLimit",
  equals: "Equals",
  deviation: "Deviation",
  digital: "Digital",
};

/** Form-based project alarm definition editor. */
export function AlarmConfig({ alarms, tags, saving, onSave }: AlarmConfigProps) {
  const [drafts, setDrafts] = useState(alarms);
  const [selectedId, setSelectedId] = useState<string | null>(alarms[0]?.id ?? null);
  const [dirty, setDirty] = useState(false);

  useEffect(() => {
    setDrafts(alarms);
    setSelectedId((current) => current ?? alarms[0]?.id ?? null);
    setDirty(false);
  }, [alarms]);

  useEffect(() => {
    if (!dirty) {
      return undefined;
    }
    const timer = setTimeout(() => {
      void onSave(drafts);
      setDirty(false);
    }, 300);
    return () => clearTimeout(timer);
  }, [dirty, drafts, onSave]);

  const byTag = useMemo(() => groupByTag(drafts), [drafts]);
  const selected = drafts.find((alarm) => alarm.id === selectedId) ?? drafts[0] ?? null;

  const updateSelected = (patch: Partial<DesignerAlarm>) => {
    if (!selected) {
      return;
    }
    updateAlarm(selected.id, patch);
  };

  const updateAlarm = (id: string, patch: Partial<DesignerAlarm>) => {
    setDrafts((current) =>
      current.map((alarm) => (alarm.id === id ? { ...alarm, ...patch } : alarm)),
    );
    setDirty(true);
  };

  const addAlarm = () => {
    const tagPath = tags[0]?.path ?? "";
    const next: DesignerAlarm = {
      id: uniqueAlarmId(drafts),
      label: "New alarm",
      priority: 3,
      tag_path: tagPath,
      condition: { kind: "high_limit", threshold: 100 },
      message: "Alarm active: {value}",
      enabled: true,
      require_ack: true,
    };
    setDrafts((current) => [...current, next]);
    setSelectedId(next.id);
    setDirty(true);
  };

  const deleteSelected = () => {
    if (!selected) {
      return;
    }
    setDrafts((current) => current.filter((alarm) => alarm.id !== selected.id));
    setSelectedId(null);
    setDirty(true);
  };

  return (
    <section style={styles.panel} aria-label="Alarm configuration">
      <header style={styles.header}>
        <div>
          <h2 style={styles.heading}>Alarms</h2>
          <div style={styles.meta}>
            {drafts.length} definitions · {saving ? "saving" : dirty ? "pending" : "saved"}
          </div>
        </div>
        <button type="button" onClick={addAlarm} style={styles.primary}>
          Add alarm
        </button>
      </header>

      <div style={styles.grid}>
        <aside style={styles.tree} aria-label="Alarms by tag">
          {Object.entries(byTag).map(([tag, items]) => (
            <section key={tag}>
              <h3 style={styles.tagHeading}>{tag || "Unassigned"}</h3>
              <div style={styles.list}>
                {items.map((alarm) => (
                  <button
                    key={alarm.id}
                    type="button"
                    onClick={() => setSelectedId(alarm.id)}
                    style={{
                      ...styles.alarmButton,
                      ...(alarm.id === selected?.id ? styles.selected : null),
                    }}
                  >
                    <span>{alarm.label}</span>
                    <small>
                      {CONDITION_LABELS[alarm.condition.kind]} · P{alarm.priority}
                    </small>
                  </button>
                ))}
              </div>
            </section>
          ))}
          {drafts.length === 0 ? <div style={styles.empty}>No alarm definitions.</div> : null}
        </aside>

        {selected ? (
          <form style={styles.form} onSubmit={(event) => event.preventDefault()}>
            <label style={styles.field}>
              Id
              <input
                value={selected.id}
                onChange={(event) => updateSelected({ id: event.currentTarget.value })}
                style={styles.input}
              />
            </label>
            <label style={styles.field}>
              Label
              <input
                value={selected.label}
                onChange={(event) => updateSelected({ label: event.currentTarget.value })}
                style={styles.input}
              />
            </label>
            <label style={styles.field}>
              Tag
              <select
                value={selected.tag_path}
                onChange={(event) => updateSelected({ tag_path: event.currentTarget.value })}
                style={styles.input}
              >
                {tags.map((tag) => (
                  <option key={tag.path} value={tag.path}>
                    {tag.path}
                  </option>
                ))}
              </select>
            </label>
            <label style={styles.field}>
              Priority
              <select
                value={selected.priority}
                onChange={(event) => updateSelected({ priority: Number(event.currentTarget.value) })}
                style={styles.input}
              >
                {[1, 2, 3, 4, 5].map((priority) => (
                  <option key={priority} value={priority}>
                    {priority}
                  </option>
                ))}
              </select>
            </label>
            <label style={styles.field}>
              Condition
              <select
                value={selected.condition.kind}
                onChange={(event) =>
                  updateSelected({
                    condition: defaultCondition(
                      event.currentTarget.value as DesignerAlarmCondition["kind"],
                    ),
                  })
                }
                style={styles.input}
              >
                {Object.entries(CONDITION_LABELS).map(([kind, label]) => (
                  <option key={kind} value={kind}>
                    {label}
                  </option>
                ))}
              </select>
            </label>
            <ConditionFields
              condition={selected.condition}
              onChange={(condition) => updateSelected({ condition })}
            />
            <label style={{ ...styles.field, ...styles.wide }}>
              Message template
              <input
                value={selected.message}
                onChange={(event) => updateSelected({ message: event.currentTarget.value })}
                style={styles.input}
              />
            </label>
            <label style={styles.check}>
              <input
                type="checkbox"
                checked={selected.enabled}
                onChange={(event) => updateSelected({ enabled: event.currentTarget.checked })}
              />
              Enabled
            </label>
            <label style={styles.check}>
              <input
                type="checkbox"
                checked={selected.require_ack}
                onChange={(event) => updateSelected({ require_ack: event.currentTarget.checked })}
              />
              Require ack
            </label>
            <div style={styles.actions}>
              <button type="button" onClick={() => void onSave(drafts)} style={styles.secondary}>
                Save now
              </button>
              <button type="button" onClick={deleteSelected} style={styles.danger}>
                Delete
              </button>
            </div>
          </form>
        ) : (
          <div style={styles.empty}>Select or add an alarm definition.</div>
        )}
      </div>
    </section>
  );
}

function ConditionFields({
  condition,
  onChange,
}: {
  condition: DesignerAlarmCondition;
  onChange: (condition: DesignerAlarmCondition) => void;
}) {
  switch (condition.kind) {
    case "high_limit":
    case "low_limit":
      return (
        <label style={styles.field}>
          Threshold
          <input
            aria-label="Alarm threshold"
            type="number"
            value={condition.threshold}
            onChange={(event) =>
              onChange({ ...condition, threshold: Number(event.currentTarget.value) })
            }
            style={styles.input}
          />
        </label>
      );
    case "deviation":
      return (
        <>
          <label style={styles.field}>
            Setpoint
            <input
              type="number"
              value={condition.setpoint}
              onChange={(event) =>
                onChange({ ...condition, setpoint: Number(event.currentTarget.value) })
              }
              style={styles.input}
            />
          </label>
          <label style={styles.field}>
            Tolerance
            <input
              type="number"
              value={condition.tolerance}
              onChange={(event) =>
                onChange({ ...condition, tolerance: Number(event.currentTarget.value) })
              }
              style={styles.input}
            />
          </label>
        </>
      );
    case "digital":
      return (
        <label style={styles.check}>
          <input
            type="checkbox"
            checked={condition.active_when}
            onChange={(event) =>
              onChange({ ...condition, active_when: event.currentTarget.checked })
            }
          />
          Active when true
        </label>
      );
    case "equals":
      return (
        <label style={styles.field}>
          Equals value
          <input
            value={String(condition.value)}
            onChange={(event) => onChange({ ...condition, value: event.currentTarget.value })}
            style={styles.input}
          />
        </label>
      );
  }
}

function defaultCondition(kind: DesignerAlarmCondition["kind"]): DesignerAlarmCondition {
  switch (kind) {
    case "low_limit":
      return { kind, threshold: 0 };
    case "equals":
      return { kind, value: "fault" };
    case "deviation":
      return { kind, setpoint: 100, tolerance: 5 };
    case "digital":
      return { kind, active_when: true };
    case "high_limit":
    default:
      return { kind: "high_limit", threshold: 100 };
  }
}

function groupByTag(alarms: DesignerAlarm[]): Record<string, DesignerAlarm[]> {
  return alarms.reduce<Record<string, DesignerAlarm[]>>((groups, alarm) => {
    const items = groups[alarm.tag_path] ?? [];
    groups[alarm.tag_path] = [...items, alarm];
    return groups;
  }, {});
}

function uniqueAlarmId(alarms: DesignerAlarm[]): string {
  let index = alarms.length + 1;
  let id = `alarm-${index}`;
  const existing = new Set(alarms.map((alarm) => alarm.id));
  while (existing.has(id)) {
    index += 1;
    id = `alarm-${index}`;
  }
  return id;
}

const styles = {
  panel: {
    minWidth: 0,
    padding: 16,
    overflow: "auto",
  },
  header: {
    display: "flex",
    justifyContent: "space-between",
    gap: 12,
    alignItems: "flex-start",
    marginBottom: 12,
  },
  heading: {
    margin: 0,
    fontSize: 18,
  },
  meta: {
    marginTop: 4,
    color: "#697586",
    fontSize: 12,
  },
  primary: {
    minHeight: 32,
    padding: "5px 10px",
    border: "1px solid #1f4e79",
    borderRadius: 6,
    background: "#1f4e79",
    color: "#ffffff",
    fontWeight: 700,
  },
  grid: {
    display: "grid",
    gridTemplateColumns: "minmax(220px, 0.8fr) minmax(320px, 1.2fr)",
    gap: 12,
  },
  tree: {
    display: "grid",
    gap: 12,
    alignContent: "start",
  },
  tagHeading: {
    margin: "0 0 6px",
    color: "#52606d",
    fontSize: 12,
  },
  list: {
    display: "grid",
    gap: 5,
  },
  alarmButton: {
    display: "grid",
    gap: 3,
    padding: "8px 10px",
    border: "1px solid #d9e2ec",
    borderRadius: 6,
    background: "#ffffff",
    color: "#1f2933",
    textAlign: "left" as const,
  },
  selected: {
    borderColor: "#1f4e79",
    background: "#eef6ff",
  },
  form: {
    display: "grid",
    gridTemplateColumns: "repeat(2, minmax(160px, 1fr))",
    gap: 10,
    alignContent: "start",
    padding: 12,
    border: "1px solid #d9e2ec",
    borderRadius: 8,
    background: "#ffffff",
  },
  field: {
    display: "grid",
    gap: 4,
    color: "#52606d",
    fontSize: 13,
  },
  wide: {
    gridColumn: "1 / -1",
  },
  input: {
    minHeight: 34,
    border: "1px solid #9aa5b1",
    borderRadius: 6,
    padding: "5px 8px",
    background: "#ffffff",
    color: "#1f2933",
  },
  check: {
    display: "flex",
    gap: 8,
    alignItems: "center",
    color: "#1f2933",
    fontSize: 13,
  },
  actions: {
    gridColumn: "1 / -1",
    display: "flex",
    gap: 8,
    justifyContent: "flex-end",
  },
  secondary: {
    minHeight: 32,
    padding: "5px 10px",
    border: "1px solid #cbd2d9",
    borderRadius: 6,
    background: "#ffffff",
  },
  danger: {
    minHeight: 32,
    padding: "5px 10px",
    border: "1px solid #fca5a5",
    borderRadius: 6,
    background: "#fef2f2",
    color: "#991b1b",
  },
  empty: {
    padding: 18,
    border: "1px dashed #cbd2d9",
    borderRadius: 6,
    color: "#697586",
  },
};
