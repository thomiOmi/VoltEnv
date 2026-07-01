pub mod platform;

use crate::utils::{VoltError, VoltResult};
use std::collections::HashMap;
use std::sync::Arc;
use tauri::Emitter;
use tokio::sync::Mutex;

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

pub struct ServiceProcesses {
    pub instances: Arc<Mutex<HashMap<String, InstanceState>>>,
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
        if !Self::is_port_available(port).await {
            return Err(VoltError::Service(format!(
                "Port {} is already in use",
                port
            )));
        }

        let mut cmd = tokio::process::Command::new(bin_path);
        cmd.args(args)
            .current_dir(cwd)
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped());

        let bin_dir = crate::paths::VoltPath::bin_dir(app);
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
            let map = self.instances.lock().await;
            map.get(&key).map(|i| i.restart_count).unwrap_or(0)
        };

        self.instances.lock().await.insert(
            key.clone(),
            InstanceState {
                pid,
                port,
                version: version.to_string(),
                bin_path: bin_path.to_string(),
                cwd: cwd.to_string(),
                args: args.to_vec(),
                manual_stop: false,
                restart_count,
            },
        );

        let _ = app.emit(
            "service-status-changed",
            serde_json::json!({ "id": id, "status": "running", "version": version, "port": port }),
        );

        let stdout = child.stdout.take();
        let stderr = child.stderr.take();
        let app_clone = app.clone();
        let sid = id.to_string();
        let ver = version.to_string();

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

        let instances = self.instances.clone();
        let app_exit = app.clone();
        let exit_id = id.to_string();
        let exit_ver = version.to_string();

        tokio::spawn(async move {
            let _ = child.wait().await;

            let (was_manual, should_restart, metadata) = {
                let mut map = instances.lock().await;
                if let Some(instance) = map.get(&key) {
                    let was_manual = instance.manual_stop;
                    let should_restart = !was_manual && instance.restart_count < 3;
                    let mut meta = instance.clone();
                    if should_restart {
                        meta.restart_count += 1;
                        // Keep instance in map for restart logic
                        map.insert(key.clone(), meta.clone());
                        (was_manual, should_restart, Some(meta))
                    } else {
                        map.remove(&key);
                        (was_manual, false, None)
                    }
                } else {
                    (true, false, None)
                }
            };

            if should_restart {
                if let Some(meta) = metadata {
                    eprintln!(
                        "[voltenv] Service {} crashed. Attempting restart {}/3...",
                        exit_id, meta.restart_count
                    );
                    tokio::time::sleep(std::time::Duration::from_secs(3)).await;

                    let state = app_exit.state::<ServiceProcesses>();
                    let _ = state
                        .start(
                            &app_exit,
                            &exit_id,
                            &exit_ver,
                            &meta.bin_path,
                            &meta.cwd,
                            &meta.args,
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
                if let Some(instance) = map.get_mut(&key) {
                    instance.manual_stop = true;
                    Some(instance.pid)
                } else {
                    None
                }
            };

            if let Some(pid) = pid {
                use platform::PlatformAdapter;
                let _ = platform::Platform::kill_process(pid, false).await;
                tokio::time::sleep(std::time::Duration::from_secs(2)).await;
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
