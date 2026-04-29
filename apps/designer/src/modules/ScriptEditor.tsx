import Editor from "@monaco-editor/react";
import type { ElementType } from "react";
import { useEffect, useRef, useState } from "react";
import type { DesignerClient, DesignerScript } from "../lib/designerClient";
import { systemStubs } from "../lib/systemStubs";

export type ScriptEditorProps = {
  client: DesignerClient;
  projectId: string;
  script: DesignerScript | null;
  goToLine?: number | null;
  editorComponent?: ElementType<EditorLikeProps>;
};

type SaveState = "saved" | "saving" | "unsaved" | "error";

/** Monaco-backed Python source editor for project scripts. */
export function ScriptEditor({
  client,
  projectId,
  script,
  goToLine,
  editorComponent,
}: ScriptEditorProps) {
  const EditorComponent = (editorComponent ?? Editor) as ElementType<EditorLikeProps>;
  const [source, setSource] = useState("");
  const [saveState, setSaveState] = useState<SaveState>("saved");
  const [error, setError] = useState<string | null>(null);
  const editorRef = useRef<MinimalEditor | null>(null);

  useEffect(() => {
    let active = true;
    setError(null);
    setSaveState("saved");
    if (!script) {
      setSource("");
      return () => {
        active = false;
      };
    }
    client
      .loadScriptSource(projectId, script.id)
      .then((loaded) => {
        if (active) {
          setSource(loaded);
          setSaveState("saved");
        }
      })
      .catch((err: unknown) => {
        if (active) {
          setError(err instanceof Error ? err.message : String(err));
          setSaveState("error");
        }
      });
    return () => {
      active = false;
    };
  }, [client, projectId, script?.id]);

  useEffect(() => {
    if (!script || saveState !== "unsaved") {
      return;
    }
    const timer = window.setTimeout(() => {
      setSaveState("saving");
      client
        .saveScriptSource(projectId, script.id, source)
        .then(() => {
          setSaveState("saved");
          setError(null);
        })
        .catch((err: unknown) => {
          setSaveState("error");
          setError(err instanceof Error ? err.message : String(err));
        });
    }, 300);
    return () => window.clearTimeout(timer);
  }, [client, projectId, saveState, script, source]);

  useEffect(() => {
    if (goToLine && editorRef.current) {
      editorRef.current.revealLineInCenter(goToLine);
      editorRef.current.setPosition({ lineNumber: goToLine, column: 1 });
      editorRef.current.focus();
    }
  }, [goToLine, script?.id]);

  if (!script) {
    return <section style={styles.panel}>Select a script.</section>;
  }

  return (
    <section style={styles.panel} aria-label="Script editor">
      <header style={styles.header}>
        <div>
          <h2 style={styles.heading}>{script.id}</h2>
          <div style={styles.meta}>{script.path}</div>
        </div>
        <div role="status" style={saveState === "error" ? styles.errorState : styles.state}>
          {saveState}
        </div>
      </header>
      {error ? <div role="alert" style={styles.error}>{error}</div> : null}
      <EditorComponent
        height="520px"
        language="python"
        theme="vs-dark"
        value={source}
        options={{ minimap: { enabled: false }, tabSize: 4, fontSize: 13 }}
        beforeMount={registerCompletions}
        onMount={(editor: MinimalEditor) => {
          editorRef.current = editor;
        }}
        onChange={(value?: string) => {
          setSource(value ?? "");
          setSaveState("unsaved");
        }}
      />
    </section>
  );
}

type MinimalEditor = {
  revealLineInCenter: (line: number) => void;
  setPosition: (position: { lineNumber: number; column: number }) => void;
  focus: () => void;
};

type EditorLikeProps = {
  height?: string;
  language?: string;
  theme?: string;
  value?: string;
  options?: unknown;
  beforeMount?: (monaco: MonacoLike) => void;
  onMount?: (editor: MinimalEditor) => void;
  onChange?: (value?: string) => void;
};

function registerCompletions(monaco: MonacoLike) {
  monaco.languages.registerCompletionItemProvider("python", {
    provideCompletionItems: () => ({
      suggestions: systemStubs.map((stub) => ({
        label: stub.label,
        kind: monaco.languages.CompletionItemKind.Function,
        insertText: stub.insertText,
        insertTextRules:
          monaco.languages.CompletionItemInsertTextRule.InsertAsSnippet,
        documentation: stub.documentation,
      })),
    }),
  });
}

type MonacoLike = {
  languages: {
    CompletionItemKind: { Function: number };
    CompletionItemInsertTextRule: { InsertAsSnippet: number };
    registerCompletionItemProvider: (
      language: string,
      provider: {
        provideCompletionItems: () => {
          suggestions: Array<{
            label: string;
            kind: number;
            insertText: string;
            insertTextRules: number;
            documentation: string;
          }>;
        };
      },
    ) => void;
  };
};

const styles = {
  panel: {
    minWidth: 0,
    display: "grid",
    gridTemplateRows: "auto auto minmax(0, 1fr)",
    background: "#ffffff",
  },
  header: {
    display: "flex",
    justifyContent: "space-between",
    alignItems: "center",
    gap: 12,
    padding: 12,
    borderBottom: "1px solid #d9e2ec",
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
  state: {
    color: "#1f4e79",
    fontSize: 12,
    fontWeight: 700,
  },
  errorState: {
    color: "#b42318",
    fontSize: 12,
    fontWeight: 700,
  },
  error: {
    padding: "8px 12px",
    background: "#fff1f0",
    color: "#b42318",
    fontSize: 13,
  },
};
