//! ADS connection configuration.

use std::net::{IpAddr, Ipv4Addr};

use openwebhmi_driver_api::{DriverError, DriverResult};
use serde::{Deserialize, Serialize};

/// Source AMS address selection.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum SourceAms {
    /// Let the `ads` crate derive source NetId from the local IPv4 address.
    #[default]
    Auto,
    /// Ask the local TwinCAT AMS router for a source port.
    Request,
    /// Use an explicit AMS NetId and optional source port.
    Explicit {
        /// Source AMS NetId, six dot-separated octets.
        net_id: String,
        /// Source AMS port. Defaults to an ephemeral client port.
        #[serde(default)]
        port: Option<u16>,
    },
}

/// Beckhoff ADS driver configuration.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AdsConfig {
    /// Target host or IP address used for ADS-over-TCP.
    pub host: String,
    /// TCP port for ADS-over-TCP. Defaults to `48898`.
    #[serde(default = "default_tcp_port")]
    pub tcp_port: u16,
    /// Target AMS NetId, six dot-separated octets such as `192.168.1.10.1.1`.
    pub ams_net_id: String,
    /// Source AMS address policy.
    #[serde(default)]
    pub source: SourceAms,
    /// ADS runtime ports to browse on connect. TwinCAT 3 PLC runtime 1 is `851`.
    #[serde(default = "default_ports")]
    pub ports: Vec<u16>,
    /// Operation timeout in milliseconds.
    #[serde(default = "default_timeout_ms")]
    pub timeout_ms: u64,
    /// Notification and fallback poll cycle in milliseconds.
    #[serde(default = "default_poll_rate_ms")]
    pub poll_rate_ms: u64,
}

impl AdsConfig {
    /// Decode config from driver JSON.
    pub fn from_value(value: serde_json::Value) -> DriverResult<Self> {
        let config: Self = serde_json::from_value(value)
            .map_err(|err| DriverError::Other(anyhow::anyhow!("invalid ADS config: {err}")))?;
        parse_ams_net_id(&config.ams_net_id)?;
        if config.ports.is_empty() {
            return Err(DriverError::InvalidAddress(
                "ADS config must include at least one port".to_string(),
            ));
        }
        Ok(config)
    }
}

impl Default for AdsConfig {
    fn default() -> Self {
        Self {
            host: "127.0.0.1".to_string(),
            tcp_port: default_tcp_port(),
            ams_net_id: "127.0.0.1.1.1".to_string(),
            source: SourceAms::Request,
            ports: default_ports(),
            timeout_ms: default_timeout_ms(),
            poll_rate_ms: default_poll_rate_ms(),
        }
    }
}

/// Parse a six-octet AMS NetId.
pub fn parse_ams_net_id(raw: &str) -> DriverResult<[u8; 6]> {
    let mut octets = [0_u8; 6];
    let parts = raw.split('.').collect::<Vec<_>>();
    if parts.len() != 6 {
        return Err(DriverError::InvalidAddress(format!(
            "AMS NetId must have six octets: {raw}"
        )));
    }
    for (index, part) in parts.iter().enumerate() {
        octets[index] = part.parse::<u8>().map_err(|_| {
            DriverError::InvalidAddress(format!("invalid AMS NetId octet '{part}' in {raw}"))
        })?;
    }
    Ok(octets)
}

/// Derive the common TwinCAT AMS NetId convention from an IPv4 address.
pub fn net_id_from_ipv4(ip: Ipv4Addr) -> String {
    let octets = ip.octets();
    format!(
        "{}.{}.{}.{}.1.1",
        octets[0], octets[1], octets[2], octets[3]
    )
}

/// Return an IPv4-derived AMS NetId when `host` is an IPv4 literal.
pub fn net_id_from_host_ipv4(host: &str) -> Option<String> {
    host.parse::<IpAddr>().ok().and_then(|ip| match ip {
        IpAddr::V4(ipv4) => Some(net_id_from_ipv4(ipv4)),
        IpAddr::V6(_) => None,
    })
}

const fn default_tcp_port() -> u16 {
    ads::PORT
}

fn default_ports() -> Vec<u16> {
    vec![851]
}

const fn default_timeout_ms() -> u64 {
    5_000
}

const fn default_poll_rate_ms() -> u64 {
    250
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_six_octet_net_id() {
        assert_eq!(
            parse_ams_net_id("192.168.1.10.1.1").unwrap(),
            [192, 168, 1, 10, 1, 1]
        );
    }

    #[test]
    fn rejects_four_octet_ip_as_net_id() {
        assert!(parse_ams_net_id("192.168.1.10").is_err());
    }

    #[test]
    fn derives_common_ipv4_net_id() {
        assert_eq!(
            net_id_from_ipv4(Ipv4Addr::new(192, 168, 1, 10)),
            "192.168.1.10.1.1"
        );
    }

    #[test]
    fn defaults_to_first_plc_runtime_port() {
        let config = AdsConfig::from_value(serde_json::json!({
            "host": "127.0.0.1",
            "ams_net_id": "127.0.0.1.1.1"
        }))
        .unwrap();

        assert_eq!(config.ports, vec![851]);
    }
}
