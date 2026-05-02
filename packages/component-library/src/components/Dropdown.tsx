import type { ComponentDefinition } from "../types";
import {
  badQualityStyle,
  baseFont,
  designerStyle,
  primitiveToTagValue,
  tagValueToPrimitive,
} from "./shared";

export type DropdownOption = {
  value: string | number | boolean;
  label: string;
};

export type DropdownProps = {
  value?: string | number | boolean;
  tagPath?: string;
  options: DropdownOption[];
  placeholder: string;
};

/** Operator selection input that writes the selected primitive value. */
export const Dropdown: ComponentDefinition<DropdownProps> = {
  kind: "Dropdown",
  defaultProps: {
    value: "",
    tagPath: "",
    options: [
      { value: "auto", label: "Auto" },
      { value: "manual", label: "Manual" },
    ],
    placeholder: "Select...",
  },
  propsSchema: {
    value: { type: "string", label: "Value", default: "" },
    tagPath: { type: "string", label: "Write tag path", default: "" },
    options: {
      type: "objectList",
      label: "Options",
      default: [
        { value: "auto", label: "Auto" },
        { value: "manual", label: "Manual" },
      ],
    },
    placeholder: { type: "string", label: "Placeholder", default: "Select..." },
  },
  bindableProps: ["value"],
  Render({ props, bindings, context }) {
    const bound = bindings.value;
    const options = normalizeOptions(props.options);
    const current = tagValueToPrimitive(bound?.value) ?? props.value ?? "";
    const selected = options.find((option) => option.value === current);
    const disabled = bound !== undefined && bound.quality !== "good";

    return (
      <select
        aria-label="Dropdown"
        disabled={disabled}
        value={selected ? encodeValue(selected.value) : ""}
        onChange={(event) => {
          if (context.mode !== "runtime" || disabled) {
            return;
          }
          const option = options.find((item) => encodeValue(item.value) === event.currentTarget.value);
          if (option) {
            context.onWriteTag(props.tagPath || "value", primitiveToTagValue(option.value));
          }
        }}
        style={{
          ...styles.select,
          ...badQualityStyle(bound),
          ...designerStyle(context.mode === "designer" ? context.isSelected : false),
        }}
      >
        <option value="" disabled>
          {props.placeholder}
        </option>
        {options.map((option) => (
          <option key={encodeValue(option.value)} value={encodeValue(option.value)}>
            {option.label}
          </option>
        ))}
      </select>
    );
  },
};

function normalizeOptions(options: DropdownOption[] | undefined): DropdownOption[] {
  return Array.isArray(options)
    ? options.filter((option) => "value" in option && typeof option.label === "string")
    : [];
}

function encodeValue(value: string | number | boolean): string {
  return `${typeof value}:${String(value)}`;
}

const styles = {
  select: {
    minWidth: 160,
    minHeight: 34,
    padding: "6px 8px",
    border: "1px solid #9aa5b1",
    borderRadius: 6,
    background: "#ffffff",
    color: "#1f2933",
    fontFamily: baseFont,
  },
};
