import {
  isServerMessage,
  type ArtifactRef,
  type ChangeAction,
  type ClientMessage,
  type View,
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

  constructor(
    private readonly url: string,
    private readonly webSocketImpl: WebSocketCtor = WebSocket,
  ) {}

  /** Open the websocket connection. */
  connect(): Promise<void> {
    if (this.ws?.readyState === WebSocket.OPEN) {
      return Promise.resolve();
    }

    return new Promise((resolve, reject) => {
      const ws = new this.webSocketImpl(this.url);
      this.ws = ws;
      ws.onopen = () => resolve();
      ws.onerror = () => reject(new Error(`failed to connect to ${this.url}`));
      ws.onmessage = (event) => this.handleMessage(event.data);
      ws.onclose = () => {
        this.rejectPending(new Error("gateway connection closed"));
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

  private send(message: ClientMessage) {
    if (this.ws?.readyState !== WebSocket.OPEN) {
      throw new Error("gateway is not connected");
    }
    this.ws.send(JSON.stringify(message));
  }

  private handleMessage(raw: string) {
    let parsed: unknown;
    try {
      parsed = JSON.parse(raw);
    } catch {
      return;
    }
    if (!isServerMessage(parsed)) {
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
  }
}
