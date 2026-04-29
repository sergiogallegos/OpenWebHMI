import { cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import { ScriptEditor } from "../modules/ScriptEditor";
import type { DesignerClient } from "../lib/designerClient";

afterEach(() => {
  cleanup();
});

describe("ScriptEditor", () => {
  it("loads source and debounces saves from a mock editor", async () => {
    const client = {
      loadScriptSource: vi.fn().mockResolvedValue("initial"),
      saveScriptSource: vi.fn().mockResolvedValue(2),
    } as unknown as DesignerClient;

    render(
      <ScriptEditor
        client={client}
        projectId="phase1-demo"
        script={{ id: "derived", path: "derived.py", enabled: true, triggers: [] }}
        editorComponent={MockEditor}
      />,
    );

    const input = await screen.findByRole("textbox");
    fireEvent.change(input, { target: { value: "initial changed" } });

    await waitFor(
      () =>
        expect(client.saveScriptSource).toHaveBeenCalledWith(
          "phase1-demo",
          "derived",
          "initial changed",
        ),
      { timeout: 1_000 },
    );
  });
});

function MockEditor({
  value,
  onChange,
  onMount,
}: {
  value?: string;
  onChange?: (value?: string) => void;
  onMount?: (editor: {
    revealLineInCenter: () => void;
    setPosition: () => void;
    focus: () => void;
  }) => void;
}) {
  onMount?.({
    revealLineInCenter: () => {},
    setPosition: () => {},
    focus: () => {},
  });
  return (
    <textarea
      aria-label="script source"
      value={value ?? ""}
      onChange={(event) => onChange?.(event.currentTarget.value)}
    />
  );
}
