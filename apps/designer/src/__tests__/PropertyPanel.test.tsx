import { cleanup, render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { afterEach, describe, expect, it, vi } from "vitest";
import type { View } from "@openwebhmi/protocol";
import { PropertyPanel } from "../modules/PropertyPanel";

afterEach(cleanup);

describe("PropertyPanel", () => {
  it("editing a prop calls the save callback", async () => {
    const user = userEvent.setup();
    const onChange = vi.fn();

    render(
      <PropertyPanel
        view={view}
        selectedComponentId="label"
        selectedTag="rockwell-1/Pressure"
        tagPaths={["rockwell-1/Pressure"]}
        debounceMs={0}
        onChange={onChange}
      />,
    );

    const input = screen.getByLabelText("Text");
    await user.clear(input);
    await user.type(input, "Line pressure");

    expect(onChange).toHaveBeenCalled();
    expect(onChange.mock.calls.at(-1)?.[0].root.children[0].props).toMatchObject({
      text: "Line pressure",
    });
  });

  it("binding a prop calls the save callback", async () => {
    const user = userEvent.setup();
    const onChange = vi.fn();

    render(
      <PropertyPanel
        view={view}
        selectedComponentId="value"
        selectedTag="rockwell-1/Pressure"
        tagPaths={["rockwell-1/Pressure"]}
        debounceMs={0}
        onChange={onChange}
      />,
    );

    await user.click(screen.getByRole("button", { name: "Bind" }));

    expect(onChange).toHaveBeenCalled();
    expect(onChange.mock.calls.at(-1)?.[0].root.children[1].bindings).toEqual([
      {
        prop: "value",
        source: { kind: "tag", path: "rockwell-1/Pressure" },
      },
    ]);
  });
});

const view: View = {
  id: "home",
  title: "Home",
  schema_version: 1,
  root: {
    id: "root",
    kind: "Container",
    props: {},
    bindings: [],
    children: [
      {
        id: "label",
        kind: "Label",
        props: { text: "Pressure", color: "#1f2933", fontSize: 16 },
        bindings: [],
        children: [],
      },
      {
        id: "value",
        kind: "ValueDisplay",
        props: { format: "number", decimals: 1, unit: "psi" },
        bindings: [],
        children: [],
      },
    ],
  },
};
