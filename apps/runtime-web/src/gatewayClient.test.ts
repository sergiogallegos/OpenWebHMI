import { describe, expect, it, vi } from "vitest";
import { GatewayClient } from "./gatewayClient";

class MockWebSocket {
  static instances: MockWebSocket[] = [];

  readonly url: string;
  readyState = 0;
  onopen: ((event: Event) => void) | null = null;
  onmessage: ((event: MessageEvent<string>) => void) | null = null;
  onclose: ((event: CloseEvent) => void) | null = null;
  onerror: ((event: Event) => void) | null = null;
  sent: string[] = [];

  constructor(url: string) {
    this.url = url;
    MockWebSocket.instances.push(this);
  }

  send(data: string) {
    this.sent.push(data);
  }

  close() {
    this.readyState = 3;
    this.onclose?.(new Event("close") as CloseEvent);
  }

  open() {
    this.readyState = 1;
    this.onopen?.(new Event("open"));
  }
}

describe("GatewayClient", () => {
  it("connects, multiplexes subscriptions, dispatches updates, and unsubscribes", () => {
    MockWebSocket.instances = [];

    const states: string[] = [];
    const updates: unknown[] = [];
    const client = new GatewayClient({
      url: "ws://127.0.0.1:8080",
      webSocketImpl: MockWebSocket,
    });
    client.onStateChange((state) => states.push(state));

    client.connect();
    expect(states).toEqual(["connecting"]);

    const first = MockWebSocket.instances[0]!;
    first.open();
    expect(states).toEqual(["connecting", "connected"]);

    const unsubscribeOne = client.subscribe("system/sim/sin", (update) =>
      updates.push(update),
    );
    const unsubscribeTwo = client.subscribe("system/sim/sin", (update) =>
      updates.push(update),
    );
    expect(first.sent).toEqual([
      '{"kind":"tag.subscribe","paths":["system/sim/sin"]}',
    ]);

    first.onmessage?.({
      data: '{"kind":"tag.update","path":"system/sim/sin","value":{"type":"real","value":0.5},"quality":"good","ts":1}',
    } as MessageEvent<string>);
    expect(updates).toHaveLength(2);

    unsubscribeOne();
    expect(first.sent).toHaveLength(1);
    unsubscribeTwo();
    expect(first.sent.at(-1)).toBe(
      '{"kind":"tag.unsubscribe","paths":["system/sim/sin"]}',
    );

    client.disconnect();
  });

  it("reconnects with backoff and resubscribes after close", async () => {
    vi.useFakeTimers();
    vi.spyOn(Math, "random").mockReturnValue(0.5);
    MockWebSocket.instances = [];

    const states: string[] = [];
    const client = new GatewayClient({
      url: "ws://127.0.0.1:8080",
      webSocketImpl: MockWebSocket,
    });
    client.onStateChange((state) => states.push(state));

    client.connect();
    client.subscribe("system/sim/sin", () => undefined);

    const first = MockWebSocket.instances[0]!;
    first.open();
    expect(first.sent).toContain(
      '{"kind":"tag.subscribe","paths":["system/sim/sin"]}',
    );

    first.close();
    expect(states).toContain("reconnecting");

    await vi.advanceTimersByTimeAsync(250);

    const second = MockWebSocket.instances[1]!;
    second.open();
    expect(second.url).toBe("ws://127.0.0.1:8080");
    expect(second.sent).toContain(
      '{"kind":"tag.subscribe","paths":["system/sim/sin"]}',
    );

    client.disconnect();
    vi.mocked(Math.random).mockRestore();
    vi.useRealTimers();
  });

  it("doubles reconnect backoff up to the cap and resets on open", async () => {
    vi.useFakeTimers();
    vi.spyOn(Math, "random").mockReturnValue(0.5);
    MockWebSocket.instances = [];

    const client = new GatewayClient({
      url: "ws://127.0.0.1:8080",
      webSocketImpl: MockWebSocket,
    });

    client.connect();
    MockWebSocket.instances[0]!.close();
    await vi.advanceTimersByTimeAsync(250);

    MockWebSocket.instances[1]!.close();
    await vi.advanceTimersByTimeAsync(500);

    MockWebSocket.instances[2]!.open();
    MockWebSocket.instances[2]!.close();
    await vi.advanceTimersByTimeAsync(250);

    expect(MockWebSocket.instances).toHaveLength(4);

    client.disconnect();
    vi.mocked(Math.random).mockRestore();
    vi.useRealTimers();
  });

  it("routes view definitions to openView callbacks", () => {
    MockWebSocket.instances = [];

    const definitions: unknown[] = [];
    const client = new GatewayClient({
      url: "ws://127.0.0.1:8080",
      webSocketImpl: MockWebSocket,
    });

    client.connect();
    const socket = MockWebSocket.instances[0]!;
    socket.open();

    client.openView("phase1-demo", "home", (definition) =>
      definitions.push(definition),
    );

    expect(socket.sent).toContain(
      '{"kind":"view.open","project_id":"phase1-demo","view_id":"home"}',
    );

    socket.onmessage?.({
      data: JSON.stringify({
        kind: "view.definition",
        project_id: "phase1-demo",
        view_id: "home",
        version: 2,
        view: {
          id: "home",
          title: "Home",
          schema_version: 1,
          root: {
            id: "root",
            kind: "Container",
            props: {},
            bindings: [],
            children: [],
          },
        },
      }),
    } as MessageEvent<string>);

    expect(definitions).toHaveLength(1);
    expect(definitions[0]).toMatchObject({
      project_id: "phase1-demo",
      view_id: "home",
      version: 2,
    });

    client.disconnect();
  });

  it("stops routing view definitions after openView unsubscriber", () => {
    MockWebSocket.instances = [];

    const definitions: unknown[] = [];
    const client = new GatewayClient({
      url: "ws://127.0.0.1:8080",
      webSocketImpl: MockWebSocket,
    });

    client.connect();
    const socket = MockWebSocket.instances[0]!;
    socket.open();

    const unsubscribe = client.openView("phase1-demo", "home", (definition) =>
      definitions.push(definition),
    );
    unsubscribe();

    expect(socket.sent.at(-1)).toBe(
      '{"kind":"view.close","project_id":"phase1-demo","view_id":"home"}',
    );

    socket.onmessage?.({
      data: JSON.stringify({
        kind: "view.definition",
        project_id: "phase1-demo",
        view_id: "home",
        version: 2,
        view: {
          id: "home",
          title: "Home",
          schema_version: 1,
          root: {
            id: "root",
            kind: "Container",
            props: {},
            bindings: [],
            children: [],
          },
        },
      }),
    } as MessageEvent<string>);

    expect(definitions).toEqual([]);

    client.disconnect();
  });

  it("sends tag.write frames", () => {
    MockWebSocket.instances = [];

    const client = new GatewayClient({
      url: "ws://127.0.0.1:8080",
      webSocketImpl: MockWebSocket,
    });

    client.connect();
    const socket = MockWebSocket.instances[0]!;
    socket.open();

    client.writeTag("rockwell-1/Setpoint", { type: "real", value: 42.5 });

    expect(socket.sent.at(-1)).toBe(
      '{"kind":"tag.write","path":"rockwell-1/Setpoint","value":{"type":"real","value":42.5}}',
    );

    client.disconnect();
  });
});
