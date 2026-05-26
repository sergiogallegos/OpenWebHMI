/* @vitest-environment jsdom */
import { cleanup, fireEvent, render, screen } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import { DEFAULT_THEME } from "@openwebhmi/protocol";
import { ThemeEditor } from "../modules/ThemeEditor";

describe("ThemeEditor", () => {
  afterEach(cleanup);

  it("renders defaults when no theme is set", () => {
    render(<ThemeEditor theme={null} saving={false} onSave={() => {}} />);

    expect(screen.getAllByDisplayValue(DEFAULT_THEME.light.primary_color).length).toBeGreaterThan(0);
  });

  it("updates root CSS variables when editing a color", () => {
    render(<ThemeEditor theme={null} saving={false} onSave={() => {}} />);

    fireEvent.change(screen.getAllByLabelText("Primary")[0], {
      target: { value: "#ff0000" },
    });

    expect(document.documentElement.style.getPropertyValue("--primary-color")).toBe("#ff0000");
  });

  it("saves the current theme shape", () => {
    const onSave = vi.fn();
    render(<ThemeEditor theme={null} saving={false} onSave={onSave} />);

    fireEvent.click(screen.getByRole("button", { name: "Save" }));

    expect(onSave).toHaveBeenCalledWith(DEFAULT_THEME);
  });
});
