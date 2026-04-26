import {
  isServerMessage,
  type ClientMessage,
  type Quality,
  type TagValue,
} from "@openwebhmi/protocol";

export type TagUpdate = {
  value: TagValue;
  quality: Quality;
  ts: number;
};

export type ConnectionState =
  | "idle"
  | "connecting"
  | "connected"
  | "reconnecting"
  | "closed";

type TagCallback = (update: TagUpdate) => void;
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
  private readonly stateCallbacks = new Set<StateCallback>();
  private currentState: ConnectionState = "idle";

  constructor(
    private readonly opts: { url: string; webSocketImpl?: WebSocketCtor },
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
    const ws = new WebSocketImpl(this.opts.url);
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

  subscribe(path: string, callback: TagCallback): () => void {
    const callbacks = this.callbacks.get(path) ?? new Set<TagCallback>();
    const wasEmpty = callbacks.size === 0;
    callbacks.add(callback);
    this.callbacks.set(path, callbacks);

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
      this.send({ kind: "tag.unsubscribe", paths: [path] });
    }
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

  private send(message: ClientMessage) {
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

    if (!isServerMessage(parsed) || parsed.kind !== "tag.update") {
      return;
    }

    const callbacks = this.callbacks.get(parsed.path);
    if (!callbacks) {
      return;
    }

    for (const callback of callbacks) {
      callback({
        value: parsed.value,
        quality: parsed.quality,
        ts: parsed.ts,
      });
    }
  }

  private resubscribeAll() {
    const paths = [...this.callbacks.keys()];
    if (paths.length > 0) {
      this.send({ kind: "tag.subscribe", paths });
    }
  }

  private scheduleReconnect() {
    if (this.manuallyClosed || this.reconnectTimer !== null) {
      return;
    }

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

function withJitter(delayMs: number): number {
  const jitter = delayMs * 0.25 * (Math.random() * 2 - 1);
  return Math.max(0, Math.round(Math.min(delayMs + jitter, MAX_BACKOFF_MS)));
}
