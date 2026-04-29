import { describe, expect, it } from "vitest";
import { defaultScriptSource, systemStubs } from "../lib/systemStubs";

describe("systemStubs", () => {
  it("locks the exposed scripting completion surface", () => {
    expect(systemStubs).toMatchInlineSnapshot(`
      [
        {
          "documentation": "Read a tag snapshot from the gateway. Returns None when no value exists.",
          "insertText": "system.tag.read("\${1:path}")",
          "label": "system.tag.read",
        },
        {
          "documentation": "Enqueue a tag write through the gateway. Driver-backed tags route to the PLC write queue.",
          "insertText": "system.tag.write("\${1:path}", \${2:value})",
          "label": "system.tag.write",
        },
        {
          "documentation": "Return current gateway time in Unix epoch milliseconds.",
          "insertText": "system.util.now()",
          "label": "system.util.now",
        },
        {
          "documentation": "Write an INFO log entry with the current script id attached.",
          "insertText": "system.util.log("\${1:message}")",
          "label": "system.util.log",
        },
        {
          "documentation": "Register a handler that runs when a tag changes.",
          "insertText": "@system.on_tag_change("\${1:tag/path}")
      def \${2:handler}(tag):
          \${3:pass}",
          "label": "system.on_tag_change",
        },
      ]
    `);
  });

  it("creates a starter script", () => {
    expect(defaultScriptSource("a/b")).toContain('@system.on_tag_change("a/b")');
  });
});
