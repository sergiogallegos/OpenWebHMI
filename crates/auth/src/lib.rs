//! Authentication and authorization primitives for OpenWebHMI.

#![deny(missing_docs)]

/// Per-view ACL evaluation.
pub mod acl;
/// Role and permission model.
pub mod roles;
/// JWT session issuing and verification.
pub mod sessions;
/// SQLite-backed local user store.
pub mod users;

pub use acl::{can_write_in_view, ViewAcl};
pub use roles::{Permission, Role};
pub use sessions::{SessionClaims, SessionError, SessionManager, VerifiedSession};
pub use users::{BootstrapAdmin, User, UserError, UserPatch, UserStore};
