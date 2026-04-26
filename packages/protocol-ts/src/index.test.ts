import { describe, expect, it } from "vitest";
import {
  isClientMessage,
  isQuality,
  isServerMessage,
  isTagValue,
  type ClientMessage,
  type ServerMessage,
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

  it("serializes ping exactly like Rust serde", () => {
    const message: ClientMessage = { kind: "ping" };

    expect(JSON.stringify(message)).toBe('{"kind":"ping"}');
    expect(isClientMessage(JSON.parse(JSON.stringify(message)))).toBe(true);
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
