import { act, cleanup, render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { afterEach, describe, expect, it, vi } from "vitest";
import { ScriptErrorPane } from "../modules/ScriptErrorPane";
import type { DesignerClient, DesignerScriptEvent } from "../lib/designerClient";

afterEach(cleanup);

describe("ScriptErrorPane", () => {
  it("renders script errors, bounds per-script events, and opens line numbers", async () => {
    const user = userEvent.setup();
    let callback: ((event: DesignerScriptEvent) => void) | null = null;
    const client = {
      subscribeScriptEvents: vi.fn((_projectId, cb) => {
        callback = cb;
        return () => {};
      }),
    } as unknown as DesignerClient;
    const onOpenScript = vi.fn();

    render(
      <ScriptErrorPane
        client={client}
        projectId="phase1-demo"
        selectedScriptId="derived"
        onOpenScript={onOpenScript}
      />,
    );

    await waitFor(() => expect(callback).not.toBeNull());
    act(() => {
      for (let index = 0; index < 21; index += 1) {
        callback?.({
          kind: "script.event",
          project_id: "phase1-demo",
          script_id: "derived",
          event_kind: "error",
          message: `Traceback line ${index + 1}`,
        });
      }
    });

    expect(screen.queryByText("Traceback line 1")).toBeNull();
    expect(screen.getByText("Traceback line 21")).toBeTruthy();
    await user.click(screen.getByText("Traceback line 21"));
    expect(onOpenScript).toHaveBeenCalledWith("derived", 21);
  });
});
