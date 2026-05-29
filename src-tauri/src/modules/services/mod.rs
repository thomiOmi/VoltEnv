use crate::modules::env::prepare_command_env;
use crate::modules::platform::Platform;
use crate::modules::platform::PlatformAdapter;
use serde::Serialize;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tauri::{AppHandle, Emitter};
use tokio::io::AsyncBufReadExt;

// LogPayload — streamed to the frontend in real time

/// A single log line emitted by a running service instance.
///
/// Sent to the frontend via the `"service-log"` Tauri event so the
/// Log Manager dashboard can display stdout / stderr in real time.
#[derive(Clone, Serialize)]
pub struct LogPayload {
    pub service_id: String,
    pub version: String,
    pub message: String,
    pub timestamp: String,
    pub is_error: bool,
}

// InstanceState — per-instance tracking metadata

/// Runtime metadata about a single running service instance.
#[derive(Clone)]
pub struct InstanceState {
    pub pid: u32,
}

// KillStrategy

/// Describes how forcefully a managed process should be terminated.
#[derive(Clone, Copy)]
pub enum KillStrategy {
    /// SIGTERM / `taskkill /PID` — polite request, may be ignored.
    Soft,
    /// SIGKILL / `taskkill /F /T` — immediate tree termination.
    Force,
    /// Soft first, then wait up to 3 s for graceful exit, then Force.
    Hybrid,
}

// Internal helpers

/// Returns an ISO‑8601 timestamp string for the current moment.
fn current_timestamp() -> String {
    chrono::Utc::now().to_rfc3339()
}

/// Polls every 250 ms until `Platform::is_process_alive` returns `false`.
async fn wait_for_exit(pid: u32) {
    let mut interval = tokio::time::interval(std::time::Duration::from_millis(250));
    while Platform::is_process_alive(pid).await {
        interval.tick().await;
    }
}

/// Reads log lines from a child process stream and emits `"service-log"`
/// events to the Tauri frontend.
///
/// Returns a `JoinHandle` so the caller can await the reader's completion
/// before declaring the service as "Stopped" — this ensures all buffered
/// log lines are flushed to the frontend.
///
/// # DOS Protection
///
/// A rate limiter caps IPC event emission at 100 lines per second per stream
/// (stdout or stderr).  When exceeded a single warning line is emitted, and
/// subsequent lines within the same 1 s window are silently dropped.  This
/// prevents a chatty service from flooding the frontend with thousands of
/// IPC events per second and locking up the UI render loop.
fn start_log_reader(
    app: AppHandle,
    service_id: String,
    version: String,
    stream: impl tokio::io::AsyncRead + Unpin + Send + 'static,
    is_error: bool,
) -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move {
        let mut lines = tokio::io::BufReader::new(stream).lines();
        let mut last_tick = tokio::time::Instant::now();
        let mut line_count = 0;
        let mut throttled = false;

        while let Ok(Some(line)) = lines.next_line().await {
            let now = tokio::time::Instant::now();
            if now.duration_since(last_tick).as_secs() >= 1 {
                last_tick = now;
                line_count = 0;
                throttled = false;
            }

            line_count += 1;
            if line_count > 100 {
                if !throttled {
                    throttled = true;
                    let warn_msg = "[VoltEnv] Log rate limit exceeded (>100 lines/s). Throttling logs to protect UI performance...".to_string();
                    let payload = LogPayload {
                        service_id: service_id.clone(),
                        version: version.clone(),
                        message: warn_msg.clone(),
                        timestamp: current_timestamp(),
                        is_error,
                    };
                    let _ = app.emit("service-log", payload);

                    // Persist the throttled warning to disk (fire-and-forget)
                    let app_handle = app.clone();
                    let s_id = service_id.clone();
                    tokio::spawn(async move {
                        if let Err(e) = crate::modules::logger::LogManager::write_service_log(
                            &app_handle,
                            &s_id,
                            &warn_msg,
                        )
                        .await
                        {
                            eprintln!("[VoltEnv Logger Error] {}", e);
                        }
                    });
                }
                continue;
            }

            // Persist this line to disk before the IPC emit so the log file
            // is never out of sync with what the UI displays.
            let log_msg = line.clone();
            let app_handle = app.clone();
            let s_id = service_id.clone();
            tokio::spawn(async move {
                if let Err(e) = crate::modules::logger::LogManager::write_service_log(
                    &app_handle,
                    &s_id,
                    &log_msg,
                )
                .await
                {
                    eprintln!("[VoltEnv Logger Error] {}", e);
                }
            });

            let payload = LogPayload {
                service_id: service_id.clone(),
                version: version.clone(),
                message: line,
                timestamp: current_timestamp(),
                is_error,
            };
            let _ = app.emit("service-log", payload);
        }
    })
}

// ServiceProcesses — multi-instance process manager

/// Manages running service processes, keyed by `"{service_id}:{version}"`.
///
/// Each instance has a dedicated `tokio` watcher that:
/// - Reads stdout/stderr line by line and emits `"service-log"` events.
/// - Waits for the child to exit (internal or external kill).
/// - Cleans up the instance map and emits `"service-status-changed"`.
///
/// Process termination uses `Platform::kill_process` directly — never a
/// cmd / shell wrapper — guaranteeing that process trees are fully reaped
/// and ports are released.
///
/// # Concurrency
///
/// The instance map is wrapped in `Arc<Mutex<...>>` so the background
/// watcher task can clean up the entry when the process exits, even if
/// killed externally.
pub struct ServiceProcesses {
    instances: Arc<Mutex<HashMap<String, InstanceState>>>,
}

impl ServiceProcesses {
    /// Creates an empty process tracker.
    pub fn new() -> Self {
        Self {
            instances: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Spawns a service binary and starts a background watcher.
    ///
    /// The key used internally is `"{id}:{version}"`, allowing multiple
    /// versions of the same service to run concurrently.
    ///
    /// # Behaviour
    ///
    /// 1. Kills any previous instance under the same key (force).
    /// 2. Prepends the central `$ROOT/bin` directory to the child process
    ///    `PATH` via `prepare_command_env` (isolated — no `setx` / profile
    ///    mutation).
    /// 3. Pipes stdout / stderr for real-time log streaming.
    /// 4. Spawns `bin_path` with `args` in `cwd`.
    /// 5. Stores the real OS PID and metadata in `InstanceState`.
    /// 6. Launches a `tokio` task that:
    ///    - Reads log lines (stdout → `is_error: false`, stderr → `is_error: true`).
    ///    - Waits for the child to exit.
    ///    - Removes the instance from tracking.
    ///    - Emits `"service-status-changed"` → `Stopped`.
    pub async fn start(
        &self,
        app: &AppHandle,
        id: &str,
        version: &str,
        bin_path: &str,
        cwd: &str,
        args: &[&str],
    ) -> Result<u32, String> {
        let key = format!("{}:{}", id, version);

        // Kill any previous instance of the same service+version
        let old_pid = {
            let mut instances = self.instances.lock().map_err(|e| e.to_string())?;
            instances.remove(&key).map(|state| state.pid)
        };
        if let Some(pid) = old_pid {
            let _ = Platform::kill_process(pid, true).await;
        }

        let mut cmd = tokio::process::Command::new(bin_path);
        cmd.args(args)
            .current_dir(cwd)
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped());

        prepare_command_env(&mut cmd, app);

        let mut child = cmd
            .spawn()
            .map_err(|e| format!("Failed to spawn {}: {}", id, e))?;

        let pid = child.id().unwrap_or(0);

        // Store instance metadata
        {
            let mut instances = self.instances.lock().map_err(|e| e.to_string())?;
            instances.insert(key.clone(), InstanceState { pid });
        }

        // Notify the frontend that the service is now running
        let _ = app.emit(
            "service-status-changed",
            serde_json::json!({
                "id": id.to_string(),
                "status": "Running",
            }),
        );

        // Prepare shared data for the watcher
        let id_owned = id.to_string();
        let ver_owned = version.to_string();
        let app_handle = app.clone();
        let instances = self.instances.clone();

        // Spawn stdout and stderr log readers, then wait for exit
        tokio::spawn(async move {
            let app_log = app_handle.clone();
            let id_log = id_owned.clone();
            let ver_log = ver_owned.clone();

            let mut stdout_handle = None;
            let mut stderr_handle = None;

            // Read stdout (non-error)
            if let Some(stdout) = child.stdout.take() {
                stdout_handle = Some(start_log_reader(
                    app_log,
                    id_log.clone(),
                    ver_log.clone(),
                    stdout,
                    false,
                ));
            }

            // Read stderr (error)
            if let Some(stderr) = child.stderr.take() {
                stderr_handle = Some(start_log_reader(
                    app_handle.clone(),
                    id_log.clone(),
                    ver_log.clone(),
                    stderr,
                    true,
                ));
            }

            // Wait for the process to exit
            child.wait().await.ok();

            // Drain all remaining buffered log lines before emitting Stopped
            if let Some(h) = stdout_handle {
                let _ = h.await;
            }
            if let Some(h) = stderr_handle {
                let _ = h.await;
            }

            // Cleanup tracking
            instances.lock().ok().and_then(|mut map| map.remove(&key));

            let _ = app_handle.emit(
                "service-status-changed",
                serde_json::json!({
                    "id": id_owned,
                    "status": "Stopped",
                }),
            );
        });

        Ok(pid)
    }

    /// Terminates a tracked instance and removes it from state.
    ///
    /// # Hybrid strategy
    ///
    /// When `strategy` is `Hybrid`:
    /// 1. Soft kill (SIGTERM / `taskkill /PID`).
    /// 2. Wait up to 3 s for graceful exit (poll every 250 ms).
    /// 3. If still alive after timeout → force kill (SIGKILL / `taskkill /F /T`).
    pub async fn stop(
        &self,
        id: &str,
        version: &str,
        strategy: KillStrategy,
    ) -> Result<(), String> {
        let key = format!("{}:{}", id, version);

        let pid = {
            let mut instances = self.instances.lock().map_err(|e| e.to_string())?;
            instances.remove(&key).map(|state| state.pid)
        };

        if let Some(pid) = pid {
            match strategy {
                KillStrategy::Hybrid => {
                    let _ = Platform::kill_process(pid, false).await;

                    let timed_out =
                        tokio::time::timeout(std::time::Duration::from_secs(3), wait_for_exit(pid))
                            .await
                            .is_err();

                    if timed_out && Platform::is_process_alive(pid).await {
                        let _ = Platform::kill_process(pid, true).await;
                    }
                }
                KillStrategy::Soft => {
                    let _ = Platform::kill_process(pid, false).await;
                }
                KillStrategy::Force => {
                    let _ = Platform::kill_process(pid, true).await;
                }
            }
        }

        Ok(())
    }

    /// Returns `true` when any version of the given service ID is currently
    /// tracked as running.
    ///
    /// Used by `switch_service_version` to refuse switching the active OS
    /// version while a process is still alive — switching the junction out
    /// from under a running binary would leave the process stranded.
    pub fn is_any_version_running(&self, id: &str) -> Result<bool, String> {
        let instances = self.instances.lock().map_err(|e| e.to_string())?;
        let prefix = format!("{}:", id);
        Ok(instances.keys().any(|k| k.starts_with(&prefix)))
    }
}

impl Default for ServiceProcesses {
    fn default() -> Self {
        Self::new()
    }
}
