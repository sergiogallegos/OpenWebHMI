/* @vitest-environment jsdom */
import { render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import { Button } from "../components/Button";

describe("theme variables", () => {
  it("uses the primary color CSS variable for Button", () => {
    document.documentElement.style.setProperty("--primary-color", "rgb(255, 0, 0)");

    render(
      <Button.Render
        props={{
          target: "driver/Start",
          label: "Start",
          writeValue: { type: "bool", value: true },
          style: "primary",
        }}
        bindings={{}}
        context={{
          mode: "runtime",
          onWriteTag: () => {},
          onSubscribeAlarms: () => () => {},
          onAckAlarm: () => {},
          onReadHistory: async () => [],
        }}
      />,
    );

    expect(screen.getByRole("button", { name: "Button" }).getAttribute("style")).toContain(
      "var(--primary-color, #1f4e79)",
    );
  });
});
