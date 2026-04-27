import { cleanup, render, screen } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import type { BoundValue } from "@openwebhmi/component-library";
import type { View } from "@openwebhmi/protocol";
import { collectTagPaths, ViewRenderer } from "../ViewRenderer";

afterEach(cleanup);

describe("ViewRenderer", () => {
  it("renders a nested tree and passes tag bindings to components", () => {
    render(
      <ViewRenderer
        view={viewWithDisplay("Pressure")}
        boundValues={{
          "rockwell-1/Pressure": realValue(42.25, "good"),
        }}
        onWriteTag={vi.fn()}
      />,
    );

    expect(screen.getByText("Pressure")).toBeTruthy();
    expect(screen.getByText("42.3")).toBeTruthy();
    expect(screen.getByText("psi")).toBeTruthy();
    expect(collectTagPaths(viewWithDisplay("Pressure"))).toEqual([
      "rockwell-1/Pressure",
    ]);
  });

  it("reflects a hot-reloaded view tree", () => {
    const { rerender } = render(
      <ViewRenderer
        view={viewWithDisplay("Pressure")}
        boundValues={{
          "rockwell-1/Pressure": realValue(41, "good"),
        }}
        onWriteTag={vi.fn()}
      />,
    );

    rerender(
      <ViewRenderer
        view={viewWithDisplay("Line pressure")}
        boundValues={{
          "rockwell-1/Pressure": realValue(43, "good"),
        }}
        onWriteTag={vi.fn()}
      />,
    );

    expect(screen.queryByText("Pressure")).toBeNull();
    expect(screen.getByText("Line pressure")).toBeTruthy();
    expect(screen.getByText("43.0")).toBeTruthy();
  });

  it("renders missing live bindings as undefined component bindings", () => {
    render(
      <ViewRenderer
        view={viewWithDisplay("Pressure")}
        boundValues={{}}
        onWriteTag={vi.fn()}
      />,
    );

    expect(screen.getByText("--")).toBeTruthy();
  });

  it("passes bad quality bound values through", () => {
    render(
      <ViewRenderer
        view={viewWithDisplay("Pressure")}
        boundValues={{
          "rockwell-1/Pressure": realValue(12, "bad"),
        }}
        onWriteTag={vi.fn()}
      />,
    );

    expect(screen.getByLabelText("Value display").getAttribute("title")).toContain(
      "bad",
    );
  });
});

function viewWithDisplay(label: string): View {
  return {
    id: "home",
    title: "Home",
    schema_version: 1,
    root: {
      id: "root",
      kind: "Container",
      props: { direction: "column", gap: 8, padding: 12, background: "#ffffff" },
      bindings: [],
      children: [
        {
          id: "label",
          kind: "Label",
          props: { text: label, color: "#1f2933", fontSize: 16 },
          bindings: [],
          children: [],
        },
        {
          id: "pressure",
          kind: "ValueDisplay",
          props: { format: "number", decimals: 1, unit: "psi" },
          bindings: [
            {
              prop: "value",
              source: { kind: "tag", path: "rockwell-1/Pressure" },
            },
          ],
          children: [],
        },
      ],
    },
  };
}

function realValue(value: number, quality: BoundValue["quality"]): BoundValue {
  return {
    value: { type: "real", value },
    quality,
    ts: Date.UTC(2026, 3, 27),
  };
}
