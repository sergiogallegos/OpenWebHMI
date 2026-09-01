import { defineConfig } from "vitest/config";
import react from "@vitejs/plugin-react";
import monacoEditorPluginModule from "vite-plugin-monaco-editor";
import { execFileSync } from "node:child_process";

const monacoEditorPlugin =
  (monacoEditorPluginModule as unknown as { default?: typeof monacoEditorPluginModule }).default ??
  monacoEditorPluginModule;

function buildCommit(): string {
  try {
    return execFileSync("git", ["rev-parse", "HEAD"], { encoding: "utf8" }).trim();
  } catch {
    // Source archives may not include .git; release automation must inject this value.
    return process.env.OPENWEBHMI_BUILD_COMMIT ?? "unknown";
  }
}

// vite-plugin-monaco-editor@1.1.0 calls fs.rmdirSync({ recursive: true }) which Node 22+ removed.
// The plugin only copies Monaco worker JS into dist/ for production, so gate it to `vite build`.
export default defineConfig(({ command }) => ({
  define: {
    __OPENWEBHMI_COMMIT__: JSON.stringify(buildCommit()),
    __OPENWEBHMI_VERSION__: JSON.stringify(process.env.OPENWEBHMI_VERSION ?? "0.0.1-dev"),
  },
  plugins: [
    react(),
    ...(command === "build"
      ? [monacoEditorPlugin({ languageWorkers: ["editorWorkerService"] })]
      : []),
  ],
  clearScreen: false,
  server: {
    port: 5174,
    strictPort: true,
  },
  test: {
    environment: "jsdom",
  },
}));
