import { useEffect, useMemo, useState } from "react";
import {
  DEFAULT_THEME,
  applyTheme,
  type Theme,
  type ThemeMode,
  type ThemeVariables,
} from "@openwebhmi/protocol";

export type ThemeEditorProps = {
  theme?: Theme | null;
  saving: boolean;
  onSave: (theme: Theme) => void;
  onCancel?: () => void;
};

const colorFields = [
  ["primary_color", "Primary"],
  ["secondary_color", "Secondary"],
  ["background", "Background"],
  ["surface", "Surface"],
  ["text_primary", "Text"],
  ["text_secondary", "Muted text"],
  ["accent", "Accent"],
  ["error", "Error"],
  ["warning", "Warning"],
] as const;

const fontOptions = [
  DEFAULT_THEME.light.font_family,
  "ui-serif, Georgia, Cambria, serif",
  "ui-sans-serif, system-ui, sans-serif",
  "ui-monospace, SFMono-Regular, Menlo, monospace",
];

export function ThemeEditor({ theme, saving, onSave, onCancel }: ThemeEditorProps) {
  const savedTheme = useMemo(() => theme ?? DEFAULT_THEME, [theme]);
  const [draft, setDraft] = useState<Theme>(savedTheme);
  const active = draft.mode === "dark" ? draft.dark : draft.light;

  useEffect(() => {
    setDraft(savedTheme);
    applyTheme(savedTheme);
  }, [savedTheme]);

  useEffect(() => {
    applyTheme(draft);
  }, [draft]);

  const updateMode = (mode: ThemeMode) => {
    const next = { ...draft, mode };
    setDraft(next);
    window.localStorage.setItem("openwebhmi.themeMode", mode);
  };
  const updateActive = <K extends keyof ThemeVariables>(key: K, value: ThemeVariables[K]) => {
    setDraft((current) => ({
      ...current,
      [current.mode]: {
        ...current[current.mode],
        [key]: value,
      },
    }));
  };

  return (
    <section style={styles.panel} aria-label="Theme editor">
      <header style={styles.header}>
        <h2 style={styles.heading}>Theme</h2>
        <div style={styles.headerControls}>
          <label style={styles.packLabel}>
            Pack
            <select
              aria-label="Widget pack"
              value={draft.pack ?? ""}
              onChange={(event) =>
                setDraft((current) => ({
                  ...current,
                  pack: event.currentTarget.value || null,
                }))
              }
              style={styles.packSelect}
            >
              <option value="">Default</option>
              <option value="material">Material Design</option>
            </select>
          </label>
          <div style={styles.modeGroup}>
            <button
              type="button"
              onClick={() => updateMode("light")}
              style={{ ...styles.segment, ...(draft.mode === "light" ? styles.segmentActive : null) }}
            >
              Light
            </button>
            <button
              type="button"
              onClick={() => updateMode("dark")}
              style={{ ...styles.segment, ...(draft.mode === "dark" ? styles.segmentActive : null) }}
            >
              Dark
            </button>
          </div>
        </div>
      </header>

      <div style={styles.grid}>
        {colorFields.map(([key, label]) => (
          <label key={key} style={styles.field}>
            <span style={styles.label}>{label}</span>
            <span style={styles.colorRow}>
              <input
                aria-label={label}
                type="color"
                value={active[key]}
                onChange={(event) => updateActive(key, event.currentTarget.value)}
              />
              <input
                value={active[key]}
                onChange={(event) => updateActive(key, event.currentTarget.value)}
                style={styles.textInput}
              />
            </span>
          </label>
        ))}
        <label style={styles.fieldWide}>
          <span style={styles.label}>Font</span>
          <select
            value={fontOptions.includes(active.font_family) ? active.font_family : "custom"}
            onChange={(event) => {
              if (event.currentTarget.value !== "custom") {
                updateActive("font_family", event.currentTarget.value);
              }
            }}
            style={styles.textInput}
          >
            {fontOptions.map((font) => (
              <option key={font} value={font}>
                {font.split(",")[0].replace(/["']/g, "")}
              </option>
            ))}
            <option value="custom">Custom</option>
          </select>
          <input
            value={active.font_family}
            onChange={(event) => updateActive("font_family", event.currentTarget.value)}
            style={styles.textInput}
          />
        </label>
        {(["font_size_base", "spacing_unit", "border_radius"] as const).map((key) => (
          <label key={key} style={styles.field}>
            <span style={styles.label}>{key.replaceAll("_", " ")}</span>
            <input
              type="number"
              min={0}
              value={active[key]}
              onChange={(event) => updateActive(key, Number(event.currentTarget.value))}
              style={styles.textInput}
            />
          </label>
        ))}
      </div>

      <footer style={styles.actions}>
        <button
          type="button"
          onClick={() => {
            if (window.confirm("Reset theme to defaults?")) {
              setDraft(DEFAULT_THEME);
            }
          }}
          style={styles.secondary}
        >
          Reset
        </button>
        <button
          type="button"
          onClick={() => {
            setDraft(savedTheme);
            applyTheme(savedTheme);
            onCancel?.();
          }}
          style={styles.secondary}
        >
          Cancel
        </button>
        <button type="button" disabled={saving} onClick={() => onSave(draft)} style={styles.primary}>
          Save
        </button>
      </footer>
    </section>
  );
}

const styles = {
  panel: { padding: 16, overflow: "auto", background: "var(--surface, #ffffff)" },
  header: { display: "flex", justifyContent: "space-between", gap: 12, marginBottom: 12 },
  heading: { margin: 0, fontSize: 18 },
  headerControls: { display: "flex", alignItems: "center", gap: 10, flexWrap: "wrap" as const },
  packLabel: { display: "flex", alignItems: "center", gap: 6, color: "var(--text-secondary, #52606d)", fontSize: 12 },
  packSelect: { minHeight: 30, border: "1px solid #cbd2d9", borderRadius: 6, background: "#fff", padding: "3px 8px" },
  modeGroup: { display: "flex", gap: 4 },
  segment: { minHeight: 30, padding: "4px 10px", border: "1px solid #cbd2d9", background: "#fff" },
  segmentActive: { background: "var(--primary-color, #1f4e79)", color: "#fff" },
  grid: { display: "grid", gridTemplateColumns: "repeat(3, minmax(0, 1fr))", gap: 12 },
  field: { display: "grid", gap: 5, minWidth: 0 },
  fieldWide: { display: "grid", gap: 5, gridColumn: "1 / -1" },
  label: { color: "var(--text-secondary, #52606d)", fontSize: 12, textTransform: "uppercase" as const },
  colorRow: { display: "grid", gridTemplateColumns: "44px minmax(0, 1fr)", gap: 6 },
  textInput: { minHeight: 32, boxSizing: "border-box" as const, border: "1px solid #cbd2d9", borderRadius: 6, padding: "4px 8px" },
  actions: { display: "flex", justifyContent: "flex-end", gap: 8, marginTop: 14 },
  primary: { minHeight: 34, padding: "6px 12px", border: "1px solid var(--primary-color, #1f4e79)", borderRadius: 6, background: "var(--primary-color, #1f4e79)", color: "#fff", fontWeight: 700 },
  secondary: { minHeight: 34, padding: "6px 12px", border: "1px solid #cbd2d9", borderRadius: 6, background: "#fff" },
};
