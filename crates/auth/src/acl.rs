//! Per-view ACL evaluation.

use serde::{Deserialize, Serialize};

use crate::roles::{Permission, Role};

/// Per-view ACL. When `allowed_roles` is `None`, role defaults apply.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ViewAcl {
    /// Roles allowed to write tags from this view.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub allowed_roles: Option<Vec<Role>>,
}

/// Return true if any role permits a tag write in a view with `acl`.
pub fn can_write_in_view(roles: &[Role], acl: Option<&ViewAcl>) -> bool {
    let role_allows_write = roles
        .iter()
        .copied()
        .any(|role| role.allows(Permission::WriteTags));
    if !role_allows_write {
        return false;
    }

    match acl.and_then(|acl| acl.allowed_roles.as_ref()) {
        Some(allowed) => roles.iter().any(|role| allowed.contains(role)),
        None => true,
    }
}
