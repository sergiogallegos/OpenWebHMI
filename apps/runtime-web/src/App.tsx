import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import {
  DEFAULT_THEME,
  applyTheme,
  themeToCss,
  type Theme,
} from "@openwebhmi/protocol";
import {
  collectTagPaths,
  ViewRenderer,
} from "./ViewRenderer";
import {
  GatewayClient,
  type ConnectionState,
} from "./gatewayClient";
import { Login } from "./modules/Login";
import { useTagBindings } from "./useTagBindings";
import { useViewSubscription } from "./useViewSubscription";

const params = new URLSearchParams(window.location.search);
const PROJECT_ID =
  params.get("project") ?? import.meta.env.VITE_PROJECT_ID ?? "phase1-demo";
const INITIAL_VIEW =
  params.get("view") ?? import.meta.env.VITE_INITIAL_VIEW ?? "home";

export function App() {
  const [sessionToken, setSessionToken] = useState(() =>
    window.localStorage.getItem("openwebhmi.sessionToken"),
  );
  const clientRef = useRef<GatewayClient | null>(null);
  if (clientRef.current === null) {
    clientRef.current = new GatewayClient({
      url: import.meta.env.VITE_GATEWAY_URL ?? "ws://localhost:8080",
      tokenProvider: () => window.localStorage.getItem("openwebhmi.sessionToken"),
    });
  }
  const client = clientRef.current;
  const [connectionState, setConnectionState] =
    useState<ConnectionState>("connecting");
  const [packId, setPackId] = useState<string | null>(null);

  useEffect(() => {
    if (!sessionToken) {
      return;
    }
    const offState = client.onStateChange(setConnectionState);
    const offError = client.onError((error) => {
      if (error.code === "auth.required") {
        window.localStorage.removeItem("openwebhmi.sessionToken");
        setSessionToken(null);
        client.disconnect();
      }
    });
    client.connect();
    return () => {
      offState();
      offError();
      client.disconnect();
    };
  }, [client, sessionToken]);

  useEffect(() => {
    if (!sessionToken || connectionState !== "connected") {
      return;
    }
    let cancelled = false;
    client
      .readTheme(PROJECT_ID)
      .then((theme) => {
        if (!cancelled) {
          applyRuntimeTheme(theme);
          setPackId(theme?.pack ?? null);
        }
      })
      .catch(() => {
        if (!cancelled) {
          removeRuntimeTheme();
          setPackId(null);
        }
      });
    return () => {
      cancelled = true;
    };
  }, [client, connectionState, sessionToken]);

  const { view, version, error } = useViewSubscription(
    client,
    PROJECT_ID,
    INITIAL_VIEW,
  );
  const tagPaths = useMemo(() => (view ? collectTagPaths(view) : []), [view]);
  const { boundValues, writeTag } = useTagBindings(client, tagPaths);
  const subscribeAlarms = useCallback(
    (options: Parameters<GatewayClient["subscribeAlarms"]>[0], callback: Parameters<GatewayClient["subscribeAlarms"]>[1]) =>
      client.subscribeAlarms(options, callback),
    [client],
  );
  const ackAlarm = useCallback(
    (alarmId: string, note?: string | null) => client.ackAlarm(alarmId, note),
    [client],
  );
  const readHistory = useCallback(
    (options: Parameters<GatewayClient["readHistory"]>[0]) => client.readHistory(options),
    [client],
  );

  if (!sessionToken) {
    return (
      <Login
        client={client}
        onAuthenticated={(token) => {
          window.localStorage.setItem("openwebhmi.sessionToken", token);
          setSessionToken(token);
        }}
      />
    );
  }

  return (
    <main style={styles.page}>
      <header style={styles.header}>
        <div>
          <h1 style={styles.title}>{view?.title ?? INITIAL_VIEW}</h1>
          <div style={styles.meta}>
            {PROJECT_ID}
            {version === null ? "" : ` · v${version}`}
          </div>
        </div>
        <ConnectionBadge state={connectionState} />
      </header>

      {connectionState === "reconnecting" || connectionState === "closed" ? (
        <div role="status" style={styles.banner}>
          Gateway disconnected. Showing the last loaded view.
        </div>
      ) : null}

      {error ? (
        <section role="alert" style={styles.error}>
          {error}
        </section>
      ) : view ? (
        <ViewRenderer
          view={view}
          projectId={PROJECT_ID}
          boundValues={boundValues}
          onWriteTag={writeTag}
          onSubscribeAlarms={subscribeAlarms}
          onAckAlarm={ackAlarm}
          onReadHistory={readHistory}
          packId={packId}
        />
      ) : (
        <section role="status" style={styles.loading}>
          {connectionState === "connecting" ? "Connecting..." : "Loading view..."}
        </section>
      )}
    </main>
  );
}

export function applyRuntimeTheme(theme: Theme | null) {
  if (!theme) {
    removeRuntimeTheme();
    return;
  }
  let style = document.getElementById("openwebhmi-project-theme") as HTMLStyleElement | null;
  if (!style) {
    style = document.createElement("style");
    style.id = "openwebhmi-project-theme";
    document.head.prepend(style);
  }
  style.textContent = themeToCss(theme);
  const storedMode = window.localStorage?.getItem("openwebhmi.themeMode");
  const mode = storedMode === "dark" || storedMode === "light" ? storedMode : theme.mode;
  applyTheme({ ...theme, mode });
}

export function removeRuntimeTheme() {
  document.getElementById("openwebhmi-project-theme")?.remove();
  applyTheme(DEFAULT_THEME);
}

function ConnectionBadge({ state }: { state: ConnectionState }) {
  const connected = state === "connected";
  return (
    <div
      aria-label="Gateway connection state"
      style={{
        ...styles.badge,
        background: connected ? "#ecfdf3" : "#fff7ed",
        borderColor: connected ? "#86efac" : "#fdba74",
        color: connected ? "#166534" : "#9a3412",
      }}
    >
      {stateLabel(state)}
    </div>
  );
}

function stateLabel(state: ConnectionState): string {
  switch (state) {
    case "connected":
      return "Connected";
    case "reconnecting":
      return "Reconnecting";
    case "closed":
      return "Closed";
    default:
      return "Connecting";
  }
}

const fontFamily =
  'Inter, ui-sans-serif, system-ui, -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif';

const styles = {
  page: {
    minHeight: "100vh",
    boxSizing: "border-box" as const,
    padding: 24,
    background: "#f4f6f8",
    color: "#1f2933",
    fontFamily,
  },
  header: {
    display: "flex",
    alignItems: "center",
    justifyContent: "space-between",
    gap: 16,
    marginBottom: 16,
  },
  title: {
    margin: 0,
    fontSize: 24,
    fontWeight: 700,
  },
  meta: {
    marginTop: 4,
    color: "#52606d",
    fontSize: 13,
  },
  badge: {
    minWidth: 116,
    boxSizing: "border-box" as const,
    padding: "6px 10px",
    border: "1px solid",
    borderRadius: 6,
    textAlign: "center" as const,
    fontSize: 13,
    fontWeight: 600,
  },
  banner: {
    marginBottom: 16,
    padding: "10px 12px",
    border: "1px solid #fdba74",
    borderRadius: 6,
    background: "#fff7ed",
    color: "#9a3412",
    fontSize: 14,
  },
  loading: {
    padding: 24,
    border: "1px solid #d9e2ec",
    borderRadius: 8,
    background: "#ffffff",
    color: "#52606d",
  },
  error: {
    padding: 24,
    border: "1px solid #fca5a5",
    borderRadius: 8,
    background: "#fef2f2",
    color: "#991b1b",
  },
};
