//! Minimal Modbus TCP slave used by OpenWebHMI driver tests.
//!
//! This simulator intentionally implements only the function codes exercised by
//! `crates/driver-modbus`: FC1/2/3/4 reads and FC5/6/15/16 writes.

use std::net::SocketAddr;
use std::sync::Arc;

use anyhow::{Context, Result, anyhow, bail};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{Mutex, Notify};

const FC_READ_COILS: u8 = 1;
const FC_READ_DISCRETE_INPUTS: u8 = 2;
const FC_READ_HOLDING_REGISTERS: u8 = 3;
const FC_READ_INPUT_REGISTERS: u8 = 4;
const FC_WRITE_SINGLE_COIL: u8 = 5;
const FC_WRITE_SINGLE_REGISTER: u8 = 6;
const FC_WRITE_MULTIPLE_COILS: u8 = 15;
const FC_WRITE_MULTIPLE_REGISTERS: u8 = 16;

const EX_ILLEGAL_FUNCTION: u8 = 1;
const EX_ILLEGAL_DATA_ADDRESS: u8 = 2;
const EX_ILLEGAL_DATA_VALUE: u8 = 3;

/// Shared simulator state.
#[derive(Debug)]
pub struct SimState {
    holding: Vec<u16>,
    input: Vec<u16>,
    coils: Vec<bool>,
    discrete: Vec<bool>,
    notify: Notify,
}

impl Default for SimState {
    fn default() -> Self {
        let mut holding = vec![0; 512];
        let f32_words = 12.5_f32.to_bits().to_be_bytes();
        holding[100] = u16::from_be_bytes([f32_words[0], f32_words[1]]);
        holding[101] = u16::from_be_bytes([f32_words[2], f32_words[3]]);
        holding[200] = 0x4f4b;
        holding[201] = 0x2100;

        let mut input = vec![0; 512];
        input[10] = 1234;

        let mut coils = vec![false; 2048];
        coils[0] = true;

        let mut discrete = vec![false; 2048];
        discrete[0] = true;

        Self {
            holding,
            input,
            coils,
            discrete,
            notify: Notify::new(),
        }
    }
}

impl SimState {
    /// Return a holding register value.
    pub async fn holding(&self, address: u16) -> Option<u16> {
        self.holding.get(address as usize).copied()
    }

    /// Return a coil value.
    pub async fn coil(&self, address: u16) -> Option<bool> {
        self.coils.get(address as usize).copied()
    }

    /// Wait until any write modifies simulator state.
    pub async fn wait_for_write(&self) {
        self.notify.notified().await;
    }
}

/// Shared simulator state handle.
pub type SharedSimState = Arc<Mutex<SimState>>;

/// Create the default simulator state.
pub fn default_state() -> SharedSimState {
    Arc::new(Mutex::new(SimState::default()))
}

/// Serve Modbus TCP on an already-bound listener.
pub async fn serve(listener: TcpListener, state: SharedSimState) -> Result<()> {
    loop {
        let (stream, peer) = listener.accept().await?;
        let state = state.clone();
        tokio::spawn(async move {
            if let Err(error) = handle_connection(stream, state).await {
                tracing::debug!(%peer, %error, "sim-modbus connection ended");
            }
        });
    }
}

/// Bind and serve Modbus TCP until the task is cancelled.
pub async fn bind_and_serve(bind: SocketAddr, state: SharedSimState) -> Result<()> {
    let listener = TcpListener::bind(bind).await?;
    tracing::info!(%bind, "sim-modbus listening");
    serve(listener, state).await
}

async fn handle_connection(mut stream: TcpStream, state: SharedSimState) -> Result<()> {
    loop {
        let mut header = [0_u8; 7];
        stream.read_exact(&mut header).await?;
        let transaction_id = u16::from_be_bytes([header[0], header[1]]);
        let protocol_id = u16::from_be_bytes([header[2], header[3]]);
        if protocol_id != 0 {
            bail!("unsupported protocol id {protocol_id}");
        }
        let length = u16::from_be_bytes([header[4], header[5]]) as usize;
        if length == 0 {
            bail!("invalid zero-length Modbus frame");
        }
        let unit_id = header[6];
        let mut pdu = vec![0_u8; length - 1];
        stream.read_exact(&mut pdu).await?;

        let response_pdu = {
            let mut guard = state.lock().await;
            match handle_pdu(&mut guard, &pdu) {
                Ok(response) => response,
                Err(exception) => vec![pdu.first().copied().unwrap_or_default() | 0x80, exception],
            }
        };

        let response_length = response_pdu.len() + 1;
        let mut response = Vec::with_capacity(7 + response_pdu.len());
        response.extend_from_slice(&transaction_id.to_be_bytes());
        response.extend_from_slice(&0_u16.to_be_bytes());
        response.extend_from_slice(&(response_length as u16).to_be_bytes());
        response.push(unit_id);
        response.extend_from_slice(&response_pdu);
        stream.write_all(&response).await?;
    }
}

fn handle_pdu(state: &mut SimState, pdu: &[u8]) -> Result<Vec<u8>, u8> {
    let function = *pdu.first().ok_or(EX_ILLEGAL_DATA_VALUE)?;
    match function {
        FC_READ_COILS => read_bits(function, &state.coils, pdu),
        FC_READ_DISCRETE_INPUTS => read_bits(function, &state.discrete, pdu),
        FC_READ_HOLDING_REGISTERS => read_registers(function, &state.holding, pdu),
        FC_READ_INPUT_REGISTERS => read_registers(function, &state.input, pdu),
        FC_WRITE_SINGLE_COIL => write_single_coil(state, pdu),
        FC_WRITE_SINGLE_REGISTER => write_single_register(state, pdu),
        FC_WRITE_MULTIPLE_COILS => write_multiple_coils(state, pdu),
        FC_WRITE_MULTIPLE_REGISTERS => write_multiple_registers(state, pdu),
        _ => Err(EX_ILLEGAL_FUNCTION),
    }
}

fn read_bits(function: u8, values: &[bool], pdu: &[u8]) -> Result<Vec<u8>, u8> {
    let (address, quantity) = read_address_and_quantity(pdu)?;
    if quantity == 0 || quantity > 2000 {
        return Err(EX_ILLEGAL_DATA_VALUE);
    }
    let end = address
        .checked_add(quantity)
        .ok_or(EX_ILLEGAL_DATA_ADDRESS)? as usize;
    let slice = values
        .get(address as usize..end)
        .ok_or(EX_ILLEGAL_DATA_ADDRESS)?;
    let byte_count = slice.len().div_ceil(8);
    let mut response = vec![function, byte_count as u8];
    response.resize(2 + byte_count, 0);
    for (index, bit) in slice.iter().copied().enumerate() {
        if bit {
            response[2 + index / 8] |= 1 << (index % 8);
        }
    }
    Ok(response)
}

fn read_registers(function: u8, values: &[u16], pdu: &[u8]) -> Result<Vec<u8>, u8> {
    let (address, quantity) = read_address_and_quantity(pdu)?;
    if quantity == 0 || quantity > 125 {
        return Err(EX_ILLEGAL_DATA_VALUE);
    }
    let end = address
        .checked_add(quantity)
        .ok_or(EX_ILLEGAL_DATA_ADDRESS)? as usize;
    let slice = values
        .get(address as usize..end)
        .ok_or(EX_ILLEGAL_DATA_ADDRESS)?;
    let mut response = vec![function, (slice.len() * 2) as u8];
    for value in slice {
        response.extend_from_slice(&value.to_be_bytes());
    }
    Ok(response)
}

fn write_single_coil(state: &mut SimState, pdu: &[u8]) -> Result<Vec<u8>, u8> {
    if pdu.len() != 5 {
        return Err(EX_ILLEGAL_DATA_VALUE);
    }
    let address = u16::from_be_bytes([pdu[1], pdu[2]]) as usize;
    let value = match u16::from_be_bytes([pdu[3], pdu[4]]) {
        0x0000 => false,
        0xff00 => true,
        _ => return Err(EX_ILLEGAL_DATA_VALUE),
    };
    *state
        .coils
        .get_mut(address)
        .ok_or(EX_ILLEGAL_DATA_ADDRESS)? = value;
    state.notify.notify_waiters();
    Ok(pdu.to_vec())
}

fn write_single_register(state: &mut SimState, pdu: &[u8]) -> Result<Vec<u8>, u8> {
    if pdu.len() != 5 {
        return Err(EX_ILLEGAL_DATA_VALUE);
    }
    let address = u16::from_be_bytes([pdu[1], pdu[2]]) as usize;
    let value = u16::from_be_bytes([pdu[3], pdu[4]]);
    *state
        .holding
        .get_mut(address)
        .ok_or(EX_ILLEGAL_DATA_ADDRESS)? = value;
    state.notify.notify_waiters();
    Ok(pdu.to_vec())
}

fn write_multiple_coils(state: &mut SimState, pdu: &[u8]) -> Result<Vec<u8>, u8> {
    if pdu.len() < 6 {
        return Err(EX_ILLEGAL_DATA_VALUE);
    }
    let (address, quantity) = read_address_and_quantity(pdu)?;
    let byte_count = pdu[5] as usize;
    if pdu.len() != 6 + byte_count {
        return Err(EX_ILLEGAL_DATA_VALUE);
    }
    let end = address
        .checked_add(quantity)
        .ok_or(EX_ILLEGAL_DATA_ADDRESS)? as usize;
    let slice = state
        .coils
        .get_mut(address as usize..end)
        .ok_or(EX_ILLEGAL_DATA_ADDRESS)?;
    for (index, value) in slice.iter_mut().enumerate() {
        *value = (pdu[6 + index / 8] & (1 << (index % 8))) != 0;
    }
    state.notify.notify_waiters();
    Ok(vec![
        FC_WRITE_MULTIPLE_COILS,
        pdu[1],
        pdu[2],
        pdu[3],
        pdu[4],
    ])
}

fn write_multiple_registers(state: &mut SimState, pdu: &[u8]) -> Result<Vec<u8>, u8> {
    if pdu.len() < 6 {
        return Err(EX_ILLEGAL_DATA_VALUE);
    }
    let (address, quantity) = read_address_and_quantity(pdu)?;
    let byte_count = pdu[5] as usize;
    if pdu.len() != 6 + byte_count || byte_count != quantity as usize * 2 {
        return Err(EX_ILLEGAL_DATA_VALUE);
    }
    let end = address
        .checked_add(quantity)
        .ok_or(EX_ILLEGAL_DATA_ADDRESS)? as usize;
    let slice = state
        .holding
        .get_mut(address as usize..end)
        .ok_or(EX_ILLEGAL_DATA_ADDRESS)?;
    for (index, value) in slice.iter_mut().enumerate() {
        let offset = 6 + index * 2;
        *value = u16::from_be_bytes([pdu[offset], pdu[offset + 1]]);
    }
    state.notify.notify_waiters();
    Ok(vec![
        FC_WRITE_MULTIPLE_REGISTERS,
        pdu[1],
        pdu[2],
        pdu[3],
        pdu[4],
    ])
}

fn read_address_and_quantity(pdu: &[u8]) -> Result<(u16, u16), u8> {
    if pdu.len() < 5 {
        return Err(EX_ILLEGAL_DATA_VALUE);
    }
    Ok((
        u16::from_be_bytes([pdu[1], pdu[2]]),
        u16::from_be_bytes([pdu[3], pdu[4]]),
    ))
}

/// Bind helper used by tests that already need the selected port.
pub async fn bind_ephemeral() -> Result<(TcpListener, SocketAddr)> {
    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .context("bind sim-modbus ephemeral port")?;
    let addr = listener.local_addr().map_err(|err| anyhow!(err))?;
    Ok((listener, addr))
}
