import { useCallback, useMemo, useRef, useState } from "react";
import type { View } from "@openwebhmi/protocol";
import { AlarmConfig } from "./modules/AlarmConfig";
import { ConnectGateway } from "./modules/ConnectGateway";
import { PreviewPane } from "./modules/PreviewPane";
import { ProjectExplorer } from "./modules/ProjectExplorer";
import { PropertyPanel } from "./modules/PropertyPanel";
import { ScriptEditor } from "./modules/ScriptEditor";
import { ScriptErrorPane } from "./modules/ScriptErrorPane";
import { ScriptList } from "./modules/ScriptList";
import { TagBrowser } from "./modules/TagBrowser";
import { ThemeEditor } from "./modules/ThemeEditor";
import { UserAdmin } from "./modules/UserAdmin";
import { ViewEditor } from "./modules/ViewEditor";
import {
  DesignerClient,
  type DesignerAlarm,
  type DesignerProject,
  type DesignerProjectChange,
} from "./lib/designerClient";
import { createComponent, findNode } from "./lib/viewMutations";

const DEFAULT_GATEWAY_URL = "ws://localhost:8080";
const DEFAULT_RUNTIME_URL = "http://localhost:5173/";
const DEFAULT_PROJECT_ID = "phase1-demo";

type SaveState = "saved" | "saving" | "unsaved" | "error";

/** Root module for the form-based OpenWebHMI designer. */
export function App() {
  const clientRef = useRef<DesignerClient | null>(null);
  const [connection, setConnection] = useState<"disconnected" | "connecting" | "connected">(
    "disconnected",
  );
  const [connectionError, setConnectionError] = useState<string | null>(null);
  const [gatewayUrl, setGatewayUrl] = useState(DEFAULT_GATEWAY_URL);
  const [project, setProject] = useState<DesignerProject | null>(null);
  const [selectedViewId, setSelectedViewId] = useState<string | null>(null);
  const [selectedScriptId, setSelectedScriptId] = useState<string | null>(null);
  const [selectedModule, setSelectedModule] = useState<"views" | "alarms" | "scripts" | "theme">("views");
  const [scriptGoToLine, setScriptGoToLine] = useState<number | null>(null);
  const [selectedComponentId, setSelectedComponentId] = useState<string | null>(null);
  const [selectedTag, setSelectedTag] = useState<string | null>(null);
  const [saveState, setSaveState] = useState<SaveState>("saved");
  const [warning, setWarning] = useState<string | null>(null);
  const [previewReload, setPreviewReload] = useState(0);

  const selectedView = useMemo(
    () => project?.views.find((view) => view.id === selectedViewId) ?? null,
    [project, selectedViewId],
  );
  const tagPaths = useMemo(
    () => project?.tags.map((tag) => tag.path).sort() ?? [],
    [project],
  );
  const selectedScript = useMemo(
    () => project?.scripts.find((script) => script.id === selectedScriptId) ?? null,
    [project, selectedScriptId],
  );

  const connect = async (url: string, username: string, password: string) => {
    setConnection("connecting");
    setConnectionError(null);
    setGatewayUrl(url);
    const client = new DesignerClient(url);
    try {
      await client.login(username, password);
      await client.connect();
      client.subscribeProject(DEFAULT_PROJECT_ID, (change) =>
        handleProjectChange(change),
      );
      const loaded = await client.loadProject(DEFAULT_PROJECT_ID);
      clientRef.current = client;
      setProject(loaded);
      setSelectedViewId(loaded.views[0]?.id ?? null);
      setSelectedScriptId(loaded.scripts[0]?.id ?? null);
      setSelectedComponentId(loaded.views[0]?.root.id ?? null);
      setConnection("connected");
      setSaveState("saved");
    } catch (error) {
      client.disconnect();
      setConnection("disconnected");
      setConnectionError(error instanceof Error ? error.message : String(error));
    }
  };

  const handleProjectChange = useCallback((change: DesignerProjectChange) => {
    if (
      change.project_id === DEFAULT_PROJECT_ID &&
      (change.artifact.kind === "view" || change.artifact.kind === "alarms" || change.artifact.kind === "theme")
    ) {
      setPreviewReload((current) => current + 1);
      setWarning(
        `Project changed at version ${change.version}. Last write wins until collaboration locking is added.`,
      );
    }
  }, []);

  const saveView = useCallback(
    async (nextView: View) => {
      const client = clientRef.current;
      if (!client || !project) {
        return;
      }
      setSaveState("saving");
      setProject({
        ...project,
        views: upsertView(project.views, nextView),
      });
      try {
        const version = await client.saveView(project.id, nextView);
        setProject((current) =>
          current
            ? {
                ...current,
                version,
                views: upsertView(current.views, nextView),
              }
            : current,
        );
        setSaveState("saved");
        setPreviewReload((current) => current + 1);
      } catch (error) {
        setSaveState("error");
        setWarning(error instanceof Error ? error.message : String(error));
      }
    },
    [project],
  );

  const saveAlarms = useCallback(
    async (nextAlarms: DesignerAlarm[]) => {
      const client = clientRef.current;
      if (!client || !project) {
        return;
      }
      setSaveState("saving");
      setProject({
        ...project,
        alarms: nextAlarms,
      });
      try {
        const version = await client.saveAlarms(project.id, nextAlarms);
        setProject((current) =>
          current
            ? {
                ...current,
                version,
                alarms: nextAlarms,
              }
            : current,
        );
        setSaveState("saved");
        setPreviewReload((current) => current + 1);
      } catch (error) {
        setSaveState("error");
        setWarning(error instanceof Error ? error.message : String(error));
      }
    },
    [project],
  );

  const saveTheme = useCallback(
    async (theme: NonNullable<DesignerProject["theme"]>) => {
      const client = clientRef.current;
      if (!client || !project) {
        return;
      }
      setSaveState("saving");
      setProject({ ...project, theme });
      try {
        const version = await client.saveTheme(project.id, theme);
        setProject((current) => (current ? { ...current, version, theme } : current));
        setSaveState("saved");
        setPreviewReload((current) => current + 1);
      } catch (error) {
        setSaveState("error");
        setWarning(error instanceof Error ? error.message : String(error));
      }
    },
    [project],
  );

  const addView = async () => {
    if (!project) {
      return;
    }
    const id = window.prompt("New view id", `view-${project.views.length + 1}`);
    if (!id) {
      return;
    }
    const view: View = {
      id,
      title: id,
      schema_version: 1,
      root: createComponent("Container", "root"),
    };
    setProject({ ...project, views: [...project.views, view] });
    setSelectedModule("views");
    setSelectedViewId(id);
    setSelectedComponentId(view.root.id);
    await saveView(view);
  };

  const renameView = async (viewId: string) => {
    const view = project?.views.find((item) => item.id === viewId);
    if (!view) {
      return;
    }
    const title = window.prompt("View title", view.title);
    if (!title) {
      return;
    }
    await saveView({ ...view, title });
  };

  if (connection !== "connected" || !project) {
    return (
      <ConnectGateway
        defaultUrl={gatewayUrl}
        connecting={connection === "connecting"}
        error={connectionError}
        onConnect={connect}
      />
    );
  }

  return (
    <main style={styles.shell}>
      <ProjectExplorer
        project={project}
        selectedViewId={selectedViewId}
        selectedModule={selectedModule}
        onOpenView={(viewId) => {
          const view = project.views.find((item) => item.id === viewId);
          setSelectedModule("views");
          setSelectedViewId(viewId);
          setSelectedComponentId(view?.root.id ?? null);
        }}
        onOpenAlarms={() => setSelectedModule("alarms")}
        onOpenScripts={() => {
          setSelectedModule("scripts");
          setSelectedScriptId((current) => current ?? project.scripts[0]?.id ?? null);
        }}
        onOpenTheme={() => setSelectedModule("theme")}
        onAddView={addView}
        onRenameView={renameView}
      />
      <section style={styles.workbench}>
        <header style={styles.topBar}>
          <div>
            <h1 style={styles.title}>Designer</h1>
            <div style={styles.meta}>
              {gatewayUrl} - {project.id} v{project.version}
            </div>
          </div>
          <div
            role="status"
            style={{
              ...styles.savePill,
              ...(saveState === "error" ? styles.saveError : null),
            }}
          >
            {saveState}
          </div>
        </header>
        {warning ? (
          <div role="alert" style={styles.warning}>
            <span>{warning}</span>
            <button type="button" onClick={() => setWarning(null)} style={styles.dismiss}>
              Dismiss
            </button>
          </div>
        ) : null}
        <div style={selectedModule === "alarms" || selectedModule === "theme" ? styles.moduleGrid : styles.editorGrid}>
          {selectedModule === "alarms" ? (
            <AlarmConfig
              alarms={project.alarms}
              tags={project.tags}
              saving={saveState === "saving"}
              onSave={saveAlarms}
            />
          ) : selectedModule === "scripts" ? (
            <>
              <ScriptList
                client={clientRef.current!}
                projectId={project.id}
                scripts={project.scripts}
                selectedScriptId={selectedScriptId}
                onSelect={(scriptId) => {
                  setSelectedScriptId(scriptId);
                  setScriptGoToLine(null);
                }}
                onScriptsChanged={(scripts) =>
                  setProject((current) => (current ? { ...current, scripts } : current))
                }
              />
              <ScriptEditor
                client={clientRef.current!}
                projectId={project.id}
                script={selectedScript}
                goToLine={scriptGoToLine}
              />
            </>
          ) : selectedView ? (
            <>
              <ViewEditor
                view={selectedView}
                selectedComponentId={selectedComponentId}
                onSelectComponent={setSelectedComponentId}
                onChange={(view) => {
                  setSaveState("unsaved");
                  void saveView(view);
                }}
              />
              <PropertyPanel
                view={selectedView}
                selectedComponentId={selectedComponentId}
                selectedTag={selectedTag}
                tagPaths={tagPaths}
                onChange={(view) => {
                  const selectedStillExists = selectedComponentId
                    ? findNode(view, selectedComponentId)
                    : null;
                  setSelectedComponentId(selectedStillExists?.id ?? view.root.id);
                  setSaveState("unsaved");
                  void saveView(view);
                }}
              />
            </>
          ) : (
            <div style={styles.empty}>Open or add a view.</div>
          )}
        </div>
        <PreviewPane
          runtimeUrl={DEFAULT_RUNTIME_URL}
          projectId={project.id}
          viewId={selectedViewId}
          reloadKey={previewReload}
        />
        {selectedModule === "scripts" ? (
          <ScriptErrorPane
            client={clientRef.current!}
            projectId={project.id}
            selectedScriptId={selectedScriptId}
            onOpenScript={(scriptId, line) => {
              setSelectedModule("scripts");
              setSelectedScriptId(scriptId);
              setScriptGoToLine(line ?? null);
            }}
          />
        ) : null}
      </section>
      <TagBrowser
        tags={project.tags}
        selectedTag={selectedTag}
        onSelectTag={setSelectedTag}
      />
      <UserAdmin client={clientRef.current!} />
    </main>
  );
}

const styles = {
  shell: {
    minHeight: "100vh",
    display: "grid",
    gridTemplateColumns: "280px minmax(0, 1fr) 280px 320px",
    background: "#f4f6f8",
    color: "#1f2933",
    fontFamily:
      'Inter, ui-sans-serif, system-ui, -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif',
  },
  workbench: {
    minWidth: 0,
    display: "grid",
    gridTemplateRows: "auto auto minmax(0, 1fr) auto",
  },
  topBar: {
    display: "flex",
    justifyContent: "space-between",
    gap: 16,
    alignItems: "center",
    padding: "14px 16px",
    borderBottom: "1px solid #d9e2ec",
    background: "#ffffff",
  },
  title: {
    margin: 0,
    fontSize: 20,
  },
  meta: {
    marginTop: 3,
    color: "#697586",
    fontSize: 12,
  },
  savePill: {
    minWidth: 82,
    padding: "5px 8px",
    border: "1px solid #86efac",
    borderRadius: 6,
    background: "#ecfdf3",
    color: "#166534",
    textAlign: "center" as const,
    fontSize: 12,
    fontWeight: 700,
  },
  saveError: {
    borderColor: "#fca5a5",
    background: "#fef2f2",
    color: "#991b1b",
  },
  warning: {
    display: "flex",
    justifyContent: "space-between",
    gap: 12,
    padding: "8px 16px",
    borderBottom: "1px solid #fdba74",
    background: "#fff7ed",
    color: "#9a3412",
    fontSize: 13,
  },
  dismiss: {
    border: "1px solid #fdba74",
    borderRadius: 6,
    background: "#ffffff",
  },
  editorGrid: {
    minHeight: 0,
    display: "grid",
    gridTemplateColumns: "minmax(0, 1fr) 320px",
  },
  moduleGrid: {
    minHeight: 0,
    display: "grid",
  },
  empty: {
    padding: 24,
    color: "#697586",
  },
};

function upsertView(views: View[], nextView: View): View[] {
  return views.some((view) => view.id === nextView.id)
    ? views.map((view) => (view.id === nextView.id ? nextView : view))
    : [...views, nextView];
}
