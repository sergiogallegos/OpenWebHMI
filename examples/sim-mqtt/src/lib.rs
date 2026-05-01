//! MQTT simulator used by OpenWebHMI driver tests.

use std::collections::HashMap;
use std::net::SocketAddr;
use std::thread::JoinHandle;
use std::time::Duration;

use anyhow::{Context, Result};
use prost::Message;
use rumqttc::{AsyncClient, MqttOptions, QoS};
use rumqttd::{Broker, Config, ConnectionSettings, RouterConfig, ServerSettings};
use tokio::net::TcpListener;

/// Running MQTT simulator handle.
pub struct SimHandle {
    addr: SocketAddr,
    _broker: JoinHandle<()>,
}

impl SimHandle {
    /// Broker socket address.
    pub fn addr(&self) -> SocketAddr {
        self.addr
    }

    /// Broker host.
    pub fn host(&self) -> String {
        self.addr.ip().to_string()
    }

    /// Broker port.
    pub fn port(&self) -> u16 {
        self.addr.port()
    }

    /// Publish the deterministic generic and Sparkplug values.
    pub async fn publish_once(&self) -> Result<()> {
        let mut options = MqttOptions::new("sim-mqtt-publisher", self.host(), self.port());
        options.set_keep_alive(Duration::from_secs(5));
        let (client, mut event_loop) = AsyncClient::new(options, 10);
        let event_task = tokio::spawn(async move { while event_loop.poll().await.is_ok() {} });

        client
            .publish("factory/line1/temperature", QoS::AtLeastOnce, false, "12.5")
            .await?;
        let birth = encode_payload(vec![Metric {
            name: Some("Pressure".to_string()),
            alias: Some(1),
            value: Some(metric::Value::DoubleValue(10.0)),
            ..Default::default()
        }]);
        client
            .publish(
                "spB/v1.0/group/DBIRTH/edge/device",
                QoS::AtLeastOnce,
                false,
                birth,
            )
            .await?;
        let data = encode_payload(vec![Metric {
            alias: Some(1),
            value: Some(metric::Value::DoubleValue(42.25)),
            ..Default::default()
        }]);
        client
            .publish(
                "spB/v1.0/group/DDATA/edge/device",
                QoS::AtLeastOnce,
                false,
                data,
            )
            .await?;
        tokio::time::sleep(Duration::from_millis(100)).await;
        event_task.abort();
        Ok(())
    }
}

#[derive(Clone, PartialEq, Message)]
struct Payload {
    #[prost(uint64, optional, tag = "1")]
    timestamp: Option<u64>,
    #[prost(message, repeated, tag = "2")]
    metrics: Vec<Metric>,
    #[prost(uint64, optional, tag = "3")]
    seq: Option<u64>,
}

#[derive(Clone, PartialEq, Message)]
struct Metric {
    #[prost(string, optional, tag = "1")]
    name: Option<String>,
    #[prost(uint64, optional, tag = "2")]
    alias: Option<u64>,
    #[prost(uint64, optional, tag = "3")]
    timestamp: Option<u64>,
    #[prost(uint32, optional, tag = "4")]
    datatype: Option<u32>,
    #[prost(bool, optional, tag = "5")]
    is_historical: Option<bool>,
    #[prost(bool, optional, tag = "6")]
    is_transient: Option<bool>,
    #[prost(bool, optional, tag = "7")]
    is_null: Option<bool>,
    #[prost(oneof = "metric::Value", tags = "10, 11, 12, 13, 14, 15, 16")]
    value: Option<metric::Value>,
}

mod metric {
    #[allow(clippy::enum_variant_names)]
    #[derive(Clone, PartialEq, ::prost::Oneof)]
    pub enum Value {
        #[prost(uint32, tag = "10")]
        IntValue(u32),
        #[prost(uint64, tag = "11")]
        LongValue(u64),
        #[prost(float, tag = "12")]
        FloatValue(f32),
        #[prost(double, tag = "13")]
        DoubleValue(f64),
        #[prost(bool, tag = "14")]
        BooleanValue(bool),
        #[prost(string, tag = "15")]
        StringValue(String),
        #[prost(bytes, tag = "16")]
        BytesValue(Vec<u8>),
    }
}

fn encode_payload(metrics: Vec<Metric>) -> Vec<u8> {
    Payload {
        timestamp: None,
        metrics,
        seq: None,
    }
    .encode_to_vec()
}

/// Start the simulator broker on an ephemeral local port.
pub async fn start_ephemeral() -> Result<SimHandle> {
    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .context("bind sim-mqtt ephemeral port")?;
    let addr = listener.local_addr()?;
    drop(listener);
    start(addr).await
}

/// Start the simulator broker on a fixed local address.
pub async fn start(addr: SocketAddr) -> Result<SimHandle> {
    let mut v4 = HashMap::new();
    v4.insert(
        "mqtt".to_string(),
        ServerSettings {
            name: "mqtt".to_string(),
            listen: addr,
            tls: None,
            next_connection_delay_ms: 1,
            connections: ConnectionSettings {
                connection_timeout_ms: 5000,
                max_payload_size: 1024 * 1024,
                max_inflight_count: 100,
                auth: None,
                external_auth: None,
                dynamic_filters: true,
            },
        },
    );
    let config = Config {
        id: 0,
        router: RouterConfig {
            max_connections: 100,
            max_outgoing_packet_count: 200,
            max_segment_size: 1024 * 1024,
            max_segment_count: 10,
            custom_segment: None,
            initialized_filters: None,
            shared_subscriptions_strategy: Default::default(),
        },
        v4: Some(v4),
        v5: None,
        ws: None,
        cluster: None,
        console: None,
        bridge: None,
        prometheus: None,
        metrics: None,
    };
    let mut broker = Broker::new(config);
    let broker_thread = std::thread::spawn(move || {
        let _ = broker.start();
    });
    tokio::time::sleep(Duration::from_millis(100)).await;
    Ok(SimHandle {
        addr,
        _broker: broker_thread,
    })
}
