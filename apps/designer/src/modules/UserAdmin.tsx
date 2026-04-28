import { useEffect, useState } from "react";
import type { AuthUser } from "@openwebhmi/protocol";
import type { DesignerClient } from "../lib/designerClient";

const ROLES = ["Administrator", "Designer", "Operator", "Viewer"];

type UserAdminProps = {
  client: DesignerClient;
};

export function UserAdmin({ client }: UserAdminProps) {
  const [users, setUsers] = useState<AuthUser[]>([]);
  const [username, setUsername] = useState("");
  const [password, setPassword] = useState("");
  const [roles, setRoles] = useState<string[]>(["Viewer"]);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    void client.listUsers().then(setUsers).catch((error: unknown) => {
      setError(error instanceof Error ? error.message : String(error));
    });
  }, [client]);

  const save = async () => {
    setError(null);
    try {
      setUsers(await client.upsertUser(username, password || null, roles));
      setUsername("");
      setPassword("");
      setRoles(["Viewer"]);
    } catch (error) {
      setError(error instanceof Error ? error.message : String(error));
    }
  };

  const remove = async (userId: string) => {
    setError(null);
    try {
      setUsers(await client.deleteUser(userId));
    } catch (error) {
      setError(error instanceof Error ? error.message : String(error));
    }
  };

  return (
    <section style={styles.panel} aria-label="User administration">
      <h2 style={styles.title}>Users</h2>
      <div style={styles.list}>
        {users.map((user) => (
          <div key={user.id} style={styles.userRow}>
            <div>
              <strong>{user.username}</strong>
              <div style={styles.roles}>{user.roles.join(", ")}</div>
            </div>
            <button type="button" style={styles.smallButton} onClick={() => remove(user.id)}>
              Delete
            </button>
          </div>
        ))}
      </div>
      <div style={styles.form}>
        <input
          aria-label="New username"
          placeholder="username"
          value={username}
          onChange={(event) => setUsername(event.currentTarget.value)}
          style={styles.input}
        />
        <input
          aria-label="New password"
          placeholder="password"
          type="password"
          value={password}
          onChange={(event) => setPassword(event.currentTarget.value)}
          style={styles.input}
        />
        <div style={styles.roleGrid}>
          {ROLES.map((role) => (
            <label key={role} style={styles.checkLabel}>
              <input
                type="checkbox"
                checked={roles.includes(role)}
                onChange={(event) => {
                  setRoles((current) =>
                    event.currentTarget.checked
                      ? [...current, role]
                      : current.filter((item) => item !== role),
                  );
                }}
              />
              {role}
            </label>
          ))}
        </div>
        <button
          type="button"
          disabled={!username || roles.length === 0}
          style={styles.button}
          onClick={() => void save()}
        >
          Save user
        </button>
      </div>
      {error ? <div role="alert" style={styles.error}>{error}</div> : null}
    </section>
  );
}

const styles = {
  panel: {
    borderTop: "1px solid #d9e2ec",
    padding: 12,
    background: "#ffffff",
  },
  title: {
    margin: "0 0 10px",
    fontSize: 14,
  },
  list: {
    display: "grid",
    gap: 6,
    marginBottom: 10,
  },
  userRow: {
    display: "flex",
    justifyContent: "space-between",
    gap: 8,
    alignItems: "center",
    padding: 8,
    border: "1px solid #d9e2ec",
    borderRadius: 6,
  },
  roles: {
    marginTop: 2,
    color: "#52606d",
    fontSize: 12,
  },
  form: {
    display: "grid",
    gap: 8,
  },
  input: {
    minHeight: 32,
    border: "1px solid #9aa5b1",
    borderRadius: 6,
    padding: "5px 8px",
    font: "inherit",
  },
  roleGrid: {
    display: "grid",
    gridTemplateColumns: "1fr 1fr",
    gap: 6,
    fontSize: 12,
  },
  checkLabel: {
    display: "flex",
    alignItems: "center",
    gap: 6,
  },
  button: {
    minHeight: 32,
    border: "1px solid #1f4e79",
    borderRadius: 6,
    background: "#1f4e79",
    color: "#ffffff",
    fontWeight: 700,
  },
  smallButton: {
    border: "1px solid #d9e2ec",
    borderRadius: 6,
    background: "#ffffff",
    padding: "4px 8px",
  },
  error: {
    marginTop: 8,
    color: "#991b1b",
    fontSize: 12,
  },
};
