use std::collections::HashMap;
use std::sync::Mutex;
use tauri::{AppHandle, Emitter};
use tauri_plugin_shell::process::{CommandChild, CommandEvent};
use tauri_plugin_shell::ShellExt;

#[allow(dead_code)]
pub struct ServiceDefinition {
    pub id: &'static str,
    pub name: &'static str,
    pub port: u16,
    pub version: &'static str,
    pub download_url: &'static str,
    pub start_args: &'static [&'static str],
    pub stop_args: &'static [&'static str],
}

pub static SERVICE_CATALOG: &[ServiceDefinition] = &[
    ServiceDefinition {
        id: "nginx",
        name: "Nginx",
        port: 80,
        version: "nginx-1.26.2",
        download_url: "https://nginx.org/download/nginx-1.26.2.zip",
        start_args: &["-c", "conf/nginx.conf"],
        stop_args: &["-s", "stop"],
    },
    ServiceDefinition {
        id: "php",
        name: "PHP-CGI",
        port: 9000,
        version: "unknown",
        download_url: "",
        start_args: &[],
        stop_args: &[],
    },
    ServiceDefinition {
        id: "mysql",
        name: "MySQL",
        port: 3306,
        version: "unknown",
        download_url: "",
        start_args: &[],
        stop_args: &[],
    },
];

impl ServiceDefinition {
    pub fn by_id(id: &str) -> Option<&'static Self> {
        SERVICE_CATALOG.iter().find(|s| s.id == id)
    }
}

/// Manages spawned child processes via `tauri-plugin-shell`.
/// Each service spawn returns an event stream; the `Terminated` event is
/// forwarded to the frontend as `service-status-changed` in real time,
/// eliminating the need for polling heartbeats.
pub struct ServiceProcesses {
    children: Mutex<HashMap<String, CommandChild>>,
}

impl ServiceProcesses {
    pub fn new() -> Self {
        Self {
            children: Mutex::new(HashMap::new()),
        }
    }

    /// Spawns a service binary via `tauri-plugin-shell` and sets up a
    /// background task that listens for OS-level termination events.
    pub async fn start(
        &self,
        app: &AppHandle,
        id: &str,
        bin_path: &str,
        cwd: &str,
        args: &[&str],
    ) -> Result<u32, String> {
        let (mut rx, child) = app
            .shell()
            .command(bin_path)
            .args(args)
            .current_dir(cwd)
            .spawn()
            .map_err(|e| format!("Failed to spawn {}: {}", id, e))?;

        let pid = child.pid();

        {
            let mut children = self.children.lock().map_err(|e| e.to_string())?;
            children.insert(id.to_string(), child);
        }

        // Real-time termination watcher — fires instantly on external kill,
        // process crash, or normal exit, without any polling.
        let id_owned = id.to_string();
        let app_handle = app.clone();

        tokio::spawn(async move {
            while let Some(event) = rx.recv().await {
                if let CommandEvent::Terminated(payload) = event {
                    let _ = app_handle.emit(
                        "service-status-changed",
                        serde_json::json!({
                            "id": id_owned,
                            "status": "Stopped",
                            "code": payload.code,
                            "signal": payload.signal,
                        }),
                    );
                    break;
                }
            }
        });

        Ok(pid)
    }

    /// Force-kills the tracked child (if any) and removes it from tracking.
    /// The spawned event watcher will also emit `service-status-changed` as
    /// a backup notification after the process exits.
    pub fn stop(&self, id: &str) -> Result<(), String> {
        let mut children = self.children.lock().map_err(|e| e.to_string())?;

        if let Some(child) = children.remove(id) {
            child
                .kill()
                .map_err(|e| format!("Failed to kill {}: {}", id, e))?;
        }

        Ok(())
    }
}
