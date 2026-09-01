import { defineConfig } from "vitest/config";
import react from "@vitejs/plugin-react";
import { execFileSync } from "node:child_process";

function buildCommit(): string {
  try {
    return execFileSync("git", ["rev-parse", "HEAD"], { encoding: "utf8" }).trim();
  } catch {
    // Source archives may not include .git; release automation must inject this value.
    return process.env.OPENWEBHMI_BUILD_COMMIT ?? "unknown";
  }
}

export default defineConfig({
  define: {
    __OPENWEBHMI_COMMIT__: JSON.stringify(buildCommit()),
    __OPENWEBHMI_VERSION__: JSON.stringify(process.env.OPENWEBHMI_VERSION ?? "0.0.1-dev"),
  },
  plugins: [react()],
  server: {
    port: 5173,
  },
  test: {
    environment: "jsdom",
  },
});
