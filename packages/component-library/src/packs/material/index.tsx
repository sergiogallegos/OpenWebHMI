import { useEffect, useRef, useState } from "react";
import type { ComponentDefinition } from "../../types";
import type { ButtonProps } from "../../components/Button";
import { Button } from "../../components/Button";
import type { CardProps } from "../../components/Card";
import { Card } from "../../components/Card";
import type { DropdownProps } from "../../components/Dropdown";
import { Dropdown } from "../../components/Dropdown";
import type { GaugeProps } from "../../components/Gauge";
import { Gauge } from "../../components/Gauge";
import type { ModalProps } from "../../components/Modal";
import { Modal } from "../../components/Modal";
import type { NumericInputProps } from "../../components/NumericInput";
import { NumericInput } from "../../components/NumericInput";
import type { SliderProps } from "../../components/Slider";
import { Slider } from "../../components/Slider";
import type { ToggleSwitchProps } from "../../components/ToggleSwitch";
import { ToggleSwitch } from "../../components/ToggleSwitch";
import {
  badQualityStyle,
  baseFont,
  clampNumber,
  designerStyle,
  primitiveToTagValue,
  tagValueFromNumber,
  tagValueToNumber,
  tagValueToPrimitive,
} from "../../components/shared";

const materialButton: ComponentDefinition<ButtonProps> = {
  ...Button,
  Render({ props, bindings, context }) {
    const bound = bindings.target;
    const boundTarget = tagValueToPrimitive(bound?.value);
    const target = typeof boundTarget === "string" ? boundTarget : props.target ?? "";
    const disabled = !target || (bound !== undefined && bound.quality !== "good") || context.mode === "designer";
    const writeValue = Array.isArray(props.writeValue)
      ? props.writeValue[0] ?? { type: "bool", value: true }
      : props.writeValue;
    return (
      <button
        className="mdc-button"
        aria-label="Button"
        type="button"
        disabled={disabled}
        onClick={() => {
          if (context.mode === "runtime" && !disabled) {
            if (props.confirmPrompt && !window.confirm(props.confirmPrompt)) {
              return;
            }
            context.onWriteTag(target, writeValue);
          }
        }}
        style={{
          ...mdStyles.button,
          ...mdButtonStyle(props.style),
          opacity: disabled ? 0.55 : 1,
          ...badQualityStyle(bound),
          ...designerStyle(context.mode === "designer" ? context.isSelected : false),
        }}
      >
        <span style={mdStyles.ripple} />
        {props.label}
      </button>
    );
  },
};

const materialToggleSwitch: ComponentDefinition<ToggleSwitchProps> = {
  ...ToggleSwitch,
  Render({ props, bindings, context }) {
    const bound = bindings.value;
    const value = bound?.value.type === "bool" ? bound.value.value : props.value ?? false;
    const disabled = bound !== undefined && bound.quality !== "good";
    return (
      <button
        className="mdc-switch"
        aria-label="Toggle switch"
        type="button"
        disabled={disabled}
        onClick={() => {
          if (context.mode === "runtime" && !disabled) {
            context.onWriteTag(props.tagPath || "value", { type: "bool", value: !value });
          }
        }}
        style={{
          ...mdStyles.switch,
          background: value ? "var(--primary-color, #6750a4)" : "#e7e0ec",
          opacity: disabled ? 0.55 : 1,
          ...badQualityStyle(bound),
          ...designerStyle(context.mode === "designer" ? context.isSelected : false),
        }}
      >
        <span
          style={{
            ...mdStyles.switchHandle,
            transform: value ? "translateX(28px)" : "translateX(4px)",
            background: value ? "#ffffff" : "#79747e",
          }}
        />
        <span style={mdStyles.switchLabel}>{value ? props.onLabel : props.offLabel}</span>
      </button>
    );
  },
};

const materialNumericInput: ComponentDefinition<NumericInputProps> = {
  ...NumericInput,
  Render({ props, bindings, context }) {
    const bound = bindings.value;
    const numericValue = tagValueToNumber(bound?.value) ?? props.value ?? 0;
    const [draft, setDraft] = useState(String(numericValue));
    useEffect(() => setDraft(String(numericValue)), [numericValue]);
    const commit = () => {
      if (context.mode !== "runtime" || props.disabled) {
        return;
      }
      const parsed = Number(draft);
      if (!Number.isFinite(parsed)) {
        setDraft(String(numericValue));
        return;
      }
      const value = clampNumber(parsed, props.min ?? Number.NEGATIVE_INFINITY, props.max ?? Number.POSITIVE_INFINITY);
      context.onWriteTag(props.tagPath || "value", tagValueFromNumber(value));
      setDraft(String(value));
    };
    return (
      <label className="mdc-text-field" style={mdStyles.field}>
        <span style={mdStyles.floatingLabel}>Value</span>
        <input
          aria-label="Numeric input"
          disabled={props.disabled}
          min={props.min}
          max={props.max}
          step={props.step}
          type="number"
          value={draft}
          onBlur={commit}
          onChange={(event) => setDraft(event.currentTarget.value)}
          onKeyDown={(event) => {
            if (event.key === "Enter") {
              commit();
            }
          }}
          style={{
            ...mdStyles.input,
            ...badQualityStyle(bound),
            ...designerStyle(context.mode === "designer" ? context.isSelected : false),
          }}
        />
      </label>
    );
  },
};

const materialSlider: ComponentDefinition<SliderProps> = {
  ...Slider,
  Render({ props, bindings, context }) {
    const bound = bindings.value;
    const max = props.max > props.min ? props.max : props.min + 1;
    const sourceValue = clampNumber(tagValueToNumber(bound?.value) ?? props.value ?? props.min, props.min, max);
    const [draft, setDraft] = useState(sourceValue);
    const lastLiveWrite = useRef(0);
    const disabled = bound !== undefined && bound.quality !== "good";
    useEffect(() => setDraft(sourceValue), [sourceValue]);
    const commit = (next: number) => {
      if (context.mode === "runtime" && !disabled) {
        context.onWriteTag(props.tagPath || "value", tagValueFromNumber(clampNumber(next, props.min, max)));
      }
    };
    return (
      <label className="mdc-slider" aria-label="Slider" style={{ ...mdStyles.slider, ...badQualityStyle(bound) }}>
        <input
          aria-label="Slider input"
          type="range"
          min={props.min}
          max={max}
          step={props.step}
          value={draft}
          disabled={disabled}
          onChange={(event) => {
            const next = Number(event.currentTarget.value);
            setDraft(next);
            if (props.commitMode === "live") {
              const now = Date.now();
              if (now - lastLiveWrite.current >= 33) {
                lastLiveWrite.current = now;
                commit(next);
              }
            }
          }}
          onMouseUp={() => commit(draft)}
          onTouchEnd={() => commit(draft)}
          style={mdStyles.range}
        />
        <span style={mdStyles.sliderValue}>{formatNumber(draft)}{props.units ? ` ${props.units}` : ""}</span>
      </label>
    );
  },
};

const materialCard: ComponentDefinition<CardProps> = {
  ...Card,
  Render({ props, context, children }) {
    return (
      <section
        className="mdc-card"
        aria-label="Card"
        style={{
          ...mdStyles.card,
          ...designerStyle(context.mode === "designer" ? context.isSelected : false),
        }}
      >
        {props.title || props.subtitle ? (
          <header style={mdStyles.cardHeader}>
            {props.title ? <strong>{props.title}</strong> : null}
            {props.subtitle ? <span style={mdStyles.cardSubtitle}>{props.subtitle}</span> : null}
          </header>
        ) : null}
        <div style={mdStyles.cardBody}>{children}</div>
      </section>
    );
  },
};

const materialDropdown: ComponentDefinition<DropdownProps> = {
  ...Dropdown,
  Render({ props, bindings, context }) {
    const bound = bindings.value;
    const options = Array.isArray(props.options) ? props.options : [];
    const current = tagValueToPrimitive(bound?.value) ?? props.value ?? "";
    const selected = options.find((option) => option.value === current);
    const disabled = bound !== undefined && bound.quality !== "good";
    return (
      <select
        className="mdc-select"
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
          ...mdStyles.select,
          ...badQualityStyle(bound),
          ...designerStyle(context.mode === "designer" ? context.isSelected : false),
        }}
      >
        <option value="" disabled>{props.placeholder}</option>
        {options.map((option) => (
          <option key={encodeValue(option.value)} value={encodeValue(option.value)}>{option.label}</option>
        ))}
      </select>
    );
  },
};

const materialGauge: ComponentDefinition<GaugeProps> = {
  ...Gauge,
  Render(props) {
    return (
      <div className="mdc-gauge">
        <Gauge.Render {...props} />
      </div>
    );
  },
};

const materialModal: ComponentDefinition<ModalProps> = {
  ...Modal,
  Render(props) {
    return (
      <div className="mdc-dialog" style={mdStyles.modalShell}>
        <Modal.Render {...props} />
      </div>
    );
  },
};

function mdButtonStyle(style: ButtonProps["style"]) {
  if (style === "danger") {
    return { background: "var(--error, #b3261e)", color: "#ffffff" };
  }
  if (style === "secondary") {
    return { background: "transparent", color: "var(--primary-color, #6750a4)", boxShadow: "none" };
  }
  return { background: "var(--primary-color, #6750a4)", color: "#ffffff" };
}

function encodeValue(value: string | number | boolean): string {
  return `${typeof value}:${String(value)}`;
}

function formatNumber(value: number) {
  return Number.isInteger(value) ? String(value) : value.toFixed(2);
}

const mdStyles = {
  button: {
    position: "relative" as const,
    overflow: "hidden",
    minHeight: 40,
    minWidth: 96,
    padding: "0 20px",
    border: 0,
    borderRadius: 20,
    boxShadow: "0 1px 2px rgba(0,0,0,0.18)",
    fontFamily: baseFont,
    fontWeight: 700,
    cursor: "pointer",
    transition: "box-shadow 180ms ease, transform 180ms ease",
  },
  ripple: {
    position: "absolute" as const,
    inset: 0,
    background: "radial-gradient(circle, rgba(255,255,255,0.28) 0, transparent 60%)",
    opacity: 0.18,
    pointerEvents: "none" as const,
  },
  switch: {
    position: "relative" as const,
    display: "inline-flex",
    alignItems: "center",
    minWidth: 92,
    height: 36,
    padding: "0 12px 0 44px",
    border: 0,
    borderRadius: 18,
    fontFamily: baseFont,
    fontWeight: 700,
    cursor: "pointer",
  },
  switchHandle: {
    position: "absolute" as const,
    left: 0,
    width: 28,
    height: 28,
    borderRadius: "50%",
    boxShadow: "0 1px 3px rgba(0,0,0,0.24)",
    transition: "transform 180ms ease",
  },
  switchLabel: { marginLeft: 4, color: "var(--text-primary, #1d1b20)" },
  field: { display: "grid", gap: 2, fontFamily: baseFont },
  floatingLabel: { fontSize: 11, color: "var(--primary-color, #6750a4)", paddingLeft: 12 },
  input: {
    width: 132,
    minHeight: 44,
    boxSizing: "border-box" as const,
    padding: "14px 12px 6px",
    border: 0,
    borderBottom: "2px solid var(--primary-color, #6750a4)",
    borderRadius: "12px 12px 0 0",
    background: "color-mix(in srgb, var(--surface, #fff) 86%, var(--primary-color, #6750a4))",
    color: "var(--text-primary, #1d1b20)",
    fontFamily: baseFont,
  },
  slider: { display: "grid", gridTemplateColumns: "minmax(140px, 1fr) auto", gap: 12, alignItems: "center", minWidth: 240, padding: 8, fontFamily: baseFont },
  range: { accentColor: "var(--primary-color, #6750a4)", width: "100%" },
  sliderValue: { minWidth: 64, textAlign: "right" as const, fontVariantNumeric: "tabular-nums" as const },
  card: { minWidth: 180, overflow: "hidden", borderRadius: 12, background: "var(--surface, #ffffff)", boxShadow: "0 2px 6px rgba(0,0,0,0.18)", fontFamily: baseFont },
  cardHeader: { display: "grid", gap: 2, padding: "14px 16px", color: "var(--text-primary, #1d1b20)" },
  cardSubtitle: { fontSize: 12, color: "var(--text-secondary, #625b71)" },
  cardBody: { padding: 16 },
  select: { minWidth: 176, minHeight: 44, padding: "8px 12px", border: 0, borderBottom: "2px solid var(--primary-color, #6750a4)", borderRadius: "12px 12px 0 0", background: "var(--surface, #ffffff)", color: "var(--text-primary, #1d1b20)", fontFamily: baseFont },
  modalShell: { borderRadius: 28, overflow: "hidden" },
};

export const materialPack: Record<string, ComponentDefinition> = {
  Button: materialButton,
  ToggleSwitch: materialToggleSwitch,
  NumericInput: materialNumericInput,
  Slider: materialSlider,
  Card: materialCard,
  Dropdown: materialDropdown,
  Gauge: materialGauge,
  Modal: materialModal,
};
