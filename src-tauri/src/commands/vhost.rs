use tauri::AppHandle;

use crate::paths::VoltPath;
use crate::vhost::{VhostInfo, VhostManager};

#[tauri::command]
pub async fn list_vhosts(app: AppHandle) -> Result<Vec<VhostInfo>, String> {
    let vhosts_dir = VoltPath::vhosts_dir(&app);
    VhostManager::list_vhosts(&vhosts_dir)
}

#[tauri::command]
pub async fn create_vhost(
    app: AppHandle,
    domain: String,
    root: String,
    port: u16,
    php_port: Option<u16>,
) -> Result<VhostInfo, String> {
    let vhosts_dir = VoltPath::vhosts_dir(&app);
    VhostManager::save_vhost(&vhosts_dir, &domain, &root, port, php_port)
}

#[tauri::command]
pub async fn delete_vhost(app: AppHandle, domain: String) -> Result<(), String> {
    let vhosts_dir = VoltPath::vhosts_dir(&app);
    VhostManager::delete_vhost(&vhosts_dir, &domain)
}
