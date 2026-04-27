import { describe, expect, it, vi } from "vitest";
import type { View } from "@openwebhmi/protocol";
import {
  addChild,
  clearBinding,
  createComponent,
  duplicateComponent,
  findNode,
  moveComponent,
  removeComponent,
  setBinding,
  setProp,
} from "../lib/viewMutations";

describe("viewMutations", () => {
  it("adds and removes child components without mutating the source view", () => {
    vi.spyOn(Math, "random").mockReturnValue(0.123456);
    const view = sampleView();
    const child = createComponent("Label");
    const added = addChild(view, "root", child);

    expect(view.root.children).toHaveLength(2);
    expect(added.root.children).toHaveLength(3);
    expect(findNode(added, child.id)?.kind).toBe("Label");

    const removed = removeComponent(added, child.id);
    expect(removed.root.children).toHaveLength(2);
    vi.mocked(Math.random).mockRestore();
  });

  it("moves and duplicates sibling components", () => {
    const view = sampleView();
    const moved = moveComponent(view, "value", "up");
    expect(moved.root.children.map((child) => child.id)).toEqual([
      "value",
      "label",
    ]);

    const duplicated = duplicateComponent(view, "label");
    expect(duplicated.root.children.map((child) => child.id)).toEqual([
      "label",
      "label-copy",
      "value",
    ]);
  });

  it("sets props and bindings", () => {
    const view = sampleView();
    const withProp = setProp(view, "label", "text", "Line pressure");
    expect(findNode(withProp, "label")?.props).toMatchObject({
      text: "Line pressure",
    });

    const withBinding = setBinding(withProp, "value", "value", {
      kind: "tag",
      path: "rockwell-1/Pressure",
    });
    expect(findNode(withBinding, "value")?.bindings).toEqual([
      {
        prop: "value",
        source: { kind: "tag", path: "rockwell-1/Pressure" },
      },
    ]);

    expect(clearBinding(withBinding, "value", "value").root.children[1]?.bindings).toEqual(
      [],
    );
  });
});

function sampleView(): View {
  return {
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
          props: { text: "Pressure" },
          bindings: [],
          children: [],
        },
        {
          id: "value",
          kind: "ValueDisplay",
          props: {},
          bindings: [],
          children: [],
        },
      ],
    },
  };
}
