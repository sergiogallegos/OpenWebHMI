import { describe, expect, it } from "vitest";
import {
  isClientMessage,
  isQuality,
  isServerMessage,
  isTagValue,
  isView,
  type ClientMessage,
  type ServerMessage,
  type View,
} from "./index";

describe("ClientMessage wire form", () => {
  it("serializes tag.subscribe exactly like Rust serde", () => {
    const message: ClientMessage = {
      kind: "tag.subscribe",
      paths: ["system/sim/sin"],
    };

    expect(JSON.stringify(message)).toBe(
      '{"kind":"tag.subscribe","paths":["system/sim/sin"]}',
    );
    expect(isClientMessage(JSON.parse(JSON.stringify(message)))).toBe(true);
  });

  it("serializes tag.unsubscribe exactly like Rust serde", () => {
    const message: ClientMessage = {
      kind: "tag.unsubscribe",
      paths: ["a/b", "c/d"],
    };

    expect(JSON.stringify(message)).toBe(
      '{"kind":"tag.unsubscribe","paths":["a/b","c/d"]}',
    );
    expect(isClientMessage(JSON.parse(JSON.stringify(message)))).toBe(true);
  });

  it("serializes tag.write exactly like Rust serde", () => {
    const message: ClientMessage = {
      kind: "tag.write",
      path: "rockwell-1/Setpoint",
      value: { type: "real", value: 42.5 },
    };

    expect(JSON.stringify(message)).toBe(
      '{"kind":"tag.write","path":"rockwell-1/Setpoint","value":{"type":"real","value":42.5}}',
    );
    expect(isClientMessage(JSON.parse(JSON.stringify(message)))).toBe(true);
  });

  it("serializes auth.login exactly like Rust serde", () => {
    const message: ClientMessage = {
      kind: "auth.login",
      username: "admin",
      password: "secret",
    };

    expect(JSON.stringify(message)).toBe(
      '{"kind":"auth.login","username":"admin","password":"secret"}',
    );
    expect(isClientMessage(JSON.parse(JSON.stringify(message)))).toBe(true);
  });

  it("serializes alarm.ack exactly like Rust serde", () => {
    const message: ClientMessage = {
      kind: "alarm.ack",
      alarm_id: "pressure-high",
      note: "checked",
    };

    expect(JSON.stringify(message)).toBe(
      '{"kind":"alarm.ack","alarm_id":"pressure-high","note":"checked"}',
    );
    expect(isClientMessage(JSON.parse(JSON.stringify(message)))).toBe(true);
  });

  it("serializes ping exactly like Rust serde", () => {
    const message: ClientMessage = { kind: "ping" };

    expect(JSON.stringify(message)).toBe('{"kind":"ping"}');
    expect(isClientMessage(JSON.parse(JSON.stringify(message)))).toBe(true);
  });

  it("serializes project.save_artifact exactly like Rust serde", () => {
    const message: ClientMessage = {
      kind: "project.save_artifact",
      request_id: "r1",
      project_id: "demo",
      artifact: { kind: "view", id: "home" },
      body: { id: "home" },
    };

    expect(JSON.stringify(message)).toBe(
      '{"kind":"project.save_artifact","request_id":"r1","project_id":"demo","artifact":{"kind":"view","id":"home"},"body":{"id":"home"}}',
    );
    expect(isClientMessage(JSON.parse(JSON.stringify(message)))).toBe(true);
  });

  it("serializes view.open exactly like Rust serde", () => {
    const message: ClientMessage = {
      kind: "view.open",
      project_id: "demo",
      view_id: "home",
    };

    expect(JSON.stringify(message)).toBe(
      '{"kind":"view.open","project_id":"demo","view_id":"home"}',
    );
    expect(isClientMessage(JSON.parse(JSON.stringify(message)))).toBe(true);
  });

  it("serializes history.read exactly like Rust serde", () => {
    const message: ClientMessage = {
      kind: "history.read",
      request_id: "r1",
      tag_path: "rockwell-1/Pressure",
      t_start_ms: 10,
      t_end_ms: 20,
      aggregation: "avg",
      max_points: 100,
    };

    expect(JSON.stringify(message)).toBe(
      '{"kind":"history.read","request_id":"r1","tag_path":"rockwell-1/Pressure","t_start_ms":10,"t_end_ms":20,"aggregation":"avg","max_points":100}',
    );
    expect(isClientMessage(JSON.parse(JSON.stringify(message)))).toBe(true);
  });

  it("serializes audit.query exactly like Rust serde", () => {
    const message: ClientMessage = {
      kind: "audit.query",
      request_id: "audit-r1",
      query: {
        from_ts_ms: 10,
        to_ts_ms: null,
        user: "admin",
        kinds: ["AuthLogin"],
        limit: 50,
        offset: 0,
      },
    };

    expect(JSON.stringify(message)).toBe(
      '{"kind":"audit.query","request_id":"audit-r1","query":{"from_ts_ms":10,"to_ts_ms":null,"user":"admin","kinds":["AuthLogin"],"limit":50,"offset":0}}',
    );
    expect(isClientMessage(JSON.parse(JSON.stringify(message)))).toBe(true);
  });

  it("serializes project export/import exactly like Rust serde", () => {
    const exportMessage: ClientMessage = {
      kind: "project.export",
      request_id: "backup-r1",
      project_id: "demo",
      include_historian: true,
      include_alarm_journal: false,
    };
    expect(JSON.stringify(exportMessage)).toBe(
      '{"kind":"project.export","request_id":"backup-r1","project_id":"demo","include_historian":true,"include_alarm_journal":false}',
    );
    expect(isClientMessage(JSON.parse(JSON.stringify(exportMessage)))).toBe(true);

    const importMessage: ClientMessage = {
      kind: "project.import",
      request_id: "restore-r1",
      project_id: "demo",
      mode: "replace",
    };
    expect(JSON.stringify(importMessage)).toBe(
      '{"kind":"project.import","request_id":"restore-r1","project_id":"demo","mode":"replace"}',
    );
    expect(isClientMessage(JSON.parse(JSON.stringify(importMessage)))).toBe(true);
  });

  it("serializes tag values and quality exactly like Rust serde", () => {
    expect(JSON.stringify({ type: "real", value: 2.5 })).toBe(
      '{"type":"real","value":2.5}',
    );
    expect(JSON.stringify("good")).toBe('"good"');
  });
});

describe("ServerMessage parsing", () => {
  it("parses tag.update into a typed value", () => {
    const parsed = JSON.parse(
      '{"kind":"tag.update","path":"system/sim/sin","value":{"type":"real","value":0.5},"quality":"good","ts":1714128000000}',
    );

    expect(isServerMessage(parsed)).toBe(true);
    const message = parsed as ServerMessage;
    expect(message.kind).toBe("tag.update");
    if (message.kind === "tag.update") {
      expect(message.value).toEqual({ type: "real", value: 0.5 });
    }
  });

  it("parses pong into a typed value", () => {
    const parsed = JSON.parse('{"kind":"pong"}');

    expect(isServerMessage(parsed)).toBe(true);
  });

  it("parses error into a typed value", () => {
    const parsed = JSON.parse(
      '{"kind":"error","code":"protocol.parse","message":"invalid JSON"}',
    );

    expect(isServerMessage(parsed)).toBe(true);
    const message = parsed as ServerMessage;
    expect(message).toEqual({
      kind: "error",
      code: "protocol.parse",
      message: "invalid JSON",
    });
  });

  it("parses view.definition into a typed value", () => {
    const view = sampleView();
    const parsed = {
      kind: "view.definition",
      project_id: "demo",
      view_id: "home",
      version: 7,
      view,
    };

    expect(isServerMessage(parsed)).toBe(true);
    expect(isView(view)).toBe(true);
  });

  it("parses history.result into a typed value", () => {
    const parsed = {
      kind: "history.result",
      request_id: "r1",
      tag_path: "rockwell-1/Pressure",
      points: [{ ts_ms: 10, value: { type: "real", value: 2.5 }, quality: "good" }],
    };

    expect(isServerMessage(parsed)).toBe(true);
  });

  it("parses auth.result and user.list into typed values", () => {
    expect(
      isServerMessage({
        kind: "auth.result",
        session_token: "jwt",
        user_id: "u1",
        roles: ["Administrator"],
        error: null,
      }),
    ).toBe(true);
    expect(
      isServerMessage({
        kind: "user.list",
        users: [{ id: "u1", username: "admin", roles: ["Administrator"] }],
      }),
    ).toBe(true);
  });

  it("parses alarm.event into a typed value", () => {
    expect(
      isServerMessage({
        kind: "alarm.event",
        alarm_id: "pressure-high",
        label: "High pressure",
        priority: 2,
        state: "active",
        tag_path: "rockwell-1/Pressure",
        value: { type: "real", value: 250 },
        quality: "good",
        activated_at_ms: 10,
        transitioned_at_ms: 10,
        message: "Pressure high: 250",
      }),
    ).toBe(true);
  });

  it("parses audit events and query results into typed values", () => {
    const entry = {
      id: 1,
      ts_ms: 10,
      user: "admin",
      session_id: null,
      source_ip: "127.0.0.1:8080",
      kind: "AuthLogin",
      payload: { type: "AuthLogin", username: "admin", success: true, reason: null },
    };

    expect(isServerMessage({ kind: "audit.event", entry })).toBe(true);
    expect(
      isServerMessage({
        kind: "audit.query_result",
        request_id: "audit-r1",
        entries: [entry],
        total: 1,
      }),
    ).toBe(true);
  });

  it("parses project backup server messages into typed values", () => {
    expect(
      isServerMessage({
        kind: "project.export_ready",
        request_id: "backup-r1",
        download_url: "/api/projects/demo/backup/token",
        size_bytes: 42,
      }),
    ).toBe(true);
    expect(
      isServerMessage({
        kind: "project.import_progress",
        request_id: "restore-r1",
        phase: "importing",
        percent: 50,
      }),
    ).toBe(true);
    expect(
      isServerMessage({
        kind: "project.import_result",
        request_id: "restore-r1",
        ok: true,
        error: null,
      }),
    ).toBe(true);
  });

  it("rejects malformed view definitions defensively", () => {
    expect(isView({ ...sampleView(), id: undefined })).toBe(false);
    expect(
      isView({
        ...sampleView(),
        root: { ...sampleView().root, kind: undefined },
      }),
    ).toBe(false);
  });

  it("rejects unknown message kinds defensively", () => {
    expect(isClientMessage({ kind: "project.delete" })).toBe(false);
    expect(isServerMessage({ kind: "project.deleted" })).toBe(false);
  });

  it("rejects malformed values defensively", () => {
    for (const value of [
      null,
      undefined,
      {},
      [],
      "tag.update",
      42,
      { kind: "unknown" },
      { kind: "tag.update" },
    ]) {
      expect(isClientMessage(value)).toBe(false);
      expect(isServerMessage(value)).toBe(false);
    }

    expect(isTagValue({ type: "real", value: "2.5" })).toBe(false);
    expect(isTagValue({ type: "bool", value: 1 })).toBe(false);
    expect(isQuality("excellent")).toBe(false);
  });
});

function sampleView(): View {
  return {
    id: "home",
    title: "Home",
    schema_version: 1,
    root: {
      id: "root",
      kind: "Container",
      props: {},
      bindings: [],
      children: [
        {
          id: "nested",
          kind: "Container",
          props: {},
          bindings: [],
          children: [
            {
              id: "pressure",
              kind: "ValueDisplay",
              props: { format: "number" },
              bindings: [
                {
                  prop: "value",
                  source: { kind: "tag", path: "rockwell-1/Pressure" },
                },
              ],
              children: [],
            },
          ],
        },
      ],
    },
  };
}
