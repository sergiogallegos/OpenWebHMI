/* @vitest-environment jsdom */
import { cleanup, render, screen } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import { getComponentDefinition } from "../registry";
import type { DesignerContext } from "../types";

const designerContext: DesignerContext = {
  mode: "designer",
  isSelected: false,
};

afterEach(() => {
  cleanup();
  vi.restoreAllMocks();
});

describe("material pack", () => {
  const materialKinds = [
    "Button",
    "ToggleSwitch",
    "NumericInput",
    "Slider",
    "Card",
    "Dropdown",
    "Gauge",
    "Modal",
  ];

  it("renders the demo widgets with Material DOM classes", () => {
    for (const kind of materialKinds) {
      const definition = getComponentDefinition(kind, "material");
      expect(definition).toBeTruthy();
      const RenderComponent = definition!.Render;
      const { container, unmount } = render(
        <RenderComponent
          props={definition!.defaultProps}
          bindings={{}}
          context={designerContext}
        />,
      );
      expect(container.querySelector("[class^='mdc-'], .mdc-card, .mdc-dialog")).toBeTruthy();
      unmount();
    }
  });

  it("falls back to default once per missing material widget type", () => {
    const warn = vi.spyOn(console, "warn").mockImplementation(() => undefined);
    const first = getComponentDefinition("Label", "material");
    const second = getComponentDefinition("Label", "material");

    expect(first?.kind).toBe("Label");
    expect(second?.kind).toBe("Label");
    expect(warn).toHaveBeenCalledTimes(1);
    expect(warn).toHaveBeenCalledWith(
      "material pack: Label rendered with default style; not in pack",
    );
  });

  it("keeps prop API parity with the default pack", () => {
    for (const kind of materialKinds) {
      const material = getComponentDefinition(kind, "material");
      const fallback = getComponentDefinition(kind);
      expect(material?.propsSchema).toEqual(fallback?.propsSchema);
      expect(material?.bindableProps).toEqual(fallback?.bindableProps);
      expect(material?.defaultProps).toEqual(fallback?.defaultProps);
    }
  });

  it("can render fallback content", () => {
    const label = getComponentDefinition("Label", "material");
    const RenderComponent = label!.Render;
    render(<RenderComponent props={label!.defaultProps} bindings={{}} context={designerContext} />);
    expect(screen.getByLabelText("Label")).toBeTruthy();
  });
});
