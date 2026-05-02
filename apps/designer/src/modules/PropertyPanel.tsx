import { useEffect, useMemo, useState } from "react";
import { componentRegistry } from "@openwebhmi/component-library";
import type { BindingSource, ComponentNode, View } from "@openwebhmi/protocol";
import {
  bindingFor,
  clearBinding,
  findNode,
  setBinding,
  setProp,
} from "../lib/viewMutations";

/** Props for the selected component property editor. */
export type PropertyPanelProps = {
  view: View | null;
  selectedComponentId: string | null;
  selectedTag: string | null;
  tagPaths: string[];
  debounceMs?: number;
  onChange: (view: View) => void;
};

/** Schema-driven property and binding editor for one component. */
export function PropertyPanel({
  view,
  selectedComponentId,
  selectedTag,
  tagPaths,
  debounceMs = 300,
  onChange,
}: PropertyPanelProps) {
  const [draftView, setDraftView] = useState<View | null>(view);
  const activeView = draftView ?? view;
  const node =
    activeView && selectedComponentId ? findNode(activeView, selectedComponentId) : null;
  const definition = node ? componentRegistry[node.kind] : null;
  const props = useMemo(() => recordProps(node?.props), [node]);

  useEffect(() => {
    setDraftView(view);
  }, [view, selectedComponentId]);

  useEffect(() => {
    if (!draftView || draftView === view) {
      return;
    }
    if (debounceMs === 0) {
      onChange(draftView);
      return;
    }
    const timer = setTimeout(() => onChange(draftView), debounceMs);
    return () => clearTimeout(timer);
  }, [debounceMs, draftView, onChange, view]);

  if (!view || !node || !definition) {
    return (
      <aside style={styles.panel} aria-label="Property panel">
        <h2 style={styles.heading}>Properties</h2>
        <div style={styles.empty}>Select a component.</div>
      </aside>
    );
  }

  const updateDraft = (next: View) => setDraftView(next);
  const sourceTag = selectedTag ?? tagPaths[0] ?? "";

  return (
    <aside style={styles.panel} aria-label="Property panel">
      <h2 style={styles.heading}>Properties</h2>
      <div style={styles.componentMeta}>
        <strong>{node.kind}</strong>
        <span>{node.id}</span>
      </div>

      <section style={styles.section}>
        <h3 style={styles.sectionTitle}>Props</h3>
        {Object.entries(definition.propsSchema).map(([prop, schema]) => (
          <label key={prop} style={styles.field}>
            {schema.label ?? prop}
            <PropInput
              type={schema.type}
              options={schema.options}
              value={props[prop] ?? schema.default ?? ""}
              onChange={(value) =>
                activeView && updateDraft(setProp(activeView, node.id, prop, value))
              }
            />
          </label>
        ))}
      </section>

      <section style={styles.section}>
        <h3 style={styles.sectionTitle}>Bindings</h3>
        {definition.bindableProps.length === 0 ? (
          <div style={styles.empty}>No bindable props.</div>
        ) : (
          definition.bindableProps.map((prop) => {
            const binding = bindingFor(node, prop);
            return (
              <div key={prop} style={styles.bindingRow}>
                <div>
                  <strong>{prop}</strong>
                  <div style={styles.bindingMeta}>
                    {binding ? describeBinding(binding.source) : "Unbound"}
                  </div>
                </div>
                <select
                  aria-label={`Binding mode for ${prop}`}
                  value={binding?.source.kind ?? "none"}
                  onChange={(event) => {
                    const mode = event.currentTarget.value;
                    if (mode === "none") {
                      activeView && updateDraft(clearBinding(activeView, node.id, prop));
                    } else {
                      activeView &&
                        updateDraft(
                          setBinding(activeView, node.id, prop, sourceForMode(mode, sourceTag)),
                        );
                    }
                  }}
                  style={styles.select}
                >
                  <option value="none">None</option>
                  <option value="tag">Tag</option>
                  <option value="constant">Constant</option>
                  <option value="expression">Expression</option>
                </select>
                <button
                  type="button"
                  disabled={!sourceTag}
                  onClick={() =>
                    activeView &&
                    updateDraft(
                        setBinding(activeView, node.id, prop, {
                          kind: "tag",
                          path: sourceTag,
                        }),
                      )
                  }
                  style={styles.button}
                >
                  Bind
                </button>
              </div>
            );
          })
        )}
      </section>
    </aside>
  );
}

function PropInput({
  type,
  options,
  value,
  onChange,
}: {
  type: string;
  options?: string[];
  value: unknown;
  onChange: (value: unknown) => void;
}) {
  if (type === "boolean") {
    return (
      <input
        type="checkbox"
        checked={Boolean(value)}
        onChange={(event) => onChange(event.currentTarget.checked)}
      />
    );
  }
  if (type === "select") {
    return (
      <select
        value={String(value)}
        onChange={(event) => onChange(event.currentTarget.value)}
        style={styles.input}
      >
        {(options ?? []).map((option) => (
          <option key={option} value={option}>
            {option}
          </option>
        ))}
      </select>
    );
  }
  if (type === "stringList") {
    return (
      <textarea
        value={Array.isArray(value) ? value.join(", ") : String(value)}
        onChange={(event) =>
          onChange(
            event.currentTarget.value
              .split(",")
              .map((item) => item.trim())
              .filter(Boolean),
          )
        }
        style={{ ...styles.input, minHeight: 70, resize: "vertical" }}
      />
    );
  }
  if (type === "objectList") {
    return (
      <textarea
        value={JSON.stringify(value ?? [], null, 2)}
        onChange={(event) => {
          try {
            const parsed = JSON.parse(event.currentTarget.value);
            onChange(Array.isArray(parsed) ? parsed : []);
          } catch {
            onChange(value);
          }
        }}
        style={{ ...styles.input, minHeight: 110, resize: "vertical", fontFamily: "monospace" }}
      />
    );
  }
  return (
    <input
      type={type === "number" ? "number" : type === "color" ? "color" : "text"}
      value={String(value)}
      onChange={(event) =>
        onChange(type === "number" ? Number(event.currentTarget.value) : event.currentTarget.value)
      }
      style={styles.input}
    />
  );
}

function sourceForMode(mode: string, path: string): BindingSource {
  if (mode === "tag") {
    return { kind: "tag", path };
  }
  if (mode === "expression") {
    return { kind: "expression", code: "" };
  }
  return { kind: "constant", value: "" };
}

function describeBinding(source: BindingSource): string {
  switch (source.kind) {
    case "tag":
      return source.path;
    case "expression":
      return "expression";
    case "constant":
      return "constant";
  }
}

function recordProps(value: ComponentNode["props"] | undefined): Record<string, unknown> {
  if (typeof value === "object" && value !== null && !Array.isArray(value)) {
    return value as Record<string, unknown>;
  }
  return {};
}

const styles = {
  panel: {
    minWidth: 300,
    padding: 16,
    borderLeft: "1px solid #d9e2ec",
    background: "#ffffff",
    overflow: "auto",
  },
  heading: {
    margin: "0 0 12px",
    fontSize: 18,
  },
  componentMeta: {
    display: "grid",
    gap: 2,
    padding: 10,
    borderRadius: 6,
    background: "#f8fafc",
    color: "#1f2933",
  },
  section: {
    marginTop: 18,
  },
  sectionTitle: {
    margin: "0 0 10px",
    color: "#52606d",
    fontSize: 12,
    textTransform: "uppercase" as const,
  },
  field: {
    display: "grid",
    gap: 6,
    marginBottom: 10,
    fontSize: 13,
    fontWeight: 600,
  },
  input: {
    minHeight: 32,
    boxSizing: "border-box" as const,
    padding: "4px 8px",
    border: "1px solid #9aa5b1",
    borderRadius: 6,
    font: "inherit",
  },
  select: {
    minHeight: 30,
    border: "1px solid #9aa5b1",
    borderRadius: 6,
    background: "#ffffff",
  },
  button: {
    minHeight: 30,
    border: "1px solid #1f4e79",
    borderRadius: 6,
    background: "#1f4e79",
    color: "#ffffff",
    fontWeight: 700,
  },
  bindingRow: {
    display: "grid",
    gridTemplateColumns: "1fr 96px 64px",
    gap: 8,
    alignItems: "center",
    marginBottom: 10,
  },
  bindingMeta: {
    marginTop: 3,
    color: "#697586",
    fontSize: 12,
  },
  empty: {
    color: "#697586",
    fontSize: 13,
  },
};
