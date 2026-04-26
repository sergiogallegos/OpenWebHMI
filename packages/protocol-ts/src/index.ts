/** A tag's current value using the Rust serde adjacent-tag wire form. */
export type TagValue =
  | { type: "bool"; value: boolean }
  | { type: "int"; value: number }
  | { type: "real"; value: number }
  | { type: "string"; value: string };

/** OPC UA-style quality carried with each tag update. */
export type Quality = "good" | "bad" | "uncertain" | "stale";

/** Messages accepted by the gateway from a runtime or designer client. */
export type ClientMessage =
  | { kind: "tag.subscribe"; paths: string[] }
  | { kind: "tag.unsubscribe"; paths: string[] }
  | { kind: "ping" };

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
  | { kind: "error"; code: string; message: string };

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
    default:
      return false;
  }
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null && !Array.isArray(value);
}

function isStringArray(value: unknown): value is string[] {
  return Array.isArray(value) && value.every((item) => typeof item === "string");
}
