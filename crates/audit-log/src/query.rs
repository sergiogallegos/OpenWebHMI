//! Audit log query filters.

use serde::{Deserialize, Serialize};

/// Audit query filter and pagination request.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuditQuery {
    /// Inclusive lower timestamp bound.
    pub from_ts_ms: Option<u64>,
    /// Inclusive upper timestamp bound.
    pub to_ts_ms: Option<u64>,
    /// Actor username filter.
    pub user: Option<String>,
    /// Event kinds; empty means all.
    #[serde(default)]
    pub kinds: Vec<String>,
    /// Result limit, capped by the store.
    pub limit: usize,
    /// Offset into ordered results.
    pub offset: usize,
}

impl Default for AuditQuery {
    fn default() -> Self {
        Self {
            from_ts_ms: None,
            to_ts_ms: None,
            user: None,
            kinds: Vec::new(),
            limit: 100,
            offset: 0,
        }
    }
}
