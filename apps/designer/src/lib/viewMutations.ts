import { components } from "@openwebhmi/component-library";
import type {
  Binding,
  BindingSource,
  ComponentNode,
  View,
} from "@openwebhmi/protocol";

/** Direction for sibling reordering inside a component tree. */
export type MoveDirection = "up" | "down";

/** Create a component node from the registered component defaults. */
export function createComponent(kind: string, id = uniqueId(kind)): ComponentNode {
  const definition = components.find((component) => component.kind === kind);
  return {
    id,
    kind,
    props: definition ? structuredCloneValue(definition.defaultProps) : {},
    bindings: [],
    children: [],
  };
}

/** Find a component node by id. */
export function findNode(view: View, id: string): ComponentNode | null {
  return findNodeInTree(view.root, id);
}

/** Add a child to a parent component. */
export function addChild(
  view: View,
  parentId: string,
  child: ComponentNode,
): View {
  return updateNode(view, parentId, (node) => ({
    ...node,
    children: [...node.children, structuredCloneValue(child)],
  }));
}

/** Remove a component. The root component cannot be removed. */
export function removeComponent(view: View, id: string): View {
  if (view.root.id === id) {
    return structuredCloneValue(view);
  }
  const next = structuredCloneValue(view);
  next.root = removeFromChildren(next.root, id);
  return next;
}

/** Move a component among its siblings. */
export function moveComponent(
  view: View,
  id: string,
  direction: MoveDirection,
): View {
  if (view.root.id === id) {
    return structuredCloneValue(view);
  }
  const next = structuredCloneValue(view);
  next.root = moveInChildren(next.root, id, direction);
  return next;
}

/** Duplicate a component and insert the copy after the original. */
export function duplicateComponent(view: View, id: string): View {
  if (view.root.id === id) {
    return structuredCloneValue(view);
  }
  const next = structuredCloneValue(view);
  next.root = duplicateInChildren(next.root, id);
  return next;
}

/** Set one prop on a component. */
export function setProp(
  view: View,
  componentId: string,
  prop: string,
  value: unknown,
): View {
  return updateNode(view, componentId, (node) => ({
    ...node,
    props: {
      ...recordProps(node.props),
      [prop]: value,
    },
  }));
}

/** Set or replace one binding on a component prop. */
export function setBinding(
  view: View,
  componentId: string,
  prop: string,
  source: BindingSource,
): View {
  return updateNode(view, componentId, (node) => {
    const bindings = node.bindings.filter((binding) => binding.prop !== prop);
    bindings.push({ prop, source });
    return { ...node, bindings };
  });
}

/** Remove a binding from a component prop. */
export function clearBinding(view: View, componentId: string, prop: string): View {
  return updateNode(view, componentId, (node) => ({
    ...node,
    bindings: node.bindings.filter((binding) => binding.prop !== prop),
  }));
}

/** Return the binding for one prop. */
export function bindingFor(
  node: ComponentNode | null,
  prop: string,
): Binding | undefined {
  return node?.bindings.find((binding) => binding.prop === prop);
}

function updateNode(
  view: View,
  id: string,
  updater: (node: ComponentNode) => ComponentNode,
): View {
  const next = structuredCloneValue(view);
  next.root = updateNodeInTree(next.root, id, updater);
  return next;
}

function updateNodeInTree(
  node: ComponentNode,
  id: string,
  updater: (node: ComponentNode) => ComponentNode,
): ComponentNode {
  if (node.id === id) {
    return updater(node);
  }
  return {
    ...node,
    children: node.children.map((child) => updateNodeInTree(child, id, updater)),
  };
}

function removeFromChildren(node: ComponentNode, id: string): ComponentNode {
  return {
    ...node,
    children: node.children
      .filter((child) => child.id !== id)
      .map((child) => removeFromChildren(child, id)),
  };
}

function moveInChildren(
  node: ComponentNode,
  id: string,
  direction: MoveDirection,
): ComponentNode {
  const children = [...node.children];
  const index = children.findIndex((child) => child.id === id);
  if (index >= 0) {
    const nextIndex = direction === "up" ? index - 1 : index + 1;
    if (nextIndex >= 0 && nextIndex < children.length) {
      const [child] = children.splice(index, 1);
      children.splice(nextIndex, 0, child);
    }
    return { ...node, children };
  }
  return {
    ...node,
    children: children.map((child) => moveInChildren(child, id, direction)),
  };
}

function duplicateInChildren(node: ComponentNode, id: string): ComponentNode {
  const children: ComponentNode[] = [];
  for (const child of node.children) {
    children.push(duplicateInChildren(child, id));
    if (child.id === id) {
      children.push(withFreshIds(child));
    }
  }
  return { ...node, children };
}

function withFreshIds(node: ComponentNode): ComponentNode {
  return {
    ...structuredCloneValue(node),
    id: `${node.id}-copy`,
    children: node.children.map((child) => withFreshIds(child)),
  };
}

function findNodeInTree(node: ComponentNode, id: string): ComponentNode | null {
  if (node.id === id) {
    return node;
  }
  for (const child of node.children) {
    const found = findNodeInTree(child, id);
    if (found) {
      return found;
    }
  }
  return null;
}

function recordProps(value: unknown): Record<string, unknown> {
  if (typeof value === "object" && value !== null && !Array.isArray(value)) {
    return value as Record<string, unknown>;
  }
  return {};
}

function uniqueId(kind: string): string {
  return `${kind.toLowerCase()}-${Math.random().toString(36).slice(2, 8)}`;
}

function structuredCloneValue<T>(value: T): T {
  return JSON.parse(JSON.stringify(value)) as T;
}
