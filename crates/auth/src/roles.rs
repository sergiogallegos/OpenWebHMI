//! Built-in roles and permissions.

use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

/// Built-in OpenWebHMI role.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum Role {
    /// Full system administrator.
    Administrator,
    /// Project author who cannot manage users.
    Designer,
    /// Runtime operator.
    Operator,
    /// Read-only runtime user.
    Viewer,
}

impl Role {
    /// Return true when this role grants `permission`.
    pub fn allows(self, permission: Permission) -> bool {
        match permission {
            Permission::ReadTags | Permission::ReadViews => true,
            Permission::WriteTags => {
                matches!(self, Role::Administrator | Role::Designer | Role::Operator)
            }
            Permission::AuthorProject => {
                matches!(self, Role::Administrator | Role::Designer)
            }
            Permission::ManageUsers => matches!(self, Role::Administrator),
        }
    }

    /// Stable role name used in JSON and JWT claims.
    pub fn as_str(self) -> &'static str {
        match self {
            Role::Administrator => "Administrator",
            Role::Designer => "Designer",
            Role::Operator => "Operator",
            Role::Viewer => "Viewer",
        }
    }
}

impl fmt::Display for Role {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for Role {
    type Err = RoleParseError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "Administrator" => Ok(Role::Administrator),
            "Designer" => Ok(Role::Designer),
            "Operator" => Ok(Role::Operator),
            "Viewer" => Ok(Role::Viewer),
            _ => Err(RoleParseError(value.to_string())),
        }
    }
}

/// A permission checked by the gateway.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Permission {
    /// Read live tag values.
    ReadTags,
    /// Write tag values.
    WriteTags,
    /// Read view definitions.
    ReadViews,
    /// Author project artifacts.
    AuthorProject,
    /// Manage local users.
    ManageUsers,
}

/// Error returned when parsing an unknown role string.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("unknown role '{0}'")]
pub struct RoleParseError(String);
