import type { DesignerClient, DesignerScript } from "../lib/designerClient";
import { defaultScriptSource } from "../lib/systemStubs";

export type ScriptListProps = {
  client: DesignerClient;
  projectId: string;
  scripts: DesignerScript[];
  selectedScriptId: string | null;
  onSelect: (scriptId: string) => void;
  onScriptsChanged: (scripts: DesignerScript[]) => void;
};

/** Script list and basic add/rename/delete commands. */
export function ScriptList({
  client,
  projectId,
  scripts,
  selectedScriptId,
  onSelect,
  onScriptsChanged,
}: ScriptListProps) {
  const addScript = async () => {
    const id = window.prompt("New script id", `script-${scripts.length + 1}`);
    if (!id) {
      return;
    }
    const script: DesignerScript = {
      id,
      path: `${id}.py`,
      enabled: true,
      triggers: [{ kind: "on_tag_change", path: "rockwell-1/Pressure" }],
    };
    await client.saveScript(projectId, script);
    await client.saveScriptSource(projectId, id, defaultScriptSource());
    onScriptsChanged([...scripts.filter((item) => item.id !== id), script]);
    onSelect(id);
  };

  const renameScript = async (script: DesignerScript) => {
    const id = window.prompt("Script id", script.id);
    if (!id || id === script.id) {
      return;
    }
    const source = await client.loadScriptSource(projectId, script.id);
    const next = { ...script, id, path: `${id}.py` };
    await client.saveScript(projectId, next);
    await client.saveScriptSource(projectId, id, source);
    await client.deleteArtifact(projectId, { kind: "script_source", id: script.id });
    await client.deleteArtifact(projectId, { kind: "script", id: script.id });
    onScriptsChanged([...scripts.filter((item) => item.id !== script.id), next]);
    onSelect(id);
  };

  const deleteScript = async (script: DesignerScript) => {
    if (!window.confirm(`Delete ${script.id}?`)) {
      return;
    }
    await client.deleteArtifact(projectId, { kind: "script_source", id: script.id });
    await client.deleteArtifact(projectId, { kind: "script", id: script.id });
    const next = scripts.filter((item) => item.id !== script.id);
    onScriptsChanged(next);
    if (selectedScriptId === script.id && next[0]) {
      onSelect(next[0].id);
    }
  };

  return (
    <section style={styles.panel} aria-label="Scripts">
      <header style={styles.header}>
        <div>
          <h2 style={styles.heading}>Scripts</h2>
          <div style={styles.meta}>{scripts.length} configured</div>
        </div>
        <button type="button" style={styles.primary} onClick={() => void addScript()}>
          Add
        </button>
      </header>
      <div style={styles.list}>
        {scripts.map((script) => (
          <div key={script.id} style={styles.row}>
            <button
              type="button"
              style={{
                ...styles.scriptButton,
                ...(selectedScriptId === script.id ? styles.selected : null),
              }}
              onClick={() => onSelect(script.id)}
            >
              <span>{script.id}</span>
              <small style={styles.path}>{script.path}</small>
            </button>
            <button type="button" style={styles.action} onClick={() => void renameScript(script)}>
              Rename
            </button>
            <button type="button" style={styles.action} onClick={() => void deleteScript(script)}>
              Delete
            </button>
          </div>
        ))}
      </div>
    </section>
  );
}

const styles = {
  panel: {
    minWidth: 220,
    borderRight: "1px solid #d9e2ec",
    background: "#ffffff",
    overflow: "auto",
  },
  header: {
    display: "flex",
    justifyContent: "space-between",
    gap: 8,
    alignItems: "center",
    padding: 12,
    borderBottom: "1px solid #d9e2ec",
  },
  heading: {
    margin: 0,
    fontSize: 16,
  },
  meta: {
    marginTop: 3,
    color: "#697586",
    fontSize: 12,
  },
  primary: {
    minHeight: 30,
    border: "1px solid #1f4e79",
    borderRadius: 6,
    background: "#1f4e79",
    color: "#ffffff",
    fontSize: 12,
    fontWeight: 700,
  },
  list: {
    display: "grid",
    gap: 6,
    padding: 12,
  },
  row: {
    display: "grid",
    gridTemplateColumns: "1fr auto auto",
    gap: 4,
  },
  scriptButton: {
    display: "grid",
    gap: 2,
    minHeight: 42,
    padding: "5px 8px",
    border: "1px solid #d9e2ec",
    borderRadius: 6,
    background: "#ffffff",
    textAlign: "left" as const,
    font: "inherit",
  },
  selected: {
    borderColor: "#1f4e79",
    background: "#eef6ff",
  },
  path: {
    color: "#697586",
    fontSize: 11,
  },
  action: {
    minHeight: 30,
    padding: "4px 8px",
    border: "1px solid #d9e2ec",
    borderRadius: 6,
    background: "#ffffff",
    fontSize: 12,
  },
};
