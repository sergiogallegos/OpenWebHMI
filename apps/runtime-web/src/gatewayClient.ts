import {
  isServerMessage,
  type ClientMessage,
  type HistoryPoint,
  type Quality,
  type ServerMessage,
  type TagValue,
} from "@openwebhmi/protocol";

export type TagUpdate = {
  value: TagValue;
  quality: Quality;
  ts: number;
};

export type ViewDefinition = Extract<
  ServerMessage,
  { kind: "view.definition" }
>;

export type ProjectChange = Extract<
  ServerMessage,
  { kind: "project.changed" }
>;
export type AlarmEvent = Extract<ServerMessage, { kind: "alarm.event" }>;
export type AlarmSubscribeOptions = {
  projectId: string;
  priorityMin?: number | null;
  priorityMax?: number | null;
};
export type HistoryReadOptions = {
  tagPath: string;
  tStartMs: number;
  tEndMs: number;
  aggregation?: string;
  maxPoints?: number;
};

export type GatewayError = Extract<ServerMessage, { kind: "error" }>;
export type AuthResult = Extract<ServerMessage, { kind: "auth.result" }>;

export type ConnectionState =
  | "idle"
  | "connecting"
  | "connected"
  | "reconnecting"
  | "closed";

type TagCallback = (update: TagUpdate) => void;
type ViewCallback = (definition: ViewDefinition) => void;
type ProjectChangeCallback = (change: ProjectChange) => void;
type AlarmCallback = (event: AlarmEvent) => void;
type ErrorCallback = (error: GatewayError) => void;
type StateCallback = (state: ConnectionState) => void;
type WebSocketCtor = new (url: string) => WebSocketLike;

type WebSocketLike = {
  readonly readyState: number;
  onopen: ((event: Event) => void) | null;
  onmessage: ((event: MessageEvent<string>) => void) | null;
  onclose: ((event: CloseEvent) => void) | null;
  onerror: ((event: Event) => void) | null;
  send(data: string): void;
  close(): void;
};

const OPEN = 1;
const INITIAL_BACKOFF_MS = 250;
const MAX_BACKOFF_MS = 8_000;

export class GatewayClient {
  private ws: WebSocketLike | null = null;
  private reconnectDelayMs = INITIAL_BACKOFF_MS;
  private reconnectTimer: ReturnType<typeof setTimeout> | null = null;
  private manuallyClosed = false;
  private readonly callbacks = new Map<string, Set<TagCallback>>();
  private readonly viewCallbacks = new Map<string, Set<ViewCallback>>();
  private readonly projectCallbacks = new Map<string, Set<ProjectChangeCallback>>();
  private readonly alarmCallbacks = new Map<string, AlarmSubscription>();
  private readonly errorCallbacks = new Set<ErrorCallback>();
  private readonly pendingHistory = new Map<
    string,
    {
      resolve: (points: HistoryPoint[]) => void;
      reject: (error: Error) => void;
    }
  >();
  private readonly stateCallbacks = new Set<StateCallback>();
  private readonly lastTagUpdates = new Map<string, TagUpdate>();
  private currentState: ConnectionState = "idle";

  constructor(
    private readonly opts: {
      url: string;
      webSocketImpl?: WebSocketCtor;
      tokenProvider?: () => string | null;
    },
  ) {}

  get state(): ConnectionState {
    return this.currentState;
  }

  connect() {
    this.manuallyClosed = false;
    this.clearReconnectTimer();
    this.setState(
      this.currentState === "idle" || this.currentState === "closed"
        ? "connecting"
        : "reconnecting",
    );

    const WebSocketImpl = this.opts.webSocketImpl ?? WebSocket;
    const ws = new WebSocketImpl(withToken(this.opts.url, this.opts.tokenProvider?.() ?? null));
    this.ws = ws;

    ws.onopen = () => {
      this.reconnectDelayMs = INITIAL_BACKOFF_MS;
      this.setState("connected");
      this.resubscribeAll();
    };
    ws.onmessage = (event: MessageEvent<string>) =>
      this.handleMessage(event.data);
    ws.onclose = () => this.scheduleReconnect();
    ws.onerror = () => {
      this.scheduleReconnect();
    };
  }

  onStateChange(callback: StateCallback): () => void {
    this.stateCallbacks.add(callback);
    return () => {
      this.stateCallbacks.delete(callback);
    };
  }

  onError(callback: ErrorCallback): () => void {
    this.errorCallbacks.add(callback);
    return () => {
      this.errorCallbacks.delete(callback);
    };
  }

  subscribe(path: string, callback: TagCallback): () => void {
    const callbacks = this.callbacks.get(path) ?? new Set<TagCallback>();
    const wasEmpty = callbacks.size === 0;
    callbacks.add(callback);
    this.callbacks.set(path, callbacks);

    const last = this.lastTagUpdates.get(path);
    if (last) {
      callback(last);
    }

    if (wasEmpty) {
      this.send({ kind: "tag.subscribe", paths: [path] });
    }

    return () => this.unsubscribe(path, callback);
  }

  unsubscribe(path: string, callback?: TagCallback) {
    const callbacks = this.callbacks.get(path);
    if (!callbacks) {
      return;
    }

    if (callback) {
      callbacks.delete(callback);
    } else {
      callbacks.clear();
    }

    if (callbacks.size === 0) {
      this.callbacks.delete(path);
      this.lastTagUpdates.delete(path);
      this.send({ kind: "tag.unsubscribe", paths: [path] });
    }
  }

  subscribeProject(
    projectId: string,
    callback: ProjectChangeCallback,
  ): () => void {
    const callbacks =
      this.projectCallbacks.get(projectId) ?? new Set<ProjectChangeCallback>();
    const wasEmpty = callbacks.size === 0;
    callbacks.add(callback);
    this.projectCallbacks.set(projectId, callbacks);

    if (wasEmpty) {
      this.send({ kind: "project.subscribe", project_id: projectId });
    }

    return () => this.unsubscribeProject(projectId, callback);
  }

  unsubscribeProject(projectId: string, callback?: ProjectChangeCallback) {
    const callbacks = this.projectCallbacks.get(projectId);
    if (!callbacks) {
      return;
    }

    if (callback) {
      callbacks.delete(callback);
    } else {
      callbacks.clear();
    }

    if (callbacks.size === 0) {
      this.projectCallbacks.delete(projectId);
      this.send({ kind: "project.unsubscribe", project_id: projectId });
    }
  }

  openView(projectId: string, viewId: string, callback: ViewCallback): () => void {
    const key = viewKey(projectId, viewId);
    const callbacks = this.viewCallbacks.get(key) ?? new Set<ViewCallback>();
    const wasEmpty = callbacks.size === 0;
    callbacks.add(callback);
    this.viewCallbacks.set(key, callbacks);

    if (wasEmpty) {
      this.requestView(projectId, viewId);
    }

    return () => this.closeView(projectId, viewId, callback);
  }

  requestView(projectId: string, viewId: string) {
    this.send({ kind: "view.open", project_id: projectId, view_id: viewId });
  }

  closeView(projectId: string, viewId: string, callback?: ViewCallback) {
    const key = viewKey(projectId, viewId);
    const callbacks = this.viewCallbacks.get(key);
    if (!callbacks) {
      return;
    }

    if (callback) {
      callbacks.delete(callback);
    } else {
      callbacks.clear();
    }

    if (callbacks.size === 0) {
      this.viewCallbacks.delete(key);
      this.send({ kind: "view.close", project_id: projectId, view_id: viewId });
    }
  }

  writeTag(path: string, value: TagValue) {
    this.send({ kind: "tag.write", path, value });
  }

  subscribeAlarms(
    options: AlarmSubscribeOptions,
    callback: AlarmCallback,
  ): () => void {
    const key = alarmKey(options);
    const subscription =
      this.alarmCallbacks.get(key) ?? {
        options: normalizedAlarmOptions(options),
        callbacks: new Set<AlarmCallback>(),
      };
    const wasEmpty = subscription.callbacks.size === 0;
    subscription.callbacks.add(callback);
    this.alarmCallbacks.set(key, subscription);

    if (wasEmpty) {
      this.send(alarmSubscribeMessage(subscription.options));
    }

    return () => this.unsubscribeAlarms(options, callback);
  }

  unsubscribeAlarms(options: AlarmSubscribeOptions, callback?: AlarmCallback) {
    const key = alarmKey(options);
    const subscription = this.alarmCallbacks.get(key);
    if (!subscription) {
      return;
    }

    if (callback) {
      subscription.callbacks.delete(callback);
    } else {
      subscription.callbacks.clear();
    }

    if (subscription.callbacks.size === 0) {
      this.alarmCallbacks.delete(key);
    }
  }

  ackAlarm(alarmId: string, note?: string | null) {
    this.send({ kind: "alarm.ack", alarm_id: alarmId, note: note ?? null });
  }

  readHistory(options: HistoryReadOptions): Promise<HistoryPoint[]> {
    const requestId = `history-${Date.now()}-${Math.random()
      .toString(36)
      .slice(2, 8)}`;
    const request = new Promise<HistoryPoint[]>((resolve, reject) => {
      this.pendingHistory.set(requestId, { resolve, reject });
    });
    this.send({
      kind: "history.read",
      request_id: requestId,
      tag_path: options.tagPath,
      t_start_ms: options.tStartMs,
      t_end_ms: options.tEndMs,
      aggregation: options.aggregation ?? "raw",
      max_points: options.maxPoints ?? 600,
    });
    return request;
  }

  disconnect() {
    this.manuallyClosed = true;
    this.clearReconnectTimer();
    this.ws?.close();
    this.ws = null;
    this.setState("closed");
  }

  ping() {
    this.send({ kind: "ping" });
  }

  login(username: string, password: string): Promise<AuthResult> {
    const WebSocketImpl = this.opts.webSocketImpl ?? WebSocket;
    return new Promise((resolve, reject) => {
      const ws = new WebSocketImpl(this.opts.url);
      ws.onopen = () => {
        ws.send(JSON.stringify({ kind: "auth.login", username, password }));
      };
      ws.onerror = () => reject(new Error("failed to connect to gateway"));
      ws.onclose = () => reject(new Error("gateway closed before login completed"));
      ws.onmessage = (event: MessageEvent<string>) => {
        try {
          const parsed: unknown = JSON.parse(event.data);
          if (isServerMessage(parsed) && parsed.kind === "auth.result") {
            ws.onclose = null;
            ws.close();
            resolve(parsed);
          } else if (isServerMessage(parsed) && parsed.kind === "error") {
            ws.onclose = null;
            ws.close();
            reject(new Error(`${parsed.code}: ${parsed.message}`));
          }
        } catch (error) {
          reject(error instanceof Error ? error : new Error(String(error)));
        }
      };
    });
  }

  private send(message: ClientMessage) {
    this.sendRaw(message);
  }

  private sendRaw(message: unknown) {
    if (this.ws?.readyState !== OPEN) {
      return;
    }
    this.ws.send(JSON.stringify(message));
  }

  private handleMessage(data: string) {
    let parsed: unknown;
    try {
      parsed = JSON.parse(data);
    } catch {
      return;
    }

    if (!isServerMessage(parsed)) {
      return;
    }

    switch (parsed.kind) {
      case "tag.update":
        this.dispatchTagUpdate(parsed);
        break;
      case "view.definition":
        this.dispatchViewDefinition(parsed);
        break;
      case "project.changed":
        this.dispatchProjectChange(parsed);
        break;
      case "history.result":
        this.dispatchHistoryResult(parsed);
        break;
      case "alarm.event":
        this.dispatchAlarmEvent(parsed);
        break;
      case "error":
        this.rejectOldestHistory(parsed);
        for (const callback of this.errorCallbacks) {
          callback(parsed);
        }
        break;
      default:
        break;
    }
  }

  private dispatchTagUpdate(parsed: Extract<ServerMessage, { kind: "tag.update" }>) {
    const update = {
      value: parsed.value,
      quality: parsed.quality,
      ts: parsed.ts,
    };
    this.lastTagUpdates.set(parsed.path, update);

    const callbacks = this.callbacks.get(parsed.path);
    if (!callbacks) {
      return;
    }

    for (const callback of callbacks) {
      callback(update);
    }
  }

  private dispatchViewDefinition(definition: ViewDefinition) {
    const callbacks = this.viewCallbacks.get(
      viewKey(definition.project_id, definition.view_id),
    );
    if (!callbacks) {
      return;
    }

    for (const callback of callbacks) {
      callback(definition);
    }
  }

  private dispatchProjectChange(change: ProjectChange) {
    const callbacks = this.projectCallbacks.get(change.project_id);
    if (!callbacks) {
      return;
    }

    for (const callback of callbacks) {
      callback(change);
    }
  }

  private dispatchAlarmEvent(event: AlarmEvent) {
    for (const subscription of this.alarmCallbacks.values()) {
      if (!alarmMatches(event, subscription.options)) {
        continue;
      }
      for (const callback of subscription.callbacks) {
        callback(event);
      }
    }
  }

  private dispatchHistoryResult(result: Extract<ServerMessage, { kind: "history.result" }>) {
    const requestId = result.request_id ?? "";
    const pending = this.pendingHistory.get(requestId);
    if (!pending) {
      return;
    }
    this.pendingHistory.delete(requestId);
    pending.resolve(result.points);
  }

  private rejectOldestHistory(error: GatewayError) {
    const [requestId, pending] = this.pendingHistory.entries().next().value ?? [];
    if (!requestId || !pending) {
      return;
    }
    this.pendingHistory.delete(requestId);
    pending.reject(new Error(`${error.code}: ${error.message}`));
  }

  private resubscribeAll() {
    const paths = [...this.callbacks.keys()];
    if (paths.length > 0) {
      this.send({ kind: "tag.subscribe", paths });
    }

    for (const projectId of this.projectCallbacks.keys()) {
      this.send({ kind: "project.subscribe", project_id: projectId });
    }

    for (const key of this.viewCallbacks.keys()) {
      const [projectId, viewId] = splitViewKey(key);
      this.requestView(projectId, viewId);
    }

    for (const subscription of this.alarmCallbacks.values()) {
      this.send(alarmSubscribeMessage(subscription.options));
    }
  }

  private scheduleReconnect() {
    if (this.manuallyClosed || this.reconnectTimer !== null) {
      return;
    }

    this.markBindingsBad();
    this.setState("reconnecting");
    const delay = withJitter(this.reconnectDelayMs);
    this.reconnectDelayMs = Math.min(this.reconnectDelayMs * 2, 8_000);
    this.reconnectTimer = setTimeout(() => {
      this.reconnectTimer = null;
      if (!this.manuallyClosed) {
        this.connect();
      }
    }, delay);
  }

  private markBindingsBad() {
    for (const [path, update] of this.lastTagUpdates) {
      if (update.quality === "bad") {
        continue;
      }

      const staleUpdate = { ...update, quality: "bad" as const };
      this.lastTagUpdates.set(path, staleUpdate);
      const callbacks = this.callbacks.get(path);
      if (!callbacks) {
        continue;
      }
      for (const callback of callbacks) {
        callback(staleUpdate);
      }
    }
  }

  private clearReconnectTimer() {
    if (this.reconnectTimer !== null) {
      clearTimeout(this.reconnectTimer);
      this.reconnectTimer = null;
    }
  }

  private setState(state: ConnectionState) {
    this.currentState = state;
    for (const callback of this.stateCallbacks) {
      callback(state);
    }
  }
}

type NormalizedAlarmSubscribeOptions = {
  projectId: string;
  priorityMin: number;
  priorityMax: number;
};

type AlarmSubscription = {
  options: NormalizedAlarmSubscribeOptions;
  callbacks: Set<AlarmCallback>;
};

function viewKey(projectId: string, viewId: string): string {
  return JSON.stringify([projectId, viewId]);
}

function splitViewKey(key: string): [string, string] {
  const parsed = JSON.parse(key) as [string, string];
  return parsed;
}

function withJitter(delayMs: number): number {
  const jitter = delayMs * 0.25 * (Math.random() * 2 - 1);
  return Math.max(0, Math.round(Math.min(delayMs + jitter, MAX_BACKOFF_MS)));
}

function normalizedAlarmOptions(
  options: AlarmSubscribeOptions,
): NormalizedAlarmSubscribeOptions {
  return {
    projectId: options.projectId,
    priorityMin: options.priorityMin ?? 1,
    priorityMax: options.priorityMax ?? 5,
  };
}

function alarmKey(options: AlarmSubscribeOptions): string {
  const normalized = normalizedAlarmOptions(options);
  return JSON.stringify([
    normalized.projectId,
    normalized.priorityMin,
    normalized.priorityMax,
  ]);
}

function alarmSubscribeMessage(options: NormalizedAlarmSubscribeOptions): ClientMessage {
  return {
    kind: "alarm.subscribe",
    project_id: options.projectId,
    priority_min: options.priorityMin,
    priority_max: options.priorityMax,
  };
}

function alarmMatches(
  event: AlarmEvent,
  options: NormalizedAlarmSubscribeOptions,
): boolean {
  return (
    event.priority >= options.priorityMin &&
    event.priority <= options.priorityMax
  );
}

function withToken(url: string, token: string | null): string {
  if (!token) {
    return url;
  }
  const parsed = new URL(url);
  parsed.searchParams.set("token", token);
  return parsed.toString();
}
