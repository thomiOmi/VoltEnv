use std::fs;
use tauri::AppHandle;

use crate::paths::VoltPath;
use crate::utils::{VoltError, VoltResult};
use crate::vhost::hosts::HostsManager;
use crate::vhost::ssl::SslManager;
use crate::vhost::{VhostInfo, VhostManager};

#[tauri::command]
pub async fn list_vhosts(app: AppHandle) -> VoltResult<Vec<VhostInfo>> {
    let vhosts_dir = VoltPath::vhosts_dir(&app);
    VhostManager::list_vhosts(&vhosts_dir).map_err(VoltError::Custom)
}

#[tauri::command]
pub async fn create_vhost(
    app: AppHandle,
    domain: String,
    root: String,
    port: u16,
    php_port: Option<u16>,
    enable_ssl: bool,
) -> VoltResult<VhostInfo> {
    let vhosts_dir = VoltPath::vhosts_dir(&app);
    let ssl_dir = VoltPath::ssl_dir(&app);
    fs::create_dir_all(&ssl_dir)?;

    let ssl_paths = if enable_ssl {
        let ca_cert_path = ssl_dir.join("rootCA.pem");
        let ca_key_path = ssl_dir.join("rootCA-key.pem");

        if !ca_cert_path.exists() || !ca_key_path.exists() {
            let (ca_cert, ca_key) = SslManager::generate_ca().map_err(VoltError::Custom)?;
            fs::write(&ca_cert_path, ca_cert)?;
            fs::write(&ca_key_path, ca_key)?;
            let _ = SslManager::install_ca(&ca_cert_path);
        }

        let ca_cert = fs::read_to_string(&ca_cert_path)?;
        let ca_key = fs::read_to_string(&ca_key_path)?;

        let (cert, key) =
            SslManager::generate_cert(&ca_cert, &ca_key, &domain).map_err(VoltError::Custom)?;
        let cert_path = ssl_dir.join(format!("{}.crt", domain));
        let key_path = ssl_dir.join(format!("{}.key", domain));

        fs::write(&cert_path, cert)?;
        fs::write(&key_path, key)?;

        if !domain.ends_with(".localhost") {
            let _ = HostsManager::add_entry(&domain);
        }

        Some((cert_path, key_path))
    } else {
        None
    };

    let ssl_ref = ssl_paths.as_ref().map(|(c, k)| (c.as_path(), k.as_path()));
    VhostManager::save_vhost(&vhosts_dir, &domain, &root, port, php_port, ssl_ref)
        .map_err(VoltError::Custom)
}

#[tauri::command]
pub async fn delete_vhost(app: AppHandle, domain: String) -> VoltResult<()> {
    let vhosts_dir = VoltPath::vhosts_dir(&app);
    VhostManager::delete_vhost(&vhosts_dir, &domain).map_err(VoltError::Custom)
}
