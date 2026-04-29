import type { DesignerProject } from "../lib/designerClient";

/** Props for the project explorer tree. */
export type ProjectExplorerProps = {
  project: DesignerProject;
  selectedViewId: string | null;
  selectedModule: "views" | "alarms" | "scripts";
  onOpenView: (viewId: string) => void;
  onOpenAlarms: () => void;
  onOpenScripts: () => void;
  onAddView: () => void;
  onRenameView: (viewId: string) => void;
};

/** Project tree for views, drivers, and tags. */
export function ProjectExplorer({
  project,
  selectedViewId,
  selectedModule,
  onOpenView,
  onOpenAlarms,
  onOpenScripts,
  onAddView,
  onRenameView,
}: ProjectExplorerProps) {
  return (
    <aside style={styles.panel} aria-label="Project explorer">
      <div style={styles.header}>
        <div>
          <h2 style={styles.heading}>{project.name}</h2>
          <div style={styles.meta}>{project.id}</div>
        </div>
        <button type="button" onClick={onAddView} style={styles.smallButton}>
          Add View
        </button>
      </div>

      <section>
        <h3 style={styles.sectionTitle}>Configuration</h3>
        <button
          type="button"
          onClick={onOpenAlarms}
          style={{
            ...styles.treeButton,
            ...(selectedModule === "alarms" ? styles.selected : null),
          }}
        >
          Alarms
        </button>
        <button
          type="button"
          onClick={onOpenScripts}
          style={{
            ...styles.treeButton,
            ...(selectedModule === "scripts" ? styles.selected : null),
          }}
        >
          Scripts
        </button>
      </section>

      <section>
        <h3 style={styles.sectionTitle}>Views</h3>
        <div style={styles.list}>
          {project.views.map((view) => (
            <div key={view.id} style={styles.row}>
              <button
                type="button"
                onClick={() => onOpenView(view.id)}
                style={{
                  ...styles.treeButton,
                  ...(view.id === selectedViewId ? styles.selected : null),
                }}
              >
                {view.id}
              </button>
              <button
                type="button"
                onClick={() => onRenameView(view.id)}
                style={styles.iconButton}
                aria-label={`Rename ${view.id}`}
              >
                Rename
              </button>
            </div>
          ))}
        </div>
      </section>

      <section>
        <h3 style={styles.sectionTitle}>Drivers</h3>
        <div style={styles.list}>
          {project.drivers.map((driver) => (
            <div key={driver.id} style={styles.readOnlyRow}>
              {driver.id}
            </div>
          ))}
        </div>
      </section>

      <section>
        <h3 style={styles.sectionTitle}>Tags</h3>
        <div style={styles.list}>
          {project.tags.map((tag) => (
            <div key={tag.path} style={styles.readOnlyRow}>
              {tag.path}
            </div>
          ))}
        </div>
      </section>
    </aside>
  );
}

const styles = {
  panel: {
    minWidth: 260,
    padding: 16,
    borderRight: "1px solid #d9e2ec",
    background: "#ffffff",
    overflow: "auto",
  },
  header: {
    display: "flex",
    gap: 8,
    justifyContent: "space-between",
    alignItems: "flex-start",
    marginBottom: 18,
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
  sectionTitle: {
    margin: "18px 0 8px",
    color: "#52606d",
    fontSize: 12,
    textTransform: "uppercase" as const,
  },
  list: {
    display: "grid",
    gap: 4,
  },
  row: {
    display: "grid",
    gridTemplateColumns: "1fr auto",
    gap: 4,
  },
  treeButton: {
    minHeight: 30,
    padding: "4px 8px",
    border: "1px solid transparent",
    borderRadius: 6,
    background: "transparent",
    textAlign: "left" as const,
    font: "inherit",
  },
  selected: {
    borderColor: "#9cc7f2",
    background: "#eef6ff",
  },
  iconButton: {
    minHeight: 30,
    padding: "4px 8px",
    border: "1px solid #d9e2ec",
    borderRadius: 6,
    background: "#ffffff",
    fontSize: 12,
  },
  smallButton: {
    minHeight: 30,
    padding: "4px 8px",
    border: "1px solid #1f4e79",
    borderRadius: 6,
    background: "#1f4e79",
    color: "#ffffff",
    fontSize: 12,
    fontWeight: 700,
  },
  readOnlyRow: {
    padding: "5px 8px",
    borderRadius: 4,
    background: "#f8fafc",
    fontSize: 13,
  },
};
