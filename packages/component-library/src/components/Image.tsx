import type { ComponentDefinition } from "../types";
import { badQualityStyle, baseFont, designerStyle, tagValueToText } from "./shared";

export type ImageProps = {
  src: string;
  width: number;
  height: number;
  fit: "contain" | "cover";
  alt: string;
};

/** Static or tag-bound image renderer for project assets and URL-backed visuals. */
export const Image: ComponentDefinition<ImageProps> = {
  kind: "Image",
  defaultProps: {
    src: "",
    width: 160,
    height: 96,
    fit: "contain",
    alt: "HMI image",
  },
  propsSchema: {
    src: { type: "string", label: "Source URL", default: "" },
    width: { type: "number", label: "Width", default: 160 },
    height: { type: "number", label: "Height", default: 96 },
    fit: {
      type: "select",
      label: "Fit",
      options: ["contain", "cover"],
      default: "contain",
    },
    alt: { type: "string", label: "Alt text", default: "HMI image" },
  },
  bindableProps: ["src"],
  Render({ props, bindings, context }) {
    const bound = bindings.src;
    const src = bound ? tagValueToText(bound.value) : props.src;

    return (
      <div
        aria-label="Image"
        style={{
          display: "inline-grid",
          placeItems: "center",
          width: props.width,
          height: props.height,
          overflow: "hidden",
          boxSizing: "border-box",
          border: "1px solid #cbd2d9",
          borderRadius: 6,
          background: "#f5f7fa",
          color: "#52606d",
          fontFamily: baseFont,
          fontSize: 12,
          ...badQualityStyle(bound),
          ...designerStyle(context.mode === "designer" ? context.isSelected : false),
        }}
      >
        {src ? (
          <img
            alt={props.alt}
            src={src}
            style={{ width: "100%", height: "100%", objectFit: props.fit }}
          />
        ) : (
          <span>No image</span>
        )}
      </div>
    );
  },
};
