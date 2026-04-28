import { componentRegistry, type BoundValue } from "@openwebhmi/component-library";
import type { AlarmEvent, AlarmSubscribeOptions } from "@openwebhmi/component-library";
import type {
  BindingSource,
  ComponentNode,
  TagValue,
  View,
} from "@openwebhmi/protocol";

type ViewRendererProps = {
  view: View;
  projectId: string;
  boundValues: Record<string, BoundValue | undefined>;
  onWriteTag: (path: string, value: TagValue) => void;
  onSubscribeAlarms: (
    options: AlarmSubscribeOptions,
    callback: (event: AlarmEvent) => void,
  ) => () => void;
  onAckAlarm: (alarmId: string, note?: string | null) => void;
};

export function ViewRenderer({
  view,
  projectId,
  boundValues,
  onWriteTag,
  onSubscribeAlarms,
  onAckAlarm,
}: ViewRendererProps) {
  return (
    <section aria-label={view.title} style={styles.surface}>
      {renderNode(view.root, boundValues, {
        projectId,
        onWriteTag,
        onSubscribeAlarms,
        onAckAlarm,
      })}
    </section>
  );
}

export function collectTagPaths(view: View): string[] {
  const paths = new Set<string>();
  walk(view.root, (node) => {
    for (const binding of node.bindings) {
      if (binding.source.kind === "tag") {
        paths.add(binding.source.path);
      }
    }
  });
  return [...paths].sort();
}

function renderNode(
  node: ComponentNode,
  boundValues: Record<string, BoundValue | undefined>,
  runtime: {
    projectId: string;
    onWriteTag: (path: string, value: TagValue) => void;
    onSubscribeAlarms: (
      options: AlarmSubscribeOptions,
      callback: (event: AlarmEvent) => void,
    ) => () => void;
    onAckAlarm: (alarmId: string, note?: string | null) => void;
  },
) {
  const definition = componentRegistry[node.kind];
  if (!definition) {
    return (
      <div key={node.id} role="alert" style={styles.unknown}>
        Unknown component: {node.kind}
      </div>
    );
  }

  const props = {
    ...definition.defaultProps,
    ...recordProps(node.props),
  };
  const bindings = bindingsForNode(node, boundValues);
  const children = node.children.map((child) =>
    renderNode(child, boundValues, runtime),
  );

  return (
    <definition.Render
      key={node.id}
      props={props}
      bindings={bindings}
      context={{ mode: "runtime", ...runtime }}
    >
      {children}
    </definition.Render>
  );
}

function bindingsForNode(
  node: ComponentNode,
  boundValues: Record<string, BoundValue | undefined>,
): Record<string, BoundValue | undefined> {
  const bindings: Record<string, BoundValue | undefined> = {};
  for (const binding of node.bindings) {
    bindings[binding.prop] = bindingFromSource(binding.source, boundValues);
  }
  return bindings;
}

function bindingFromSource(
  source: BindingSource,
  boundValues: Record<string, BoundValue | undefined>,
): BoundValue | undefined {
  if (source.kind === "tag") {
    return boundValues[source.path];
  }
  if (source.kind === "constant") {
    const value = tagValueFromUnknown(source.value);
    return value ? { value, quality: "good", ts: 0 } : undefined;
  }
  return undefined;
}

function tagValueFromUnknown(value: unknown): TagValue | undefined {
  switch (typeof value) {
    case "boolean":
      return { type: "bool", value };
    case "number":
      return Number.isInteger(value)
        ? { type: "int", value }
        : { type: "real", value };
    case "string":
      return { type: "string", value };
    default:
      return undefined;
  }
}

function recordProps(value: unknown): Record<string, unknown> {
  if (typeof value === "object" && value !== null && !Array.isArray(value)) {
    return value as Record<string, unknown>;
  }
  return {};
}

function walk(node: ComponentNode, visit: (node: ComponentNode) => void) {
  visit(node);
  for (const child of node.children) {
    walk(child, visit);
  }
}

const styles = {
  surface: {
    minHeight: 0,
  },
  unknown: {
    padding: 12,
    border: "1px solid #f59e0b",
    borderRadius: 6,
    background: "#fffbeb",
    color: "#92400e",
    fontFamily:
      'Inter, ui-sans-serif, system-ui, -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif',
  },
};
