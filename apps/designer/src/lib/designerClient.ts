import {
  isServerMessage,
  type ArtifactRef,
  type ChangeAction,
  type ClientMessage,
  type ServerMessage,
  type TagValue,
  type View,
  type AuthUser,
} from "@openwebhmi/protocol";

/** Driver config shape consumed by the designer project tree. */
export type DesignerDriver = {
  id: string;
  type: string;
  config?: unknown;
};

/** Tag config shape consumed by the designer tag browser. */
export type DesignerTag = {
  path: string;
  driver: string;
  address: string;
};

export type DesignerAlarmCondition =
  | { kind: "high_limit"; threshold: number }
  | { kind: "low_limit"; threshold: number }
  | { kind: "equals"; value: TagValue["value"] }
  | { kind: "deviation"; setpoint: number; tolerance: number }
  | { kind: "digital"; active_when: boolean };

export type DesignerAlarm = {
  id: string;
  label: string;
  priority: number;
  tag_path: string;
  condition: DesignerAlarmCondition;
  message: string;
  enabled: boolean;
  require_ack: boolean;
};

export type DesignerScriptTrigger =
  | { kind: "on_tag_change"; path: string }
  | { kind: "on_timer"; every_ms: number }
  | { kind: "on_alarm"; alarm_id: string }
  | { kind: "on_button_click"; component_id: string };

export type DesignerScript = {
  id: string;
  path: string;
  enabled: boolean;
  triggers: DesignerScriptTrigger[];
  handler_timeout_ms?: number | null;
};

export type DesignerScriptEvent = Extract<ServerMessage, { kind: "script.event" }>;

/** Project snapshot loaded from the gateway. */
export type DesignerProject = {
  id: string;
  schema_version: number;
  name: string;
  version: number;
  drivers: DesignerDriver[];
  tags: DesignerTag[];
  alarms: DesignerAlarm[];
  scripts: DesignerScript[];
  views: View[];
};

/** Project change notification routed from the gateway. */
export type DesignerProjectChange = {
  project_id: string;
  version: number;
  artifact: ArtifactRef;
  action: ChangeAction;
};

type WebSocketCtor = new (url: string) => WebSocket;
type ProjectChangeCallback = (change: DesignerProjectChange) => void;
type ScriptEventCallback = (event: DesignerScriptEvent) => void;

/** Gateway client used by the form-based designer. */
export class DesignerClient {
  private ws: WebSocket | null = null;
  private readonly projectChangeCallbacks = new Set<ProjectChangeCallback>();
  private readonly scriptEventCallbacks = new Map<string, Set<ScriptEventCallback>>();
  private readonly pendingProjectLoads: Array<{
    resolve: (project: DesignerProject) => void;
    reject: (error: Error) => void;
  }> = [];
  private readonly pendingSaves = new Map<
    string,
    {
      resolve: (version: number) => void;
      reject: (error: Error) => void;
    }
  >();
  private readonly pendingArtifactReads = new Map<
    string,
    {
      resolve: (body: unknown) => void;
      reject: (error: Error) => void;
    }
  >();
  private readonly pendingUserLists: Array<{
    resolve: (users: AuthUser[]) => void;
    reject: (error: Error) => void;
  }> = [];

  constructor(
    private readonly url: string,
    private token: string | null = null,
    private readonly webSocketImpl: WebSocketCtor = WebSocket,
  ) {}

  /** Open the websocket connection. */
  connect(): Promise<void> {
    if (this.ws?.readyState === WebSocket.OPEN) {
      return Promise.resolve();
    }

    return new Promise((resolve, reject) => {
      const ws = new this.webSocketImpl(withToken(this.url, this.token));
      this.ws = ws;
      ws.onopen = () => resolve();
      ws.onerror = () => reject(new Error(`failed to connect to ${this.url}`));
      ws.onmessage = (event) => this.handleMessage(event.data);
      ws.onclose = () => {
        this.rejectPending(new Error("gateway connection closed"));
      };
    });
  }

  /** Authenticate and store the returned JWT for the next connection. */
  login(username: string, password: string): Promise<void> {
    return new Promise((resolve, reject) => {
      const ws = new this.webSocketImpl(this.url);
      ws.onopen = () => {
        ws.send(JSON.stringify({ kind: "auth.login", username, password }));
      };
      ws.onerror = () => reject(new Error(`failed to connect to ${this.url}`));
      ws.onclose = () => reject(new Error("gateway closed before login completed"));
      ws.onmessage = (event) => {
        const parsed = parseServerMessage(event.data);
        if (!parsed) {
          return;
        }
        if (parsed.kind === "auth.result") {
          if (!parsed.session_token) {
            reject(new Error(parsed.error ?? "login failed"));
            return;
          }
          this.token = parsed.session_token;
          ws.onclose = null;
          ws.close();
          resolve();
        } else if (parsed.kind === "error") {
          reject(new Error(`${parsed.code}: ${parsed.message}`));
        }
      };
    });
  }

  /** Close the websocket connection. */
  disconnect() {
    this.ws?.close();
    this.ws = null;
  }

  /** Load a project snapshot. */
  loadProject(projectId: string): Promise<DesignerProject> {
    const request = new Promise<DesignerProject>((resolve, reject) => {
      this.pendingProjectLoads.push({ resolve, reject });
    });
    this.send({ kind: "project.load", project_id: projectId });
    return request;
  }

  /** Subscribe to project changes. */
  subscribeProject(
    projectId: string,
    callback: ProjectChangeCallback,
  ): () => void {
    this.projectChangeCallbacks.add(callback);
    this.send({ kind: "project.subscribe", project_id: projectId });
    return () => {
      this.projectChangeCallbacks.delete(callback);
      this.send({ kind: "project.unsubscribe", project_id: projectId });
    };
  }

  /** Save a view artifact through the gateway. */
  saveView(projectId: string, view: View): Promise<number> {
    return this.saveArtifact(projectId, { kind: "view", id: view.id }, view);
  }

  /** Save the project alarm definition artifact. */
  saveAlarms(projectId: string, alarms: DesignerAlarm[]): Promise<number> {
    return this.saveArtifact(projectId, { kind: "alarms" }, { alarms });
  }

  private saveArtifact(
    projectId: string,
    artifact: ArtifactRef,
    body: unknown,
  ): Promise<number> {
    const requestId = `save-${Date.now()}-${randomSuffix()}`;
    const request = new Promise<number>((resolve, reject) => {
      this.pendingSaves.set(requestId, { resolve, reject });
    });
    this.send({
      kind: "project.save_artifact",
      request_id: requestId,
      project_id: projectId,
      artifact,
      body,
    });
    return request;
  }

  /** Load raw Python source for a project script. */
  async loadScriptSource(projectId: string, scriptId: string): Promise<string> {
    const requestId = `script-read-${scriptId}-${Date.now()}-${randomSuffix()}`;
    const request = new Promise<unknown>((resolve, reject) => {
      this.pendingArtifactReads.set(requestId, { resolve, reject });
    });
    this.send({
      kind: "project.read_artifact",
      request_id: requestId,
      project_id: projectId,
      artifact: { kind: "script_source", id: scriptId },
    });
    const body = await request;
    return isSourceBody(body) ? body.source : "";
  }

  /** Save raw Python source for a project script. */
  saveScriptSource(projectId: string, scriptId: string, source: string): Promise<number> {
    return this.saveArtifact(projectId, { kind: "script_source", id: scriptId }, { source });
  }

  /** Save one script config envelope. */
  saveScript(projectId: string, script: DesignerScript): Promise<number> {
    return this.saveArtifact(projectId, { kind: "script", id: script.id }, script);
  }

  /** Delete one project artifact. */
  deleteArtifact(projectId: string, artifact: ArtifactRef): Promise<void> {
    const requestId = `delete-${Date.now()}-${randomSuffix()}`;
    const request = new Promise<void>((resolve, reject) => {
      this.pendingSaves.set(requestId, {
        resolve: () => resolve(),
        reject,
      });
    });
    this.send({
      kind: "project.delete_artifact",
      request_id: requestId,
      project_id: projectId,
      artifact,
    });
    return request;
  }

  /** Subscribe to gateway script events for a project. */
  subscribeScriptEvents(
    projectId: string,
    callback: ScriptEventCallback,
  ): () => void {
    const callbacks = this.scriptEventCallbacks.get(projectId) ?? new Set<ScriptEventCallback>();
    const first = callbacks.size === 0;
    callbacks.add(callback);
    this.scriptEventCallbacks.set(projectId, callbacks);
    if (first) {
      this.send({ kind: "script.subscribe", project_id: projectId });
    }
    return () => {
      callbacks.delete(callback);
      if (callbacks.size === 0) {
        this.scriptEventCallbacks.delete(projectId);
        this.send({ kind: "script.unsubscribe", project_id: projectId });
      }
    };
  }

  /** List local users. */
  listUsers(): Promise<AuthUser[]> {
    const request = new Promise<AuthUser[]>((resolve, reject) => {
      this.pendingUserLists.push({ resolve, reject });
    });
    this.send({ kind: "user.list" });
    return request;
  }

  /** Create or update a local user. */
  upsertUser(username: string, password: string | null, roles: string[]): Promise<AuthUser[]> {
    const request = new Promise<AuthUser[]>((resolve, reject) => {
      this.pendingUserLists.push({ resolve, reject });
    });
    this.send({ kind: "user.upsert", username, password, roles });
    return request;
  }

  /** Delete a local user. */
  deleteUser(userId: string): Promise<AuthUser[]> {
    const request = new Promise<AuthUser[]>((resolve, reject) => {
      this.pendingUserLists.push({ resolve, reject });
    });
    this.send({ kind: "user.delete", user_id: userId });
    return request;
  }

  private send(message: ClientMessage) {
    if (this.ws?.readyState !== WebSocket.OPEN) {
      throw new Error("gateway is not connected");
    }
    this.ws.send(JSON.stringify(message));
  }

  private handleMessage(raw: string) {
    const parsed = parseServerMessage(raw);
    if (!parsed) {
      return;
    }

    switch (parsed.kind) {
      case "project.snapshot": {
        const pending = this.pendingProjectLoads.shift();
        pending?.resolve(parsed.project as DesignerProject);
        break;
      }
      case "project.save_result": {
        const requestId = parsed.request_id ?? "";
        const pending = this.pendingSaves.get(requestId);
        if (pending) {
          this.pendingSaves.delete(requestId);
          pending.resolve(parsed.version);
        }
        break;
      }
      case "project.delete_result": {
        const requestId = parsed.request_id ?? "";
        const pending = this.pendingSaves.get(requestId);
        if (pending) {
          this.pendingSaves.delete(requestId);
          pending.resolve(0);
        }
        break;
      }
      case "project.artifact": {
        const requestId = parsed.request_id ?? "";
        const pending = this.pendingArtifactReads.get(requestId);
        if (pending) {
          this.pendingArtifactReads.delete(requestId);
          pending.resolve(parsed.body);
        }
        break;
      }
      case "project.changed":
        for (const callback of this.projectChangeCallbacks) {
          callback(parsed);
        }
        break;
      case "script.event": {
        const callbacks = this.scriptEventCallbacks.get(parsed.project_id);
        if (callbacks) {
          for (const callback of callbacks) {
            callback(parsed);
          }
        }
        break;
      }
      case "user.list": {
        const pending = this.pendingUserLists.shift();
        pending?.resolve(parsed.users);
        break;
      }
      case "error":
        this.rejectNext(new Error(`${parsed.code}: ${parsed.message}`));
        break;
      default:
        break;
    }
  }

  private rejectNext(error: Error) {
    const load = this.pendingProjectLoads.shift();
    if (load) {
      load.reject(error);
      return;
    }
    const [requestId, save] = this.pendingSaves.entries().next().value ?? [];
    if (requestId && save) {
      this.pendingSaves.delete(requestId);
      save.reject(error);
    }
  }

  private rejectPending(error: Error) {
    for (const pending of this.pendingProjectLoads.splice(0)) {
      pending.reject(error);
    }
    for (const [requestId, pending] of this.pendingSaves) {
      this.pendingSaves.delete(requestId);
      pending.reject(error);
    }
    for (const [requestId, pending] of this.pendingArtifactReads) {
      this.pendingArtifactReads.delete(requestId);
      pending.reject(error);
    }
    for (const pending of this.pendingUserLists.splice(0)) {
      pending.reject(error);
    }
  }
}

function randomSuffix(): string {
  return Math.random().toString(36).slice(2, 8);
}

function isSourceBody(value: unknown): value is { source: string } {
  return (
    typeof value === "object" &&
    value !== null &&
    "source" in value &&
    typeof (value as { source?: unknown }).source === "string"
  );
}

function parseServerMessage(raw: string): ServerMessage | null {
  try {
    const parsed: unknown = JSON.parse(raw);
    return isServerMessage(parsed) ? parsed : null;
  } catch {
    return null;
  }
}

function withToken(url: string, token: string | null): string {
  if (!token) {
    return url;
  }
  const parsed = new URL(url);
  parsed.searchParams.set("token", token);
  return parsed.toString();
}
