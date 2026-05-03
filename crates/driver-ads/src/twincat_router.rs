//! Windows TwinCAT-router ADS backend using Beckhoff `TcAdsDll.dll`.

use std::ffi::{CString, OsStr, c_char, c_void};
use std::os::windows::ffi::OsStrExt;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex as StdMutex};
use std::thread::JoinHandle;
use std::time::Duration;

use async_trait::async_trait;
use openwebhmi_driver_api::{DriverError, DriverResult};
use tokio::sync::mpsc;

use crate::connection::{AdsConfig, parse_ams_net_id};
use crate::driver::{
    AdsClientLike, SubscriptionEntry, SubscriptionGuard, now_ms, spawn_blocking_driver,
    update_from_result,
};
use crate::symbols::{AdsDataType, SymbolEntry, SymbolTable};

const ADSIGRP_SYM_HNDBYNAME: u32 = 0xF003;
const ADSIGRP_SYM_VALBYHND: u32 = 0xF005;
const ADSIGRP_SYM_RELEASEHND: u32 = 0xF006;
const ADSIGRP_SYM_UPLOAD: u32 = 0xF00B;
const ADSIGRP_SYM_UPLOADINFO: u32 = 0xF00C;
const ADSIGRP_SYM_UPLOADINFO2: u32 = 0xF00F;

#[repr(C)]
#[derive(Clone, Copy)]
struct AmsNetId {
    b: [u8; 6],
}

#[repr(C)]
#[derive(Clone, Copy)]
struct AmsAddr {
    net_id: AmsNetId,
    port: u16,
}

type AdsPortOpenEx = unsafe extern "system" fn() -> i32;
type AdsPortCloseEx = unsafe extern "system" fn(i32) -> i32;
type AdsSyncReadReqEx2 =
    unsafe extern "system" fn(i32, *const AmsAddr, u32, u32, u32, *mut c_void, *mut u32) -> i32;
type AdsSyncWriteReqEx =
    unsafe extern "system" fn(i32, *const AmsAddr, u32, u32, u32, *const c_void) -> i32;
type AdsSyncReadWriteReqEx2 = unsafe extern "system" fn(
    i32,
    *const AmsAddr,
    u32,
    u32,
    u32,
    *mut c_void,
    u32,
    *const c_void,
    *mut u32,
) -> i32;

#[link(name = "kernel32")]
unsafe extern "system" {
    fn LoadLibraryW(name: *const u16) -> *mut c_void;
    fn GetProcAddress(module: *mut c_void, name: *const c_char) -> *mut c_void;
    fn FreeLibrary(module: *mut c_void) -> i32;
}

struct LibraryHandle(*mut c_void);

unsafe impl Send for LibraryHandle {}
unsafe impl Sync for LibraryHandle {}

impl Drop for LibraryHandle {
    fn drop(&mut self) {
        let _ = unsafe { FreeLibrary(self.0) };
    }
}

struct TcAdsApi {
    _library: LibraryHandle,
    port_open: AdsPortOpenEx,
    port_close: AdsPortCloseEx,
    read: AdsSyncReadReqEx2,
    write: AdsSyncWriteReqEx,
    read_write: AdsSyncReadWriteReqEx2,
}

impl TcAdsApi {
    fn load() -> DriverResult<Self> {
        let mut errors = Vec::new();
        for path in tc_ads_dll_candidates() {
            let encoded = wide_null(&path);
            let module = unsafe { LoadLibraryW(encoded.as_ptr()) };
            if module.is_null() {
                errors.push(path);
            } else {
                return unsafe { Self::from_library(LibraryHandle(module)) };
            }
        }

        Err(DriverError::Other(anyhow::anyhow!(
            "could not load Beckhoff TcAdsDll.dll from candidates: {}",
            errors.join("; ")
        )))
    }

    unsafe fn from_library(library: LibraryHandle) -> DriverResult<Self> {
        unsafe {
            let port_open = load_symbol::<AdsPortOpenEx>(&library, "AdsPortOpenEx")?;
            let port_close = load_symbol::<AdsPortCloseEx>(&library, "AdsPortCloseEx")?;
            let read = load_symbol::<AdsSyncReadReqEx2>(&library, "AdsSyncReadReqEx2")?;
            let write = load_symbol::<AdsSyncWriteReqEx>(&library, "AdsSyncWriteReqEx")?;
            let read_write =
                load_symbol::<AdsSyncReadWriteReqEx2>(&library, "AdsSyncReadWriteReqEx2")?;

            Ok(Self {
                _library: library,
                port_open,
                port_close,
                read,
                write,
                read_write,
            })
        }
    }
}

unsafe fn load_symbol<T>(library: &LibraryHandle, name: &str) -> DriverResult<T>
where
    T: Copy,
{
    unsafe {
        let raw = CString::new(name).expect("static export names do not contain nul");
        let symbol = GetProcAddress(library.0, raw.as_ptr());
        if symbol.is_null() {
            return Err(DriverError::Other(anyhow::anyhow!(
                "TcAdsDll.dll does not export {name}"
            )));
        }
        Ok(std::mem::transmute_copy(&symbol))
    }
}

fn tc_ads_dll_candidates() -> Vec<String> {
    vec![
        "TcAdsDll.dll".to_string(),
        r"C:\TwinCAT\Common64\TcAdsDll.dll".to_string(),
        r"C:\TwinCAT\AdsApi\TcAdsDll\x64\TcAdsDll.dll".to_string(),
        r"C:\TwinCAT\Common32\TcAdsDll.dll".to_string(),
        r"C:\TwinCAT\AdsApi\TcAdsDll\TcAdsDll.dll".to_string(),
    ]
}

fn wide_null(path: &str) -> Vec<u16> {
    OsStr::new(path).encode_wide().chain(Some(0)).collect()
}

/// ADS client routed through the locally installed TwinCAT router.
pub(crate) struct TcAdsClient {
    api: Arc<TcAdsApi>,
    call_lock: Arc<StdMutex<()>>,
    port: i32,
    target_net_id: [u8; 6],
}

impl TcAdsClient {
    pub(crate) fn connect(config: AdsConfig) -> DriverResult<Self> {
        let api = Arc::new(TcAdsApi::load()?);
        let port = unsafe { (api.port_open)() };
        if port <= 0 {
            return Err(DriverError::RemoteFault {
                code: "ads-dll-open-port".to_string(),
                message: format!("AdsPortOpenEx returned {port}"),
            });
        }

        Ok(Self {
            api,
            call_lock: Arc::new(StdMutex::new(())),
            port,
            target_net_id: parse_ams_net_id(&config.ams_net_id)?,
        })
    }

    fn target(&self, port: u16) -> AmsAddr {
        AmsAddr {
            net_id: AmsNetId {
                b: self.target_net_id,
            },
            port,
        }
    }
}

impl Drop for TcAdsClient {
    fn drop(&mut self) {
        let _ = unsafe { (self.api.port_close)(self.port) };
    }
}

#[async_trait]
impl AdsClientLike for TcAdsClient {
    async fn symbol_table(&self, port: u16) -> DriverResult<SymbolTable> {
        let api = self.api.clone();
        let call_lock = self.call_lock.clone();
        let ads_port = self.port;
        let target = self.target(port);
        spawn_blocking_driver(move || {
            let _guard = call_lock
                .lock()
                .map_err(|_| DriverError::Other(anyhow::anyhow!("TcAdsDll mutex poisoned")))?;
            read_symbol_table(&api, ads_port, &target)
        })
        .await
    }

    async fn read_symbol(&self, port: u16, symbol: &str, size: usize) -> DriverResult<Vec<u8>> {
        let api = self.api.clone();
        let call_lock = self.call_lock.clone();
        let ads_port = self.port;
        let target = self.target(port);
        let symbol = symbol.to_string();
        spawn_blocking_driver(move || {
            let _guard = call_lock
                .lock()
                .map_err(|_| DriverError::Other(anyhow::anyhow!("TcAdsDll mutex poisoned")))?;
            read_symbol_value(&api, ads_port, &target, &symbol, size)
        })
        .await
    }

    async fn write_symbol(&self, port: u16, symbol: &str, data: Vec<u8>) -> DriverResult<()> {
        let api = self.api.clone();
        let call_lock = self.call_lock.clone();
        let ads_port = self.port;
        let target = self.target(port);
        let symbol = symbol.to_string();
        spawn_blocking_driver(move || {
            let _guard = call_lock
                .lock()
                .map_err(|_| DriverError::Other(anyhow::anyhow!("TcAdsDll mutex poisoned")))?;
            write_symbol_value(&api, ads_port, &target, &symbol, &data)
        })
        .await
    }

    async fn subscribe_symbols(
        &self,
        entries: Vec<SubscriptionEntry>,
        poll_rate_ms: u64,
        updates: mpsc::UnboundedSender<openwebhmi_driver_api::DriverUpdate>,
    ) -> DriverResult<SubscriptionGuard> {
        let api = self.api.clone();
        let call_lock = self.call_lock.clone();
        let ads_port = self.port;
        let target_net_id = self.target_net_id;
        let stop = Arc::new(AtomicBool::new(false));
        let worker_stop = stop.clone();
        let join = std::thread::spawn(move || {
            while !worker_stop.load(Ordering::Relaxed) {
                for entry in &entries {
                    let target = AmsAddr {
                        net_id: AmsNetId { b: target_net_id },
                        port: entry.port,
                    };
                    let result = call_lock
                        .lock()
                        .map_err(|_| DriverError::Other(anyhow::anyhow!("TcAdsDll mutex poisoned")))
                        .and_then(|_guard| {
                            read_symbol_indexed(
                                &api,
                                ads_port,
                                &target,
                                entry.index_group,
                                entry.index_offset,
                                entry.size,
                            )
                        })
                        .and_then(|bytes| {
                            crate::symbols::decode_ads_value(entry.data_type, &bytes)
                        });
                    let update = update_from_result(entry.raw.clone(), result, now_ms());
                    if updates.send(update).is_err() {
                        return;
                    }
                }
                std::thread::sleep(Duration::from_millis(poll_rate_ms));
            }
        });

        Ok(SubscriptionGuard::backend(PollGuard {
            stop,
            join: StdMutex::new(Some(join)),
        }))
    }
}

struct PollGuard {
    stop: Arc<AtomicBool>,
    join: StdMutex<Option<JoinHandle<()>>>,
}

impl Drop for PollGuard {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Relaxed);
        if let Ok(mut join) = self.join.lock() {
            if let Some(join) = join.take() {
                let _ = join.join();
            }
        }
    }
}

fn read_symbol_table(api: &TcAdsApi, port: i32, target: &AmsAddr) -> DriverResult<SymbolTable> {
    let mut info = [0_u8; 24];
    let mut bytes_read = 0_u32;
    let err = unsafe {
        (api.read)(
            port,
            target,
            ADSIGRP_SYM_UPLOADINFO2,
            0,
            info.len() as u32,
            info.as_mut_ptr().cast(),
            &mut bytes_read,
        )
    };
    if err != 0 {
        let mut legacy = [0_u8; 8];
        read_raw(api, port, target, ADSIGRP_SYM_UPLOADINFO, 0, &mut legacy)?;
        info[..8].copy_from_slice(&legacy);
    } else if bytes_read < 8 {
        return Err(short_read("ADS symbol upload info", bytes_read as usize, 8));
    }

    let symbol_bytes = u32::from_le_bytes(info[4..8].try_into().expect("slice len")) as usize;
    let mut data = vec![0; symbol_bytes];
    read_raw(api, port, target, ADSIGRP_SYM_UPLOAD, 0, &mut data)?;
    parse_symbol_upload(&data)
}

fn parse_symbol_upload(data: &[u8]) -> DriverResult<SymbolTable> {
    let mut offset = 0_usize;
    let mut symbols = Vec::new();
    while offset + 30 <= data.len() {
        let entry = &data[offset..];
        let entry_len = u32_at(entry, 0)? as usize;
        if entry_len == 0 || offset + entry_len > data.len() || entry_len < 30 {
            return Err(DriverError::RemoteFault {
                code: "ads-symbol-upload".to_string(),
                message: format!("invalid symbol entry length {entry_len} at offset {offset}"),
            });
        }

        let index_group = u32_at(entry, 4)?;
        let index_offset = u32_at(entry, 8)?;
        let size = u32_at(entry, 12)? as usize;
        let base_type = u32_at(entry, 16)?;
        let name_len = u16_at(entry, 24)? as usize;
        let type_len = u16_at(entry, 26)? as usize;
        let comment_len = u16_at(entry, 28)? as usize;
        let strings = &entry[30..entry_len];
        let required = name_len + type_len + comment_len + 3;
        if strings.len() < required {
            return Err(DriverError::RemoteFault {
                code: "ads-symbol-upload".to_string(),
                message: format!("truncated symbol strings at offset {offset}"),
            });
        }

        let name = string_field(strings, 0, name_len)?;
        let type_start = name_len + 1;
        let type_name = string_field(strings, type_start, type_len)?;
        symbols.push(SymbolEntry {
            data_type: AdsDataType::from_ads(base_type, &type_name),
            name,
            index_group,
            index_offset,
            type_name,
            size,
            base_type,
        });
        offset += entry_len;
    }

    Ok(SymbolTable::new(symbols))
}

fn read_symbol_value(
    api: &TcAdsApi,
    port: i32,
    target: &AmsAddr,
    symbol: &str,
    size: usize,
) -> DriverResult<Vec<u8>> {
    let handle = symbol_handle(api, port, target, symbol)?;
    let result = read_symbol_indexed(api, port, target, ADSIGRP_SYM_VALBYHND, handle, size);
    let release = release_handle(api, port, target, handle);
    match (result, release) {
        (Ok(bytes), Ok(())) => Ok(bytes),
        (Err(err), _) | (_, Err(err)) => Err(err),
    }
}

fn write_symbol_value(
    api: &TcAdsApi,
    port: i32,
    target: &AmsAddr,
    symbol: &str,
    data: &[u8],
) -> DriverResult<()> {
    let handle = symbol_handle(api, port, target, symbol)?;
    let result = write_raw(api, port, target, ADSIGRP_SYM_VALBYHND, handle, data);
    let release = release_handle(api, port, target, handle);
    result.and(release)
}

fn symbol_handle(api: &TcAdsApi, port: i32, target: &AmsAddr, symbol: &str) -> DriverResult<u32> {
    let mut name = symbol.as_bytes().to_vec();
    name.push(0);
    let mut handle = [0_u8; 4];
    let mut bytes_read = 0_u32;
    let err = unsafe {
        (api.read_write)(
            port,
            target,
            ADSIGRP_SYM_HNDBYNAME,
            0,
            handle.len() as u32,
            handle.as_mut_ptr().cast(),
            name.len() as u32,
            name.as_ptr().cast(),
            &mut bytes_read,
        )
    };
    ads_result(err)?;
    if bytes_read < 4 {
        return Err(short_read("ADS symbol handle", bytes_read as usize, 4));
    }
    Ok(u32::from_le_bytes(handle))
}

fn release_handle(api: &TcAdsApi, port: i32, target: &AmsAddr, handle: u32) -> DriverResult<()> {
    write_raw(
        api,
        port,
        target,
        ADSIGRP_SYM_RELEASEHND,
        0,
        &handle.to_le_bytes(),
    )
}

fn read_symbol_indexed(
    api: &TcAdsApi,
    port: i32,
    target: &AmsAddr,
    index_group: u32,
    index_offset: u32,
    size: usize,
) -> DriverResult<Vec<u8>> {
    let mut data = vec![0; size];
    read_raw(api, port, target, index_group, index_offset, &mut data)?;
    Ok(data)
}

fn read_raw(
    api: &TcAdsApi,
    port: i32,
    target: &AmsAddr,
    index_group: u32,
    index_offset: u32,
    data: &mut [u8],
) -> DriverResult<()> {
    let mut bytes_read = 0_u32;
    let err = unsafe {
        (api.read)(
            port,
            target,
            index_group,
            index_offset,
            data.len() as u32,
            data.as_mut_ptr().cast(),
            &mut bytes_read,
        )
    };
    ads_result(err)?;
    if bytes_read < data.len() as u32 {
        return Err(short_read("ADS read", bytes_read as usize, data.len()));
    }
    Ok(())
}

fn write_raw(
    api: &TcAdsApi,
    port: i32,
    target: &AmsAddr,
    index_group: u32,
    index_offset: u32,
    data: &[u8],
) -> DriverResult<()> {
    ads_result(unsafe {
        (api.write)(
            port,
            target,
            index_group,
            index_offset,
            data.len() as u32,
            data.as_ptr().cast(),
        )
    })
}

fn ads_result(err: i32) -> DriverResult<()> {
    if err == 0 {
        Ok(())
    } else {
        Err(DriverError::RemoteFault {
            code: format!("ads-dll-0x{err:x}"),
            message: format!("TcAdsDll returned ADS error {err}"),
        })
    }
}

fn short_read(context: &str, actual: usize, expected: usize) -> DriverError {
    DriverError::RemoteFault {
        code: "ads-short-read".to_string(),
        message: format!("{context}: expected {expected} bytes, got {actual}"),
    }
}

fn u32_at(bytes: &[u8], offset: usize) -> DriverResult<u32> {
    let slice = bytes
        .get(offset..offset + 4)
        .ok_or_else(|| DriverError::RemoteFault {
            code: "ads-symbol-upload".to_string(),
            message: format!("truncated u32 at offset {offset}"),
        })?;
    Ok(u32::from_le_bytes(slice.try_into().expect("slice len")))
}

fn u16_at(bytes: &[u8], offset: usize) -> DriverResult<u16> {
    let slice = bytes
        .get(offset..offset + 2)
        .ok_or_else(|| DriverError::RemoteFault {
            code: "ads-symbol-upload".to_string(),
            message: format!("truncated u16 at offset {offset}"),
        })?;
    Ok(u16::from_le_bytes(slice.try_into().expect("slice len")))
}

fn string_field(bytes: &[u8], start: usize, len: usize) -> DriverResult<String> {
    let field = bytes
        .get(start..start + len)
        .ok_or_else(|| DriverError::RemoteFault {
            code: "ads-symbol-upload".to_string(),
            message: format!("truncated string at offset {start}"),
        })?;
    String::from_utf8(field.to_vec()).map_err(|err| DriverError::RemoteFault {
        code: "ads-symbol-upload".to_string(),
        message: err.to_string(),
    })
}
