import type React from "react";
import type { Quality, TagValue } from "@openwebhmi/protocol";

export type PropSchemaField = {
  type: "string" | "number" | "boolean" | "color" | "select";
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

export type RuntimeContext = {
  mode: "runtime";
  onWriteTag: (path: string, value: TagValue) => void;
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
