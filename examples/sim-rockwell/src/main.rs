//! Minimal EtherNet/IP simulator for Phase 1 Rockwell driver tests.

mod tag_config;

use std::net::{IpAddr, SocketAddr};
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Instant;

use anyhow::Context;
use clap::Parser;
use tag_config::{SimValue, TagMap, load_tags, update_dynamic_tags, write_tag};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::Mutex;
use tokio::time::{self, Duration};
use tracing::{info, warn};
use tracing_subscriber::EnvFilter;

const CMD_REGISTER_SESSION: u16 = 0x0065;
const CMD_SEND_RR_DATA: u16 = 0x006F;

const CIP_MULTIPLE_SERVICE: u8 = 0x0A;
const CIP_READ_TAG: u8 = 0x4C;
const CIP_WRITE_TAG: u8 = 0x4D;

const CIP_REPLY_MULTIPLE: u8 = 0x8A;
const CIP_REPLY_READ: u8 = 0xCC;
const CIP_REPLY_WRITE: u8 = 0xCD;

const CIP_TYPE_BOOL: u16 = 0x00C1;
const CIP_TYPE_DINT: u16 = 0x00C4;
const CIP_TYPE_REAL: u16 = 0x00CA;
const CIP_TYPE_STRING: u16 = 0x00CE;

#[derive(Debug, Parser)]
#[command(author, version, about)]
struct Args {
    /// IP address to bind.
    #[arg(long, default_value = "127.0.0.1")]
    bind: IpAddr,
    /// TCP port to bind.
    #[arg(long, default_value_t = 44818)]
    port: u16,
    /// Optional simulator tag config.
    #[arg(long)]
    config: Option<PathBuf>,
    /// Log level used when RUST_LOG is not set.
    #[arg(long, default_value = "info")]
    log_level: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    init_tracing(&args.log_level)?;

    let config = args.config.or_else(default_config);
    let tags = Arc::new(Mutex::new(load_tags(config.as_deref())?));
    spawn_tag_updater(Arc::clone(&tags));

    let addr = SocketAddr::new(args.bind, args.port);
    let listener = TcpListener::bind(addr)
        .await
        .with_context(|| format!("failed to bind simulator at {addr}"))?;
    info!(local_addr = %listener.local_addr()?, "sim-rockwell listening");

    loop {
        let (stream, peer) = listener.accept().await.context("accept failed")?;
        let tags = Arc::clone(&tags);
        tokio::spawn(async move {
            if let Err(err) = handle_connection(stream, tags).await {
                warn!(%peer, error = %err, "client connection ended");
            }
        });
    }
}

fn init_tracing(log_level: &str) -> anyhow::Result<()> {
    let filter = EnvFilter::try_from_default_env().or_else(|_| EnvFilter::try_new(log_level))?;
    tracing_subscriber::fmt().with_env_filter(filter).init();
    Ok(())
}

fn default_config() -> Option<PathBuf> {
    let path = PathBuf::from("examples/sim-rockwell/Sim.toml");
    path.exists().then_some(path)
}

fn spawn_tag_updater(tags: Arc<Mutex<TagMap>>) {
    tokio::spawn(async move {
        let started = Instant::now();
        let mut interval = time::interval(Duration::from_millis(50));
        loop {
            interval.tick().await;
            let mut guard = tags.lock().await;
            update_dynamic_tags(&mut guard, started);
        }
    });
}

async fn handle_connection(mut stream: TcpStream, tags: Arc<Mutex<TagMap>>) -> anyhow::Result<()> {
    loop {
        let mut header = [0_u8; 24];
        stream.read_exact(&mut header).await?;

        let command = u16::from_le_bytes([header[0], header[1]]);
        let length = u16::from_le_bytes([header[2], header[3]]) as usize;
        let session_handle = u32::from_le_bytes([header[4], header[5], header[6], header[7]]);

        let mut payload = vec![0_u8; length];
        if length > 0 {
            stream.read_exact(&mut payload).await?;
        }

        match command {
            CMD_REGISTER_SESSION => stream.write_all(&build_register_session_response()).await?,
            CMD_SEND_RR_DATA => {
                let cip_response = build_cip_response(&payload, &tags).await;
                let response = build_send_rr_response(session_handle, &cip_response);
                stream.write_all(&response).await?;
            }
            _ => anyhow::bail!("unsupported EtherNet/IP command 0x{command:04X}"),
        }
    }
}

fn build_register_session_response() -> Vec<u8> {
    let session_handle = 0x1234_5678_u32;
    let mut response = Vec::with_capacity(28);
    response.extend_from_slice(&CMD_REGISTER_SESSION.to_le_bytes());
    response.extend_from_slice(&4_u16.to_le_bytes());
    response.extend_from_slice(&session_handle.to_le_bytes());
    response.extend_from_slice(&0_u32.to_le_bytes());
    response.extend_from_slice(&[0_u8; 8]);
    response.extend_from_slice(&0_u32.to_le_bytes());
    response.extend_from_slice(&[0_u8; 4]);
    response
}

fn build_send_rr_response(session_handle: u32, cip_response: &[u8]) -> Vec<u8> {
    let mut data = Vec::new();
    data.extend_from_slice(&0_u32.to_le_bytes());
    data.extend_from_slice(&0_u16.to_le_bytes());
    data.extend_from_slice(&2_u16.to_le_bytes());
    data.extend_from_slice(&0_u16.to_le_bytes());
    data.extend_from_slice(&0_u16.to_le_bytes());
    data.extend_from_slice(&0x00B2_u16.to_le_bytes());
    data.extend_from_slice(&(cip_response.len() as u16).to_le_bytes());
    data.extend_from_slice(cip_response);

    let mut response = Vec::with_capacity(24 + data.len());
    response.extend_from_slice(&CMD_SEND_RR_DATA.to_le_bytes());
    response.extend_from_slice(&(data.len() as u16).to_le_bytes());
    response.extend_from_slice(&session_handle.to_le_bytes());
    response.extend_from_slice(&0_u32.to_le_bytes());
    response.extend_from_slice(&[0_u8; 8]);
    response.extend_from_slice(&0_u32.to_le_bytes());
    response.extend_from_slice(&data);
    response
}

async fn build_cip_response(payload: &[u8], tags: &Arc<Mutex<TagMap>>) -> Vec<u8> {
    let cip_request = extract_cip_request(payload);
    match cip_request.first().copied().unwrap_or(0) {
        CIP_READ_TAG => build_read_response(&cip_request, tags).await,
        CIP_WRITE_TAG => {
            handle_write(&cip_request, tags).await;
            vec![CIP_REPLY_WRITE, 0x00, 0x00, 0x00]
        }
        CIP_MULTIPLE_SERVICE => build_multiple_service_response(&cip_request, tags).await,
        _ => vec![CIP_REPLY_READ, 0x00, 0x01, 0x00],
    }
}

async fn build_read_response(cip_request: &[u8], tags: &Arc<Mutex<TagMap>>) -> Vec<u8> {
    let Some((tag_name, _)) = parse_tag_and_path(cip_request) else {
        return vec![CIP_REPLY_READ, 0x00, 0x04, 0x00];
    };
    let tags = tags.lock().await;
    let value = tags
        .get(&tag_name)
        .map(|tag| tag.value.clone())
        .unwrap_or(SimValue::Dint(0));
    build_value_response(value)
}

async fn build_multiple_service_response(cip_request: &[u8], tags: &Arc<Mutex<TagMap>>) -> Vec<u8> {
    let requests = parse_multiple_service_requests(cip_request);
    let mut replies = Vec::new();
    for request in requests {
        let reply = match request.first().copied().unwrap_or(0) {
            CIP_READ_TAG => build_read_response(&request, tags).await,
            CIP_WRITE_TAG => {
                handle_write(&request, tags).await;
                vec![CIP_REPLY_WRITE, 0x00, 0x00, 0x00]
            }
            _ => vec![CIP_REPLY_READ, 0x00, 0x01, 0x00],
        };
        replies.push(reply);
    }

    let mut response = vec![CIP_REPLY_MULTIPLE, 0x00, 0x00, 0x00];
    response.extend_from_slice(&(replies.len() as u16).to_le_bytes());
    let mut offset = 2 + replies.len() * 2;
    for reply in &replies {
        response.extend_from_slice(&(offset as u16).to_le_bytes());
        offset += reply.len();
    }
    for reply in replies {
        response.extend_from_slice(&reply);
    }
    response
}

fn parse_multiple_service_requests(cip_request: &[u8]) -> Vec<Vec<u8>> {
    if cip_request.len() < 7 {
        return Vec::new();
    }
    let count_pos = 2 + cip_request[1] as usize * 2;
    if cip_request.len() < count_pos + 2 {
        return Vec::new();
    }
    let count = u16::from_le_bytes([cip_request[count_pos], cip_request[count_pos + 1]]) as usize;
    let offsets_start = count_pos + 2;
    if cip_request.len() < offsets_start + count * 2 {
        return Vec::new();
    }

    let service_base = count_pos;
    let mut offsets = Vec::with_capacity(count);
    for i in 0..count {
        let pos = offsets_start + i * 2;
        offsets.push(u16::from_le_bytes([cip_request[pos], cip_request[pos + 1]]) as usize);
    }

    offsets
        .iter()
        .enumerate()
        .filter_map(|(i, offset)| {
            let start = service_base + offset;
            let end = offsets
                .get(i + 1)
                .map(|next| service_base + next)
                .unwrap_or(cip_request.len());
            (start < end && end <= cip_request.len()).then(|| cip_request[start..end].to_vec())
        })
        .collect()
}

async fn handle_write(cip_request: &[u8], tags: &Arc<Mutex<TagMap>>) {
    let Some((tag_name, _)) = parse_tag_and_path(cip_request) else {
        return;
    };
    let path_words = cip_request.get(1).copied().unwrap_or(0) as usize;
    let path_end = 2 + path_words * 2;
    if cip_request.len() < path_end + 4 {
        return;
    }

    let data_type = u16::from_le_bytes([cip_request[path_end], cip_request[path_end + 1]]);
    let data_start = path_end + 4;
    let value = match data_type {
        CIP_TYPE_BOOL => cip_request
            .get(data_start)
            .map(|value| SimValue::Bool(*value != 0)),
        CIP_TYPE_DINT if cip_request.len() >= data_start + 4 => {
            Some(SimValue::Dint(i32::from_le_bytes(
                cip_request[data_start..data_start + 4]
                    .try_into()
                    .unwrap_or([0; 4]),
            )))
        }
        CIP_TYPE_REAL if cip_request.len() >= data_start + 4 => {
            Some(SimValue::Real(f32::from_le_bytes(
                cip_request[data_start..data_start + 4]
                    .try_into()
                    .unwrap_or([0; 4]),
            )))
        }
        CIP_TYPE_STRING if cip_request.len() >= data_start + 4 => {
            let len = u32::from_le_bytes(
                cip_request[data_start..data_start + 4]
                    .try_into()
                    .unwrap_or([0; 4]),
            ) as usize;
            let string_start = data_start + 4;
            (cip_request.len() >= string_start + len).then(|| {
                SimValue::String(
                    String::from_utf8_lossy(&cip_request[string_start..string_start + len])
                        .to_string(),
                )
            })
        }
        _ => None,
    };

    if let Some(value) = value {
        let mut guard = tags.lock().await;
        write_tag(&mut guard, tag_name, value);
    }
}

fn build_value_response(value: SimValue) -> Vec<u8> {
    let mut response = vec![CIP_REPLY_READ, 0x00, 0x00, 0x00];
    match value {
        SimValue::Bool(value) => {
            response.extend_from_slice(&CIP_TYPE_BOOL.to_le_bytes());
            response.push(if value { 0xFF } else { 0x00 });
        }
        SimValue::Dint(value) => {
            response.extend_from_slice(&CIP_TYPE_DINT.to_le_bytes());
            response.extend_from_slice(&value.to_le_bytes());
        }
        SimValue::Real(value) => {
            response.extend_from_slice(&CIP_TYPE_REAL.to_le_bytes());
            response.extend_from_slice(&value.to_le_bytes());
        }
        SimValue::String(value) => {
            response.extend_from_slice(&CIP_TYPE_STRING.to_le_bytes());
            response.extend_from_slice(&(value.len() as u32).to_le_bytes());
            response.extend_from_slice(value.as_bytes());
        }
    }
    response
}

fn extract_cip_request(payload: &[u8]) -> Vec<u8> {
    if payload.len() < 8 {
        return Vec::new();
    }
    let item_count = u16::from_le_bytes([payload[6], payload[7]]);
    let mut pos = 8;
    for _ in 0..item_count {
        if pos + 4 > payload.len() {
            break;
        }
        let item_type = u16::from_le_bytes([payload[pos], payload[pos + 1]]);
        let item_len = u16::from_le_bytes([payload[pos + 2], payload[pos + 3]]) as usize;
        pos += 4;
        if pos + item_len > payload.len() {
            break;
        }
        if item_type == 0x00B2 {
            let ucmm = &payload[pos..pos + item_len];
            if ucmm.len() < 10 {
                return Vec::new();
            }
            let msg_len = u16::from_le_bytes([ucmm[8], ucmm[9]]) as usize;
            let start = 10;
            let end = usize::min(start + msg_len, ucmm.len());
            return ucmm[start..end].to_vec();
        }
        pos += item_len;
    }
    Vec::new()
}

fn parse_tag_and_path(cip_request: &[u8]) -> Option<(String, Option<usize>)> {
    if cip_request.len() < 2 {
        return None;
    }

    let path_words = cip_request[1] as usize;
    let path_bytes = path_words * 2;
    if cip_request.len() < 2 + path_bytes {
        return None;
    }

    let path = &cip_request[2..2 + path_bytes];
    let mut pos = 0;
    let mut tag_name = None;
    let mut element_index = None;

    while pos < path.len() {
        match path[pos] {
            0x91 => {
                let len = *path.get(pos + 1)? as usize;
                let start = pos + 2;
                let end = start + len;
                if end > path.len() {
                    break;
                }
                if tag_name.is_none() {
                    tag_name = Some(String::from_utf8_lossy(&path[start..end]).to_string());
                }
                pos = end + (len % 2);
            }
            0x28 => {
                element_index = path.get(pos + 1).map(|value| *value as usize);
                pos += 2;
            }
            _ => pos += 1,
        }
    }

    tag_name.map(|name| (name, element_index))
}
