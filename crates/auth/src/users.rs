//! SQLite-backed local user store.

use std::path::Path;
use std::str::FromStr;
use std::sync::{Arc, Mutex};

use base64::Engine;
use bcrypt::{DEFAULT_COST, hash, verify};
use rand::RngCore;
use rusqlite::{Connection, OptionalExtension, params};
use uuid::Uuid;

use crate::roles::Role;

/// User returned by the local user store.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct User {
    /// Stable user id.
    pub id: String,
    /// Login username.
    pub username: String,
    /// Assigned roles.
    pub roles: Vec<Role>,
}

/// User mutation payload.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UserPatch {
    /// Login username.
    pub username: String,
    /// Optional new plaintext password.
    pub password: Option<String>,
    /// Assigned roles.
    pub roles: Vec<Role>,
}

/// Result of first-run admin bootstrap.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BootstrapAdmin {
    /// Admin user.
    pub user: User,
    /// Generated password to show exactly once, or `None` when the user already existed.
    pub generated_password: Option<String>,
}

/// SQLite-backed local user store.
#[derive(Clone)]
pub struct UserStore {
    conn: Arc<Mutex<Connection>>,
}

impl UserStore {
    /// Open or create a user store at `path`.
    pub fn open(path: impl AsRef<Path>) -> Result<Self, UserError> {
        let conn = Connection::open(path)?;
        let store = Self {
            conn: Arc::new(Mutex::new(conn)),
        };
        store.init()?;
        Ok(store)
    }

    /// Open an in-memory user store.
    pub fn memory() -> Result<Self, UserError> {
        let conn = Connection::open_in_memory()?;
        let store = Self {
            conn: Arc::new(Mutex::new(conn)),
        };
        store.init()?;
        Ok(store)
    }

    /// Create a first-run admin if no users exist.
    pub fn bootstrap_admin(
        &self,
        override_password: Option<&str>,
    ) -> Result<BootstrapAdmin, UserError> {
        if let Some(user) = self.find_by_username("admin")? {
            return Ok(BootstrapAdmin {
                user,
                generated_password: None,
            });
        }
        if self.user_count()? > 0 {
            return Err(UserError::AdminMissing);
        }

        let password = override_password
            .map(ToOwned::to_owned)
            .unwrap_or_else(generate_password);
        let user = self.create_user("admin", &password, vec![Role::Administrator])?;
        Ok(BootstrapAdmin {
            user,
            generated_password: override_password.is_none().then_some(password),
        })
    }

    /// Create a user.
    pub fn create_user(
        &self,
        username: &str,
        password: &str,
        roles: Vec<Role>,
    ) -> Result<User, UserError> {
        validate_username(username)?;
        validate_roles(&roles)?;
        let id = Uuid::new_v4().to_string();
        let password_hash = hash(password, DEFAULT_COST)?;
        let roles_json = serde_json::to_string(&roles)?;
        self.lock()?.execute(
            "INSERT INTO users (id, username, password_hash, roles)
             VALUES (?1, ?2, ?3, ?4)",
            params![id, username, password_hash, roles_json],
        )?;
        Ok(User {
            id,
            username: username.to_string(),
            roles,
        })
    }

    /// Upsert a user by username.
    pub fn upsert_user(&self, patch: UserPatch) -> Result<User, UserError> {
        validate_username(&patch.username)?;
        validate_roles(&patch.roles)?;
        if let Some(existing) = self.find_by_username(&patch.username)? {
            let roles_json = serde_json::to_string(&patch.roles)?;
            if let Some(password) = patch.password {
                let password_hash = hash(password, DEFAULT_COST)?;
                self.lock()?.execute(
                    "UPDATE users SET password_hash = ?1, roles = ?2 WHERE id = ?3",
                    params![password_hash, roles_json, existing.id],
                )?;
            } else {
                self.lock()?.execute(
                    "UPDATE users SET roles = ?1 WHERE id = ?2",
                    params![roles_json, existing.id],
                )?;
            }
            Ok(User {
                roles: patch.roles,
                ..existing
            })
        } else {
            let Some(password) = patch.password else {
                return Err(UserError::PasswordRequired);
            };
            self.create_user(&patch.username, &password, patch.roles)
        }
    }

    /// Delete a user by id.
    pub fn delete_user(&self, user_id: &str) -> Result<(), UserError> {
        self.lock()?
            .execute("DELETE FROM users WHERE id = ?1", params![user_id])?;
        Ok(())
    }

    /// List users ordered by username.
    pub fn list_users(&self) -> Result<Vec<User>, UserError> {
        let conn = self.lock()?;
        let mut stmt = conn.prepare("SELECT id, username, roles FROM users ORDER BY username")?;
        let rows = stmt.query_map([], |row| {
            let roles_json: String = row.get(2)?;
            let roles = parse_roles_json(&roles_json)?;
            Ok(User {
                id: row.get(0)?,
                username: row.get(1)?,
                roles,
            })
        })?;
        rows.collect::<Result<Vec<_>, _>>().map_err(Into::into)
    }

    /// Authenticate a username and password.
    pub fn authenticate(&self, username: &str, password: &str) -> Result<Option<User>, UserError> {
        let Some((user, password_hash)) = self.find_with_hash(username)? else {
            return Ok(None);
        };
        if verify(password, &password_hash)? {
            Ok(Some(user))
        } else {
            Ok(None)
        }
    }

    /// Find a user by username.
    pub fn find_by_username(&self, username: &str) -> Result<Option<User>, UserError> {
        self.find_with_hash(username)
            .map(|user| user.map(|(user, _)| user))
    }

    fn init(&self) -> Result<(), UserError> {
        self.lock()?.execute_batch(
            "CREATE TABLE IF NOT EXISTS users (
                id TEXT PRIMARY KEY,
                username TEXT UNIQUE NOT NULL,
                password_hash TEXT NOT NULL,
                roles TEXT NOT NULL
            );",
        )?;
        Ok(())
    }

    fn user_count(&self) -> Result<u64, UserError> {
        let count: i64 = self
            .lock()?
            .query_row("SELECT COUNT(*) FROM users", [], |row| row.get(0))?;
        Ok(count.max(0) as u64)
    }

    fn find_with_hash(&self, username: &str) -> Result<Option<(User, String)>, UserError> {
        self.lock()?
            .query_row(
                "SELECT id, username, password_hash, roles FROM users WHERE username = ?1",
                params![username],
                |row| {
                    let roles_json: String = row.get(3)?;
                    let roles = parse_roles_json(&roles_json)?;
                    Ok((
                        User {
                            id: row.get(0)?,
                            username: row.get(1)?,
                            roles,
                        },
                        row.get(2)?,
                    ))
                },
            )
            .optional()
            .map_err(Into::into)
    }

    fn lock(&self) -> Result<std::sync::MutexGuard<'_, Connection>, UserError> {
        self.conn.lock().map_err(|_| UserError::Poisoned)
    }
}

/// User-store error.
#[derive(Debug, thiserror::Error)]
pub enum UserError {
    /// SQLite error.
    #[error(transparent)]
    Sqlite(#[from] rusqlite::Error),
    /// Bcrypt error.
    #[error(transparent)]
    Bcrypt(#[from] bcrypt::BcryptError),
    /// Serde error.
    #[error(transparent)]
    Serde(#[from] serde_json::Error),
    /// The first user store has users but no admin account.
    #[error("user store contains users but no admin account")]
    AdminMissing,
    /// New users require a password.
    #[error("password is required for a new user")]
    PasswordRequired,
    /// Username is invalid.
    #[error("username must be 1-64 ASCII alphanumeric, dot, underscore, or hyphen characters")]
    InvalidUsername,
    /// Roles list is empty.
    #[error("at least one role is required")]
    EmptyRoles,
    /// SQLite lock is poisoned.
    #[error("user-store sqlite lock poisoned")]
    Poisoned,
}

fn parse_roles_json(json: &str) -> Result<Vec<Role>, rusqlite::Error> {
    serde_json::from_str(json).map_err(|err| rusqlite::Error::ToSqlConversionFailure(Box::new(err)))
}

fn validate_username(username: &str) -> Result<(), UserError> {
    let valid = !username.is_empty()
        && username.len() <= 64
        && username
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '.' | '_' | '-'));
    if valid {
        Ok(())
    } else {
        Err(UserError::InvalidUsername)
    }
}

fn validate_roles(roles: &[Role]) -> Result<(), UserError> {
    if roles.is_empty() {
        Err(UserError::EmptyRoles)
    } else {
        Ok(())
    }
}

fn generate_password() -> String {
    let mut bytes = [0_u8; 18];
    rand::thread_rng().fill_bytes(&mut bytes);
    base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(bytes)
}

impl FromStr for User {
    type Err = UserError;

    fn from_str(_s: &str) -> Result<Self, Self::Err> {
        Err(UserError::InvalidUsername)
    }
}
