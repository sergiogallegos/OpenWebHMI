/* @vitest-environment jsdom */
import { describe, expect, it } from "vitest";
import { DEFAULT_THEME } from "@openwebhmi/protocol";
import { applyRuntimeTheme, removeRuntimeTheme } from "../App";

describe("runtime theme loading", () => {
  it("injects a style block for a project theme", () => {
    applyRuntimeTheme(DEFAULT_THEME);

    const style = document.getElementById("openwebhmi-project-theme");
    expect(style?.textContent).toContain("--primary-color");
  });

  it("removes the style block when no theme is returned", () => {
    applyRuntimeTheme(DEFAULT_THEME);
    applyRuntimeTheme(null);

    expect(document.getElementById("openwebhmi-project-theme")).toBeNull();
  });

  it("can explicitly remove the runtime theme", () => {
    applyRuntimeTheme(DEFAULT_THEME);
    removeRuntimeTheme();

    expect(document.getElementById("openwebhmi-project-theme")).toBeNull();
  });
});
