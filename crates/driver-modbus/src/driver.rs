//! Modbus driver implementation.

use std::collections::VecDeque;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use async_trait::async_trait;
use futures_util::stream::{self, BoxStream};
use openwebhmi_driver_api::{
    Capabilities, Driver, DriverError, DriverMetadata, DriverResult, DriverUpdate, TagAddress,
    TagNode, make_metadata,
};
use openwebhmi_protocol::{Quality, TagValue};
use tokio::sync::Mutex;
use tokio::time;
use tokio_modbus::ExceptionCode;
use tokio_modbus::client::{rtu, tcp};
use tokio_modbus::prelude::{Reader, Slave, SlaveContext, Writer};
use tokio_serial::{DataBits, FlowControl};

use crate::address::{Area, EncodedValue, ModbusAddress};
use crate::connection::{Connection, ModbusConfig, Parity, StopBits};

type SharedClient = Arc<Mutex<Box<dyn ModbusClientLike>>>;
type ModbusRequestResult<T> = Result<Result<T, ExceptionCode>, tokio_modbus::Error>;

#[async_trait]
pub(crate) trait ModbusClientLike: Send {
    async fn read_coils(&mut self, address: u16, quantity: u16) -> ModbusRequestResult<Vec<bool>>;
    async fn read_discrete_inputs(
        &mut self,
        address: u16,
        quantity: u16,
    ) -> ModbusRequestResult<Vec<bool>>;
    async fn read_holding_registers(
        &mut self,
        address: u16,
        quantity: u16,
    ) -> ModbusRequestResult<Vec<u16>>;
    async fn read_input_registers(
        &mut self,
        address: u16,
        quantity: u16,
    ) -> ModbusRequestResult<Vec<u16>>;
    async fn write_single_coil(&mut self, address: u16, value: bool) -> ModbusRequestResult<()>;
    async fn write_single_register(&mut self, address: u16, value: u16) -> ModbusRequestResult<()>;
    async fn write_multiple_coils(
        &mut self,
        address: u16,
        values: Vec<bool>,
    ) -> ModbusRequestResult<()>;
    async fn write_multiple_registers(
        &mut self,
        address: u16,
        values: Vec<u16>,
    ) -> ModbusRequestResult<()>;
    fn set_slave(&mut self, slave: Slave);
}

struct RealModbusClient {
    context: tokio_modbus::client::Context,
}

#[async_trait]
impl ModbusClientLike for RealModbusClient {
    async fn read_coils(&mut self, address: u16, quantity: u16) -> ModbusRequestResult<Vec<bool>> {
        self.context.read_coils(address, quantity).await
    }

    async fn read_discrete_inputs(
        &mut self,
        address: u16,
        quantity: u16,
    ) -> ModbusRequestResult<Vec<bool>> {
        self.context.read_discrete_inputs(address, quantity).await
    }

    async fn read_holding_registers(
        &mut self,
        address: u16,
        quantity: u16,
    ) -> ModbusRequestResult<Vec<u16>> {
        self.context.read_holding_registers(address, quantity).await
    }

    async fn read_input_registers(
        &mut self,
        address: u16,
        quantity: u16,
    ) -> ModbusRequestResult<Vec<u16>> {
        self.context.read_input_registers(address, quantity).await
    }

    async fn write_single_coil(&mut self, address: u16, value: bool) -> ModbusRequestResult<()> {
        self.context.write_single_coil(address, value).await
    }

    async fn write_single_register(&mut self, address: u16, value: u16) -> ModbusRequestResult<()> {
        self.context.write_single_register(address, value).await
    }

    async fn write_multiple_coils(
        &mut self,
        address: u16,
        values: Vec<bool>,
    ) -> ModbusRequestResult<()> {
        self.context.write_multiple_coils(address, &values).await
    }

    async fn write_multiple_registers(
        &mut self,
        address: u16,
        values: Vec<u16>,
    ) -> ModbusRequestResult<()> {
        self.context
            .write_multiple_registers(address, &values)
            .await
    }

    fn set_slave(&mut self, slave: Slave) {
        self.context.set_slave(slave);
    }
}

/// Modbus TCP/RTU client driver.
#[derive(Default)]
pub struct ModbusDriver {
    client: Option<SharedClient>,
    config: Option<ModbusConfig>,
}

impl ModbusDriver {
    /// Construct a disconnected Modbus driver.
    pub fn new() -> Self {
        Self::default()
    }

    #[cfg(test)]
    pub(crate) fn with_client(client: Box<dyn ModbusClientLike>, config: ModbusConfig) -> Self {
        Self {
            client: Some(Arc::new(Mutex::new(client))),
            config: Some(config),
        }
    }

    fn client(&self) -> DriverResult<SharedClient> {
        self.client.clone().ok_or(DriverError::NotConnected)
    }
}

#[async_trait]
impl Driver for ModbusDriver {
    fn metadata(&self) -> DriverMetadata {
        make_metadata(
            "Modbus",
            "Modbus TCP/RTU",
            env!("CARGO_PKG_VERSION"),
            Capabilities {
                native_subscribe: false,
                browse: false,
                batch_read: true,
                batch_write: true,
            },
        )
    }

    async fn connect(&mut self, config: serde_json::Value) -> DriverResult<()> {
        let config = ModbusConfig::from_value(config)
            .map_err(|err| DriverError::Other(anyhow::anyhow!("invalid Modbus config: {err}")))?;
        let client = connect_client(&config).await?;
        self.client = Some(Arc::new(Mutex::new(Box::new(client))));
        self.config = Some(config);
        Ok(())
    }

    async fn disconnect(&mut self) -> DriverResult<()> {
        self.client = None;
        Ok(())
    }

    async fn browse(&self, _path: Option<&str>) -> DriverResult<Vec<TagNode>> {
        Err(DriverError::UnsupportedType {
            device_type: "Modbus has no browse service".to_string(),
        })
    }

    async fn read(&self, address: &TagAddress) -> DriverResult<TagValue> {
        let address = parse_address(address)?;
        let client = self.client()?;
        let mut guard = client.lock().await;
        guard.set_slave(Slave(address.unit_id));

        match address.area {
            Area::Coils => {
                let bits = flatten(guard.read_coils(address.address, address.count).await)?;
                address.decode_bits(&bits)
            }
            Area::Discrete => {
                let bits = flatten(
                    guard
                        .read_discrete_inputs(address.address, address.count)
                        .await,
                )?;
                address.decode_bits(&bits)
            }
            Area::Holding => {
                let registers = flatten(
                    guard
                        .read_holding_registers(address.address, address.count)
                        .await,
                )?;
                address.decode_registers(&registers)
            }
            Area::Input => {
                let registers = flatten(
                    guard
                        .read_input_registers(address.address, address.count)
                        .await,
                )?;
                address.decode_registers(&registers)
            }
        }
    }

    async fn write(&self, address: &TagAddress, value: TagValue) -> DriverResult<()> {
        let address = parse_address(address)?;
        let encoded = address.encode_value(value)?;
        let client = self.client()?;
        let mut guard = client.lock().await;
        guard.set_slave(Slave(address.unit_id));

        match encoded {
            EncodedValue::Bits(bits) if bits.len() == 1 => {
                flatten(guard.write_single_coil(address.address, bits[0]).await)
            }
            EncodedValue::Bits(bits) => {
                flatten(guard.write_multiple_coils(address.address, bits).await)
            }
            EncodedValue::Registers(registers) if registers.len() == 1 => flatten(
                guard
                    .write_single_register(address.address, registers[0])
                    .await,
            ),
            EncodedValue::Registers(registers) => flatten(
                guard
                    .write_multiple_registers(address.address, registers)
                    .await,
            ),
        }
    }

    async fn subscribe(
        &self,
        addresses: Vec<TagAddress>,
    ) -> DriverResult<BoxStream<'static, DriverUpdate>> {
        if addresses.is_empty() {
            return Ok(Box::pin(stream::empty()));
        }
        let requests = addresses
            .into_iter()
            .map(|address| {
                let parsed = parse_address(&address)?;
                Ok(SubscriptionAddress {
                    raw: address,
                    parsed,
                })
            })
            .collect::<DriverResult<Vec<_>>>()?;

        let client = self.client()?;
        let poll_rate_ms = self
            .config
            .as_ref()
            .map(|config| config.poll_rate_ms)
            .unwrap_or(250)
            .max(10);

        let state = SubscriptionState {
            client,
            groups: build_read_groups(requests),
            interval: time::interval(Duration::from_millis(poll_rate_ms)),
            pending: VecDeque::new(),
        };

        Ok(Box::pin(stream::unfold(state, |mut state| async move {
            loop {
                if let Some(update) = state.pending.pop_front() {
                    return Some((update, state));
                }

                state.interval.tick().await;
                let ts_ms = now_ms();
                state
                    .pending
                    .extend(poll_groups(&state.client, &state.groups, ts_ms).await);
            }
        })))
    }
}

struct SubscriptionState {
    client: SharedClient,
    groups: Vec<ReadGroup>,
    interval: time::Interval,
    pending: VecDeque<DriverUpdate>,
}

#[derive(Clone)]
struct SubscriptionAddress {
    raw: TagAddress,
    parsed: ModbusAddress,
}

#[derive(Clone)]
struct ReadGroup {
    unit_id: u8,
    area: Area,
    start: u16,
    count: u16,
    items: Vec<SubscriptionAddress>,
}

async fn poll_groups(client: &SharedClient, groups: &[ReadGroup], ts_ms: u64) -> Vec<DriverUpdate> {
    let mut updates = Vec::new();
    for group in groups {
        match read_group(client, group).await {
            Ok(GroupPayload::Bits(bits)) => {
                updates.extend(group.items.iter().map(|item| {
                    let offset = (item.parsed.address - group.start) as usize;
                    let count = item.parsed.count as usize;
                    let result = item.parsed.decode_bits(&bits[offset..offset + count]);
                    update_from_result(item.raw.clone(), result, ts_ms)
                }));
            }
            Ok(GroupPayload::Registers(registers)) => {
                updates.extend(group.items.iter().map(|item| {
                    let offset = (item.parsed.address - group.start) as usize;
                    let count = item.parsed.count as usize;
                    let result = item
                        .parsed
                        .decode_registers(&registers[offset..offset + count]);
                    update_from_result(item.raw.clone(), result, ts_ms)
                }));
            }
            Err(error) => {
                updates.extend(group.items.iter().map(|item| DriverUpdate {
                    address: item.raw.clone(),
                    value: TagValue::String(error.to_string()),
                    quality: error.into_quality(),
                    ts_ms,
                }));
            }
        }
    }
    updates
}

async fn read_group(client: &SharedClient, group: &ReadGroup) -> DriverResult<GroupPayload> {
    let mut guard = client.lock().await;
    guard.set_slave(Slave(group.unit_id));
    match group.area {
        Area::Coils => Ok(GroupPayload::Bits(flatten(
            guard.read_coils(group.start, group.count).await,
        )?)),
        Area::Discrete => Ok(GroupPayload::Bits(flatten(
            guard.read_discrete_inputs(group.start, group.count).await,
        )?)),
        Area::Holding => Ok(GroupPayload::Registers(flatten(
            guard.read_holding_registers(group.start, group.count).await,
        )?)),
        Area::Input => Ok(GroupPayload::Registers(flatten(
            guard.read_input_registers(group.start, group.count).await,
        )?)),
    }
}

enum GroupPayload {
    Bits(Vec<bool>),
    Registers(Vec<u16>),
}

fn update_from_result(
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

fn build_read_groups(mut requests: Vec<SubscriptionAddress>) -> Vec<ReadGroup> {
    requests.sort_by_key(|request| {
        (
            request.parsed.unit_id,
            area_sort_key(request.parsed.area),
            request.parsed.address,
        )
    });

    let mut groups: Vec<ReadGroup> = Vec::new();
    for request in requests {
        let max_count = max_group_count(request.parsed.area);
        let request_start = request.parsed.address;
        let request_end = request_start.saturating_add(request.parsed.count);

        if let Some(group) = groups.last_mut() {
            let group_end = group.start.saturating_add(group.count);
            let merged_end = group_end.max(request_end);
            if group.unit_id == request.parsed.unit_id
                && group.area == request.parsed.area
                && request_start <= group_end
                && merged_end - group.start <= max_count
            {
                group.count = merged_end - group.start;
                group.items.push(request);
                continue;
            }
        }

        groups.push(ReadGroup {
            unit_id: request.parsed.unit_id,
            area: request.parsed.area,
            start: request_start,
            count: request.parsed.count,
            items: vec![request],
        });
    }
    groups
}

fn max_group_count(area: Area) -> u16 {
    if area.is_bit_area() { 2000 } else { 125 }
}

fn area_sort_key(area: Area) -> u8 {
    match area {
        Area::Coils => 0,
        Area::Discrete => 1,
        Area::Input => 2,
        Area::Holding => 3,
    }
}

async fn connect_client(config: &ModbusConfig) -> DriverResult<RealModbusClient> {
    let context = match &config.connection {
        Connection::Tcp { host, port } => {
            let socket_addr = format!("{host}:{port}")
                .parse::<SocketAddr>()
                .map_err(|err| {
                    DriverError::Other(anyhow::anyhow!("invalid TCP endpoint: {err}"))
                })?;
            time::timeout(
                Duration::from_millis(config.connection_timeout_ms),
                tcp::connect_slave(socket_addr, Slave(255)),
            )
            .await
            .map_err(|_| DriverError::NotConnected)?
            .map_err(DriverError::Io)?
        }
        Connection::Rtu {
            device,
            baud,
            parity,
            data_bits,
            stop_bits,
        } => {
            let builder = tokio_serial::new(device, *baud)
                .parity(match parity {
                    Parity::None => tokio_serial::Parity::None,
                    Parity::Odd => tokio_serial::Parity::Odd,
                    Parity::Even => tokio_serial::Parity::Even,
                })
                .data_bits(match data_bits {
                    5 => DataBits::Five,
                    6 => DataBits::Six,
                    7 => DataBits::Seven,
                    _ => DataBits::Eight,
                })
                .stop_bits(match stop_bits {
                    StopBits::One => tokio_serial::StopBits::One,
                    StopBits::Two => tokio_serial::StopBits::Two,
                })
                .flow_control(FlowControl::None);
            let stream = tokio_serial::SerialStream::open(&builder)
                .map_err(|err| DriverError::Other(anyhow::anyhow!(err)))?;
            rtu::attach_slave(stream, Slave(1))
        }
    };

    Ok(RealModbusClient { context })
}

fn parse_address(address: &TagAddress) -> DriverResult<ModbusAddress> {
    ModbusAddress::parse(&address.raw).map_err(|err| DriverError::InvalidAddress(err.to_string()))
}

fn flatten<T>(result: ModbusRequestResult<T>) -> DriverResult<T> {
    result
        .map_err(map_transport_error)?
        .map_err(map_exception_code)
}

fn map_exception_code(exception: ExceptionCode) -> DriverError {
    match exception {
        ExceptionCode::IllegalDataAddress => DriverError::InvalidAddress(exception.to_string()),
        ExceptionCode::IllegalFunction | ExceptionCode::IllegalDataValue => {
            DriverError::UnsupportedType {
                device_type: exception.to_string(),
            }
        }
        other => DriverError::RemoteFault {
            code: format!("modbus-exception-{}", u8::from(other)),
            message: other.to_string(),
        },
    }
}

fn map_transport_error(error: tokio_modbus::Error) -> DriverError {
    match error {
        tokio_modbus::Error::Transport(error) => DriverError::Io(error),
        tokio_modbus::Error::Protocol(error) => DriverError::RemoteFault {
            code: "protocol".to_string(),
            message: error.to_string(),
        },
    }
}

fn now_ms() -> u64 {
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
    struct MockClient {
        reads_u16: VecDeque<ModbusRequestResult<Vec<u16>>>,
        reads_bool: VecDeque<ModbusRequestResult<Vec<bool>>>,
        writes_bool: Vec<(u16, bool)>,
        slave: Option<Slave>,
    }

    #[async_trait]
    impl ModbusClientLike for MockClient {
        async fn read_coils(
            &mut self,
            _address: u16,
            _quantity: u16,
        ) -> ModbusRequestResult<Vec<bool>> {
            self.reads_bool.pop_front().unwrap_or(Ok(Ok(vec![false])))
        }

        async fn read_discrete_inputs(
            &mut self,
            _address: u16,
            _quantity: u16,
        ) -> ModbusRequestResult<Vec<bool>> {
            self.reads_bool.pop_front().unwrap_or(Ok(Ok(vec![false])))
        }

        async fn read_holding_registers(
            &mut self,
            _address: u16,
            _quantity: u16,
        ) -> ModbusRequestResult<Vec<u16>> {
            self.reads_u16.pop_front().unwrap_or(Ok(Ok(vec![0])))
        }

        async fn read_input_registers(
            &mut self,
            _address: u16,
            _quantity: u16,
        ) -> ModbusRequestResult<Vec<u16>> {
            self.reads_u16.pop_front().unwrap_or(Ok(Ok(vec![0])))
        }

        async fn write_single_coil(
            &mut self,
            address: u16,
            value: bool,
        ) -> ModbusRequestResult<()> {
            self.writes_bool.push((address, value));
            Ok(Ok(()))
        }

        async fn write_single_register(
            &mut self,
            _address: u16,
            _value: u16,
        ) -> ModbusRequestResult<()> {
            Ok(Ok(()))
        }

        async fn write_multiple_coils(
            &mut self,
            address: u16,
            values: Vec<bool>,
        ) -> ModbusRequestResult<()> {
            for (offset, value) in values.into_iter().enumerate() {
                self.writes_bool.push((address + offset as u16, value));
            }
            Ok(Ok(()))
        }

        async fn write_multiple_registers(
            &mut self,
            _address: u16,
            _values: Vec<u16>,
        ) -> ModbusRequestResult<()> {
            Ok(Ok(()))
        }

        fn set_slave(&mut self, slave: Slave) {
            self.slave = Some(slave);
        }
    }

    #[tokio::test]
    async fn read_holding_register_maps_to_value() {
        let mut mock = MockClient::default();
        mock.reads_u16.push_back(Ok(Ok(vec![0x4148, 0x0000])));
        let driver = ModbusDriver::with_client(Box::new(mock), ModbusConfig::default());

        let value = driver
            .read(&TagAddress::new("1/holding/100:2:f32"))
            .await
            .unwrap();

        assert_eq!(value, TagValue::Real(12.5));
    }

    #[tokio::test]
    async fn write_coil_uses_bit_write_path() {
        let mock = MockClient::default();
        let shared = Arc::new(Mutex::new(Box::new(mock) as Box<dyn ModbusClientLike>));
        let driver = ModbusDriver {
            client: Some(shared),
            config: Some(ModbusConfig::default()),
        };

        driver
            .write(&TagAddress::new("1/coils/7"), TagValue::Bool(true))
            .await
            .unwrap();
    }

    #[test]
    fn maps_illegal_data_address_to_invalid_address() {
        let error = map_exception_code(ExceptionCode::IllegalDataAddress);
        assert!(matches!(error, DriverError::InvalidAddress(_)));
    }

    #[test]
    fn groups_overlapping_and_adjacent_subscription_reads() {
        let groups = build_read_groups(vec![
            subscription_address("1/holding/100:2:f32"),
            subscription_address("1/holding/102:u16"),
            subscription_address("1/holding/101:2:u32"),
            subscription_address("2/holding/100:u16"),
        ]);

        assert_eq!(groups.len(), 2);
        assert_eq!(groups[0].unit_id, 1);
        assert_eq!(groups[0].start, 100);
        assert_eq!(groups[0].count, 3);
        assert_eq!(groups[0].items.len(), 3);
        assert_eq!(groups[1].unit_id, 2);
    }

    fn subscription_address(raw: &str) -> SubscriptionAddress {
        SubscriptionAddress {
            raw: TagAddress::new(raw),
            parsed: ModbusAddress::parse(raw).unwrap(),
        }
    }
}
