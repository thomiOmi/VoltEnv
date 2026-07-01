use std::fs;
use tauri::AppHandle;

use crate::paths::VoltPath;
use crate::settings::Settings;
use crate::utils::{VoltError, VoltResult};
use crate::vhost::ssl::SslManager;
use crate::vhost::VhostManager;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QuickCreateResult {
    pub project_name: String,
    pub domain: String,
    pub root_path: String,
    pub created_vhost: bool,
    pub created_database: bool,
}

#[tauri::command]
pub async fn quick_create(
    app: AppHandle,
    project_name: String,
    create_database: bool,
) -> VoltResult<QuickCreateResult> {
    let slug: String = project_name
        .chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '-' || c == '_' {
                c
            } else {
                '-'
            }
        })
        .collect();
    let slug_lower = slug.to_lowercase();

    let domain = format!("{}.localhost", slug_lower);
    let www_dir = VoltPath::www_dir(&app);
    let project_dir = www_dir.join(&slug_lower);
    let root_path = project_dir.to_string_lossy().to_string();

    std::fs::create_dir_all(&project_dir)?;

    let index_php = format!(
        r#"<?php
echo "<h1>{} — Ready</h1>";
echo "<p>Project created by VoltEnv.</p>";
phpinfo();
"#,
        project_name
    );
    std::fs::write(project_dir.join("index.php"), &index_php)?;

    let settings = Settings::load(&app);
    let nginx_port = settings
        .resolved_ports
        .get("nginx")
        .copied()
        .or_else(|| settings.preferred_ports.get("nginx").copied())
        .unwrap_or(80);

    let php_port = settings
        .resolved_ports
        .get("php")
        .copied()
        .or_else(|| settings.preferred_ports.get("php").copied());

    let ssl_dir = VoltPath::ssl_dir(&app);
    let _ = fs::create_dir_all(&ssl_dir);

    let ca_cert_path = ssl_dir.join("rootCA.pem");
    let ca_key_path = ssl_dir.join("rootCA-key.pem");

    let ssl_paths = if !ca_cert_path.exists() || !ca_key_path.exists() {
        if let Ok((ca_cert, ca_key)) = SslManager::generate_ca() {
            let _ = fs::write(&ca_cert_path, ca_cert);
            let _ = fs::write(&ca_key_path, ca_key);
            let _ = SslManager::install_ca(&ca_cert_path);
        }
        None
    } else {
        let ca_cert = fs::read_to_string(&ca_cert_path).unwrap_or_default();
        let ca_key = fs::read_to_string(&ca_key_path).unwrap_or_default();
        if let Ok((cert, key)) = SslManager::generate_cert(&ca_cert, &ca_key, &domain) {
            let cert_path = ssl_dir.join(format!("{}.crt", domain));
            let key_path = ssl_dir.join(format!("{}.key", domain));
            let _ = fs::write(&cert_path, cert);
            let _ = fs::write(&key_path, key);
            Some((cert_path, key_path))
        } else {
            None
        }
    };

    let vhosts_dir = VoltPath::vhosts_dir(&app);
    VhostManager::save_vhost(
        &vhosts_dir,
        &domain,
        &root_path,
        nginx_port,
        php_port,
        ssl_paths.as_ref().map(|(c, k)| (c.as_path(), k.as_path())),
    )
    .map_err(VoltError::Custom)?;

    let mut created_db = false;
    if create_database {
        let sanitized: String = slug_lower
            .chars()
            .filter(|c| c.is_alphanumeric() || *c == '_')
            .collect();
        if !sanitized.is_empty() {
            match crate::commands::database::create_database_inner(&app, &sanitized).await {
                Ok(()) => created_db = true,
                Err(e) => eprintln!("[voltenv] Quick create database failed: {}", e),
            }
        }
    }

    Ok(QuickCreateResult {
        project_name,
        domain,
        root_path,
        created_vhost: true,
        created_database: created_db,
    })
}
