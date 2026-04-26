import { useEffect, useRef, useState } from "react";
import type { Quality, TagValue } from "@openwebhmi/protocol";
import {
  GatewayClient,
  type ConnectionState,
  type TagUpdate,
} from "./gatewayClient";

type LiveTag = {
  value: TagValue | null;
  quality: Quality | null;
  ts: number | null;
};

const EMPTY_TAG: LiveTag = { value: null, quality: null, ts: null };

export function App() {
  const clientRef = useRef<GatewayClient | null>(null);
  if (clientRef.current === null) {
    clientRef.current = new GatewayClient({
      url: import.meta.env.VITE_GATEWAY_URL ?? "ws://localhost:8080",
    });
  }
  const client = clientRef.current;
  const [connectionState, setConnectionState] =
    useState<ConnectionState>("connecting");
  const [sin, setSin] = useState<LiveTag>(EMPTY_TAG);
  const [counter, setCounter] = useState<LiveTag>(EMPTY_TAG);

  useEffect(() => {
    const offState = client.onStateChange(setConnectionState);
    client.connect();

    const unsubscribeSin = client.subscribe("system/sim/sin", (update) =>
      setSin(toLiveTag(update)),
    );
    const unsubscribeCounter = client.subscribe(
      "system/sim/counter",
      (update) => setCounter(toLiveTag(update)),
    );

    return () => {
      unsubscribeSin();
      unsubscribeCounter();
      offState();
      client.disconnect();
    };
  }, [client]);

  return (
    <main style={styles.page}>
      <section style={styles.panel}>
        <h1 style={styles.heading}>OpenWebHMI Runtime</h1>
        <div style={styles.status}>Gateway: {connectionState}</div>
        <TagRow label="system/sim/sin" tag={sin} />
        <TagRow label="system/sim/counter" tag={counter} />
      </section>
    </main>
  );
}

function TagRow({ label, tag }: { label: string; tag: LiveTag }) {
  return (
    <div style={styles.row}>
      <div>
        <div style={styles.path}>{label}</div>
        <div style={styles.meta}>
          {tag.quality ?? "unknown"} ·{" "}
          {tag.ts === null ? "no updates yet" : new Date(tag.ts).toLocaleTimeString()}
        </div>
      </div>
      <div style={styles.value}>{formatTagValue(tag.value)}</div>
    </div>
  );
}

function toLiveTag(update: TagUpdate): LiveTag {
  return {
    value: update.value,
    quality: update.quality,
    ts: update.ts,
  };
}

function formatTagValue(value: TagValue | null): string {
  if (value === null) {
    return "—";
  }

  if (value.type === "real") {
    return value.value.toFixed(4);
  }

  return String(value.value);
}

const styles = {
  page: {
    margin: 0,
    minHeight: "100vh",
    display: "grid",
    placeItems: "center",
    background: "#f4f6f8",
    color: "#1f2933",
    fontFamily:
      'Inter, ui-sans-serif, system-ui, -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif',
  },
  panel: {
    width: "min(720px, calc(100vw - 32px))",
    background: "#ffffff",
    border: "1px solid #d9e2ec",
    borderRadius: 8,
    padding: 24,
    boxShadow: "0 12px 32px rgba(16, 24, 40, 0.08)",
  },
  heading: {
    margin: "0 0 8px",
    fontSize: 24,
    fontWeight: 700,
  },
  status: {
    marginBottom: 20,
    color: "#52606d",
    fontSize: 14,
  },
  row: {
    display: "flex",
    justifyContent: "space-between",
    alignItems: "center",
    gap: 16,
    padding: "16px 0",
    borderTop: "1px solid #e4e7eb",
  },
  path: {
    fontFamily: "ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace",
    fontSize: 14,
  },
  meta: {
    marginTop: 4,
    color: "#697586",
    fontSize: 13,
  },
  value: {
    fontFamily: "ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace",
    fontSize: 28,
    fontWeight: 700,
    minWidth: 160,
    textAlign: "right" as const,
  },
};
