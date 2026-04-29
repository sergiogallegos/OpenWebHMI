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
  allowedRoles?: string[] | null;
  root: ComponentNode;
  schema_version: number;
};

export type HistoryPoint = {
  ts_ms: number;
  value: TagValue;
  quality: Quality;
};

export type AuthUser = {
  id: string;
  username: string;
  roles: string[];
};

export type AlarmState = "clear" | "active" | "acked" | "cleared";

/** Messages accepted by the gateway from a runtime or designer client. */
export type ClientMessage =
  | { kind: "auth.login"; username: string; password: string }
  | { kind: "auth.logout" }
  | {
      kind: "alarm.subscribe";
      project_id: string;
      priority_min?: number | null;
      priority_max?: number | null;
    }
  | { kind: "alarm.ack"; alarm_id: string; note?: string | null }
  | { kind: "tag.subscribe"; paths: string[] }
  | { kind: "tag.unsubscribe"; paths: string[] }
  | { kind: "tag.write"; path: string; value: TagValue }
  | { kind: "ping" }
  | { kind: "project.subscribe"; project_id: string }
  | { kind: "project.unsubscribe"; project_id: string }
  | { kind: "project.load"; project_id: string }
  | {
      kind: "history.read";
      request_id?: string | null;
      tag_path: string;
      t_start_ms: number;
      t_end_ms: number;
      aggregation: string;
      max_points: number;
    }
  | {
      kind: "project.save_artifact";
      request_id?: string | null;
      project_id: string;
      artifact: ArtifactRef;
      body: unknown;
    }
  | { kind: "view.open"; project_id: string; view_id: string }
  | { kind: "view.close"; project_id: string; view_id: string }
  | { kind: "user.list" }
  | {
      kind: "user.upsert";
      username: string;
      password?: string | null;
      roles: string[];
    }
  | { kind: "user.delete"; user_id: string }
  | {
      kind: "script.run";
      request_id?: string | null;
      script_id: string;
      trigger: string;
      args?: unknown;
    };

/** Messages emitted by the gateway to runtime or designer clients. */
export type ServerMessage =
  | {
      kind: "auth.result";
      session_token?: string | null;
      user_id?: string | null;
      roles: string[];
      error?: string | null;
    }
  | {
      kind: "alarm.event";
      alarm_id: string;
      label: string;
      priority: number;
      state: AlarmState;
      tag_path: string;
      value: TagValue;
      quality: Quality;
      activated_at_ms?: number | null;
      transitioned_at_ms: number;
      who?: string | null;
      note?: string | null;
      message: string;
    }
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
      kind: "history.result";
      request_id?: string | null;
      tag_path: string;
      points: HistoryPoint[];
    }
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
    }
  | { kind: "user.list"; users: AuthUser[] }
  | {
      kind: "script.result";
      request_id?: string | null;
      script_id: string;
      result: unknown;
    }
  | {
      kind: "script.error";
      request_id?: string | null;
      script_id: string;
      message: string;
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
    case "auth.login":
      return (
        typeof value.username === "string" &&
        typeof value.password === "string"
      );
    case "auth.logout":
      return true;
    case "alarm.subscribe":
      return (
        typeof value.project_id === "string" &&
        optionalFiniteNumber(value, "priority_min") &&
        optionalFiniteNumber(value, "priority_max")
      );
    case "alarm.ack":
      return (
        typeof value.alarm_id === "string" &&
        ("note" in value
          ? value.note === null || typeof value.note === "string"
          : true)
      );
    case "tag.subscribe":
    case "tag.unsubscribe":
      return isStringArray(value.paths);
    case "tag.write":
      return typeof value.path === "string" && isTagValue(value.value);
    case "ping":
      return true;
    case "project.subscribe":
    case "project.unsubscribe":
    case "project.load":
      return typeof value.project_id === "string";
    case "history.read":
      return (
        typeof value.tag_path === "string" &&
        typeof value.t_start_ms === "number" &&
        Number.isFinite(value.t_start_ms) &&
        typeof value.t_end_ms === "number" &&
        Number.isFinite(value.t_end_ms) &&
        typeof value.aggregation === "string" &&
        typeof value.max_points === "number" &&
        Number.isFinite(value.max_points) &&
        ("request_id" in value
          ? value.request_id === null || typeof value.request_id === "string"
          : true)
      );
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
    case "user.list":
      return true;
    case "user.upsert":
      return (
        typeof value.username === "string" &&
        isStringArray(value.roles) &&
        ("password" in value
          ? value.password === null || typeof value.password === "string"
          : true)
      );
    case "user.delete":
      return typeof value.user_id === "string";
    case "script.run":
      return (
        typeof value.script_id === "string" &&
        typeof value.trigger === "string" &&
        ("request_id" in value
          ? value.request_id === null || typeof value.request_id === "string"
          : true)
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
    case "auth.result":
      return (
        isStringArray(value.roles) &&
        ("session_token" in value
          ? value.session_token === null ||
            typeof value.session_token === "string"
          : true) &&
        ("user_id" in value
          ? value.user_id === null || typeof value.user_id === "string"
          : true) &&
        ("error" in value
          ? value.error === null || typeof value.error === "string"
          : true)
      );
    case "alarm.event":
      return (
        typeof value.alarm_id === "string" &&
        typeof value.label === "string" &&
        typeof value.priority === "number" &&
        Number.isFinite(value.priority) &&
        isAlarmState(value.state) &&
        typeof value.tag_path === "string" &&
        isTagValue(value.value) &&
        isQuality(value.quality) &&
        optionalFiniteNumber(value, "activated_at_ms") &&
        typeof value.transitioned_at_ms === "number" &&
        Number.isFinite(value.transitioned_at_ms) &&
        ("who" in value
          ? value.who === null || typeof value.who === "string"
          : true) &&
        ("note" in value
          ? value.note === null || typeof value.note === "string"
          : true) &&
        typeof value.message === "string"
      );
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
    case "history.result":
      return (
        typeof value.tag_path === "string" &&
        Array.isArray(value.points) &&
        value.points.every(isHistoryPoint) &&
        ("request_id" in value
          ? value.request_id === null || typeof value.request_id === "string"
          : true)
      );
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
    case "user.list":
      return Array.isArray(value.users) && value.users.every(isAuthUser);
    case "script.result":
      return (
        typeof value.script_id === "string" &&
        ("request_id" in value
          ? value.request_id === null || typeof value.request_id === "string"
          : true) &&
        "result" in value
      );
    case "script.error":
      return (
        typeof value.script_id === "string" &&
        typeof value.message === "string" &&
        ("request_id" in value
          ? value.request_id === null || typeof value.request_id === "string"
          : true)
      );
    default:
      return false;
  }
}

function isAuthUser(value: unknown): value is AuthUser {
  return (
    isRecord(value) &&
    typeof value.id === "string" &&
    typeof value.username === "string" &&
    isStringArray(value.roles)
  );
}

function isAlarmState(value: unknown): value is AlarmState {
  return (
    value === "clear" ||
    value === "active" ||
    value === "acked" ||
    value === "cleared"
  );
}

function optionalFiniteNumber(
  value: Record<string, unknown>,
  key: string,
): boolean {
  return key in value
    ? value[key] === null ||
        (typeof value[key] === "number" && Number.isFinite(value[key]))
    : true;
}

function isHistoryPoint(value: unknown): value is HistoryPoint {
  return (
    isRecord(value) &&
    typeof value.ts_ms === "number" &&
    Number.isFinite(value.ts_ms) &&
    isTagValue(value.value) &&
    isQuality(value.quality)
  );
}

export function isView(value: unknown): value is View {
  return (
    isRecord(value) &&
    typeof value.id === "string" &&
    typeof value.title === "string" &&
    ("allowedRoles" in value
      ? value.allowedRoles === null || isStringArray(value.allowedRoles)
      : true) &&
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
