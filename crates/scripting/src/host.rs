//! Script host orchestration.

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use openwebhmi_project_store::ScriptConfig;
use openwebhmi_tag_engine::TagStore;
use rand::Rng;
use tokio::sync::{broadcast, mpsc, oneshot};
use tracing::{info, warn};

use crate::TagWriteSink;
use crate::triggers::TriggerRegistration;
use crate::worker::WorkerProc;

const INITIAL_BACKOFF: Duration = Duration::from_millis(250);
const MAX_BACKOFF: Duration = Duration::from_secs(8);

/// Default per-handler timeout.
pub const DEFAULT_TIMEOUT: Duration = Duration::from_secs(5);

/// Script host runtime options.
#[derive(Debug, Clone)]
pub struct ScriptHostOptions {
    /// Default per-handler timeout.
    pub handler_timeout: Duration,
}

impl Default for ScriptHostOptions {
    fn default() -> Self {
        Self {
            handler_timeout: DEFAULT_TIMEOUT,
        }
    }
}

/// Current script lifecycle status.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ScriptStatus {
    /// Worker is starting.
    Starting,
    /// Worker is ready.
    Ready,
    /// Worker is backing off before restart.
    Restarting,
    /// Worker has stopped because the host is shutting down.
    Stopped,
}

/// Script host event stream item.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ScriptEvent {
    /// Script status changed.
    Status {
        /// Project id.
        project_id: String,
        /// Script id.
        script_id: String,
        /// New status.
        status: ScriptStatus,
    },
    /// Script wrote `system.util.log`.
    Log {
        /// Project id.
        project_id: String,
        /// Script id.
        script_id: String,
        /// Log message.
        message: String,
    },
    /// Script produced an error.
    Error {
        /// Project id.
        project_id: String,
        /// Script id.
        script_id: String,
        /// Error message.
        message: String,
    },
}

/// Running host for project scripts.
pub struct ScriptHost {
    control: mpsc::Sender<ControlMessage>,
    events: broadcast::Sender<ScriptEvent>,
    task: tokio::task::JoinHandle<()>,
}

/// Cloneable handle to a running script host.
#[derive(Clone)]
pub struct ScriptHostHandle {
    control: mpsc::Sender<ControlMessage>,
    events: broadcast::Sender<ScriptEvent>,
}

#[derive(Debug)]
enum ControlMessage {
    Kill {
        script_id: String,
        response: oneshot::Sender<anyhow::Result<()>>,
    },
    Shutdown,
}

impl ScriptHost {
    /// Spawn a script host for the supplied project script configs.
    pub fn spawn(
        project_id: impl Into<String>,
        store: TagStore,
        write_sink: Arc<dyn TagWriteSink>,
        scripts: Vec<ScriptConfig>,
        options: ScriptHostOptions,
    ) -> ScriptHost {
        let project_id = project_id.into();
        let (control, control_rx) = mpsc::channel(16);
        let (events, _) = broadcast::channel(1024);
        let task_events = events.clone();
        let task_control = control.clone();
        let task = tokio::spawn(async move {
            run_host(
                project_id,
                store,
                write_sink,
                scripts,
                options,
                control_rx,
                task_events,
            )
            .await;
        });
        ScriptHost {
            control: task_control,
            events,
            task,
        }
    }

    /// Return a cloneable handle for tests and embedding code.
    pub fn handle(&self) -> ScriptHostHandle {
        ScriptHostHandle {
            control: self.control.clone(),
            events: self.events.clone(),
        }
    }

    /// Subscribe to script events.
    pub fn subscribe_events(&self) -> broadcast::Receiver<ScriptEvent> {
        self.events.subscribe()
    }

    /// Stop the host task.
    pub async fn shutdown(self) {
        let _ = self.control.send(ControlMessage::Shutdown).await;
        let _ = self.task.await;
    }
}

impl ScriptHostHandle {
    /// Subscribe to script events.
    pub fn subscribe_events(&self) -> broadcast::Receiver<ScriptEvent> {
        self.events.subscribe()
    }

    /// Kill one script worker. The supervisor will restart it.
    pub async fn kill_worker(&self, script_id: impl Into<String>) -> anyhow::Result<()> {
        let (response, rx) = oneshot::channel();
        self.control
            .send(ControlMessage::Kill {
                script_id: script_id.into(),
                response,
            })
            .await?;
        rx.await?
    }
}

async fn run_host(
    project_id: String,
    store: TagStore,
    write_sink: Arc<dyn TagWriteSink>,
    scripts: Vec<ScriptConfig>,
    options: ScriptHostOptions,
    mut control: mpsc::Receiver<ControlMessage>,
    events: broadcast::Sender<ScriptEvent>,
) {
    let mut worker_controls = HashMap::new();
    for script in scripts.into_iter().filter(|script| script.enabled) {
        let (tx, rx) = mpsc::channel(8);
        worker_controls.insert(script.id.clone(), tx);
        tokio::spawn(run_script_supervisor(
            store.clone(),
            write_sink.clone(),
            project_id.clone(),
            script,
            options.clone(),
            events.clone(),
            rx,
        ));
    }

    while let Some(message) = control.recv().await {
        match message {
            ControlMessage::Kill {
                script_id,
                response,
            } => {
                let result = match worker_controls.get(&script_id) {
                    Some(tx) => {
                        let (worker_response, rx) = oneshot::channel();
                        if tx
                            .send(WorkerControl::Kill {
                                response: worker_response,
                            })
                            .await
                            .is_err()
                        {
                            Err(anyhow::anyhow!("script worker supervisor stopped"))
                        } else {
                            rx.await.unwrap_or_else(|_| {
                                Err(anyhow::anyhow!("script worker response channel closed"))
                            })
                        }
                    }
                    None => Err(anyhow::anyhow!("unknown script id '{script_id}'")),
                };
                let _ = response.send(result);
            }
            ControlMessage::Shutdown => {
                for tx in worker_controls.values() {
                    let _ = tx.send(WorkerControl::Shutdown).await;
                }
                return;
            }
        }
    }
}

#[derive(Debug)]
enum WorkerControl {
    Kill {
        response: oneshot::Sender<anyhow::Result<()>>,
    },
    Shutdown,
}

async fn run_script_supervisor(
    store: TagStore,
    write_sink: Arc<dyn TagWriteSink>,
    project_id: String,
    script: ScriptConfig,
    options: ScriptHostOptions,
    events: broadcast::Sender<ScriptEvent>,
    mut control: mpsc::Receiver<WorkerControl>,
) {
    let tag_paths = TriggerRegistration::tag_change_paths(&script);
    if tag_paths.is_empty() {
        info!(script_id = %script.id, "script has no v1 on_tag_change triggers");
        return;
    }

    let timeout = script
        .handler_timeout_ms
        .map(Duration::from_millis)
        .unwrap_or(options.handler_timeout);
    let mut backoff = INITIAL_BACKOFF;
    loop {
        let _ = events.send(ScriptEvent::Status {
            project_id: project_id.clone(),
            script_id: script.id.clone(),
            status: ScriptStatus::Starting,
        });
        match WorkerProc::spawn(&script.id, PathBuf::from(&script.path)).await {
            Ok(mut worker) => {
                info!(
                    script_id = %script.id,
                    pid = ?worker.id(),
                    "script worker ready"
                );
                let _ = events.send(ScriptEvent::Status {
                    project_id: project_id.clone(),
                    script_id: script.id.clone(),
                    status: ScriptStatus::Ready,
                });
                backoff = INITIAL_BACKOFF;
                let runtime = ReadyWorkerRuntime {
                    store: &store,
                    write_sink: write_sink.clone(),
                    project_id: &project_id,
                    script_id: &script.id,
                    tag_paths: &tag_paths,
                    timeout,
                    events: &events,
                };
                match run_ready_worker(runtime, &mut worker, &mut control).await {
                    Err(err) => {
                        warn!(script_id = %script.id, error = %err, "script worker restarting");
                    }
                    _ => {
                        let _ = events.send(ScriptEvent::Status {
                            project_id: project_id.clone(),
                            script_id: script.id.clone(),
                            status: ScriptStatus::Stopped,
                        });
                        return;
                    }
                }
            }
            Err(err) => warn!(script_id = %script.id, error = %err, "script worker spawn failed"),
        }

        let _ = events.send(ScriptEvent::Status {
            project_id: project_id.clone(),
            script_id: script.id.clone(),
            status: ScriptStatus::Restarting,
        });
        tokio::time::sleep(jittered(backoff)).await;
        backoff = (backoff * 2).min(MAX_BACKOFF);
    }
}

struct ReadyWorkerRuntime<'a> {
    store: &'a TagStore,
    write_sink: Arc<dyn TagWriteSink>,
    project_id: &'a str,
    script_id: &'a str,
    tag_paths: &'a [String],
    timeout: Duration,
    events: &'a broadcast::Sender<ScriptEvent>,
}

async fn run_ready_worker(
    runtime: ReadyWorkerRuntime<'_>,
    worker: &mut WorkerProc,
    control: &mut mpsc::Receiver<WorkerControl>,
) -> anyhow::Result<()> {
    let _reader = worker.start_reader(
        runtime.store.clone(),
        runtime.write_sink,
        runtime.project_id.to_string(),
        runtime.events.clone(),
    )?;
    let mut receivers = runtime
        .tag_paths
        .iter()
        .map(|path| (path.clone(), runtime.store.subscribe(path)))
        .collect::<Vec<_>>();

    loop {
        tokio::select! {
            biased;
            message = control.recv() => {
                match message {
                    Some(WorkerControl::Kill { response }) => {
                        let result = worker.kill().await.map_err(Into::into);
                        let _ = response.send(result);
                        return Err(anyhow::anyhow!("worker killed by control request"));
                    }
                    Some(WorkerControl::Shutdown) | None => {
                        let _ = worker.kill().await;
                        return Ok(());
                    }
                }
            }
            status = worker.wait() => {
                let status = status?;
                return Err(anyhow::anyhow!("worker exited with {status}"));
            }
            result = recv_any(&mut receivers) => {
                let snapshot = result?;
                if let Err(err) = worker.invoke_tag_change(snapshot, runtime.timeout).await {
                    warn!(script_id = %runtime.script_id, error = %err, "script handler failed");
                    let _ = runtime.events.send(ScriptEvent::Error {
                        project_id: runtime.project_id.to_string(),
                        script_id: runtime.script_id.to_string(),
                        message: err.to_string(),
                    });
                    let _ = worker.kill().await;
                    return Err(err);
                }
            }
        }
    }
}

async fn recv_any(
    receivers: &mut [(
        String,
        tokio::sync::broadcast::Receiver<openwebhmi_tag_engine::TagSnapshot>,
    )],
) -> anyhow::Result<openwebhmi_tag_engine::TagSnapshot> {
    loop {
        for (_, rx) in receivers.iter_mut() {
            match rx.try_recv() {
                Ok(snapshot) => return Ok(snapshot),
                Err(tokio::sync::broadcast::error::TryRecvError::Empty) => {}
                Err(tokio::sync::broadcast::error::TryRecvError::Lagged(_)) => {}
                Err(tokio::sync::broadcast::error::TryRecvError::Closed) => {
                    anyhow::bail!("tag subscription closed")
                }
            }
        }
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
}

fn jittered(duration: Duration) -> Duration {
    let millis = duration.as_millis() as u64;
    if millis == 0 {
        return duration;
    }
    let min = millis.saturating_mul(75) / 100;
    let max = millis.saturating_mul(125) / 100;
    Duration::from_millis(rand::thread_rng().gen_range(min..=max))
}
