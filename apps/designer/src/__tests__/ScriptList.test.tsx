import { cleanup, render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { afterEach, describe, expect, it, vi } from "vitest";
import { ScriptList } from "../modules/ScriptList";
import type { DesignerClient, DesignerScript } from "../lib/designerClient";

afterEach(() => {
  cleanup();
  vi.restoreAllMocks();
});

describe("ScriptList", () => {
  it("adds and deletes script config plus source artifacts", async () => {
    const user = userEvent.setup();
    vi.spyOn(window, "prompt").mockReturnValue("new-script");
    vi.spyOn(window, "confirm").mockReturnValue(true);
    const client = {
      saveScript: vi.fn().mockResolvedValue(2),
      saveScriptSource: vi.fn().mockResolvedValue(3),
      deleteArtifact: vi.fn().mockResolvedValue(undefined),
      loadScriptSource: vi.fn().mockResolvedValue("old"),
    } as unknown as DesignerClient;
    const onScriptsChanged = vi.fn();
    const onSelect = vi.fn();
    const scripts: DesignerScript[] = [
      { id: "old-script", path: "old-script.py", enabled: true, triggers: [] },
    ];

    render(
      <ScriptList
        client={client}
        projectId="phase1-demo"
        scripts={scripts}
        selectedScriptId="old-script"
        onSelect={onSelect}
        onScriptsChanged={onScriptsChanged}
      />,
    );

    await user.click(screen.getByRole("button", { name: "Add" }));
    expect(client.saveScript).toHaveBeenCalledWith(
      "phase1-demo",
      expect.objectContaining({ id: "new-script" }),
    );
    expect(client.saveScriptSource).toHaveBeenCalledWith(
      "phase1-demo",
      "new-script",
      expect.stringContaining("system"),
    );

    await user.click(screen.getByRole("button", { name: "Delete" }));
    expect(client.deleteArtifact).toHaveBeenCalledWith("phase1-demo", {
      kind: "script_source",
      id: "old-script",
    });
    expect(client.deleteArtifact).toHaveBeenCalledWith("phase1-demo", {
      kind: "script",
      id: "old-script",
    });
  });
});
