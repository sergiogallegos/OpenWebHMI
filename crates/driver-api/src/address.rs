//! Driver-specific tag addressing and browse nodes.

use serde::{Deserialize, Serialize};

/// Driver-specific tag address syntax.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TagAddress {
    /// Raw driver-specific address, for example `Line1.MotorRPM`.
    pub raw: String,
}

impl TagAddress {
    /// Create a new raw tag address.
    pub fn new(raw: impl Into<String>) -> Self {
        Self { raw: raw.into() }
    }
}

/// A node returned by driver browse operations.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TagNode {
    /// Display name of this node.
    pub name: String,
    /// Address for leaf nodes. Folders use `None`.
    pub address: Option<TagAddress>,
    /// Driver-specific data type, for example `DINT`, `REAL`, or `UDT:Recipe`.
    pub data_type: Option<String>,
    /// Child nodes below this node.
    pub children: Vec<TagNode>,
}
