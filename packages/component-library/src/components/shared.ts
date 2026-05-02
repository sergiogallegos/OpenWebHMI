import type { CSSProperties } from "react";
import type { BoundValue } from "../types";
import type { TagValue } from "@openwebhmi/protocol";

export const baseFont =
  'Inter, ui-sans-serif, system-ui, -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif';

export function isBadQuality(bound: BoundValue | undefined): boolean {
  return bound !== undefined && bound.quality !== "good";
}

export function badQualityStyle(bound: BoundValue | undefined): CSSProperties {
  if (!isBadQuality(bound)) {
    return {};
  }

  return {
    border: "1px solid #d64545",
    color: "#7b8794",
  };
}

export function designerStyle(isSelected: boolean | undefined): CSSProperties {
  if (!isSelected) {
    return {};
  }

  return {
    outline: "2px solid #2563eb",
    outlineOffset: 2,
  };
}

export function tagValueToText(value: TagValue | undefined): string {
  if (value === undefined) {
    return "";
  }

  return String(value.value);
}

export function tagValueToNumber(value: TagValue | undefined): number | null {
  if (value === undefined) {
    return null;
  }

  if (value.type === "int" || value.type === "real") {
    return value.value;
  }

  const parsed = Number(value.value);
  return Number.isFinite(parsed) ? parsed : null;
}

export function tagValueFromNumber(value: number): TagValue {
  return Number.isInteger(value)
    ? { type: "int", value }
    : { type: "real", value };
}

export function tagValueToPrimitive(
  value: TagValue | undefined,
): string | number | boolean | undefined {
  return value?.value;
}

export function primitiveToTagValue(value: string | number | boolean): TagValue {
  if (typeof value === "boolean") {
    return { type: "bool", value };
  }
  if (typeof value === "number") {
    return tagValueFromNumber(value);
  }
  return { type: "string", value };
}

export function clampNumber(value: number, min: number, max: number): number {
  if (!Number.isFinite(value)) {
    return min;
  }
  return Math.min(max, Math.max(min, value));
}
