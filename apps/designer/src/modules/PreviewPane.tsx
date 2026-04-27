/** Props for the runtime preview iframe. */
export type PreviewPaneProps = {
  runtimeUrl: string;
  projectId: string;
  viewId: string | null;
  reloadKey: number;
};

/** Runtime iframe preview for the currently selected view. */
export function PreviewPane({
  runtimeUrl,
  projectId,
  viewId,
  reloadKey,
}: PreviewPaneProps) {
  if (!viewId) {
    return (
      <section style={styles.panel} aria-label="Preview pane">
        <div style={styles.empty}>Open a view to preview it.</div>
      </section>
    );
  }

  const url = new URL(runtimeUrl);
  url.searchParams.set("project", projectId);
  url.searchParams.set("view", viewId);
  url.searchParams.set("preview", String(reloadKey));

  return (
    <section style={styles.panel} aria-label="Preview pane">
      <iframe title="Runtime preview" src={url.toString()} style={styles.iframe} />
    </section>
  );
}

const styles = {
  panel: {
    minHeight: 260,
    borderTop: "1px solid #d9e2ec",
    background: "#ffffff",
  },
  iframe: {
    display: "block",
    width: "100%",
    height: 320,
    border: 0,
  },
  empty: {
    padding: 16,
    color: "#697586",
    fontSize: 13,
  },
};
