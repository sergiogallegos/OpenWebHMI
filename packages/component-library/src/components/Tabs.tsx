import { useState } from "react";
import type { ComponentDefinition } from "../types";
import { badQualityStyle, baseFont, designerStyle, tagValueToText } from "./shared";

export type TabItem = { id: string; label: string };

export type TabsProps = {
  tabs: TabItem[];
  activeTab: string;
  tabPosition: "top" | "left";
};

/** Multi-page container that maps child index to configured tab index. */
export const Tabs: ComponentDefinition<TabsProps> = {
  kind: "Tabs",
  defaultProps: {
    tabs: [
      { id: "overview", label: "Overview" },
      { id: "details", label: "Details" },
    ],
    activeTab: "overview",
    tabPosition: "top",
  },
  propsSchema: {
    tabs: {
      type: "objectList",
      label: "Tabs",
      default: [
        { id: "overview", label: "Overview" },
        { id: "details", label: "Details" },
      ],
    },
    activeTab: { type: "string", label: "Active tab", default: "overview" },
    tabPosition: {
      type: "select",
      label: "Tab position",
      options: ["top", "left"],
      default: "top",
    },
  },
  bindableProps: ["activeTab"],
  Render({ props, bindings, context, children }) {
    const tabs = normalizeTabs(props.tabs);
    const bound = bindings.activeTab;
    const boundActive = tagValueToText(bound?.value);
    const first = tabs[0]?.id ?? "";
    const boundSelected = tabs.some((tab) => tab.id === boundActive)
      ? boundActive
      : tabs.some((tab) => tab.id === props.activeTab)
        ? props.activeTab
        : first;
    const [localActive, setLocalActive] = useState(boundSelected);
    const selected = bound && bound.quality !== "good" ? first : bound ? boundSelected : localActive || first;
    const activeIndex = Math.max(0, tabs.findIndex((tab) => tab.id === selected));
    const childArray = Array.isArray(children) ? children : children ? [children] : [];
    const vertical = props.tabPosition === "left";

    const selectIndex = (index: number) => {
      const next = tabs[(index + tabs.length) % tabs.length];
      if (next) {
        setLocalActive(next.id);
      }
    };

    return (
      <section
        aria-label="Tabs"
        style={{
          ...styles.shell,
          ...(vertical ? styles.shellLeft : null),
          ...badQualityStyle(bound),
          ...designerStyle(context.mode === "designer" ? context.isSelected : false),
        }}
      >
        <div
          role="tablist"
          aria-orientation={vertical ? "vertical" : "horizontal"}
          style={{ ...styles.tabList, ...(vertical ? styles.tabListLeft : null) }}
          onKeyDown={(event) => {
            if (event.key === "ArrowRight" || event.key === "ArrowDown") selectIndex(activeIndex + 1);
            if (event.key === "ArrowLeft" || event.key === "ArrowUp") selectIndex(activeIndex - 1);
            if (event.key === "Home") selectIndex(0);
            if (event.key === "End") selectIndex(tabs.length - 1);
          }}
        >
          {tabs.map((tab, index) => (
            <button
              key={tab.id}
              role="tab"
              aria-selected={index === activeIndex}
              type="button"
              onClick={() => setLocalActive(tab.id)}
              style={{ ...styles.tab, ...(index === activeIndex ? styles.tabActive : null) }}
            >
              {tab.label}
            </button>
          ))}
        </div>
        <div role="tabpanel" style={styles.panel}>
          {childArray[activeIndex] ?? <div style={styles.empty}>No content</div>}
        </div>
      </section>
    );
  },
};

function normalizeTabs(tabs: TabItem[] | undefined): TabItem[] {
  return Array.isArray(tabs)
    ? tabs.filter((tab) => tab.id && tab.label)
    : [];
}

const styles = {
  shell: {
    display: "grid",
    gap: 10,
    minWidth: 220,
    padding: 10,
    border: "1px solid #cbd2d9",
    borderRadius: 8,
    background: "#ffffff",
    fontFamily: baseFont,
  },
  shellLeft: { gridTemplateColumns: "auto 1fr" },
  tabList: { display: "flex", gap: 6, borderBottom: "1px solid #e5e7eb" },
  tabListLeft: { flexDirection: "column" as const, borderBottom: "none", borderRight: "1px solid #e5e7eb" },
  tab: {
    padding: "7px 10px",
    border: "1px solid transparent",
    borderRadius: 6,
    background: "transparent",
    color: "#52606d",
    cursor: "pointer",
  },
  tabActive: { background: "#dbeafe", color: "#1e3a8a", fontWeight: 700 },
  panel: { minHeight: 80 },
  empty: { color: "#697586", fontSize: 13 },
};
