import { cleanup, render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { afterEach, describe, expect, it, vi } from "vitest";
import { ProjectExplorer } from "../modules/ProjectExplorer";
import type { DesignerProject } from "../lib/designerClient";

afterEach(cleanup);

describe("ProjectExplorer", () => {
  it("renders a project tree and fires view actions", async () => {
    const user = userEvent.setup();
    const onOpenView = vi.fn();
    const onOpenAlarms = vi.fn();
    const onOpenScripts = vi.fn();
    const onAddView = vi.fn();
    const onRenameView = vi.fn();

    render(
      <ProjectExplorer
        project={project}
        selectedViewId="home"
        selectedModule="views"
        onOpenView={onOpenView}
        onOpenAlarms={onOpenAlarms}
        onOpenScripts={onOpenScripts}
        onAddView={onAddView}
        onRenameView={onRenameView}
      />,
    );

    expect(screen.getByText("Demo")).toBeTruthy();
    expect(screen.getByText("rockwell-1")).toBeTruthy();
    expect(screen.getByText("rockwell-1/Pressure")).toBeTruthy();

    await user.click(screen.getByRole("button", { name: "home" }));
    await user.click(screen.getByRole("button", { name: "Alarms" }));
    await user.click(screen.getByRole("button", { name: "Scripts" }));
    await user.click(screen.getByRole("button", { name: "Add View" }));
    await user.click(screen.getByRole("button", { name: "Rename home" }));

    expect(onOpenView).toHaveBeenCalledWith("home");
    expect(onOpenAlarms).toHaveBeenCalledOnce();
    expect(onOpenScripts).toHaveBeenCalledOnce();
    expect(onAddView).toHaveBeenCalledOnce();
    expect(onRenameView).toHaveBeenCalledWith("home");
  });
});

const project: DesignerProject = {
  id: "phase1-demo",
  schema_version: 1,
  name: "Demo",
  version: 1,
  drivers: [{ id: "rockwell-1", type: "rockwell" }],
  tags: [{ path: "rockwell-1/Pressure", driver: "rockwell-1", address: "Pressure" }],
  alarms: [],
  scripts: [],
  views: [
    {
      id: "home",
      title: "Home",
      schema_version: 1,
      root: {
        id: "root",
        kind: "Container",
        props: {},
        bindings: [],
        children: [],
      },
    },
  ],
};
