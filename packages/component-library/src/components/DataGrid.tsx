import { useMemo, useState } from "react";
import type { ComponentDefinition } from "../types";
import { badQualityStyle, baseFont, designerStyle } from "./shared";

export type DataGridColumn = {
  key: string;
  label: string;
  type: "string" | "number" | "boolean" | "datetime";
  width?: number;
};

export type DataGridProps = {
  rows?: Record<string, unknown>[];
  columns: DataGridColumn[];
  pageSize: number;
  sortBy?: string;
  sortDirection: "asc" | "desc";
};

/** Sortable, paginated table bound to JSON row data. */
export const DataGrid: ComponentDefinition<DataGridProps> = {
  kind: "DataGrid",
  defaultProps: {
    rows: [
      { tag: "Pressure", value: 120.5, state: "Good" },
      { tag: "Counter", value: 42, state: "Good" },
    ],
    columns: [
      { key: "tag", label: "Tag", type: "string" },
      { key: "value", label: "Value", type: "number" },
      { key: "state", label: "State", type: "string" },
    ],
    pageSize: 25,
    sortDirection: "asc",
  },
  propsSchema: {
    rows: { type: "objectList", label: "Rows", default: [] },
    columns: {
      type: "objectList",
      label: "Columns",
      default: [
        { key: "tag", label: "Tag", type: "string" },
        { key: "value", label: "Value", type: "number" },
      ],
    },
    pageSize: { type: "number", label: "Page size", default: 25 },
    sortBy: { type: "string", label: "Sort by" },
    sortDirection: { type: "select", label: "Sort direction", options: ["asc", "desc"], default: "asc" },
  },
  bindableProps: ["rows"],
  Render({ props, bindings, context }) {
    const bound = bindings.rows;
    const columns = normalizeColumns(props.columns);
    const rows = rowsFromBound(bound?.value) ?? normalizeRows(props.rows);
    const [sort, setSort] = useState({ key: props.sortBy ?? "", direction: props.sortDirection });
    const [page, setPage] = useState(0);
    const pageSize = Math.max(1, props.pageSize || 25);
    const sorted = useMemo(() => sort.key ? sortRows(rows, sort.key, sort.direction) : rows, [rows, sort]);
    const totalPages = Math.max(1, Math.ceil(sorted.length / pageSize));
    const visible = sorted.slice(page * pageSize, page * pageSize + pageSize);

    return (
      <section
        aria-label="Data grid"
        style={{
          ...styles.shell,
          ...badQualityStyle(bound),
          ...designerStyle(context.mode === "designer" ? context.isSelected : false),
        }}
      >
        {visible.length === 0 ? <div style={styles.empty}>No data</div> : (
          <table style={styles.table}>
            <thead>
              <tr>
                {columns.map((column) => (
                  <th key={column.key} style={{ ...styles.th, width: column.width }} scope="col">
                    <button
                      type="button"
                      style={styles.headerButton}
                      onClick={() => {
                        setSort((current) => ({
                          key: column.key,
                          direction: current.key === column.key && current.direction === "asc" ? "desc" : "asc",
                        }));
                        setPage(0);
                      }}
                    >
                      {column.label} {sort.key === column.key ? (sort.direction === "asc" ? "▲" : "▼") : ""}
                    </button>
                  </th>
                ))}
              </tr>
            </thead>
            <tbody>
              {visible.map((row, rowIndex) => (
                <tr key={rowIndex}>
                  {columns.map((column) => (
                    <td key={column.key} style={styles.td}>{formatCell(row[column.key], column.type)}</td>
                  ))}
                </tr>
              ))}
            </tbody>
          </table>
        )}
        <div style={styles.pager}>
          <button type="button" onClick={() => setPage(Math.max(0, page - 1))} disabled={page === 0}>Prev</button>
          <span>Page {page + 1} / {totalPages}</span>
          <button type="button" onClick={() => setPage(Math.min(totalPages - 1, page + 1))} disabled={page >= totalPages - 1}>Next</button>
        </div>
      </section>
    );
  },
};

function normalizeColumns(columns: DataGridColumn[] | undefined): DataGridColumn[] {
  return Array.isArray(columns) ? columns.filter((column) => column.key && column.label) : [];
}

function normalizeRows(rows: Record<string, unknown>[] | undefined): Record<string, unknown>[] {
  return Array.isArray(rows) ? rows.filter((row) => typeof row === "object" && row !== null) : [];
}

function rowsFromBound(value: unknown): Record<string, unknown>[] | null {
  if (!value || typeof value !== "object" || !("type" in value)) return null;
  const tag = value as { type: string; value: unknown };
  if (tag.type !== "string" || typeof tag.value !== "string") return null;
  try {
    return normalizeRows(JSON.parse(tag.value));
  } catch {
    return null;
  }
}

function sortRows(rows: Record<string, unknown>[], key: string, direction: "asc" | "desc") {
  const multiplier = direction === "asc" ? 1 : -1;
  return [...rows].sort((left, right) => compare(left[key], right[key]) * multiplier);
}

function compare(left: unknown, right: unknown): number {
  if (typeof left === "number" && typeof right === "number") return left - right;
  return String(left ?? "").localeCompare(String(right ?? ""));
}

function formatCell(value: unknown, type: DataGridColumn["type"]): string {
  if (value === undefined || value === null) return "";
  if (type === "datetime") {
    const date = new Date(String(value));
    return Number.isNaN(date.valueOf()) ? String(value) : date.toLocaleString();
  }
  return String(value);
}

const styles = {
  shell: { display: "grid", gap: 8, minWidth: 280, padding: 8, border: "1px solid #cbd2d9", borderRadius: 8, background: "#ffffff", fontFamily: baseFont },
  table: { width: "100%", borderCollapse: "collapse" as const, fontSize: 13 },
  th: { padding: 0, borderBottom: "1px solid #cbd2d9", textAlign: "left" as const },
  headerButton: { width: "100%", padding: "7px 8px", border: 0, background: "#f8fafc", textAlign: "left" as const, fontWeight: 700 },
  td: { padding: "7px 8px", borderBottom: "1px solid #eef2f7" },
  pager: { display: "flex", gap: 8, alignItems: "center", justifyContent: "flex-end", fontSize: 12 },
  empty: { color: "#697586", fontSize: 13 },
};
