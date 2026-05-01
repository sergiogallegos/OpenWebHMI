//! OPC UA NodeId address parsing.

use std::str::FromStr;

use opcua::types::{ByteString, Guid, NodeId};
use thiserror::Error;

/// OPC UA NodeId identifier form.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NodeIdForm {
    /// String NodeId (`s=`).
    String,
    /// Numeric NodeId (`i=`).
    Numeric,
    /// GUID NodeId (`g=`).
    Guid,
    /// Opaque byte-string NodeId (`b=`).
    Opaque,
}

/// Parsed OPC UA node address.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OpcUaAddress {
    /// Namespace index.
    pub namespace: u16,
    /// Identifier form.
    pub form: NodeIdForm,
    /// Identifier payload as written by the user.
    pub id: String,
}

impl OpcUaAddress {
    /// Parse `ns=<index>;<form>=<id>`.
    pub fn parse(raw: &str) -> Result<Self, OpcUaAddressError> {
        raw.parse()
    }

    /// Convert to an OPC UA crate NodeId.
    pub fn to_node_id(&self) -> Result<NodeId, OpcUaAddressError> {
        match self.form {
            NodeIdForm::String => Ok(NodeId::new(self.namespace, self.id.clone())),
            NodeIdForm::Numeric => Ok(NodeId::new(
                self.namespace,
                self.id
                    .parse::<u32>()
                    .map_err(|_| OpcUaAddressError::InvalidIdentifier(self.id.clone()))?,
            )),
            NodeIdForm::Guid => Ok(NodeId::new(
                self.namespace,
                self.id
                    .parse::<Guid>()
                    .map_err(|_| OpcUaAddressError::InvalidIdentifier(self.id.clone()))?,
            )),
            NodeIdForm::Opaque => Ok(NodeId::new(
                self.namespace,
                ByteString::from_base64(&self.id)
                    .ok_or_else(|| OpcUaAddressError::InvalidIdentifier(self.id.clone()))?,
            )),
        }
    }
}

impl FromStr for OpcUaAddress {
    type Err = OpcUaAddressError;

    fn from_str(raw: &str) -> Result<Self, Self::Err> {
        let (ns_part, id_part) = raw
            .split_once(';')
            .ok_or_else(|| OpcUaAddressError::InvalidShape(raw.to_string()))?;
        let namespace = ns_part
            .strip_prefix("ns=")
            .ok_or_else(|| OpcUaAddressError::InvalidShape(raw.to_string()))?
            .parse::<u16>()
            .map_err(|_| OpcUaAddressError::InvalidNamespace(ns_part.to_string()))?;
        let (form, id) = id_part
            .split_once('=')
            .ok_or_else(|| OpcUaAddressError::InvalidShape(raw.to_string()))?;
        if id.contains('=') || id.is_empty() {
            return Err(OpcUaAddressError::InvalidIdentifier(id.to_string()));
        }
        let form = match form {
            "s" => NodeIdForm::String,
            "i" => NodeIdForm::Numeric,
            "g" => NodeIdForm::Guid,
            "b" => NodeIdForm::Opaque,
            other => return Err(OpcUaAddressError::UnknownForm(other.to_string())),
        };

        Ok(Self {
            namespace,
            form,
            id: id.to_string(),
        })
    }
}

/// OPC UA address parse failure.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum OpcUaAddressError {
    /// Address did not match `ns=<index>;<form>=<id>`.
    #[error("invalid OPC UA address shape: {0}")]
    InvalidShape(String),
    /// Namespace index is invalid.
    #[error("invalid OPC UA namespace index: {0}")]
    InvalidNamespace(String),
    /// Identifier form is unknown.
    #[error("unknown OPC UA NodeId form: {0}")]
    UnknownForm(String),
    /// Identifier payload is invalid for its form.
    #[error("invalid OPC UA NodeId identifier: {0}")]
    InvalidIdentifier(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_all_node_id_forms() {
        assert_eq!(
            OpcUaAddress::parse("ns=2;s=Pressure").unwrap().form,
            NodeIdForm::String
        );
        assert_eq!(
            OpcUaAddress::parse("ns=4;i=1234").unwrap().form,
            NodeIdForm::Numeric
        );
        assert_eq!(
            OpcUaAddress::parse("ns=1;g=00000000-0000-0000-0000-000000000001")
                .unwrap()
                .form,
            NodeIdForm::Guid
        );
        assert_eq!(
            OpcUaAddress::parse("ns=1;b=AQID").unwrap().form,
            NodeIdForm::Opaque
        );
    }

    #[test]
    fn rejects_malformed_addresses() {
        assert!(OpcUaAddress::parse("2;s=Pressure").is_err());
        assert!(OpcUaAddress::parse("ns=x;s=Pressure").is_err());
        assert!(OpcUaAddress::parse("ns=2;x=Pressure").is_err());
        assert!(OpcUaAddress::parse("ns=2;s=").is_err());
        assert!(OpcUaAddress::parse("ns=2;s=a=b").is_err());
    }
}
