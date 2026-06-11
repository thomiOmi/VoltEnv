pub mod platform;

use std::collections::HashMap;
use std::sync::Arc;
use tauri::Emitter;
use tokio::sync::Mutex;
use crate::errors::VoltResult;

#[derive(Clone, Debug)]
pub struct InstanceState {
    pub pid: u32,
    pub port: u16,
    pub version: String,
    pub bin_path: String,
    pub cwd: String,
    pub args: Vec<String>,
    pub restart_count: u32,
    pub manual_stop: bool,
}

pub struct ServiceProcesses {
    pub instances: Arc<Mutex<HashMap<String, InstanceState>>>,
}

impl ServiceProcesses {
    pub fn new() -> Self {
        Self {
            instances: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub async fn start(
        &self,
        app: &tauri::AppHandle,
        id: &str,
        version: &str,
        bin_path: &str,
        cwd: &str,
        args: &[String],
        port: u16,
    ) -> VoltResult<u32> {
        self.start_internal(app, id, version, bin_path, cwd, args, port, 0).await
    }

    fn start_internal(
        &self,
        app: &tauri::AppHandle,
        id: &str,
        version: &str,
        bin_path: &str,
        cwd: &str,
        args: &[String],
        port: u16,
        restart_count: u32,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = VoltResult<u32>> + Send + '_>> {
        let app = app.clone();
        let id = id.to_string();
        let version = version.to_string();
        let bin_path = bin_path.to_string();
        let cwd = cwd.to_string();
        let args = args.to_vec();

        Box::pin(async move {
            let mut cmd = tokio::process::Command::new(&bin_path);
            cmd.args(&args)
                .current_dir(&cwd)
                .stdout(std::process::Stdio::piped())
                .stderr(std::process::Stdio::piped());

            let bin_dir = crate::paths::VoltPath::bin_dir(&app);
            let current_path = std::env::var("PATH").unwrap_or_default();
            cmd.env(
                "PATH",
                format!("{}{}{}", bin_dir.display(), crate::path_sep(), current_path),
            );

            let mut child = cmd.spawn().map_err(|e| format!("Failed to spawn: {}", e))?;
            let pid = child.id().ok_or("Failed to get PID")?;

            let key = format!("{}:{}", id, version);
            self.instances.lock().await.insert(
                key.clone(),
                InstanceState {
                    pid,
                    port,
                    version: version.clone(),
                    bin_path: bin_path.clone(),
                    cwd: cwd.clone(),
                    args: args.clone(),
                    restart_count,
                    manual_stop: false,
                },
            );

            let _ = app.emit(
                "service-status-changed",
                serde_json::json!({ "id": id, "status": "running", "version": version }),
            );

            let stdout = child.stdout.take();
            let stderr = child.stderr.take();
            let app_clone = app.clone();
            let sid = id.clone();
            let ver = version.clone();

            if let Some(out) = stdout {
                let a = app_clone.clone();
                let i = sid.clone();
                let v = ver.clone();
                tokio::spawn(async move {
                    let h = platform::start_log_reader(a, &i, &v, out, false);
                    let _ = h.await;
                });
            }
            if let Some(err) = stderr {
                let a = app_clone.clone();
                let i = sid.clone();
                let v = ver.clone();
                tokio::spawn(async move {
                    let h = platform::start_log_reader_err(a, &i, &v, err, true);
                    let _ = h.await;
                });
            }

            let instances_arc = self.instances.clone();
            let app_exit = app.clone();
            let exit_id = id.clone();
            let exit_ver = version.clone();

            // For auto-restart, we need a way to call start_internal again
            // We'll use a clone of ServiceProcesses (which is just an Arc wrap anyway)
            let service_proc = ServiceProcesses { instances: self.instances.clone() };

            tokio::spawn(async move {
                let _ = child.wait().await;

                let mut instances = instances_arc.lock().await;
                let key = format!("{}:{}", exit_id, exit_ver);

                if let Some(state) = instances.get(&key).cloned() {
                    if !state.manual_stop && state.restart_count < 3 {
                        instances.remove(&key);
                        drop(instances);

                        let next_restart = state.restart_count + 1;
                        let _ = app_exit.emit("system-log", serde_json::json!({
                            "level": "warn",
                            "message": format!("Service {} crashed. Restarting (attempt {}/3)...", exit_id, next_restart),
                            "timestamp": chrono::Local::now().to_rfc3339()
                        }));

                        tokio::time::sleep(std::time::Duration::from_secs(3)).await;
                        let _ = service_proc.start_internal(
                            &app_exit,
                            &exit_id,
                            &exit_ver,
                            &state.bin_path,
                            &state.cwd,
                            &state.args,
                            state.port,
                            next_restart
                        ).await;
                    } else {
                        instances.remove(&key);
                        let _ = app_exit.emit(
                            "service-status-changed",
                            serde_json::json!({ "id": exit_id, "status": "stopped", "version": exit_ver }),
                        );
                        if !state.manual_stop {
                            let _ = app_exit.emit("system-log", serde_json::json!({
                                "level": "error",
                                "message": format!("Service {} stopped unexpectedly after {} restart attempts.", exit_id, state.restart_count),
                                "timestamp": chrono::Local::now().to_rfc3339()
                            }));
                        }
                    }
                }
            });

            Ok(pid)
        })
    }

    pub async fn stop_all_by_id(&self, id: &str) -> VoltResult<()> {
        let keys: Vec<String> = {
            let map = self.instances.lock().await;
            map.keys()
                .filter(|k| k.starts_with(&format!("{}:", id)))
                .cloned()
                .collect()
        };

        for key in keys {
            let pid = {
                let mut map = self.instances.lock().await;
                if let Some(state) = map.get_mut(&key) {
                    state.manual_stop = true;
                    Some(state.pid)
                } else {
                    None
                }
            };

            if let Some(pid) = pid {
                use platform::PlatformAdapter;
                let _ = platform::Platform::kill_process(pid, false).await;
                tokio::time::sleep(std::time::Duration::from_secs(1)).await;
                if platform::Platform::is_process_alive(pid).await {
                    let _ = platform::Platform::kill_process(pid, true).await;
                }
                self.instances.lock().await.remove(&key);
            }
        }

        Ok(())
    }

    pub async fn is_any_running(&self) -> bool {
        !self.instances.lock().await.is_empty()
    }
}

impl Default for ServiceProcesses {
    fn default() -> Self {
        Self::new()
    }
}
