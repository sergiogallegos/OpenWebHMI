import {
  isServerMessage,
  type ArtifactRef,
  type ChangeAction,
  type ClientMessage,
  type ServerMessage,
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

/** Project snapshot loaded from the gateway. */
export type DesignerProject = {
  id: string;
  schema_version: number;
  name: string;
  version: number;
  drivers: DesignerDriver[];
  tags: DesignerTag[];
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

/** Gateway client used by the form-based designer. */
export class DesignerClient {
  private ws: WebSocket | null = null;
  private readonly projectChangeCallbacks = new Set<ProjectChangeCallback>();
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
    const requestId = `view-${view.id}-${Date.now()}-${Math.random()
      .toString(36)
      .slice(2, 8)}`;
    const request = new Promise<number>((resolve, reject) => {
      this.pendingSaves.set(requestId, { resolve, reject });
    });
    this.send({
      kind: "project.save_artifact",
      request_id: requestId,
      project_id: projectId,
      artifact: { kind: "view", id: view.id },
      body: view,
    });
    return request;
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
      case "project.changed":
        for (const callback of this.projectChangeCallbacks) {
          callback(parsed);
        }
        break;
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
    for (const pending of this.pendingUserLists.splice(0)) {
      pending.reject(error);
    }
  }
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
