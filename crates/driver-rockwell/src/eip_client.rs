//! Internal abstraction over `rust-ethernet-ip` for unit tests.

use std::time::Duration;

use async_trait::async_trait;
use rust_ethernet_ip::{
    EipClient, EtherNetIpError, PlcValue, RoutePath, TagGroupEvent, TagGroupSubscription,
};
use tokio::time::timeout;

use crate::config::RockwellConfig;

/// Result alias for upstream EtherNet/IP operations.
pub type EipResult<T> = Result<T, EtherNetIpError>;

#[async_trait]
/// Minimal tag-group subscription surface used by the wrapper.
pub trait TagGroupSubscriptionLike: Send {
    /// Wait for the next upstream tag-group event.
    async fn wait_for_update(&self) -> Option<TagGroupEvent>;

    /// Stop the underlying subscription.
    fn stop(&self);
}

#[async_trait]
impl TagGroupSubscriptionLike for TagGroupSubscription {
    async fn wait_for_update(&self) -> Option<TagGroupEvent> {
        TagGroupSubscription::wait_for_update(self).await
    }

    fn stop(&self) {
        TagGroupSubscription::stop(self);
    }
}

#[async_trait]
/// Minimal upstream client surface used by `RockwellDriver`.
pub trait EipClientLike: Send + Sync {
    /// Read one tag from the PLC.
    async fn read_tag(&mut self, tag_name: &str) -> EipResult<PlcValue>;

    /// Write one tag to the PLC.
    async fn write_tag(&mut self, tag_name: &str, value: PlcValue) -> EipResult<()>;

    /// Register or update a polling tag group.
    async fn upsert_tag_group(
        &mut self,
        group_name: &str,
        tags: &[&str],
        update_rate_ms: u32,
    ) -> EipResult<()>;

    /// Subscribe to a previously registered tag group.
    async fn subscribe_tag_group(
        &mut self,
        group_name: &str,
    ) -> EipResult<Box<dyn TagGroupSubscriptionLike>>;
}

pub(crate) struct RealEipClient {
    inner: EipClient,
}

impl RealEipClient {
    pub(crate) async fn connect(config: &RockwellConfig) -> EipResult<Self> {
        let endpoint = config.endpoint();
        let route = parse_route(config)?;
        let connect = async {
            match route {
                Some(route) => EipClient::with_route_path(&endpoint, route).await,
                None => EipClient::connect(&endpoint).await,
            }
        };

        let inner = timeout(
            Duration::from_millis(config.connection_timeout_ms.into()),
            connect,
        )
        .await
        .map_err(|_| {
            EtherNetIpError::Timeout(Duration::from_millis(config.connection_timeout_ms.into()))
        })??;

        Ok(Self { inner })
    }
}

#[async_trait]
impl EipClientLike for RealEipClient {
    async fn read_tag(&mut self, tag_name: &str) -> EipResult<PlcValue> {
        self.inner.read_tag(tag_name).await
    }

    async fn write_tag(&mut self, tag_name: &str, value: PlcValue) -> EipResult<()> {
        self.inner.write_tag(tag_name, value).await
    }

    async fn upsert_tag_group(
        &mut self,
        group_name: &str,
        tags: &[&str],
        update_rate_ms: u32,
    ) -> EipResult<()> {
        self.inner
            .upsert_tag_group(group_name, tags, update_rate_ms)
            .await
    }

    async fn subscribe_tag_group(
        &mut self,
        group_name: &str,
    ) -> EipResult<Box<dyn TagGroupSubscriptionLike>> {
        Ok(Box::new(self.inner.subscribe_tag_group(group_name).await?))
    }
}

fn parse_route(config: &RockwellConfig) -> EipResult<Option<RoutePath>> {
    if let Some(route) = config
        .route
        .as_deref()
        .filter(|route| !route.trim().is_empty())
    {
        let parts = route
            .split(',')
            .map(|part| part.trim().parse::<u8>())
            .collect::<Result<Vec<_>, _>>()
            .map_err(|err| {
                EtherNetIpError::Protocol(format!("invalid route path '{}': {err}", route))
            })?;

        if parts.len() == 2 && parts[0] == 1 {
            return Ok(Some(RoutePath::new().add_slot(parts[1])));
        }

        return Err(EtherNetIpError::Protocol(format!(
            "unsupported route path '{}'; expected backplane form '1,<slot>'",
            route
        )));
    }

    if config.slot > 0 {
        Ok(Some(RoutePath::new().add_slot(config.slot)))
    } else {
        Ok(None)
    }
}
