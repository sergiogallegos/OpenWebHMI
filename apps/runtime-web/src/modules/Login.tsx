import { useState } from "react";
import type { FormEvent } from "react";
import type { GatewayClient } from "../gatewayClient";

type LoginProps = {
  client: GatewayClient;
  onAuthenticated(token: string): void;
};

export function Login({ client, onAuthenticated }: LoginProps) {
  const [username, setUsername] = useState("admin");
  const [password, setPassword] = useState("");
  const [error, setError] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);

  const submit = async (event: FormEvent) => {
    event.preventDefault();
    setBusy(true);
    setError(null);
    try {
      const result = await client.login(username, password);
      if (!result.session_token) {
        setError(result.error ?? "Login failed");
        return;
      }
      onAuthenticated(result.session_token);
    } catch (error) {
      setError(error instanceof Error ? error.message : String(error));
    } finally {
      setBusy(false);
    }
  };

  return (
    <main style={styles.page}>
      <form onSubmit={submit} style={styles.form}>
        <h1 style={styles.title}>OpenWebHMI</h1>
        <label style={styles.label}>
          Username
          <input
            value={username}
            onChange={(event) => setUsername(event.target.value)}
            autoComplete="username"
            style={styles.input}
          />
        </label>
        <label style={styles.label}>
          Password
          <input
            type="password"
            value={password}
            onChange={(event) => setPassword(event.target.value)}
            autoComplete="current-password"
            style={styles.input}
          />
        </label>
        {error ? <div role="alert" style={styles.error}>{error}</div> : null}
        <button type="submit" disabled={busy} style={styles.button}>
          {busy ? "Signing in..." : "Sign in"}
        </button>
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
    width: "min(360px, calc(100vw - 32px))",
    display: "grid",
    gap: 14,
    padding: 24,
    border: "1px solid #d9e2ec",
    borderRadius: 8,
    background: "#ffffff",
  },
  title: {
    margin: 0,
    fontSize: 24,
  },
  label: {
    display: "grid",
    gap: 6,
    fontSize: 13,
    fontWeight: 600,
  },
  input: {
    boxSizing: "border-box" as const,
    width: "100%",
    border: "1px solid #bcccdc",
    borderRadius: 6,
    padding: "9px 10px",
    font: "inherit",
  },
  button: {
    border: 0,
    borderRadius: 6,
    padding: "10px 12px",
    background: "#2563eb",
    color: "#ffffff",
    font: "inherit",
    fontWeight: 700,
  },
  error: {
    border: "1px solid #fca5a5",
    borderRadius: 6,
    padding: "8px 10px",
    background: "#fef2f2",
    color: "#991b1b",
    fontSize: 13,
  },
};
