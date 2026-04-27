/* @vitest-environment jsdom */
import { cleanup, render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { afterEach, describe, expect, it, vi } from "vitest";
import type { TagValue } from "@openwebhmi/protocol";
import { componentRegistry } from "../../registry";
import type { BoundValue, DesignerContext, RuntimeContext } from "../../types";
import { Container } from "../Container";
import { Image } from "../Image";
import { Indicator } from "../Indicator";
import { Label } from "../Label";
import { NumericInput } from "../NumericInput";
import { ValueDisplay } from "../ValueDisplay";

const runtimeContext: RuntimeContext = {
  mode: "runtime",
  onWriteTag: vi.fn(),
};

const designerContext: DesignerContext = {
  mode: "designer",
  isSelected: false,
};

const goodString = bound({ type: "string", value: "Bound" }, "good");
const badString = bound({ type: "string", value: "Faulted" }, "bad");
const goodNumber = bound({ type: "real", value: 12.345 }, "good");
const badNumber = bound({ type: "real", value: 99.5 }, "bad");
const goodBool = bound({ type: "bool", value: true }, "good");
const badBool = bound({ type: "bool", value: false }, "bad");

afterEach(() => {
  cleanup();
});

describe("componentRegistry", () => {
  it("exports all six v1 components by kind", () => {
    expect(Object.keys(componentRegistry).sort()).toEqual([
      "Container",
      "Image",
      "Indicator",
      "Label",
      "NumericInput",
      "ValueDisplay",
    ]);
  });
});

describe("Label", () => {
  it("renders with default props", () => {
    render(<Label.Render props={Label.defaultProps} bindings={{}} context={designerContext} />);
    expect(screen.getByText("Label")).toBeTruthy();
  });

  it("renders with a bound good value", () => {
    render(
      <Label.Render
        props={Label.defaultProps}
        bindings={{ text: goodString }}
        context={runtimeContext}
      />,
    );
    expect(screen.getByText("Bound")).toBeTruthy();
  });

  it("renders the bad-quality visual when bound to bad quality", () => {
    render(
      <Label.Render
        props={Label.defaultProps}
        bindings={{ text: badString }}
        context={runtimeContext}
      />,
    );
    expectBadQuality(screen.getByLabelText("Label"));
  });

  it("renders correctly when the binding is missing", () => {
    render(
      <Label.Render
        props={{ ...Label.defaultProps, text: "Fallback" }}
        bindings={{ text: undefined }}
        context={runtimeContext}
      />,
    );
    expect(screen.getByText("Fallback")).toBeTruthy();
  });
});

describe("ValueDisplay", () => {
  it("renders with default props", () => {
    render(
      <ValueDisplay.Render
        props={ValueDisplay.defaultProps}
        bindings={{}}
        context={designerContext}
      />,
    );
    expect(screen.getByText("--")).toBeTruthy();
  });

  it("renders with a bound good value", () => {
    render(
      <ValueDisplay.Render
        props={{ ...ValueDisplay.defaultProps, decimals: 1, unit: "psi" }}
        bindings={{ value: goodNumber }}
        context={runtimeContext}
      />,
    );
    expect(screen.getByText("12.3")).toBeTruthy();
    expect(screen.getByText("psi")).toBeTruthy();
  });

  it("renders the bad-quality visual when bound to bad quality", () => {
    render(
      <ValueDisplay.Render
        props={ValueDisplay.defaultProps}
        bindings={{ value: badNumber }}
        context={runtimeContext}
      />,
    );
    expectBadQuality(screen.getByLabelText("Value display"));
  });

  it("renders correctly when the binding is missing", () => {
    render(
      <ValueDisplay.Render
        props={{ ...ValueDisplay.defaultProps, value: { type: "int", value: 7 } }}
        bindings={{ value: undefined }}
        context={runtimeContext}
      />,
    );
    expect(screen.getByText("7.00")).toBeTruthy();
  });
});

describe("NumericInput", () => {
  it("renders with default props", () => {
    render(
      <NumericInput.Render
        props={NumericInput.defaultProps}
        bindings={{}}
        context={designerContext}
      />,
    );
    expect((screen.getByLabelText("Numeric input") as HTMLInputElement).value).toBe("0");
  });

  it("renders with a bound good value", () => {
    render(
      <NumericInput.Render
        props={NumericInput.defaultProps}
        bindings={{ value: goodNumber }}
        context={runtimeContext}
      />,
    );
    expect((screen.getByLabelText("Numeric input") as HTMLInputElement).value).toBe("12.345");
  });

  it("renders the bad-quality visual when bound to bad quality", () => {
    render(
      <NumericInput.Render
        props={NumericInput.defaultProps}
        bindings={{ value: badNumber }}
        context={runtimeContext}
      />,
    );
    expectBadQuality(screen.getByLabelText("Numeric input"));
  });

  it("renders correctly when the binding is missing", () => {
    render(
      <NumericInput.Render
        props={{ ...NumericInput.defaultProps, value: 42 }}
        bindings={{ value: undefined }}
        context={runtimeContext}
      />,
    );
    expect((screen.getByLabelText("Numeric input") as HTMLInputElement).value).toBe("42");
  });

  it("typing a value and pressing Enter writes to the runtime context", async () => {
    const user = userEvent.setup();
    const onWriteTag = vi.fn();
    render(
      <NumericInput.Render
        props={{ ...NumericInput.defaultProps, tagPath: "tank/level" }}
        bindings={{}}
        context={{ mode: "runtime", onWriteTag }}
      />,
    );

    const input = screen.getByLabelText("Numeric input");
    await user.clear(input);
    await user.type(input, "123.5{Enter}");

    expect(onWriteTag).toHaveBeenCalledWith("tank/level", {
      type: "real",
      value: 123.5,
    });
  });
});

describe("Indicator", () => {
  it("renders with default props", () => {
    render(
      <Indicator.Render
        props={Indicator.defaultProps}
        bindings={{}}
        context={designerContext}
      />,
    );
    expect(screen.getByRole("status", { name: "Indicator false" })).toBeTruthy();
  });

  it("renders with a bound good value", () => {
    render(
      <Indicator.Render
        props={Indicator.defaultProps}
        bindings={{ state: goodBool }}
        context={runtimeContext}
      />,
    );
    expect(screen.getByRole("status", { name: "Indicator true" })).toBeTruthy();
  });

  it("renders the bad-quality visual when bound to bad quality", () => {
    render(
      <Indicator.Render
        props={Indicator.defaultProps}
        bindings={{ state: badBool }}
        context={runtimeContext}
      />,
    );
    expectBadQuality(screen.getByRole("status", { name: "Indicator false" }));
    expect(screen.getByText("?")).toBeTruthy();
  });

  it("renders correctly when the binding is missing", () => {
    render(
      <Indicator.Render
        props={{ ...Indicator.defaultProps, state: "RUN", mapping: { RUN: "#2563eb" } }}
        bindings={{ state: undefined }}
        context={runtimeContext}
      />,
    );
    expect(screen.getByText("RUN")).toBeTruthy();
  });
});

describe("Image", () => {
  it("renders with default props", () => {
    render(<Image.Render props={Image.defaultProps} bindings={{}} context={designerContext} />);
    expect(screen.getByText("No image")).toBeTruthy();
  });

  it("renders with a bound good value", () => {
    render(
      <Image.Render
        props={{ ...Image.defaultProps, alt: "Pump" }}
        bindings={{ src: goodString }}
        context={runtimeContext}
      />,
    );
    expect((screen.getByAltText("Pump") as HTMLImageElement).src).toContain("Bound");
  });

  it("renders the bad-quality visual when bound to bad quality", () => {
    render(
      <Image.Render
        props={Image.defaultProps}
        bindings={{ src: badString }}
        context={runtimeContext}
      />,
    );
    expectBadQuality(screen.getByLabelText("Image"));
  });

  it("renders correctly when the binding is missing", () => {
    render(
      <Image.Render
        props={{ ...Image.defaultProps, src: "/asset.png", alt: "Asset" }}
        bindings={{ src: undefined }}
        context={runtimeContext}
      />,
    );
    expect(screen.getByAltText("Asset")).toBeTruthy();
  });
});

describe("Container", () => {
  it("renders with default props", () => {
    render(
      <Container.Render props={Container.defaultProps} bindings={{}} context={designerContext} />,
    );
    expect(screen.getByLabelText("Container")).toBeTruthy();
  });

  it("renders with an empty bindings map", () => {
    render(
      <Container.Render
        props={{ ...Container.defaultProps, direction: "row" }}
        bindings={{}}
        context={runtimeContext}
      >
        <span>Child</span>
      </Container.Render>,
    );
    expect(screen.getByText("Child")).toBeTruthy();
  });

  it("renders without a bad-quality visual because it has no bindable props", () => {
    render(
      <Container.Render props={Container.defaultProps} bindings={{}} context={runtimeContext} />,
    );
    expect(screen.getByLabelText("Container").style.border).toContain("dashed");
  });

  it("renders correctly when a binding key is missing entirely", () => {
    render(
      <Container.Render
        props={{ ...Container.defaultProps, background: "#f5f7fa" }}
        bindings={{ missing: undefined }}
        context={runtimeContext}
      />,
    );
    expect(screen.getByLabelText("Container")).toBeTruthy();
  });
});

function bound(value: TagValue, quality: BoundValue["quality"]): BoundValue {
  return {
    value,
    quality,
    ts: 1_714_000_000_000,
  };
}

function expectBadQuality(element: HTMLElement) {
  expect(element.style.border).toContain("214, 69, 69");
}
