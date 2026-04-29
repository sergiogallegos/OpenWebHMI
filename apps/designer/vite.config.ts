import { defineConfig } from "vitest/config";
import react from "@vitejs/plugin-react";
import monacoEditorPluginModule from "vite-plugin-monaco-editor";

const monacoEditorPlugin =
  (monacoEditorPluginModule as unknown as { default?: typeof monacoEditorPluginModule }).default ??
  monacoEditorPluginModule;

// vite-plugin-monaco-editor@1.1.0 calls fs.rmdirSync({ recursive: true }) which Node 22+ removed.
// The plugin only copies Monaco worker JS into dist/ for production, so gate it to `vite build`.
export default defineConfig(({ command }) => ({
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
