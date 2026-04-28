/* @vitest-environment jsdom */
import { cleanup, render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { afterEach, describe, expect, it, vi } from "vitest";
import { AlarmTable } from "../AlarmTable";
import type { AlarmEvent, RuntimeContext } from "../../types";

const runtimeContext: RuntimeContext = {
  mode: "runtime",
  onWriteTag: vi.fn(),
};

afterEach(() => {
  cleanup();
});

describe("AlarmTable", () => {
  it("renders synthetic alarm events sorted by priority", () => {
    render(
      <AlarmTable.Render
        props={{ ...AlarmTable.defaultProps, stateFilter: "all", events }}
        bindings={{}}
        context={runtimeContext}
      />,
    );

    expect(screen.getByText("High pressure")).toBeTruthy();
    expect(screen.getByText("Low flow")).toBeTruthy();
    expect(screen.getByTitle("Quality: bad")).toBeTruthy();
  });

  it("ack button calls callback with id and note", async () => {
    const user = userEvent.setup();
    const onAck = vi.fn();
    render(
      <AlarmTable.Render
        props={{ ...AlarmTable.defaultProps, events, onAck }}
        bindings={{}}
        context={runtimeContext}
      />,
    );

    await user.click(screen.getByText("High pressure"));
    await user.type(screen.getByLabelText("Ack note"), "checked");
    await user.click(screen.getByRole("button", { name: "Ack" }));

    expect(onAck).toHaveBeenCalledWith("pressure-high", "checked");
  });

  it("priority filtering hides out-of-range rows", async () => {
    const user = userEvent.setup();
    render(
      <AlarmTable.Render
        props={{ ...AlarmTable.defaultProps, stateFilter: "all", events }}
        bindings={{}}
        context={runtimeContext}
      />,
    );

    await user.click(screen.getByRole("button", { name: "info" }));

    expect(screen.queryByText("High pressure")).toBeNull();
    expect(screen.getByText("Low flow")).toBeTruthy();
  });
});

const events: AlarmEvent[] = [
  event({
    alarm_id: "flow-low",
    label: "Low flow",
    priority: 4,
    value: { type: "real", value: 12 },
    quality: "good",
  }),
  event({
    alarm_id: "pressure-high",
    label: "High pressure",
    priority: 1,
    value: { type: "real", value: 250 },
    quality: "bad",
  }),
];

function event(overrides: Partial<AlarmEvent>): AlarmEvent {
  return {
    kind: "alarm.event",
    alarm_id: "alarm",
    label: "Alarm",
    priority: 3,
    state: "active",
    tag_path: "rockwell-1/Pressure",
    value: { type: "real", value: 0 },
    quality: "good",
    activated_at_ms: 1_000,
    transitioned_at_ms: 1_000,
    who: null,
    note: null,
    message: "Alarm message",
    ...overrides,
  };
}
