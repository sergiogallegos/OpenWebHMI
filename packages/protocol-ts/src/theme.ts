export type ThemeMode = "light" | "dark";

export type ThemeVariables = {
  primary_color: string;
  secondary_color: string;
  background: string;
  surface: string;
  text_primary: string;
  text_secondary: string;
  accent: string;
  error: string;
  warning: string;
  font_family: string;
  font_size_base: number;
  spacing_unit: number;
  border_radius: number;
};

export type Theme = {
  light: ThemeVariables;
  dark: ThemeVariables;
  mode: ThemeMode;
  pack?: string | null;
};

export const DEFAULT_THEME: Theme = {
  mode: "light",
  pack: null,
  light: {
    primary_color: "#1f4e79",
    secondary_color: "#52606d",
    background: "#f4f6f8",
    surface: "#ffffff",
    text_primary: "#1f2933",
    text_secondary: "#52606d",
    accent: "#2563eb",
    error: "#dc2626",
    warning: "#f59e0b",
    font_family:
      'Inter, ui-sans-serif, system-ui, -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif',
    font_size_base: 14,
    spacing_unit: 8,
    border_radius: 6,
  },
  dark: {
    primary_color: "#60a5fa",
    secondary_color: "#94a3b8",
    background: "#111827",
    surface: "#1f2937",
    text_primary: "#f9fafb",
    text_secondary: "#cbd5e1",
    accent: "#38bdf8",
    error: "#f87171",
    warning: "#fbbf24",
    font_family:
      'Inter, ui-sans-serif, system-ui, -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif',
    font_size_base: 14,
    spacing_unit: 8,
    border_radius: 6,
  },
};

export function themeToCss(theme: Theme): string {
  return `:root { ${variablesToCss(theme.light)} }\n:root[data-theme="dark"] { ${variablesToCss(theme.dark)} }`;
}

export function applyTheme(theme: Theme, root: HTMLElement = document.documentElement) {
  const active = theme.mode === "dark" ? theme.dark : theme.light;
  for (const [name, value] of Object.entries(toCssVariables(active))) {
    root.style.setProperty(name, value);
  }
  root.dataset.theme = theme.mode;
}

function variablesToCss(variables: ThemeVariables): string {
  return Object.entries(toCssVariables(variables))
    .map(([name, value]) => `${name}: ${value};`)
    .join(" ");
}

function toCssVariables(variables: ThemeVariables): Record<string, string> {
  return {
    "--primary-color": variables.primary_color,
    "--secondary-color": variables.secondary_color,
    "--background": variables.background,
    "--surface": variables.surface,
    "--text-primary": variables.text_primary,
    "--text-secondary": variables.text_secondary,
    "--accent": variables.accent,
    "--error": variables.error,
    "--warning": variables.warning,
    "--font-family": variables.font_family,
    "--font-size-base": `${variables.font_size_base}px`,
    "--spacing-unit": `${variables.spacing_unit}px`,
    "--border-radius": `${variables.border_radius}px`,
  };
}
