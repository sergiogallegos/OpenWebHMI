import { describe, expect, it } from "vitest";
import type { ComponentNode } from "@openwebhmi/protocol";
import { exportWidgetJson, importWidget } from "../modules/widget-io";

describe("widget-io", () => {
  it("round-trips a complex widget without changing props", () => {
    const widget = sampleWidget();
    const json = exportWidgetJson(widget, {
      now: () => 1_779_711_200_000,
      openwebhmiVersion: "test",
    });

    const result = importWidget(json, {
      tagPaths: ["rockwell/Foo"],
      idFactory: (kind) => `${kind}-imported`,
    });

    expect(result.warnings).toEqual([]);
    expect(result.widget.kind).toBe("Trend");
    expect(result.widget.props).toEqual(widget.props);
    expect(result.widget.bindings).toEqual(widget.bindings);
  });

  it("warns for missing binding paths but still imports", () => {
    const result = importWidget(exportWidgetJson(sampleWidget()), {
      tagPaths: [],
      idFactory: (kind) => kind,
    });

    expect(result.widget.kind).toBe("Trend");
    expect(result.warnings).toEqual([
      "Binding path `rockwell/Foo` was not found in this project's tag namespace.",
    ]);
  });

  it("rejects unknown widget types", () => {
    const json = JSON.stringify({
      schema_version: 1,
      widget_type: "NonExistent",
      props: {},
      bindings: [],
      children: [],
      exported_at: 0,
      openwebhmi_version: "test",
    });

    expect(() => importWidget(json)).toThrow("Unknown widget type `NonExistent`");
  });

  it("rejects unsupported schema versions", () => {
    const value = JSON.parse(exportWidgetJson(sampleWidget()));
    value.schema_version = 999;

    expect(() => importWidget(JSON.stringify(value))).toThrow(
      "Unsupported widget export schema_version 999",
    );
  });

  it("pins the representative export format", () => {
    expect(
      exportWidgetJson(sampleWidget(), {
        now: () => 1_779_711_200_000,
        openwebhmiVersion: "test",
      }),
    ).toMatchInlineSnapshot(`
      "{
        "schema_version": 1,
        "widget_type": "Trend",
        "props": {
          "title": "Line pressure",
          "pens": [
            {
              "label": "Pressure",
              "tagPath": "rockwell/Foo",
              "color": "#2563eb"
            }
          ],
          "windowMinutes": 15,
          "showLegend": true,
          "thresholds": {
            "high": 90,
            "low": 10
          }
        },
        "bindings": [
          {
            "prop": "value",
            "source": {
              "kind": "tag",
              "path": "rockwell/Foo"
            }
          }
        ],
        "children": [],
        "exported_at": 1779711200000,
        "openwebhmi_version": "test"
      }
      "
    `);
  });
});

function sampleWidget(): ComponentNode {
  return {
    id: "trend-1",
    kind: "Trend",
    props: {
      title: "Line pressure",
      pens: [{ label: "Pressure", tagPath: "rockwell/Foo", color: "#2563eb" }],
      windowMinutes: 15,
      showLegend: true,
      thresholds: { high: 90, low: 10 },
    },
    bindings: [{ prop: "value", source: { kind: "tag", path: "rockwell/Foo" } }],
    children: [],
  };
}
