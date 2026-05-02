/* @vitest-environment jsdom */
import { cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { afterEach, describe, expect, it, vi } from "vitest";
import type { TagValue } from "@openwebhmi/protocol";
import { componentRegistry } from "../../registry";
import type { AlarmEvent, BoundValue, DesignerContext, RuntimeContext } from "../../types";
import { AlarmBanner } from "../AlarmBanner";
import { Button } from "../Button";
import { Container } from "../Container";
import { Dropdown } from "../Dropdown";
import { Gauge } from "../Gauge";
import { Image } from "../Image";
import { Indicator } from "../Indicator";
import { Label } from "../Label";
import { MultiState } from "../MultiState";
import { NumericInput } from "../NumericInput";
import { ProgressBar } from "../ProgressBar";
import { Slider } from "../Slider";
import { ToggleSwitch } from "../ToggleSwitch";
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
  it("exports the registered components by kind", () => {
    expect(Object.keys(componentRegistry).sort()).toEqual([
      "AlarmBanner",
      "AlarmTable",
      "Button",
      "Container",
      "Dropdown",
      "Gauge",
      "Image",
      "Indicator",
      "Label",
      "MultiState",
      "NumericInput",
      "ProgressBar",
      "Slider",
      "ToggleSwitch",
      "Trend",
      "ValueDisplay",
    ]);
  });
});

describe("Gauge", () => {
  it("renders with default props and a bound value", () => {
    render(<Gauge.Render props={Gauge.defaultProps} bindings={{ value: goodNumber }} context={runtimeContext} />);
    expect(screen.getByRole("img", { name: "Gauge chart" })).toBeTruthy();
    expect(screen.getByText("12.3")).toBeTruthy();
  });

  it("renders the bad-quality visual", () => {
    render(<Gauge.Render props={Gauge.defaultProps} bindings={{ value: badNumber }} context={runtimeContext} />);
    expectBadQuality(screen.getByLabelText("Gauge"));
  });
});

describe("ProgressBar", () => {
  it("renders fill from bound value", () => {
    render(<ProgressBar.Render props={ProgressBar.defaultProps} bindings={{ value: goodNumber }} context={runtimeContext} />);
    expect(screen.getByTestId("progress-fill").style.width).toBe("12.345%");
  });

  it("renders bad quality", () => {
    render(<ProgressBar.Render props={ProgressBar.defaultProps} bindings={{ value: badNumber }} context={runtimeContext} />);
    expectBadQuality(screen.getByLabelText("Progress bar"));
  });
});

describe("Slider", () => {
  it("writes on release in runtime mode", async () => {
    const onWriteTag = vi.fn();
    render(<Slider.Render props={Slider.defaultProps} bindings={{}} context={{ mode: "runtime", onWriteTag }} />);
    const input = screen.getByLabelText("Slider input");
    fireEvent.change(input, { target: { value: "75" } });
    fireEvent.mouseUp(input);
    expect(onWriteTag).toHaveBeenCalledWith("value", { type: "int", value: 75 });
  });

  it("does not write in designer mode", async () => {
    const user = userEvent.setup();
    const onWriteTag = vi.fn();
    render(<Slider.Render props={Slider.defaultProps} bindings={{}} context={designerContext} />);
    await user.type(screen.getByLabelText("Slider input"), "75");
    expect(onWriteTag).not.toHaveBeenCalled();
  });
});

describe("Dropdown", () => {
  it("writes selected option in runtime mode", async () => {
    const user = userEvent.setup();
    const onWriteTag = vi.fn();
    render(<Dropdown.Render props={Dropdown.defaultProps} bindings={{}} context={{ mode: "runtime", onWriteTag }} />);
    await user.selectOptions(screen.getByLabelText("Dropdown"), "string:manual");
    expect(onWriteTag).toHaveBeenCalledWith("value", { type: "string", value: "manual" });
  });

  it("renders bad quality", () => {
    render(<Dropdown.Render props={Dropdown.defaultProps} bindings={{ value: badString }} context={runtimeContext} />);
    expectBadQuality(screen.getByLabelText("Dropdown"));
  });
});

describe("ToggleSwitch", () => {
  it("writes inverse boolean in runtime mode", async () => {
    const user = userEvent.setup();
    const onWriteTag = vi.fn();
    render(<ToggleSwitch.Render props={ToggleSwitch.defaultProps} bindings={{ value: goodBool }} context={{ mode: "runtime", onWriteTag }} />);
    await user.click(screen.getByLabelText("Toggle switch"));
    expect(onWriteTag).toHaveBeenCalledWith("value", { type: "bool", value: false });
  });

  it("does not write in designer mode", async () => {
    const user = userEvent.setup();
    const onWriteTag = vi.fn();
    render(<ToggleSwitch.Render props={ToggleSwitch.defaultProps} bindings={{}} context={designerContext} />);
    await user.click(screen.getByLabelText("Toggle switch"));
    expect(onWriteTag).not.toHaveBeenCalled();
  });
});

describe("Button", () => {
  it("writes configured value to bound target", async () => {
    const user = userEvent.setup();
    const onWriteTag = vi.fn();
    render(
      <Button.Render
        props={{ ...Button.defaultProps, writeValue: { type: "int", value: 7 } }}
        bindings={{ target: bound({ type: "string", value: "pump/start" }, "good") }}
        context={{ mode: "runtime", onWriteTag }}
      />,
    );
    await user.click(screen.getByLabelText("Button"));
    expect(onWriteTag).toHaveBeenCalledWith("pump/start", { type: "int", value: 7 });
  });

  it("does not write in designer mode", async () => {
    const user = userEvent.setup();
    const onWriteTag = vi.fn();
    render(<Button.Render props={{ ...Button.defaultProps, target: "pump/start" }} bindings={{}} context={designerContext} />);
    await user.click(screen.getByLabelText("Button"));
    expect(onWriteTag).not.toHaveBeenCalled();
  });
});

describe("MultiState", () => {
  it("renders matching state label", () => {
    render(<MultiState.Render props={MultiState.defaultProps} bindings={{ value: bound({ type: "string", value: "stop" }, "good") }} context={runtimeContext} />);
    expect(screen.getByText("Stopped")).toBeTruthy();
  });

  it("renders bad quality", () => {
    render(<MultiState.Render props={MultiState.defaultProps} bindings={{ value: badString }} context={runtimeContext} />);
    expectBadQuality(screen.getByLabelText("MultiState"));
  });
});

describe("AlarmBanner", () => {
  it("subscribes and counts active alarm bands", async () => {
    let callback: ((event: any) => void) | undefined;
    const onSubscribeAlarms = vi.fn((_options, next) => {
      callback = next;
      return vi.fn();
    });
    render(
      <AlarmBanner.Render
        props={{ ...AlarmBanner.defaultProps, showZero: true }}
        bindings={{}}
        context={{ mode: "runtime", onWriteTag: vi.fn(), projectId: "phase1-demo", onSubscribeAlarms }}
      />,
    );
    callback?.(alarmEvent({ alarm_id: "a1", priority: 2, state: "active" }));
    callback?.(alarmEvent({ alarm_id: "a2", priority: 3, state: "acked" }));
    expect(onSubscribeAlarms).toHaveBeenCalled();
    await waitFor(() =>
      expect(screen.getByRole("button", { name: "critical alarms" }).textContent).toContain("1"),
    );
    expect(screen.getByRole("button", { name: "warning alarms" }).textContent).toContain("1");
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

function alarmEvent(overrides: Partial<AlarmEvent>): AlarmEvent {
  return {
    kind: "alarm.event",
    alarm_id: "alarm",
    label: "Alarm",
    priority: 2,
    state: "active",
    tag_path: "rockwell-1/Pressure",
    value: { type: "real", value: 100 },
    quality: "good",
    activated_at_ms: 1,
    transitioned_at_ms: 1,
    message: "Alarm",
    ...overrides,
  };
}

function expectBadQuality(element: HTMLElement) {
  expect(element.style.border).toContain("214, 69, 69");
}
