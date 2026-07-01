use tauri::Manager;

pub mod platform;

use crate::utils::{VoltError, VoltResult};
use std::collections::HashMap;
use std::sync::Arc;
use tauri::Emitter;
use tokio::sync::Mutex;
use futures_util::future::{BoxFuture, FutureExt};

#[derive(Clone, Debug)]
pub struct InstanceState {
    pub pid: u32,
    pub port: u16,
    pub version: String,
    pub bin_path: String,
    pub cwd: String,
    pub args: Vec<String>,
    pub manual_stop: bool,
    pub restart_count: u32,
}

#[derive(Clone)]
pub struct ServiceProcesses {
    pub instances: Arc<Mutex<HashMap<String, InstanceState>>>,
}

impl Default for ServiceProcesses {
    fn default() -> Self {
        Self::new()
    }
}
impl ServiceProcesses {
    pub fn new() -> Self {
        Self {
            instances: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    async fn is_port_available(port: u16) -> bool {
        tokio::net::TcpListener::bind(("127.0.0.1", port))
            .await
            .is_ok()
    }

    #[allow(clippy::too_many_arguments)]
    pub fn start(
        &self,
        app: tauri::AppHandle,
        id: String,
        version: String,
        bin_path: String,
        cwd: String,
        args: Vec<String>,
        port: u16,
    ) -> BoxFuture<'static, VoltResult<u32>> {
        let instances = self.instances.clone();
        async move {
            if !Self::is_port_available(port).await {
                return Err(VoltError::Service(format!(
                    "Port {} is already in use",
                    port
                )));
            }

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

            let mut child = cmd
                .spawn()
                .map_err(|e| VoltError::Process(format!("Failed to spawn {}: {}", id, e)))?;
            let pid = child
                .id()
                .ok_or_else(|| VoltError::Process("Failed to get PID".to_string()))?;

            let key = format!("{}:{}", id, version);

            let restart_count = {
                let map = instances.lock().await;
                map.get(&key).map(|i| i.restart_count).unwrap_or(0)
            };

            {
                let mut map = instances.lock().await;
                map.insert(
                    key.clone(),
                    InstanceState {
                        pid,
                        port,
                        version: version.clone(),
                        bin_path: bin_path.clone(),
                        cwd: cwd.clone(),
                        args: args.clone(),
                        manual_stop: false,
                        restart_count,
                    },
                );
            }

            let _ = app.emit(
                "service-status-changed",
                serde_json::json!({ "id": id, "status": "running", "version": version, "port": port }),
            );

            if let Some(out) = child.stdout.take() {
                platform::start_log_reader(app.clone(), &id, &version, out, false);
            }
            if let Some(err) = child.stderr.take() {
                platform::start_log_reader_err(app.clone(), &id, &version, err, true);
            }

            let instances_clone = instances.clone();
            let app_exit = app.clone();
            let exit_id = id.clone();
            let exit_ver = version.clone();

            tokio::spawn(async move {
                let _ = child.wait().await;

                let (was_manual, should_restart, metadata) = {
                    let mut map = instances_clone.lock().await;
                    if let Some(instance) = map.get(&key) {
                        let was_manual = instance.manual_stop;
                        let should_restart = !was_manual && instance.restart_count < 10;
                        let mut meta = instance.clone();
                        if should_restart {
                            meta.restart_count += 1;
                            map.insert(key.clone(), meta.clone());
                            (was_manual, should_restart, Some(meta))
                        } else {
                            map.remove(&key);
                            (was_manual, false, None)
                        }
                    } else {
                        (false, false, None)
                    }
                };

                if should_restart {
                    if let Some(meta) = metadata {
                        println!(
                            "[voltenv] Service {} crashed. Restarting (attempt {})...",
                            exit_id, meta.restart_count
                        );
                        tokio::time::sleep(std::time::Duration::from_secs(3)).await;

                        let state = {
                            let s = app_exit.state::<ServiceProcesses>();
                            s.inner().clone()
                        };

                        let _ = state
                            .start(
                                app_exit.clone(),
                                exit_id.clone(),
                                exit_ver.clone(),
                                meta.bin_path,
                                meta.cwd,
                                meta.args,
                                meta.port,
                            )
                            .await;
                        return;
                    }
                }

                if !was_manual {
                    let _ = app_exit.emit(
                        "service-status-changed",
                        serde_json::json!({ "id": exit_id, "status": "stopped", "version": exit_ver }),
                    );
                }
            });

            Ok(pid)
        }.boxed()
    }

    pub async fn stop(&self, id: &str, version: &str) -> VoltResult<()> {
        let key = format!("{}:{}", id, version);
        let pid = {
            let mut map = self.instances.lock().await;
            if let Some(instance) = map.get_mut(&key) {
                instance.manual_stop = true;
                instance.pid
            } else {
                return Ok(());
            }
        };

        let _ = <platform::Platform as platform::PlatformAdapter>::kill_process(pid, false).await;
        Ok(())
    }

    pub async fn stop_all_by_id(&self, id: &str) -> VoltResult<()> {
        let mut pids = Vec::new();
        {
            let mut map = self.instances.lock().await;
            for (key, instance) in map.iter_mut() {
                if key.starts_with(&format!("{}:", id)) {
                    instance.manual_stop = true;
                    pids.push(instance.pid);
                }
            }
        }

        for pid in pids {
            let _ = <platform::Platform as platform::PlatformAdapter>::kill_process(pid, false).await;
        }
        Ok(())
    }

    pub async fn get_status(&self, id: &str, version: &str) -> String {
        let key = format!("{}:{}", id, version);
        let map = self.instances.lock().await;
        if map.contains_key(&key) {
            "running".to_string()
        } else {
            "stopped".to_string()
        }
    }
}
