/** Python scripting stubs surfaced in Monaco completion. */
export const systemStubs = [
  {
    label: "system.tag.read",
    insertText: 'system.tag.read("${1:path}")',
    documentation: "Read a tag snapshot from the gateway. Returns None when no value exists.",
  },
  {
    label: "system.tag.write",
    insertText: 'system.tag.write("${1:path}", ${2:value})',
    documentation: "Enqueue a tag write through the gateway. Driver-backed tags route to the PLC write queue.",
  },
  {
    label: "system.util.now",
    insertText: "system.util.now()",
    documentation: "Return current gateway time in Unix epoch milliseconds.",
  },
  {
    label: "system.util.log",
    insertText: 'system.util.log("${1:message}")',
    documentation: "Write an INFO log entry with the current script id attached.",
  },
  {
    label: "system.on_tag_change",
    insertText:
      '@system.on_tag_change("${1:tag/path}")\ndef ${2:handler}(tag):\n    ${3:pass}',
    documentation: "Register a handler that runs when a tag changes.",
  },
] as const;

/** Starter script used when creating a new script from the designer. */
export function defaultScriptSource(tagPath = "rockwell-1/Pressure"): string {
  return `import system


@system.on_tag_change("${tagPath}")
def on_change(tag):
    system.util.log("tag changed")
`;
}
