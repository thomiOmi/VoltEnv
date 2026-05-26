use std::process::Stdio;
use tokio::process::{Child, Command};
use tokio::sync::Mutex;
use std::collections::HashMap;
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
    // Menyimpan handle process yang sedang berjalan
    pub children: Arc<Mutex<HashMap<String, Child>>>,
}

impl ProcessManager {
    pub fn new() -> Self {
        Self {
            children: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Menjalankan service berdasarkan nama binary
    pub async fn spawn_service(&self, id: &str, args: Vec<&str>) -> Result<(), String> {
        let bin_path = VoltPath::get_binary_path(id);

        if !bin_path.exists() {
            return Err(format!("Binary untuk {} tidak ditemukan di {:?}", id, bin_path));
        }

        let mut command = Command::new(bin_path);
        command.args(args)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        // Di Windows, tambahkan flag agar tidak muncul jendela console baru
        #[cfg(target_os = "windows")]
        {
            use std::os::windows::process::CommandExt;
            const CREATE_NO_WINDOW: u32 = 0x08000000;
            command.creation_flags(CREATE_NO_WINDOW);
        }

        let child = command.spawn()
            .map_err(|e| format!("Gagal menjalankan {}: {}", id, e))?;

        let mut children = self.children.lock().await;
        children.insert(id.to_string(), child);

        Ok(())
    }

    /// Menghentikan service
    pub async fn kill_service(&self, id: &str) -> Result<(), String> {
        let mut children = self.children.lock().await;
        if let Some(mut child) = children.remove(id) {
            child.kill().await
                .map_err(|e| format!("Gagal mematikan {}: {}", id, e))?;
        }
        Ok(())
    }

    /// Mengecek apakah service masih berjalan
    pub async fn is_running(&self, id: &str) -> bool {
        let mut children = self.children.lock().await;
        if let Some(child) = children.get_mut(id) {
            match child.try_wait() {
                Ok(None) => true, // Masih berjalan
                _ => {
                    children.remove(id); // Sudah mati
                    false
                }
            }
        } else {
            false
        }
    }
}
