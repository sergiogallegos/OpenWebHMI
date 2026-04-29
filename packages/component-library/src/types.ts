import type React from "react";
import type { HistoryPoint, Quality, ServerMessage, TagValue } from "@openwebhmi/protocol";

export type PropSchemaField = {
  type: "string" | "number" | "boolean" | "color" | "select" | "stringList";
  label?: string;
  options?: string[];
  default?: unknown;
};

export type PropSchema<Props> = {
  [Key in keyof Props & string]: PropSchemaField;
};

export type BindableProp<Props = any> = keyof Props & string;

export type BoundValue = {
  value: TagValue;
  quality: Quality;
  ts: number;
};

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
  aggregation: string;
  maxPoints: number;
};

export type RuntimeContext = {
  mode: "runtime";
  onWriteTag: (path: string, value: TagValue) => void;
  liveValues?: Record<string, BoundValue | undefined>;
  onReadHistory?: (options: HistoryReadOptions) => Promise<HistoryPoint[]>;
  projectId?: string;
  onSubscribeAlarms?: (
    options: AlarmSubscribeOptions,
    callback: (event: AlarmEvent) => void,
  ) => () => void;
  onAckAlarm?: (alarmId: string, note?: string | null) => void;
};

export type DesignerContext = {
  mode: "designer";
  isSelected: boolean;
};

export type RenderProps<Props> = {
  props: Props;
  bindings: Record<string, BoundValue | undefined>;
  context: RuntimeContext | DesignerContext;
  children?: React.ReactNode;
};

export type ComponentDefinition<Props = any> = {
  kind: string;
  defaultProps: Props;
  propsSchema: PropSchema<Props>;
  bindableProps: BindableProp<Props>[];
  Render: React.FC<RenderProps<Props>>;
  ThumbnailIcon?: React.FC;
};
