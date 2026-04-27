import type { DesignerTag } from "../lib/designerClient";

/** Props for the tag browser. */
export type TagBrowserProps = {
  tags: DesignerTag[];
  selectedTag: string | null;
  onSelectTag: (path: string) => void;
};

/** List of known project and memory tags used by binding controls. */
export function TagBrowser({ tags, selectedTag, onSelectTag }: TagBrowserProps) {
  const memoryTags = tags
    .map((tag) => `system/drivers/${tag.driver}/status`)
    .filter((path, index, all) => all.indexOf(path) === index);
  const paths = [...tags.map((tag) => tag.path), ...memoryTags].sort();

  return (
    <aside style={styles.panel} aria-label="Tag browser">
      <h2 style={styles.heading}>Tags</h2>
      <div style={styles.list}>
        {paths.map((path) => (
          <button
            type="button"
            key={path}
            onClick={() => onSelectTag(path)}
            style={{
              ...styles.tagButton,
              ...(path === selectedTag ? styles.selected : null),
            }}
          >
            {path}
          </button>
        ))}
      </div>
    </aside>
  );
}

const styles = {
  panel: {
    minWidth: 260,
    padding: 16,
    borderLeft: "1px solid #d9e2ec",
    background: "#ffffff",
    overflow: "auto",
  },
  heading: {
    margin: "0 0 12px",
    fontSize: 18,
  },
  list: {
    display: "grid",
    gap: 6,
  },
  tagButton: {
    minHeight: 30,
    padding: "5px 8px",
    border: "1px solid #d9e2ec",
    borderRadius: 6,
    background: "#ffffff",
    textAlign: "left" as const,
    fontFamily: "ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace",
    fontSize: 12,
  },
  selected: {
    borderColor: "#1f4e79",
    background: "#eef6ff",
  },
};
