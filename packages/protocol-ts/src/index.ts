/** A tag's current value using the Rust serde adjacent-tag wire form. */
export type TagValue =
  | { type: "bool"; value: boolean }
  | { type: "int"; value: number }
  | { type: "real"; value: number }
  | { type: "string"; value: string };

/** OPC UA-style quality carried with each tag update. */
export type Quality = "good" | "bad" | "uncertain" | "stale";

export type ArtifactRef =
  | { kind: "project_meta" }
  | { kind: "view"; id: string }
  | { kind: "tags" }
  | { kind: "alarms" }
  | { kind: "script"; id: string };

export type ChangeAction = "created" | "updated" | "deleted";

export type BindingSource =
  | { kind: "tag"; path: string }
  | { kind: "expression"; code: string }
  | { kind: "constant"; value: unknown };

export type Binding = {
  prop: string;
  source: BindingSource;
};

export type ComponentNode = {
  id: string;
  kind: string;
  props: unknown;
  bindings: Binding[];
  children: ComponentNode[];
};

export type View = {
  id: string;
  title: string;
  root: ComponentNode;
  schema_version: number;
};

/** Messages accepted by the gateway from a runtime or designer client. */
export type ClientMessage =
  | { kind: "tag.subscribe"; paths: string[] }
  | { kind: "tag.unsubscribe"; paths: string[] }
  | { kind: "ping" }
  | { kind: "project.subscribe"; project_id: string }
  | { kind: "project.unsubscribe"; project_id: string }
  | { kind: "project.load"; project_id: string }
  | {
      kind: "project.save_artifact";
      request_id?: string | null;
      project_id: string;
      artifact: ArtifactRef;
      body: unknown;
    }
  | { kind: "view.open"; project_id: string; view_id: string }
  | { kind: "view.close"; project_id: string; view_id: string };

/** Messages emitted by the gateway to runtime or designer clients. */
export type ServerMessage =
  | {
      kind: "tag.update";
      path: string;
      value: TagValue;
      quality: Quality;
      ts: number;
    }
  | { kind: "pong" }
  | { kind: "error"; code: string; message: string }
  | { kind: "project.snapshot"; project: unknown; version: number }
  | {
      kind: "project.changed";
      project_id: string;
      version: number;
      artifact: ArtifactRef;
      action: ChangeAction;
    }
  | {
      kind: "project.save_result";
      request_id?: string | null;
      project_id: string;
      version: number;
    }
  | {
      kind: "view.definition";
      project_id: string;
      view_id: string;
      version: number;
      view: View;
    };

/** Return true when `value` is a valid OpenWebHMI tag value envelope. */
export function isTagValue(value: unknown): value is TagValue {
  if (!isRecord(value) || typeof value.type !== "string") {
    return false;
  }

  switch (value.type) {
    case "bool":
      return typeof value.value === "boolean";
    case "int":
    case "real":
      return typeof value.value === "number" && Number.isFinite(value.value);
    case "string":
      return typeof value.value === "string";
    default:
      return false;
  }
}

/** Return true when `value` is one of the stable quality literals. */
export function isQuality(value: unknown): value is Quality {
  return (
    value === "good" ||
    value === "bad" ||
    value === "uncertain" ||
    value === "stale"
  );
}

/** Return true when `value` is a client-to-gateway protocol message. */
export function isClientMessage(value: unknown): value is ClientMessage {
  if (!isRecord(value) || typeof value.kind !== "string") {
    return false;
  }

  switch (value.kind) {
    case "tag.subscribe":
    case "tag.unsubscribe":
      return isStringArray(value.paths);
    case "ping":
      return true;
    case "project.subscribe":
    case "project.unsubscribe":
    case "project.load":
      return typeof value.project_id === "string";
    case "project.save_artifact":
      return (
        typeof value.project_id === "string" &&
        isArtifactRef(value.artifact) &&
        ("request_id" in value
          ? value.request_id === null || typeof value.request_id === "string"
          : true)
      );
    case "view.open":
    case "view.close":
      return (
        typeof value.project_id === "string" &&
        typeof value.view_id === "string"
      );
    default:
      return false;
  }
}

/** Return true when `value` is a gateway-to-client protocol message. */
export function isServerMessage(value: unknown): value is ServerMessage {
  if (!isRecord(value) || typeof value.kind !== "string") {
    return false;
  }

  switch (value.kind) {
    case "tag.update":
      return (
        typeof value.path === "string" &&
        isTagValue(value.value) &&
        isQuality(value.quality) &&
        typeof value.ts === "number" &&
        Number.isFinite(value.ts)
      );
    case "pong":
      return true;
    case "error":
      return typeof value.code === "string" && typeof value.message === "string";
    case "project.snapshot":
      return typeof value.version === "number" && Number.isFinite(value.version);
    case "project.changed":
      return (
        typeof value.project_id === "string" &&
        typeof value.version === "number" &&
        Number.isFinite(value.version) &&
        isArtifactRef(value.artifact) &&
        isChangeAction(value.action)
      );
    case "project.save_result":
      return (
        typeof value.project_id === "string" &&
        typeof value.version === "number" &&
        Number.isFinite(value.version) &&
        ("request_id" in value
          ? value.request_id === null || typeof value.request_id === "string"
          : true)
      );
    case "view.definition":
      return (
        typeof value.project_id === "string" &&
        typeof value.view_id === "string" &&
        typeof value.version === "number" &&
        Number.isFinite(value.version) &&
        isView(value.view)
      );
    default:
      return false;
  }
}

export function isView(value: unknown): value is View {
  return (
    isRecord(value) &&
    typeof value.id === "string" &&
    typeof value.title === "string" &&
    typeof value.schema_version === "number" &&
    isComponentNode(value.root)
  );
}

function isComponentNode(value: unknown): value is ComponentNode {
  return (
    isRecord(value) &&
    typeof value.id === "string" &&
    typeof value.kind === "string" &&
    Array.isArray(value.bindings) &&
    value.bindings.every(isBinding) &&
    Array.isArray(value.children) &&
    value.children.every(isComponentNode) &&
    "props" in value
  );
}

function isBinding(value: unknown): value is Binding {
  return (
    isRecord(value) &&
    typeof value.prop === "string" &&
    isBindingSource(value.source)
  );
}

function isBindingSource(value: unknown): value is BindingSource {
  if (!isRecord(value) || typeof value.kind !== "string") {
    return false;
  }
  switch (value.kind) {
    case "tag":
      return typeof value.path === "string";
    case "expression":
      return typeof value.code === "string";
    case "constant":
      return "value" in value;
    default:
      return false;
  }
}

function isArtifactRef(value: unknown): value is ArtifactRef {
  if (!isRecord(value) || typeof value.kind !== "string") {
    return false;
  }
  switch (value.kind) {
    case "project_meta":
    case "tags":
    case "alarms":
      return true;
    case "view":
    case "script":
      return typeof value.id === "string";
    default:
      return false;
  }
}

function isChangeAction(value: unknown): value is ChangeAction {
  return value === "created" || value === "updated" || value === "deleted";
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null && !Array.isArray(value);
}

function isStringArray(value: unknown): value is string[] {
  return Array.isArray(value) && value.every((item) => typeof item === "string");
}
