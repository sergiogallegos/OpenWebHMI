//! Minimal OPC UA simulator used by OpenWebHMI driver tests.

use std::sync::Arc;

use anyhow::{anyhow, Context, Result};
use opcua::server::address_space::{AccessLevel, VariableBuilder};
use opcua::server::node_manager::memory::{simple_node_manager_imports, SimpleNodeManager};
use opcua::server::{ServerBuilder, ServerHandle, ANONYMOUS_USER_TOKEN_ID};
use opcua::types::{
    DataTypeId, DataValue, MessageSecurityMode, NodeId, NodeSetNamespaceMapper, ObjectId,
};
use tokio::net::TcpListener;
use tokio::task::JoinHandle;

const NAMESPACE_URI: &str = "urn:openwebhmi:sim-opcua";
// async-opcua's generated diagnostics node manager claims namespace 1 before custom
// managers. The simulator reserves a padding namespace and installs test nodes into
// the namespace owned by its SimpleNodeManager so service routing is unambiguous.
const RESERVED_NAMESPACE_URI: &str = "urn:openwebhmi:sim-opcua:reserved";
const NODESET_RESERVED_NAMESPACE_INDEX: u16 = 1;
const NODESET_NAMESPACE_INDEX: u16 = NODESET_RESERVED_NAMESPACE_INDEX;

/// Running simulator handle.
pub struct SimHandle {
    endpoint: String,
    namespace_index: u16,
    server: ServerHandle,
    _task: JoinHandle<()>,
    manager: Arc<SimpleNodeManager>,
}

impl SimHandle {
    /// OPC UA endpoint URL.
    pub fn endpoint(&self) -> &str {
        &self.endpoint
    }

    /// Namespace index assigned to simulator nodes.
    pub fn namespace_index(&self) -> u16 {
        self.namespace_index
    }

    /// Write the pressure node and notify subscribers.
    pub fn set_pressure(&self, value: f64) -> Result<()> {
        self.manager
            .set_value(
                self.server.subscriptions(),
                &pressure_node_id(self.namespace_index),
                None,
                DataValue::new_now(value),
            )
            .map_err(|status| anyhow!("set Pressure failed: {status}"))
    }

    /// Stop the simulator.
    pub fn stop(&self) {
        self.server.cancel();
    }
}

/// Start on an ephemeral local port.
pub async fn start_ephemeral() -> Result<SimHandle> {
    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .context("bind sim-opcua ephemeral port")?;
    start_with_listener(listener).await
}

/// Start with an already-bound listener.
pub async fn start_with_listener(listener: TcpListener) -> Result<SimHandle> {
    let addr = listener.local_addr()?;
    let endpoint = format!("opc.tcp://127.0.0.1:{}/", addr.port());
    let builder = ServerBuilder::new()
        .application_name("OpenWebHMI sim-opcua")
        .application_uri("urn:openwebhmi:sim-opcua")
        .product_uri("urn:openwebhmi:sim-opcua")
        .create_sample_keypair(true)
        .host("127.0.0.1")
        .pki_dir("./target/sim-opcua-pki")
        .discovery_urls(vec![endpoint.clone()])
        .with_node_manager(simple_node_manager_imports(
            vec![Box::new(SimNodeSetImport)],
            "openwebhmi-sim",
        ))
        .add_endpoint(
            "none",
            (
                "/",
                opcua::crypto::SecurityPolicy::None,
                MessageSecurityMode::None,
                &[ANONYMOUS_USER_TOKEN_ID] as &[&str],
            ),
        );
    let (server, handle) = builder.build().map_err(|err| anyhow!(err))?;
    let manager = handle
        .node_managers()
        .get_of_type::<SimpleNodeManager>()
        .ok_or_else(|| anyhow!("simple node manager missing"))?;

    let namespace_index = *manager
        .namespaces()
        .iter()
        .find(|(_, uri)| uri.as_str() == RESERVED_NAMESPACE_URI)
        .map(|(index, _)| index)
        .ok_or_else(|| anyhow!("sim-opcua namespace missing"))?;
    install_nodes(&manager, namespace_index)?;

    let task = tokio::spawn(async move {
        let _ = server.run_with(listener).await;
    });

    Ok(SimHandle {
        endpoint,
        namespace_index,
        server: handle,
        _task: task,
        manager,
    })
}

fn install_nodes(manager: &SimpleNodeManager, namespace_index: u16) -> Result<()> {
    let mut address_space = manager.address_space().write();
    let pressure = pressure_node_id(namespace_index);
    let counter = counter_node_id(namespace_index);
    let pressure_inserted = VariableBuilder::new(&pressure, "Pressure", "Pressure")
        .value(12.5_f64)
        .data_type(DataTypeId::Double)
        .access_level(AccessLevel::CURRENT_READ | AccessLevel::CURRENT_WRITE)
        .user_access_level(AccessLevel::CURRENT_READ | AccessLevel::CURRENT_WRITE)
        .organized_by(ObjectId::ObjectsFolder)
        .insert(&mut *address_space);
    let counter_inserted = VariableBuilder::new(&counter, "Counter", "Counter")
        .value(0_i64)
        .data_type(DataTypeId::Int64)
        .access_level(AccessLevel::CURRENT_READ | AccessLevel::CURRENT_WRITE)
        .user_access_level(AccessLevel::CURRENT_READ | AccessLevel::CURRENT_WRITE)
        .organized_by(ObjectId::ObjectsFolder)
        .insert(&mut *address_space);
    if !pressure_inserted || !counter_inserted {
        return Err(anyhow!("sim-opcua failed to insert variable nodes"));
    }
    Ok(())
}

struct SimNodeSetImport;

impl opcua::nodes::NodeSetImport for SimNodeSetImport {
    fn register_namespaces(&self, namespaces: &mut NodeSetNamespaceMapper) {
        namespaces.add_namespace(RESERVED_NAMESPACE_URI, NODESET_RESERVED_NAMESPACE_INDEX);
        namespaces.add_namespace(NAMESPACE_URI, NODESET_NAMESPACE_INDEX);
    }

    fn get_own_namespaces(&self) -> Vec<String> {
        vec![
            RESERVED_NAMESPACE_URI.to_string(),
            NAMESPACE_URI.to_string(),
        ]
    }

    fn load<'a>(
        &'a self,
        _namespaces: &'a NodeSetNamespaceMapper,
    ) -> Box<dyn Iterator<Item = opcua::nodes::ImportedItem> + 'a> {
        Box::new(std::iter::empty())
    }
}

/// Pressure node id in a specific namespace.
pub fn pressure_node_id(namespace_index: u16) -> NodeId {
    NodeId::new(namespace_index, "Pressure")
}

/// Counter node id.
pub fn counter_node_id(namespace_index: u16) -> NodeId {
    NodeId::new(namespace_index, "Counter")
}

/// Default state placeholder retained for light unit tests.
pub fn default_state() -> (f64, i64) {
    (12.5, 0)
}
