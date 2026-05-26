import { componentRegistry } from "@openwebhmi/component-library";
import type { Binding, ComponentNode } from "@openwebhmi/protocol";

export const WIDGET_EXPORT_SCHEMA_VERSION = 1;

export type ExportedWidgetChild = {
  widget_type: string;
  props: unknown;
  bindings: Binding[];
  children: ExportedWidgetChild[];
};

export type ExportedWidget = {
  schema_version: number;
  widget_type: string;
  props: unknown;
  bindings: Binding[];
  children: ExportedWidgetChild[];
  exported_at: number;
  openwebhmi_version: string;
};

export type ImportResult = {
  widget: ComponentNode;
  warnings: string[];
};

export type ImportWidgetOptions = {
  tagPaths?: string[];
  now?: () => number;
  idFactory?: (kind: string) => string;
  openwebhmiVersion?: string;
};

export function exportWidget(
  widget: ComponentNode,
  options: ImportWidgetOptions = {},
): ExportedWidget {
  return {
    schema_version: WIDGET_EXPORT_SCHEMA_VERSION,
    widget_type: widget.kind,
    props: cloneValue(widget.props),
    bindings: cloneValue(widget.bindings),
    children: widget.children.map(componentToExportedChild),
    exported_at: options.now?.() ?? Date.now(),
    openwebhmi_version: options.openwebhmiVersion ?? "0.0.1",
  };
}

export function exportWidgetJson(
  widget: ComponentNode,
  options: ImportWidgetOptions = {},
): string {
  return `${JSON.stringify(exportWidget(widget, options), null, 2)}\n`;
}

export function importWidget(
  json: string,
  options: ImportWidgetOptions = {},
): ImportResult {
  const value = parseExport(json);
  if (value.schema_version !== WIDGET_EXPORT_SCHEMA_VERSION) {
    throw new Error(
      `Unsupported widget export schema_version ${value.schema_version}; expected ${WIDGET_EXPORT_SCHEMA_VERSION}.`,
    );
  }
  validateKnownWidget(value.widget_type);
  const tagPaths = new Set(options.tagPaths ?? []);
  const warnings = bindingWarnings(value, tagPaths);
  return {
    widget: exportedToComponent(value, options.idFactory ?? defaultId),
    warnings,
  };
}

export function downloadExportedWidget(widget: ComponentNode, viewTitle: string) {
  const blob = new Blob([exportWidgetJson(widget)], { type: "application/json" });
  const anchor = document.createElement("a");
  anchor.href = URL.createObjectURL(blob);
  anchor.download = `${widget.kind.toLowerCase()}-${slugify(viewTitle)}.owhmi-widget`;
  anchor.click();
  URL.revokeObjectURL(anchor.href);
}

function parseExport(json: string): ExportedWidget {
  const value = JSON.parse(json) as Partial<ExportedWidget>;
  if (!value || typeof value !== "object") {
    throw new Error("Widget export must be a JSON object.");
  }
  if (typeof value.schema_version !== "number") {
    throw new Error("Widget export is missing schema_version.");
  }
  if (typeof value.widget_type !== "string" || value.widget_type.length === 0) {
    throw new Error("Widget export is missing widget_type.");
  }
  return {
    schema_version: value.schema_version,
    widget_type: value.widget_type,
    props: value.props ?? {},
    bindings: Array.isArray(value.bindings) ? value.bindings : [],
    children: Array.isArray(value.children) ? value.children : [],
    exported_at: typeof value.exported_at === "number" ? value.exported_at : 0,
    openwebhmi_version:
      typeof value.openwebhmi_version === "string" ? value.openwebhmi_version : "unknown",
  };
}

function componentToExportedChild(component: ComponentNode): ExportedWidgetChild {
  return {
    widget_type: component.kind,
    props: cloneValue(component.props),
    bindings: cloneValue(component.bindings),
    children: component.children.map(componentToExportedChild),
  };
}

function exportedToComponent(
  exported: ExportedWidget | ExportedWidgetChild,
  idFactory: (kind: string) => string,
): ComponentNode {
  validateKnownWidget(exported.widget_type);
  return {
    id: idFactory(exported.widget_type),
    kind: exported.widget_type,
    props: cloneValue(exported.props),
    bindings: cloneValue(exported.bindings),
    children: exported.children.map((child) => exportedToComponent(child, idFactory)),
  };
}

function bindingWarnings(exported: ExportedWidget, tagPaths: Set<string>): string[] {
  if (tagPaths.size === 0) {
    return tagBindingPaths(exported).map(
      (path) => `Binding path \`${path}\` was not found in this project's tag namespace.`,
    );
  }
  return tagBindingPaths(exported)
    .filter((path) => !tagPaths.has(path))
    .map((path) => `Binding path \`${path}\` was not found in this project's tag namespace.`);
}

function tagBindingPaths(exported: ExportedWidget | ExportedWidgetChild): string[] {
  const paths = exported.bindings.flatMap((binding) =>
    binding.source.kind === "tag" ? [binding.source.path] : [],
  );
  for (const child of exported.children) {
    paths.push(...tagBindingPaths(child));
  }
  return [...new Set(paths)];
}

function validateKnownWidget(kind: string) {
  if (!componentRegistry[kind]) {
    throw new Error(`Unknown widget type \`${kind}\`; import aborted.`);
  }
}

function defaultId(kind: string): string {
  return `${kind.toLowerCase()}-${Math.random().toString(36).slice(2, 8)}`;
}

function slugify(value: string): string {
  return value.toLowerCase().replace(/[^a-z0-9]+/g, "-").replace(/^-|-$/g, "") || "view";
}

function cloneValue<T>(value: T): T {
  return JSON.parse(JSON.stringify(value)) as T;
}
