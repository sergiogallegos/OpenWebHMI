//! ADS address parser.

use std::fmt;
use std::str::FromStr;

use thiserror::Error;

/// Parsed OpenWebHMI ADS address.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdsAddress {
    /// Target ADS port, such as `851` for the first TwinCAT 3 PLC runtime.
    pub port: u16,
    /// TwinCAT symbol path, preserving the case as entered.
    pub symbol: String,
}

impl AdsAddress {
    /// Parse `<port>:<symbol>`.
    pub fn parse(raw: &str) -> Result<Self, AdsAddressError> {
        raw.parse()
    }
}

impl FromStr for AdsAddress {
    type Err = AdsAddressError;

    fn from_str(raw: &str) -> Result<Self, Self::Err> {
        let (port_raw, symbol_raw) = raw
            .split_once(':')
            .ok_or(AdsAddressError::MissingSeparator)?;
        let port = port_raw
            .parse::<u16>()
            .map_err(|_| AdsAddressError::InvalidPort(port_raw.to_string()))?;
        if symbol_raw.is_empty() {
            return Err(AdsAddressError::EmptySymbol);
        }
        if !symbol_raw
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '_' | '.' | '[' | ']'))
        {
            return Err(AdsAddressError::InvalidSymbol(symbol_raw.to_string()));
        }
        Ok(Self {
            port,
            symbol: symbol_raw.to_string(),
        })
    }
}

impl fmt::Display for AdsAddress {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.port, self.symbol)
    }
}

/// ADS address parse failure.
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum AdsAddressError {
    /// Missing `:` between port and symbol.
    #[error("ADS address must be '<port>:<symbol>'")]
    MissingSeparator,
    /// ADS port is not a valid unsigned 16-bit integer.
    #[error("invalid ADS port '{0}'")]
    InvalidPort(String),
    /// Symbol path is empty.
    #[error("ADS symbol must not be empty")]
    EmptySymbol,
    /// Symbol path contains characters outside the v1 symbol grammar.
    #[error("invalid ADS symbol '{0}'")]
    InvalidSymbol(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_port_and_symbol() {
        let address = AdsAddress::parse("851:MAIN.fbMotor.fActualSpeed").unwrap();
        assert_eq!(address.port, 851);
        assert_eq!(address.symbol, "MAIN.fbMotor.fActualSpeed");
    }

    #[test]
    fn accepts_array_indices() {
        let address = AdsAddress::parse("851:GVL.Values[3]").unwrap();
        assert_eq!(address.symbol, "GVL.Values[3]");
    }

    #[test]
    fn rejects_malformed_addresses() {
        assert!(AdsAddress::parse("MAIN.x").is_err());
        assert!(AdsAddress::parse("99999:MAIN.x").is_err());
        assert!(AdsAddress::parse("851:").is_err());
        assert!(AdsAddress::parse("851:MAIN x").is_err());
    }
}
