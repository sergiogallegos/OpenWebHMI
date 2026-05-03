//! ADS driver implementation.

use std::collections::BTreeMap;
use std::sync::{Arc, Mutex as StdMutex};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use async_trait::async_trait;
use futures_util::Stream;
use futures_util::stream::{self, BoxStream};
use openwebhmi_driver_api::{
    Capabilities, Driver, DriverError, DriverMetadata, DriverResult, DriverUpdate, TagAddress,
    TagNode, make_metadata,
};
use openwebhmi_protocol::{Quality, TagValue};
use tokio::sync::mpsc;
use tokio_stream::wrappers::UnboundedReceiverStream;

use crate::address::AdsAddress;
use crate::connection::{AdsBackend, AdsConfig, SourceAms, parse_ams_net_id};
use crate::symbols::{
    AdsDataType, SymbolEntry, SymbolTable, decode_ads_value, encode_ads_value, table_from_ads,
};

type SharedClient = Arc<dyn AdsClientLike>;

#[async_trait]
pub(crate) trait AdsClientLike: Send + Sync {
    async fn symbol_table(&self, port: u16) -> DriverResult<SymbolTable>;
    async fn read_symbol(&self, port: u16, symbol: &str, size: usize) -> DriverResult<Vec<u8>>;
    async fn write_symbol(&self, port: u16, symbol: &str, data: Vec<u8>) -> DriverResult<()>;
    async fn subscribe_symbols(
        &self,
        entries: Vec<SubscriptionEntry>,
        poll_rate_ms: u64,
        updates: mpsc::UnboundedSender<DriverUpdate>,
    ) -> DriverResult<SubscriptionGuard>;
}

struct RealAdsClient {
    client: Arc<StdMutex<ads::Client>>,
    target_net_id: ads::AmsNetId,
}

impl RealAdsClient {
    fn connect(config: AdsConfig) -> DriverResult<Self> {
        let target_net_id = ads::AmsNetId(parse_ams_net_id(&config.ams_net_id)?);
        let source = source_from_config(&config.source)?;
        let timeouts = ads::Timeouts::new(Duration::from_millis(config.timeout_ms));
        let endpoint = (config.host.as_str(), config.tcp_port);
        let client = ads::Client::new(endpoint, timeouts, source).map_err(map_ads_error)?;
        Ok(Self {
            client: Arc::new(StdMutex::new(client)),
            target_net_id,
        })
    }

    fn target(&self, port: u16) -> ads::AmsAddr {
        ads::AmsAddr::new(self.target_net_id, port)
    }
}

#[async_trait]
impl AdsClientLike for RealAdsClient {
    async fn symbol_table(&self, port: u16) -> DriverResult<SymbolTable> {
        let client = self.client.clone();
        let target = self.target(port);
        spawn_blocking_driver(move || {
            let guard = client
                .lock()
                .map_err(|_| DriverError::Other(anyhow::anyhow!("ADS client mutex poisoned")))?;
            let device = guard.device(target);
            let (symbols, _types) = ads::symbol::get_symbol_info(device).map_err(map_ads_error)?;
            Ok(table_from_ads(symbols))
        })
        .await
    }

    async fn read_symbol(&self, port: u16, symbol: &str, size: usize) -> DriverResult<Vec<u8>> {
        let client = self.client.clone();
        let target = self.target(port);
        let symbol = symbol.to_string();
        spawn_blocking_driver(move || {
            let guard = client
                .lock()
                .map_err(|_| DriverError::Other(anyhow::anyhow!("ADS client mutex poisoned")))?;
            let device = guard.device(target);
            let handle = ads::Handle::new(device, &symbol).map_err(map_ads_error)?;
            let mut bytes = vec![0; size];
            handle.read(&mut bytes).map_err(map_ads_error)?;
            Ok(bytes)
        })
        .await
    }

    async fn write_symbol(&self, port: u16, symbol: &str, data: Vec<u8>) -> DriverResult<()> {
        let client = self.client.clone();
        let target = self.target(port);
        let symbol = symbol.to_string();
        spawn_blocking_driver(move || {
            let guard = client
                .lock()
                .map_err(|_| DriverError::Other(anyhow::anyhow!("ADS client mutex poisoned")))?;
            let device = guard.device(target);
            let handle = ads::Handle::new(device, &symbol).map_err(map_ads_error)?;
            handle.write(&data).map_err(map_ads_error)
        })
        .await
    }

    async fn subscribe_symbols(
        &self,
        entries: Vec<SubscriptionEntry>,
        poll_rate_ms: u64,
        updates: mpsc::UnboundedSender<DriverUpdate>,
    ) -> DriverResult<SubscriptionGuard> {
        let client = self.client.clone();
        let target_net_id = self.target_net_id;
        spawn_blocking_driver(move || {
            let mut entry_map = BTreeMap::new();
            let mut handles = Vec::new();
            let receiver = {
                let guard = client.lock().map_err(|_| {
                    DriverError::Other(anyhow::anyhow!("ADS client mutex poisoned"))
                })?;
                let receiver = guard.get_notification_channel();
                for entry in &entries {
                    let device = guard.device(ads::AmsAddr::new(target_net_id, entry.port));
                    let attributes = ads::notif::Attributes::new(
                        entry.size,
                        ads::notif::TransmissionMode::ServerOnChange,
                        Duration::from_millis(poll_rate_ms),
                        Duration::from_millis(poll_rate_ms),
                    );
                    let handle = device
                        .add_notification(entry.index_group, entry.index_offset, &attributes)
                        .map_err(map_ads_error)?;
                    handles.push((entry.port, handle));
                    entry_map.insert(handle, entry.clone());
                }
                receiver
            };

            std::thread::spawn(move || {
                while let Ok(notification) = receiver.recv() {
                    for sample in notification.samples() {
                        let Some(entry) = entry_map.get(&sample.handle) else {
                            continue;
                        };
                        let result = decode_ads_value(entry.data_type, sample.data);
                        let update = update_from_result(entry.raw.clone(), result, now_ms());
                        if updates.send(update).is_err() {
                            return;
                        }
                    }
                }
            });

            Ok(SubscriptionGuard {
                client: Some(client),
                target_net_id,
                handles,
                backend_guard: None,
            })
        })
        .await
    }
}

/// Beckhoff TwinCAT ADS client driver.
#[derive(Default)]
pub struct AdsDriver {
    client: Option<SharedClient>,
    config: Option<AdsConfig>,
    symbols_by_port: BTreeMap<u16, SymbolTable>,
}

impl AdsDriver {
    /// Construct a disconnected ADS driver.
    pub fn new() -> Self {
        Self::default()
    }

    #[cfg(test)]
    fn with_client(client: Arc<dyn AdsClientLike>, config: AdsConfig, table: SymbolTable) -> Self {
        Self {
            client: Some(client),
            config: Some(config),
            symbols_by_port: BTreeMap::from([(851, table)]),
        }
    }

    fn client(&self) -> DriverResult<SharedClient> {
        self.client.clone().ok_or(DriverError::NotConnected)
    }

    async fn resolve(&self, address: &TagAddress) -> DriverResult<ResolvedSymbol> {
        let address = parse_address(address)?;
        let table = self.symbols_by_port.get(&address.port).ok_or_else(|| {
            DriverError::InvalidAddress(format!("ADS port {} was not browsed", address.port))
        })?;
        let entry = table
            .get(&address.symbol)
            .ok_or_else(|| DriverError::InvalidAddress(address.symbol.clone()))?;
        let data_type = entry
            .data_type
            .ok_or_else(|| DriverError::UnsupportedType {
                device_type: entry.type_name.clone(),
            })?;
        Ok(ResolvedSymbol {
            port: address.port,
            symbol: address.symbol,
            entry: entry.clone(),
            data_type,
        })
    }
}

#[async_trait]
impl Driver for AdsDriver {
    fn metadata(&self) -> DriverMetadata {
        make_metadata(
            "Beckhoff",
            "TwinCAT ADS",
            env!("CARGO_PKG_VERSION"),
            Capabilities {
                native_subscribe: true,
                browse: true,
                batch_read: true,
                batch_write: true,
            },
        )
    }

    async fn connect(&mut self, config: serde_json::Value) -> DriverResult<()> {
        let config = AdsConfig::from_value(config)?;
        let client = connect_client(config.clone())?;
        let mut symbols_by_port = BTreeMap::new();
        for port in &config.ports {
            symbols_by_port.insert(*port, client.symbol_table(*port).await?);
        }
        self.symbols_by_port = symbols_by_port;
        self.client = Some(client);
        self.config = Some(config);
        Ok(())
    }

    async fn disconnect(&mut self) -> DriverResult<()> {
        self.client = None;
        self.symbols_by_port.clear();
        Ok(())
    }

    async fn browse(&self, path: Option<&str>) -> DriverResult<Vec<TagNode>> {
        Ok(self
            .symbols_by_port
            .iter()
            .flat_map(|(port, table)| {
                table.browse(path).into_iter().map(move |mut node| {
                    if let Some(address) = &node.address {
                        node.address = Some(TagAddress::new(format!("{port}:{}", address.raw)));
                    }
                    node
                })
            })
            .collect())
    }

    async fn read(&self, address: &TagAddress) -> DriverResult<TagValue> {
        let resolved = self.resolve(address).await?;
        let bytes = self
            .client()?
            .read_symbol(resolved.port, &resolved.symbol, resolved.entry.size)
            .await?;
        decode_ads_value(resolved.data_type, &bytes)
    }

    async fn write(&self, address: &TagAddress, value: TagValue) -> DriverResult<()> {
        let resolved = self.resolve(address).await?;
        let bytes = encode_ads_value(resolved.data_type, value, resolved.entry.size)?;
        self.client()?
            .write_symbol(resolved.port, &resolved.symbol, bytes)
            .await
    }

    async fn subscribe(
        &self,
        addresses: Vec<TagAddress>,
    ) -> DriverResult<BoxStream<'static, DriverUpdate>> {
        if addresses.is_empty() {
            return Ok(Box::pin(stream::empty()));
        }

        let mut entries = Vec::with_capacity(addresses.len());
        for address in addresses {
            let resolved = self.resolve(&address).await?;
            entries.push(SubscriptionEntry {
                raw: address,
                port: resolved.port,
                index_group: resolved.entry.index_group,
                index_offset: resolved.entry.index_offset,
                size: resolved.entry.size,
                data_type: resolved.data_type,
            });
        }

        let (tx, rx) = mpsc::unbounded_channel();
        let poll_rate_ms = self
            .config
            .as_ref()
            .map(|config| config.poll_rate_ms)
            .unwrap_or(250)
            .max(10);
        let guard = self
            .client()?
            .subscribe_symbols(entries, poll_rate_ms, tx)
            .await?;

        Ok(Box::pin(GuardedUpdateStream {
            inner: UnboundedReceiverStream::new(rx),
            _guard: guard,
        }))
    }
}

#[derive(Clone)]
pub(crate) struct SubscriptionEntry {
    pub(crate) raw: TagAddress,
    pub(crate) port: u16,
    pub(crate) index_group: u32,
    pub(crate) index_offset: u32,
    pub(crate) size: usize,
    pub(crate) data_type: AdsDataType,
}

pub(crate) struct SubscriptionGuard {
    client: Option<Arc<StdMutex<ads::Client>>>,
    target_net_id: ads::AmsNetId,
    handles: Vec<(u16, ads::notif::Handle)>,
    backend_guard: Option<Box<dyn Send + Sync>>,
}

impl SubscriptionGuard {
    pub(crate) fn backend(guard: impl Send + Sync + 'static) -> Self {
        Self {
            client: None,
            target_net_id: ads::AmsNetId([0; 6]),
            handles: Vec::new(),
            backend_guard: Some(Box::new(guard)),
        }
    }
}

impl Drop for SubscriptionGuard {
    fn drop(&mut self) {
        let _ = self.backend_guard.take();
        let Some(client) = self.client.take() else {
            return;
        };
        let target_net_id = self.target_net_id;
        let handles = std::mem::take(&mut self.handles);
        std::thread::spawn(move || {
            let Ok(guard) = client.lock() else {
                return;
            };
            for (port, handle) in handles {
                let device = guard.device(ads::AmsAddr::new(target_net_id, port));
                let _ = device.delete_notification(handle);
            }
        });
    }
}

struct GuardedUpdateStream {
    inner: UnboundedReceiverStream<DriverUpdate>,
    _guard: SubscriptionGuard,
}

impl Stream for GuardedUpdateStream {
    type Item = DriverUpdate;

    fn poll_next(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        std::pin::Pin::new(&mut self.inner).poll_next(cx)
    }
}

struct ResolvedSymbol {
    port: u16,
    symbol: String,
    entry: SymbolEntry,
    data_type: AdsDataType,
}

fn source_from_config(source: &SourceAms) -> DriverResult<ads::Source> {
    match source {
        SourceAms::Auto => Ok(ads::Source::Auto),
        SourceAms::Request => Ok(ads::Source::Request),
        SourceAms::Explicit { net_id, port } => Ok(ads::Source::Addr(ads::AmsAddr::new(
            ads::AmsNetId(parse_ams_net_id(net_id)?),
            port.unwrap_or(58_913),
        ))),
    }
}

fn connect_client(config: AdsConfig) -> DriverResult<SharedClient> {
    match config.backend {
        AdsBackend::AdsRsTcp => Ok(Arc::new(RealAdsClient::connect(config)?) as SharedClient),
        AdsBackend::TwincatRouter => connect_twin_cat_router(config),
        AdsBackend::Auto => connect_auto(config),
    }
}

fn connect_auto(config: AdsConfig) -> DriverResult<SharedClient> {
    #[cfg(windows)]
    {
        match connect_twin_cat_router(config.clone()) {
            Ok(client) => return Ok(client),
            Err(err) => {
                tracing::debug!(
                    "TwinCAT router ADS backend unavailable, falling back to ads-rs: {err}"
                );
            }
        }
    }

    Ok(Arc::new(RealAdsClient::connect(config)?) as SharedClient)
}

fn connect_twin_cat_router(config: AdsConfig) -> DriverResult<SharedClient> {
    #[cfg(windows)]
    {
        Ok(Arc::new(crate::twincat_router::TcAdsClient::connect(config)?) as SharedClient)
    }

    #[cfg(not(windows))]
    {
        let _ = config;
        Err(DriverError::UnsupportedType {
            device_type: "TwinCAT router backend requires Windows and Beckhoff TcAdsDll.dll"
                .to_string(),
        })
    }
}

pub(crate) async fn spawn_blocking_driver<T>(
    f: impl FnOnce() -> DriverResult<T> + Send + 'static,
) -> DriverResult<T>
where
    T: Send + 'static,
{
    tokio::task::spawn_blocking(f)
        .await
        .map_err(|err| DriverError::Other(anyhow::anyhow!("ADS blocking task failed: {err}")))?
}

fn parse_address(address: &TagAddress) -> DriverResult<AdsAddress> {
    AdsAddress::parse(&address.raw).map_err(|err| DriverError::InvalidAddress(err.to_string()))
}

fn map_ads_error(error: ads::Error) -> DriverError {
    let message = error.to_string();
    if message.to_ascii_lowercase().contains("symbol") {
        DriverError::InvalidAddress(message)
    } else {
        DriverError::RemoteFault {
            code: "ads".to_string(),
            message,
        }
    }
}

pub(crate) fn update_from_result(
    address: TagAddress,
    result: DriverResult<TagValue>,
    ts_ms: u64,
) -> DriverUpdate {
    match result {
        Ok(value) => DriverUpdate {
            address,
            value,
            quality: Quality::Good,
            ts_ms,
        },
        Err(error) => DriverUpdate {
            address,
            value: TagValue::String(error.to_string()),
            quality: error.into_quality(),
            ts_ms,
        },
    }
}

pub(crate) fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis() as u64)
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use std::collections::VecDeque;

    use super::*;

    #[derive(Default)]
    struct MockAdsClient {
        reads: StdMutex<VecDeque<Vec<u8>>>,
        writes: StdMutex<Vec<(u16, String, Vec<u8>)>>,
    }

    #[async_trait]
    impl AdsClientLike for MockAdsClient {
        async fn symbol_table(&self, _port: u16) -> DriverResult<SymbolTable> {
            Ok(test_table())
        }

        async fn read_symbol(
            &self,
            _port: u16,
            _symbol: &str,
            _size: usize,
        ) -> DriverResult<Vec<u8>> {
            Ok(self.reads.lock().unwrap().pop_front().unwrap_or_default())
        }

        async fn write_symbol(&self, port: u16, symbol: &str, data: Vec<u8>) -> DriverResult<()> {
            self.writes
                .lock()
                .unwrap()
                .push((port, symbol.to_string(), data));
            Ok(())
        }

        async fn subscribe_symbols(
            &self,
            entries: Vec<SubscriptionEntry>,
            _poll_rate_ms: u64,
            updates: mpsc::UnboundedSender<DriverUpdate>,
        ) -> DriverResult<SubscriptionGuard> {
            for entry in entries {
                let _ = updates.send(update_from_result(
                    entry.raw,
                    Ok(TagValue::Real(12.5)),
                    now_ms(),
                ));
            }
            Ok(SubscriptionGuard {
                client: None,
                target_net_id: ads::AmsNetId([0; 6]),
                handles: Vec::new(),
                backend_guard: None,
            })
        }
    }

    #[tokio::test]
    async fn reads_symbol_by_handle_path() {
        let client = Arc::new(MockAdsClient::default());
        client
            .reads
            .lock()
            .unwrap()
            .push_back(12.5_f32.to_le_bytes().to_vec());
        let driver = AdsDriver::with_client(client, AdsConfig::default(), test_table());

        let value = driver
            .read(&TagAddress::new("851:MAIN.Speed"))
            .await
            .unwrap();

        assert_eq!(value, TagValue::Real(12.5));
    }

    #[tokio::test]
    async fn writes_symbol_bytes() {
        let client = Arc::new(MockAdsClient::default());
        let driver = AdsDriver::with_client(client.clone(), AdsConfig::default(), test_table());

        driver
            .write(&TagAddress::new("851:MAIN.Speed"), TagValue::Real(10.0))
            .await
            .unwrap();

        let writes = client.writes.lock().unwrap();
        assert_eq!(writes.len(), 1);
        assert_eq!(writes[0].0, 851);
        assert_eq!(writes[0].1, "MAIN.Speed");
        assert_eq!(writes[0].2, 10.0_f32.to_le_bytes());
    }

    #[tokio::test]
    async fn subscribes_with_native_updates() {
        let client = Arc::new(MockAdsClient::default());
        let driver = AdsDriver::with_client(client, AdsConfig::default(), test_table());
        let mut stream = driver
            .subscribe(vec![TagAddress::new("851:MAIN.Speed")])
            .await
            .unwrap();

        let update = futures_util::StreamExt::next(&mut stream).await.unwrap();
        assert_eq!(update.address.raw, "851:MAIN.Speed");
        assert_eq!(update.quality, Quality::Good);
    }

    fn test_table() -> SymbolTable {
        SymbolTable::new([SymbolEntry {
            name: "MAIN.Speed".to_string(),
            index_group: 0x4020,
            index_offset: 0,
            type_name: "REAL".to_string(),
            size: 4,
            base_type: 4,
            data_type: Some(AdsDataType::Real),
        }])
    }
}
