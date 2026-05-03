//! SQLite-backed audit log for security-relevant OpenWebHMI events.

#![deny(missing_docs)]

mod event;
mod query;
mod store;

pub use event::{AuditEntry, AuditEvent, UserAdminAction, WriteSource};
pub use query::AuditQuery;
pub use store::AuditLog;
