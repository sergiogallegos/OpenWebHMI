//! CPython worker subprocess management.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::sync::{
    atomic::{AtomicU64, Ordering},
    Arc,
};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use openwebhmi_tag_engine::TagStore;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader, Lines};
use tokio::process::{Child, ChildStdin, ChildStdout, Command};
use tokio::sync::{broadcast, oneshot, Mutex};
use tracing::{info, warn};

use crate::host::ScriptEvent;
use crate::rpc::{
    HostFrame, RpcMethod, TagChangeArgs, TagReadArgs, TagWriteArgs, UtilLogArgs, WorkerFrame,
};
use crate::sink::TagWriteSink;

type TriggerCompletion = oneshot::Sender<Result<(), String>>;
type PendingTriggers = Arc<Mutex<HashMap<String, TriggerCompletion>>>;

/// Timeout used while waiting for the initial worker ready frame.
pub const READY_TIMEOUT: Duration = Duration::from_secs(5);

/// A running CPython script worker.
pub struct WorkerProc {
    script_id: String,
    child: Child,
    stdin: Arc<Mutex<ChildStdin>>,
    reader: Option<Lines<BufReader<ChildStdout>>>,
    pending: PendingTriggers,
    next_id: Arc<AtomicU64>,
}

impl WorkerProc {
    /// Spawn a worker for a script source file.
    pub async fn spawn(
        script_id: impl Into<String>,
        script_path: impl AsRef<Path>,
    ) -> anyhow::Result<Self> {
        let script_id = script_id.into();
        let python = python_command();
        let runner = runner_path();
        let mut child = Command::new(python)
            .arg("-u")
            .arg(runner)
            .arg("--script-id")
            .arg(&script_id)
            .arg(script_path.as_ref())
            .env("PYTHONUNBUFFERED", "1")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .spawn()?;

        let stdin = child
            .stdin
            .take()
            .ok_or_else(|| anyhow::anyhow!("worker stdin unavailable"))?;
        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| anyhow::anyhow!("worker stdout unavailable"))?;
        let mut reader = BufReader::new(stdout).lines();

        let ready = tokio::time::timeout(READY_TIMEOUT, reader.next_line()).await??;
        let Some(line) = ready else {
            anyhow::bail!("worker exited before ready handshake");
        };
        match serde_json::from_str::<WorkerFrame>(&line)? {
            WorkerFrame::Ready {
                script_id: ready_id,
            } if ready_id == script_id => {}
            WorkerFrame::Ready { script_id } => {
                anyhow::bail!("worker ready script_id mismatch: {script_id}")
            }
            frame => anyhow::bail!("expected ready handshake, got {frame:?}"),
        }

        Ok(Self {
            script_id,
            child,
            stdin: Arc::new(Mutex::new(stdin)),
            reader: Some(reader),
            pending: Arc::new(Mutex::new(HashMap::new())),
            next_id: Arc::new(AtomicU64::new(1)),
        })
    }

    /// Start the worker stdout reader loop.
    pub fn start_reader(
        &mut self,
        store: TagStore,
        write_sink: Arc<dyn TagWriteSink>,
        events: broadcast::Sender<ScriptEvent>,
    ) -> anyhow::Result<tokio::task::JoinHandle<()>> {
        let Some(mut reader) = self.reader.take() else {
            anyhow::bail!("worker reader already started");
        };
        let stdin = self.stdin.clone();
        let pending = self.pending.clone();
        let script_id = self.script_id.clone();
        Ok(tokio::spawn(async move {
            while let Ok(Some(line)) = reader.next_line().await {
                let frame = match serde_json::from_str::<WorkerFrame>(&line) {
                    Ok(frame) => frame,
                    Err(err) => {
                        warn!(%script_id, error = %err, line = %line, "invalid worker frame");
                        continue;
                    }
                };
                match frame {
                    WorkerFrame::Ready { .. } => {}
                    WorkerFrame::Rpc { id, method, args } => {
                        let response = handle_rpc(
                            &script_id,
                            &store,
                            write_sink.as_ref(),
                            &events,
                            method,
                            args,
                        )
                        .await;
                        let frame = match response {
                            Ok(result) => HostFrame::RpcResult { id, result },
                            Err(error) => HostFrame::RpcError { id, error },
                        };
                        if let Err(err) = send_frame(&stdin, &frame).await {
                            warn!(%script_id, error = %err, "failed to write rpc response");
                            return;
                        }
                    }
                    WorkerFrame::TriggerDone { id } => {
                        if let Some(tx) = pending.lock().await.remove(&id) {
                            let _ = tx.send(Ok(()));
                        }
                    }
                    WorkerFrame::TriggerError { id, error } => {
                        warn!(%script_id, error = %error, "script trigger failed");
                        let _ = events.send(ScriptEvent::Error {
                            script_id: script_id.clone(),
                            message: error.clone(),
                        });
                        if let Some(tx) = pending.lock().await.remove(&id) {
                            let _ = tx.send(Err(error));
                        }
                    }
                    WorkerFrame::ScriptError { message, traceback } => {
                        warn!(%script_id, %message, ?traceback, "script error");
                        let _ = events.send(ScriptEvent::Error {
                            script_id: script_id.clone(),
                            message,
                        });
                    }
                }
            }
            let mut pending = pending.lock().await;
            for (_, tx) in pending.drain() {
                let _ = tx.send(Err("worker stdout closed".to_string()));
            }
        }))
    }

    /// Invoke an `on_tag_change` handler and wait for completion.
    pub async fn invoke_tag_change(
        &self,
        snapshot: openwebhmi_tag_engine::TagSnapshot,
        timeout: Duration,
    ) -> anyhow::Result<()> {
        let id = format!("trigger-{}", self.next_id.fetch_add(1, Ordering::Relaxed));
        let (tx, rx) = oneshot::channel();
        self.pending.lock().await.insert(id.clone(), tx);
        let frame = HostFrame::Trigger {
            id: id.clone(),
            trigger: "on_tag_change".to_string(),
            args: TagChangeArgs {
                tag_path: snapshot.path,
                value: snapshot.value,
                quality: snapshot.quality,
                ts_ms: snapshot.ts,
            },
        };
        if let Err(err) = send_frame(&self.stdin, &frame).await {
            self.pending.lock().await.remove(&id);
            return Err(err);
        }

        match tokio::time::timeout(timeout, rx).await {
            Ok(Ok(Ok(()))) => Ok(()),
            Ok(Ok(Err(err))) => anyhow::bail!(err),
            Ok(Err(_)) => anyhow::bail!("worker reader stopped before trigger completed"),
            Err(_) => {
                self.pending.lock().await.remove(&id);
                anyhow::bail!("script handler timed out after {} ms", timeout.as_millis())
            }
        }
    }

    /// Wait for the worker process to exit.
    pub async fn wait(&mut self) -> std::io::Result<std::process::ExitStatus> {
        self.child.wait().await
    }

    /// Kill the worker process.
    pub async fn kill(&mut self) -> std::io::Result<()> {
        self.child.kill().await
    }

    /// Return the operating-system process id, when available.
    pub fn id(&self) -> Option<u32> {
        self.child.id()
    }
}

async fn handle_rpc(
    script_id: &str,
    store: &TagStore,
    write_sink: &dyn TagWriteSink,
    events: &broadcast::Sender<ScriptEvent>,
    method: RpcMethod,
    args: serde_json::Value,
) -> Result<serde_json::Value, String> {
    match method {
        RpcMethod::TagRead => {
            let args: TagReadArgs = serde_json::from_value(args).map_err(|err| err.to_string())?;
            Ok(match store.get(&args.path) {
                Some(snapshot) => serde_json::json!({
                    "path": snapshot.path,
                    "value": snapshot.value,
                    "quality": snapshot.quality,
                    "ts_ms": snapshot.ts,
                }),
                None => serde_json::Value::Null,
            })
        }
        RpcMethod::TagWrite => {
            let args: TagWriteArgs = serde_json::from_value(args).map_err(|err| err.to_string())?;
            info!(
                script_id,
                path = %args.path,
                value = ?args.value,
                "script tag write"
            );
            match write_sink.enqueue(&args.path, args.value) {
                Ok(()) => Ok(serde_json::Value::Null),
                Err(err) => {
                    warn!(
                        script_id,
                        path = %args.path,
                        error = %err,
                        "script tag write rejected"
                    );
                    Err(err.to_string())
                }
            }
        }
        RpcMethod::UtilNow => Ok(serde_json::json!(now_ms())),
        RpcMethod::UtilLog => {
            let args: UtilLogArgs = serde_json::from_value(args).map_err(|err| err.to_string())?;
            info!(script_id, message = %args.message, "script log");
            let _ = events.send(ScriptEvent::Log {
                script_id: script_id.to_string(),
                message: args.message,
            });
            Ok(serde_json::Value::Null)
        }
    }
}

async fn send_frame(stdin: &Arc<Mutex<ChildStdin>>, frame: &HostFrame) -> anyhow::Result<()> {
    let mut stdin = stdin.lock().await;
    let line = serde_json::to_string(frame)?;
    stdin.write_all(line.as_bytes()).await?;
    stdin.write_all(b"\n").await?;
    stdin.flush().await?;
    Ok(())
}

fn python_command() -> String {
    std::env::var("OPENWEBHMI_PYTHON").unwrap_or_else(|_| "python3".to_string())
}

fn runner_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("python/_runner.py")
}

fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}
