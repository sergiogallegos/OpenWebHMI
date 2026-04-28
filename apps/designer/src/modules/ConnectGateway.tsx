import { useState } from "react";

/** Props for the gateway connection form. */
export type ConnectGatewayProps = {
  defaultUrl: string;
  connecting: boolean;
  error: string | null;
  onConnect: (url: string, username: string, password: string) => void;
};

/** Gateway URL picker for the anonymous Phase 2 designer connection. */
export function ConnectGateway({
  defaultUrl,
  connecting,
  error,
  onConnect,
}: ConnectGatewayProps) {
  const [url, setUrl] = useState(defaultUrl);
  const [username, setUsername] = useState("admin");
  const [password, setPassword] = useState("");

  return (
    <main style={styles.page}>
      <form
        style={styles.form}
        onSubmit={(event) => {
          event.preventDefault();
          onConnect(url, username, password);
        }}
      >
        <h1 style={styles.title}>OpenWebHMI Designer</h1>
        <label style={styles.label}>
          Gateway URL
          <input
            aria-label="Gateway URL"
            value={url}
            onChange={(event) => setUrl(event.currentTarget.value)}
            style={styles.input}
          />
        </label>
        <label style={styles.label}>
          Username
          <input
            aria-label="Username"
            value={username}
            onChange={(event) => setUsername(event.currentTarget.value)}
            autoComplete="username"
            style={styles.input}
          />
        </label>
        <label style={styles.label}>
          Password
          <input
            aria-label="Password"
            type="password"
            value={password}
            onChange={(event) => setPassword(event.currentTarget.value)}
            autoComplete="current-password"
            style={styles.input}
          />
        </label>
        <button type="submit" disabled={connecting} style={styles.button}>
          {connecting ? "Connecting" : "Connect"}
        </button>
        {error ? (
          <div role="alert" style={styles.error}>
            {error}
          </div>
        ) : null}
      </form>
    </main>
  );
}

const styles = {
  page: {
    minHeight: "100vh",
    display: "grid",
    placeItems: "center",
    background: "#f4f6f8",
    color: "#1f2933",
    fontFamily:
      'Inter, ui-sans-serif, system-ui, -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif',
  },
  form: {
    width: "min(440px, calc(100vw - 32px))",
    display: "grid",
    gap: 14,
    padding: 24,
    border: "1px solid #d9e2ec",
    borderRadius: 8,
    background: "#ffffff",
  },
  title: {
    margin: 0,
    fontSize: 22,
  },
  label: {
    display: "grid",
    gap: 6,
    fontSize: 13,
    fontWeight: 600,
  },
  input: {
    minHeight: 36,
    padding: "6px 8px",
    border: "1px solid #9aa5b1",
    borderRadius: 6,
    font: "inherit",
  },
  button: {
    minHeight: 36,
    border: "1px solid #1f4e79",
    borderRadius: 6,
    background: "#1f4e79",
    color: "#ffffff",
    fontWeight: 700,
  },
  error: {
    padding: 10,
    border: "1px solid #fca5a5",
    borderRadius: 6,
    background: "#fef2f2",
    color: "#991b1b",
    fontSize: 13,
  },
};
