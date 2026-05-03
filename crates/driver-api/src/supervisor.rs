//! Best-effort driver supervision.

use std::panic::AssertUnwindSafe;
use std::sync::{Arc, RwLock};
use std::time::Duration;

use futures_util::FutureExt;
use openwebhmi_protocol::TagValue;
use rand::Rng;
use serde_json::Value;
use tokio::sync::{mpsc, oneshot};
use tokio::task::JoinHandle;
use tokio::time;
use tracing::{debug, warn};

use crate::{Driver, DriverError, DriverResult, TagAddress, TagNode};

const INITIAL_BACKOFF: Duration = Duration::from_millis(250);
const MAX_BACKOFF: Duration = Duration::from_secs(8);

/// Driver lifecycle state visible to the gateway.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DriverStatus {
    /// Driver is disconnected and no connect attempt is active.
    Disconnected,
    /// Driver is connecting or waiting for reconnect backoff.
    Connecting,
    /// Driver is connected and accepting commands.
    Connected,
    /// Driver faulted. The supervisor may still recover after backoff.
    Faulted {
        /// Human-readable fault reason.
        reason: String,
    },
}

/// Supervisor constructor.
///
/// The supervisor is an in-process, best-effort mitigation. It catches Rust
/// panics observed through async task unwinding when the binary is compiled
/// with `panic = "unwind"`. It does not and cannot contain native faults,
/// aborting panics, undefined behavior, stack overflow, or resource exhaustion;
/// those terminate the gateway process as documented in `docs/architecture.md`
/// §4.4.
pub struct DriverSupervisor;

/// Handle used by gateway code to inspect and command a supervised driver.
pub struct SupervisorHandle {
    status: Arc<RwLock<DriverStatus>>,
    tx: mpsc::Sender<Command>,
    task: JoinHandle<()>,
}

enum Command {
    Read {
        address: TagAddress,
        reply: oneshot::Sender<DriverResult<TagValue>>,
    },
    Write {
        address: TagAddress,
        value: TagValue,
        reply: oneshot::Sender<DriverResult<()>>,
    },
    Browse {
        path: Option<String>,
        reply: oneshot::Sender<DriverResult<Vec<TagNode>>>,
    },
    Shutdown {
        reply: oneshot::Sender<()>,
    },
}

impl DriverSupervisor {
    /// Spawn a supervised driver task and return its handle.
    pub fn spawn(mut driver: Box<dyn Driver>, config: Value) -> SupervisorHandle {
        let (tx, mut rx) = mpsc::channel::<Command>(128);
        let status = Arc::new(RwLock::new(DriverStatus::Disconnected));
        let task_status = Arc::clone(&status);

        let task = tokio::spawn(async move {
            let mut backoff = INITIAL_BACKOFF;
            connect_until_success(driver.as_mut(), config.clone(), &task_status, &mut backoff)
                .await;

            while let Some(command) = rx.recv().await {
                match command {
                    Command::Read { address, reply } => {
                        let result = run_driver_future(
                            AssertUnwindSafe(driver.read(&address)).catch_unwind(),
                            &task_status,
                        )
                        .await;
                        if result.is_err() {
                            recover(driver.as_mut(), config.clone(), &task_status, &mut backoff)
                                .await;
                        }
                        let _ = reply.send(result);
                    }
                    Command::Write {
                        address,
                        value,
                        reply,
                    } => {
                        let result = run_driver_future(
                            AssertUnwindSafe(driver.write(&address, value)).catch_unwind(),
                            &task_status,
                        )
                        .await;
                        if result.is_err() {
                            recover(driver.as_mut(), config.clone(), &task_status, &mut backoff)
                                .await;
                        }
                        let _ = reply.send(result);
                    }
                    Command::Browse { path, reply } => {
                        let result = run_driver_future(
                            AssertUnwindSafe(driver.browse(path.as_deref())).catch_unwind(),
                            &task_status,
                        )
                        .await;
                        if result.is_err() {
                            recover(driver.as_mut(), config.clone(), &task_status, &mut backoff)
                                .await;
                        }
                        let _ = reply.send(result);
                    }
                    Command::Shutdown { reply } => {
                        let _ = driver.disconnect().await;
                        set_status(&task_status, DriverStatus::Disconnected);
                        let _ = reply.send(());
                        break;
                    }
                }
            }
        });

        SupervisorHandle { status, tx, task }
    }
}

impl SupervisorHandle {
    /// Return the current driver status.
    pub fn status(&self) -> DriverStatus {
        self.status
            .read()
            .map(|status| status.clone())
            .unwrap_or_else(|_| DriverStatus::Faulted {
                reason: "status lock poisoned".to_string(),
            })
    }

    /// Read through the supervised driver.
    pub async fn read(&self, address: TagAddress) -> DriverResult<TagValue> {
        let (reply, rx) = oneshot::channel();
        self.tx
            .send(Command::Read { address, reply })
            .await
            .map_err(|_| DriverError::NotConnected)?;
        rx.await.map_err(|_| DriverError::NotConnected)?
    }

    /// Write through the supervised driver.
    pub async fn write(&self, address: TagAddress, value: TagValue) -> DriverResult<()> {
        let (reply, rx) = oneshot::channel();
        self.tx
            .send(Command::Write {
                address,
                value,
                reply,
            })
            .await
            .map_err(|_| DriverError::NotConnected)?;
        rx.await.map_err(|_| DriverError::NotConnected)?
    }

    /// Browse through the supervised driver.
    pub async fn browse(&self, path: Option<String>) -> DriverResult<Vec<TagNode>> {
        let (reply, rx) = oneshot::channel();
        self.tx
            .send(Command::Browse { path, reply })
            .await
            .map_err(|_| DriverError::NotConnected)?;
        rx.await.map_err(|_| DriverError::NotConnected)?
    }

    /// Gracefully stop the supervised driver.
    pub async fn shutdown(self) {
        let (reply, rx) = oneshot::channel();
        let _ = self.tx.send(Command::Shutdown { reply }).await;
        let _ = rx.await;
        let _ = self.task.await;
    }
}

async fn connect_until_success(
    driver: &mut dyn Driver,
    config: Value,
    status: &Arc<RwLock<DriverStatus>>,
    backoff: &mut Duration,
) {
    loop {
        set_status(status, DriverStatus::Connecting);
        match AssertUnwindSafe(driver.connect(config.clone()))
            .catch_unwind()
            .await
        {
            Ok(Ok(())) => {
                *backoff = INITIAL_BACKOFF;
                set_status(status, DriverStatus::Connected);
                return;
            }
            Ok(Err(err)) => {
                warn!(error = %err, "driver connect failed");
            }
            Err(_) => {
                set_status(
                    status,
                    DriverStatus::Faulted {
                        reason: "driver panicked during connect".to_string(),
                    },
                );
            }
        }

        sleep_backoff(backoff).await;
    }
}

async fn recover(
    driver: &mut dyn Driver,
    config: Value,
    status: &Arc<RwLock<DriverStatus>>,
    backoff: &mut Duration,
) {
    let _ = driver.disconnect().await;
    connect_until_success(driver, config, status, backoff).await;
}

async fn run_driver_future<T>(
    future: impl std::future::Future<Output = Result<DriverResult<T>, Box<dyn std::any::Any + Send>>>,
    status: &Arc<RwLock<DriverStatus>>,
) -> DriverResult<T> {
    match future.await {
        Ok(result) => result,
        Err(_) => {
            set_status(
                status,
                DriverStatus::Faulted {
                    reason: "driver task panicked".to_string(),
                },
            );
            Err(DriverError::Other(anyhow::anyhow!("driver task panicked")))
        }
    }
}

async fn sleep_backoff(backoff: &mut Duration) {
    let delay = jittered(*backoff);
    *backoff = (*backoff * 2).min(MAX_BACKOFF);
    debug!(?delay, "driver reconnect backoff");
    time::sleep(delay).await;
}

fn jittered(duration: Duration) -> Duration {
    let base_ms = duration.as_millis() as u64;
    let spread = base_ms / 4;
    let jitter = if spread == 0 {
        0_i64
    } else {
        rand::thread_rng().gen_range(-(spread as i64)..=(spread as i64))
    };
    Duration::from_millis(base_ms.saturating_add_signed(jitter))
}

fn set_status(status: &Arc<RwLock<DriverStatus>>, next: DriverStatus) {
    if let Ok(mut guard) = status.write() {
        *guard = next;
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
    use std::sync::{Arc, Mutex};

    use async_trait::async_trait;
    use openwebhmi_protocol::TagValue;
    use serde_json::json;
    use tokio::time::{sleep, timeout};

    use super::*;
    use crate::{Capabilities, DriverMetadata, trait_def::make_metadata};

    #[derive(Clone)]
    struct TestDriver {
        connect_failures: Arc<AtomicUsize>,
        panic_reads: Arc<AtomicBool>,
        values: Arc<Mutex<HashMap<String, TagValue>>>,
    }

    impl TestDriver {
        fn new() -> Self {
            let mut values = HashMap::new();
            values.insert("Counter".to_string(), TagValue::Int(1));
            Self {
                connect_failures: Arc::new(AtomicUsize::new(0)),
                panic_reads: Arc::new(AtomicBool::new(false)),
                values: Arc::new(Mutex::new(values)),
            }
        }
    }

    #[async_trait]
    impl Driver for TestDriver {
        fn metadata(&self) -> DriverMetadata {
            make_metadata("test", "mock", "0.0.0", Capabilities::NONE)
        }

        async fn connect(&mut self, _config: serde_json::Value) -> DriverResult<()> {
            let remaining = self.connect_failures.load(Ordering::SeqCst);
            if remaining > 0 {
                self.connect_failures.fetch_sub(1, Ordering::SeqCst);
                return Err(DriverError::Connecting);
            }
            Ok(())
        }

        async fn disconnect(&mut self) -> DriverResult<()> {
            Ok(())
        }

        async fn browse(&self, _path: Option<&str>) -> DriverResult<Vec<TagNode>> {
            Ok(Vec::new())
        }

        async fn read(&self, address: &TagAddress) -> DriverResult<TagValue> {
            if self.panic_reads.swap(false, Ordering::SeqCst) {
                panic!("intentional read panic");
            }
            self.values
                .lock()
                .map_err(|_| DriverError::Other(anyhow::anyhow!("lock poisoned")))?
                .get(&address.raw)
                .cloned()
                .ok_or_else(|| DriverError::InvalidAddress(address.raw.clone()))
        }

        async fn write(&self, address: &TagAddress, value: TagValue) -> DriverResult<()> {
            self.values
                .lock()
                .map_err(|_| DriverError::Other(anyhow::anyhow!("lock poisoned")))?
                .insert(address.raw.clone(), value);
            Ok(())
        }
    }

    #[tokio::test]
    async fn supervisor_retries_connect_until_success() {
        let driver = TestDriver::new();
        driver.connect_failures.store(2, Ordering::SeqCst);
        let handle = DriverSupervisor::spawn(Box::new(driver), json!({}));

        timeout(Duration::from_secs(2), async {
            loop {
                if handle.status() == DriverStatus::Connected {
                    break;
                }
                sleep(Duration::from_millis(20)).await;
            }
        })
        .await
        .expect("driver should connect after retries");

        handle.shutdown().await;
    }

    #[tokio::test]
    async fn supervisor_recovers_after_read_panic() {
        let driver = TestDriver::new();
        driver.panic_reads.store(true, Ordering::SeqCst);
        let handle = DriverSupervisor::spawn(Box::new(driver), json!({}));

        timeout(Duration::from_secs(1), async {
            while handle.status() != DriverStatus::Connected {
                sleep(Duration::from_millis(10)).await;
            }
        })
        .await
        .unwrap();

        let result = handle.read(TagAddress::new("Counter")).await;
        assert!(result.is_err());

        timeout(Duration::from_secs(1), async {
            loop {
                if handle.status() == DriverStatus::Connected {
                    break;
                }
                sleep(Duration::from_millis(10)).await;
            }
        })
        .await
        .expect("driver should reconnect after panic");

        let value = handle.read(TagAddress::new("Counter")).await.unwrap();
        assert_eq!(value, TagValue::Int(1));
        handle.shutdown().await;
    }
}
