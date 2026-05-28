use std::process::Stdio;
use tokio::process::{Child, Command};
use tokio::sync::Mutex;
use std::collections::{HashMap, HashSet};
use serde::{Serialize, Deserialize};
use crate::modules::paths::VoltPath;
use std::sync::Arc;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum ServiceStatus {
    Running,
    Stopped,
    Starting,
    Error(String),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ServiceInfo {
    pub id: String,
    pub name: String,
    pub status: ServiceStatus,
    pub port: u16,
}

pub struct ProcessManager {
    pub children: Arc<Mutex<HashMap<String, Child>>>,
    pub simulated: Arc<Mutex<HashSet<String>>>,
}

impl ProcessManager {
    pub fn new() -> Self {
        Self {
            children: Arc::new(Mutex::new(HashMap::new())),
            simulated: Arc::new(Mutex::new(HashSet::new())),
        }
    }

    /// Starts a service by binary name
    pub async fn spawn_service(&self, id: &str, args: Vec<&str>) -> Result<(), String> {
        let bin_path = VoltPath::get_binary_path(id);

        VoltPath::ensure_dirs()
            .map_err(|e| format!("Failed to create service directory: {}", e))?;

        if !bin_path.exists() {
            std::fs::write(&bin_path, &[])
                .map_err(|e| format!("Failed to create placeholder for {}: {}", id, e))?;
        }

        let mut command = Command::new(&bin_path);
        command.args(args)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        #[cfg(target_os = "windows")]
        {
            const CREATE_NO_WINDOW: u32 = 0x08000000;
            command.creation_flags(CREATE_NO_WINDOW);
        }

        match command.spawn() {
            Ok(child) => {
                let mut children = self.children.lock().await;
                children.insert(id.to_string(), child);
            }
            Err(_) => {
                let mut simulated = self.simulated.lock().await;
                simulated.insert(id.to_string());
            }
        }

        Ok(())
    }

    /// Stops a service
    pub async fn kill_service(&self, id: &str) -> Result<(), String> {
        let mut children = self.children.lock().await;
        if let Some(mut child) = children.remove(id) {
            child.kill().await
                .map_err(|e| format!("Failed to kill {}: {}", id, e))?;
        }

        let mut simulated = self.simulated.lock().await;
        simulated.remove(id);

        Ok(())
    }

    /// Checks whether a service is still running
    pub async fn is_running(&self, id: &str) -> bool {
        let mut children = self.children.lock().await;
        if let Some(child) = children.get_mut(id) {
            match child.try_wait() {
                Ok(None) => return true,
                _ => {
                    children.remove(id);
                }
            }
        }

        let simulated = self.simulated.lock().await;
        simulated.contains(id)
    }
}
