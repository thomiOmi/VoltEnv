pub mod platform;

use std::collections::HashMap;
use std::sync::Arc;
use tauri::Emitter;
use tokio::sync::Mutex;

#[derive(Clone, Debug)]
pub struct InstanceState {
    pub pid: u32,
    pub port: u16,
    pub version: String,
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
    ) -> Result<u32, String> {
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

        let mut child = cmd.spawn().map_err(|e| format!("Failed to spawn: {}", e))?;
        let pid = child.id().ok_or("Failed to get PID")?;

        self.instances.lock().await.insert(
            format!("{}:{}", id, version),
            InstanceState {
                pid,
                port,
                version: version.to_string(),
            },
        );

        let _ = app.emit(
            "service-status-changed",
            serde_json::json!({ "id": id, "status": "running", "version": version }),
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
            instances
                .lock()
                .await
                .remove(&format!("{}:{}", exit_id, exit_ver));
            let _ = app_exit.emit(
                "service-status-changed",
                serde_json::json!({ "id": exit_id, "status": "stopped", "version": exit_ver }),
            );
        });

        Ok(pid)
    }

    pub async fn stop_all_by_id(&self, id: &str) -> Result<(), String> {
        let keys: Vec<String> = {
            let map = self.instances.lock().await;
            map.keys()
                .filter(|k| k.starts_with(&format!("{}:", id)))
                .cloned()
                .collect()
        };

        for key in keys {
            let pid = {
                let map = self.instances.lock().await;
                map.get(&key).map(|s| s.pid)
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
