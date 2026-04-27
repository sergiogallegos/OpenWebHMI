import { components } from "@openwebhmi/component-library";
import type { ComponentNode, View } from "@openwebhmi/protocol";
import {
  addChild,
  createComponent,
  duplicateComponent,
  moveComponent,
  removeComponent,
} from "../lib/viewMutations";

/** Props for the form-based view tree editor. */
export type ViewEditorProps = {
  view: View;
  selectedComponentId: string | null;
  onSelectComponent: (id: string) => void;
  onChange: (view: View) => void;
};

/** Nested card editor for the view component tree. */
export function ViewEditor({
  view,
  selectedComponentId,
  onSelectComponent,
  onChange,
}: ViewEditorProps) {
  return (
    <section style={styles.panel} aria-label="View editor">
      <div style={styles.header}>
        <div>
          <h2 style={styles.heading}>{view.title}</h2>
          <div style={styles.meta}>{view.id}</div>
        </div>
      </div>
      <ComponentCard
        node={view.root}
        depth={0}
        selectedComponentId={selectedComponentId}
        onSelectComponent={onSelectComponent}
        onChange={onChange}
        view={view}
      />
    </section>
  );
}

function ComponentCard({
  node,
  depth,
  selectedComponentId,
  onSelectComponent,
  onChange,
  view,
}: {
  node: ComponentNode;
  depth: number;
  selectedComponentId: string | null;
  onSelectComponent: (id: string) => void;
  onChange: (view: View) => void;
  view: View;
}) {
  const canHaveChildren = node.kind === "Container";
  const selected = node.id === selectedComponentId;

  return (
    <div
      style={{
        ...styles.card,
        marginLeft: depth * 18,
        ...(selected ? styles.selectedCard : null),
      }}
    >
      <div style={styles.cardHeader}>
        <button
          type="button"
          style={styles.cardTitle}
          onClick={() => onSelectComponent(node.id)}
        >
          <strong>{node.kind}</strong>
          <span style={styles.nodeId}>{node.id}</span>
        </button>
        <div style={styles.actions}>
          <button type="button" style={styles.action} onClick={() => onSelectComponent(node.id)}>
            Edit
          </button>
          <button
            type="button"
            style={styles.action}
            disabled={view.root.id === node.id}
            onClick={() => onChange(removeComponent(view, node.id))}
          >
            Delete
          </button>
          <button
            type="button"
            style={styles.action}
            disabled={view.root.id === node.id}
            onClick={() => onChange(moveComponent(view, node.id, "up"))}
          >
            Move up
          </button>
          <button
            type="button"
            style={styles.action}
            disabled={view.root.id === node.id}
            onClick={() => onChange(moveComponent(view, node.id, "down"))}
          >
            Move down
          </button>
          <button
            type="button"
            style={styles.action}
            disabled={view.root.id === node.id}
            onClick={() => onChange(duplicateComponent(view, node.id))}
          >
            Duplicate
          </button>
        </div>
      </div>

      {canHaveChildren ? (
        <label style={styles.addLabel}>
          Add child
          <select
            aria-label={`Add child to ${node.id}`}
            value=""
            onChange={(event) => {
              const kind = event.currentTarget.value;
              if (kind) {
                const child = createComponent(kind);
                onChange(addChild(view, node.id, child));
                onSelectComponent(child.id);
              }
            }}
            style={styles.select}
          >
            <option value="">Choose component</option>
            {components.map((component) => (
              <option key={component.kind} value={component.kind}>
                {component.kind}
              </option>
            ))}
          </select>
        </label>
      ) : null}

      <div style={styles.children}>
        {node.children.map((child) => (
          <ComponentCard
            key={child.id}
            node={child}
            depth={depth + 1}
            selectedComponentId={selectedComponentId}
            onSelectComponent={onSelectComponent}
            onChange={onChange}
            view={view}
          />
        ))}
      </div>
    </div>
  );
}

const styles = {
  panel: {
    minWidth: 0,
    padding: 16,
    overflow: "auto",
  },
  header: {
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
  card: {
    display: "grid",
    gap: 10,
    marginBottom: 10,
    padding: 12,
    border: "1px solid #d9e2ec",
    borderRadius: 8,
    background: "#ffffff",
  },
  selectedCard: {
    borderColor: "#1f4e79",
    boxShadow: "0 0 0 2px #d9ebff",
  },
  cardHeader: {
    display: "grid",
    gridTemplateColumns: "minmax(120px, 1fr) auto",
    gap: 8,
    alignItems: "center",
  },
  cardTitle: {
    display: "flex",
    gap: 8,
    alignItems: "baseline",
    padding: 0,
    border: 0,
    background: "transparent",
    color: "#1f2933",
    textAlign: "left" as const,
    font: "inherit",
  },
  nodeId: {
    color: "#697586",
    fontFamily: "ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace",
    fontSize: 12,
  },
  actions: {
    display: "flex",
    gap: 6,
    flexWrap: "wrap" as const,
    justifyContent: "flex-end",
  },
  action: {
    minHeight: 28,
    padding: "4px 8px",
    border: "1px solid #d9e2ec",
    borderRadius: 6,
    background: "#ffffff",
    fontSize: 12,
  },
  addLabel: {
    display: "grid",
    gridTemplateColumns: "80px minmax(160px, 240px)",
    gap: 8,
    alignItems: "center",
    color: "#52606d",
    fontSize: 13,
  },
  select: {
    minHeight: 32,
    border: "1px solid #9aa5b1",
    borderRadius: 6,
    background: "#ffffff",
  },
  children: {
    display: "grid",
    gap: 4,
  },
};
