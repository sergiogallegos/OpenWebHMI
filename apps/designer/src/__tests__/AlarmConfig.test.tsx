import { cleanup, fireEvent, render, screen } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import { AlarmConfig } from "../modules/AlarmConfig";
import type { DesignerAlarm, DesignerTag } from "../lib/designerClient";

afterEach(cleanup);

describe("AlarmConfig", () => {
  it("editing a threshold updates the alarm definition and saves", async () => {
    vi.useFakeTimers();
    const onSave = vi.fn();

    render(
      <AlarmConfig
        alarms={alarms}
        tags={tags}
        saving={false}
        onSave={onSave}
      />,
    );

    const threshold = screen.getByLabelText("Alarm threshold");
    fireEvent.change(threshold, { target: { value: "250" } });
    await vi.advanceTimersByTimeAsync(300);

    expect(onSave).toHaveBeenCalledWith([
      {
        ...alarms[0],
        condition: { kind: "high_limit", threshold: 250 },
      },
    ]);
    vi.useRealTimers();
  });

  it("adding an alarm creates a draft and schedules a save", async () => {
    vi.useFakeTimers();
    const onSave = vi.fn();

    render(
      <AlarmConfig
        alarms={[]}
        tags={tags}
        saving={false}
        onSave={onSave}
      />,
    );

    fireEvent.click(screen.getByRole("button", { name: "Add alarm" }));
    await vi.advanceTimersByTimeAsync(300);

    expect(onSave).toHaveBeenCalledWith([
      expect.objectContaining({
        id: "alarm-1",
        tag_path: "rockwell-1/Pressure",
      }),
    ]);
    vi.useRealTimers();
  });
});

const tags: DesignerTag[] = [
  { path: "rockwell-1/Pressure", driver: "rockwell-1", address: "Pressure" },
];

const alarms: DesignerAlarm[] = [
  {
    id: "pressure-high",
    label: "High pressure",
    priority: 2,
    tag_path: "rockwell-1/Pressure",
    condition: { kind: "high_limit", threshold: 200 },
    message: "Pressure high: {value}",
    enabled: true,
    require_ack: true,
  },
];
