import { useEffect, useRef, useState } from "react";
import type { ComponentDefinition } from "../types";
import { badQualityStyle, baseFont, designerStyle } from "./shared";

export type ModalProps = {
  title: string;
  open: boolean;
  confirmLabel: string;
  cancelLabel: string;
  confirmTagPath?: string;
  cancelTagPath?: string;
  confirmStyle: "primary" | "danger";
};

/** Operator confirmation dialog with optional write-back actions. */
export const Modal: ComponentDefinition<ModalProps> = {
  kind: "Modal",
  defaultProps: {
    title: "Confirm",
    open: false,
    confirmLabel: "Confirm",
    cancelLabel: "Cancel",
    confirmStyle: "primary",
  },
  propsSchema: {
    title: { type: "string", label: "Title", default: "Confirm" },
    open: { type: "boolean", label: "Open", default: false },
    confirmLabel: { type: "string", label: "Confirm label", default: "Confirm" },
    cancelLabel: { type: "string", label: "Cancel label", default: "Cancel" },
    confirmTagPath: { type: "string", label: "Confirm tag path" },
    cancelTagPath: { type: "string", label: "Cancel tag path" },
    confirmStyle: { type: "select", label: "Confirm style", options: ["primary", "danger"], default: "primary" },
  },
  bindableProps: ["open"],
  Render({ props, bindings, context, children }) {
    const bound = bindings.open;
    const boundOpen = bound?.value.type === "bool" ? bound.value.value : props.open;
    const [localOpen, setLocalOpen] = useState(boundOpen);
    const dialogRef = useRef<HTMLDialogElement>(null);

    useEffect(() => setLocalOpen(boundOpen), [boundOpen]);
    useEffect(() => {
      const dialog = dialogRef.current;
      if (!dialog || context.mode === "designer") return;
      if (localOpen && !dialog.open) {
        if (typeof dialog.showModal === "function") {
          dialog.showModal();
        } else {
          dialog.setAttribute("open", "");
        }
      }
      if (!localOpen && dialog.open && typeof dialog.close === "function") {
        dialog.close();
      } else if (!localOpen) {
        dialog.removeAttribute("open");
      }
    }, [context.mode, localOpen]);

    const write = (path: string | undefined) => {
      if (context.mode === "runtime" && path) {
        context.onWriteTag(path, { type: "bool", value: true });
      }
      setLocalOpen(false);
    };

    const body = (
      <section
        aria-label="Modal body"
        style={{
          ...styles.body,
          ...badQualityStyle(bound),
          ...designerStyle(context.mode === "designer" ? context.isSelected : false),
        }}
      >
        <header style={styles.header}>
          {context.mode === "designer" ? <span style={styles.badge}>Modal preview</span> : null}
          <strong>{props.title}</strong>
        </header>
        <div style={styles.content}>{children}</div>
        <footer style={styles.footer}>
          <button type="button" style={styles.cancel} onClick={() => write(props.cancelTagPath)}>
            {props.cancelLabel}
          </button>
          <button type="button" style={{ ...styles.confirm, ...(props.confirmStyle === "danger" ? styles.danger : null) }} onClick={() => write(props.confirmTagPath)}>
            {props.confirmLabel}
          </button>
        </footer>
      </section>
    );

    if (context.mode === "designer") {
      return body;
    }
    return (
      <dialog
        ref={dialogRef}
        aria-label="Modal"
        onCancel={(event) => {
          event.preventDefault();
          write(props.cancelTagPath);
        }}
        onClick={(event) => {
          if (event.target === dialogRef.current) write(props.cancelTagPath);
        }}
        style={styles.dialog}
      >
        {body}
      </dialog>
    );
  },
};

const styles = {
  dialog: { border: "none", background: "transparent", padding: 0 },
  body: { minWidth: 280, border: "1px solid #cbd2d9", borderRadius: 8, background: "#ffffff", fontFamily: baseFont, boxShadow: "0 10px 30px rgba(0,0,0,0.2)" },
  header: { display: "flex", gap: 8, alignItems: "center", padding: 12, borderBottom: "1px solid #e5e7eb" },
  badge: { padding: "2px 6px", borderRadius: 999, background: "#dbeafe", color: "#1e3a8a", fontSize: 11 },
  content: { padding: 12 },
  footer: { display: "flex", justifyContent: "flex-end", gap: 8, padding: 12, borderTop: "1px solid #e5e7eb" },
  cancel: { padding: "7px 12px", border: "1px solid #9aa5b1", borderRadius: 6, background: "#ffffff" },
  confirm: { padding: "7px 12px", border: "1px solid #163a5a", borderRadius: 6, background: "#1f4e79", color: "#ffffff" },
  danger: { borderColor: "#b91c1c", background: "#dc2626" },
};
