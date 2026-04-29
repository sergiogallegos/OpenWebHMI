/* @vitest-environment jsdom */
import { cleanup, render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { afterEach, describe, expect, it, vi } from "vitest";
import type { HistoryPoint } from "@openwebhmi/protocol";
import { Trend } from "../Trend";
import type { RuntimeContext } from "../../types";

afterEach(() => {
  cleanup();
});

describe("Trend", () => {
  it("renders loading while initial history fetch is in flight", () => {
    render(
      <Trend.Render
        props={Trend.defaultProps}
        bindings={{}}
        context={runtimeContext(() => new Promise(() => undefined))}
      />,
    );

    expect(screen.getByText("loading...")).toBeTruthy();
  });

  it("renders fetched history as an SVG path", async () => {
    render(
      <Trend.Render
        props={{ ...Trend.defaultProps, tagPaths: ["rockwell-1/Pressure"] }}
        bindings={{}}
        context={runtimeContext(async () => history(10, 20))}
      />,
    );

    await waitFor(() => expect(screen.queryByText("loading...")).toBeNull());
    expect(screen.getByRole("img", { name: "Trend chart" }).querySelectorAll("path").length).toBeGreaterThan(0);
    expect(screen.getByText("rockwell-1/Pressure")).toBeTruthy();
    expect(screen.getByText("20.00")).toBeTruthy();
  });

  it("appends live tag updates without replacing the existing SVG element", async () => {
    const readHistory = async () => history(10);
    const { rerender } = render(
      <Trend.Render
        props={{ ...Trend.defaultProps, tagPaths: ["rockwell-1/Pressure"] }}
        bindings={{}}
        context={runtimeContext(readHistory)}
      />,
    );

    await waitFor(() => expect(screen.queryByText("loading...")).toBeNull());
    const svg = screen.getByRole("img", { name: "Trend chart" });
    const firstPath = svg.querySelector("path");

    rerender(
      <Trend.Render
        props={{ ...Trend.defaultProps, tagPaths: ["rockwell-1/Pressure"] }}
        bindings={{}}
        context={{
          ...runtimeContext(readHistory),
          liveValues: {
            "rockwell-1/Pressure": {
              value: { type: "real", value: 30 },
              quality: "good",
              ts: Date.now(),
            },
          },
        }}
      />,
    );

    await waitFor(() => expect(screen.getAllByText("30.00").length).toBeGreaterThan(0));
    expect(screen.getByRole("img", { name: "Trend chart" })).toBe(svg);
    expect(svg.querySelector("path")).toBe(firstPath);
  });

  it("renders two pens with distinct colors in the legend", async () => {
    render(
      <Trend.Render
        props={{
          ...Trend.defaultProps,
          tagPaths: ["rockwell-1/Pressure", "rockwell-1/Counter"],
        }}
        bindings={{}}
        context={runtimeContext(async () => history(1, 2))}
      />,
    );

    await waitFor(() => expect(screen.queryByText("loading...")).toBeNull());
    const pressure = screen.getByRole("button", { name: /rockwell-1\/Pressure/ });
    const counter = screen.getByRole("button", { name: /rockwell-1\/Counter/ });
    expect(pressure.querySelector("span")?.getAttribute("style")).not.toBe(
      counter.querySelector("span")?.getAttribute("style"),
    );
  });

  it("renders bad-quality samples as dashed paths", async () => {
    render(
      <Trend.Render
        props={Trend.defaultProps}
        bindings={{}}
        context={runtimeContext(async () => [
          point(1_000, 10, "good"),
          point(2_000, 12, "bad"),
        ])}
      />,
    );

    await waitFor(() => expect(screen.queryByText("loading...")).toBeNull());
    expect(
      screen.getByRole("img", { name: "Trend chart" }).querySelector(
        'path[stroke-dasharray="5 4"]',
      ),
    ).toBeTruthy();
  });

  it("toggles pen visibility from the legend", async () => {
    const user = userEvent.setup();
    render(
      <Trend.Render
        props={Trend.defaultProps}
        bindings={{}}
        context={runtimeContext(async () => history(10, 20))}
      />,
    );
    await waitFor(() => expect(screen.queryByText("loading...")).toBeNull());

    await user.click(screen.getByRole("button", { name: /rockwell-1\/Pressure/ }));
    expect(screen.getByRole("button", { name: /rockwell-1\/Pressure/ }).style.opacity).toBe("0.45");
  });
});

function runtimeContext(
  onReadHistory: RuntimeContext["onReadHistory"],
): RuntimeContext {
  return {
    mode: "runtime",
    onWriteTag: vi.fn(),
    onReadHistory,
    liveValues: {},
  };
}

function history(...values: number[]): HistoryPoint[] {
  return values.map((value, index) => point((index + 1) * 1_000, value, "good"));
}

function point(
  tsMs: number,
  value: number,
  quality: HistoryPoint["quality"],
): HistoryPoint {
  return {
    ts_ms: tsMs,
    value: { type: "real", value },
    quality,
  };
}
