use std::collections::HashMap;
use tauri::{AppHandle, State, Manager};
use crate::paths::VoltPath;
use crate::settings::Settings;
use crate::service::ServiceDefinition;
use crate::utils::{VoltResult, VoltError};

#[tauri::command]
pub async fn get_settings(app: AppHandle) -> VoltResult<Settings> {
    Ok(Settings::load(&app))
}

#[tauri::command]
pub async fn update_settings(app: AppHandle, settings: Settings) -> VoltResult<()> {
    settings.save(&app).map_err(VoltError::Custom)
}

#[tauri::command]
pub async fn get_service_logs(app: AppHandle, id: String, version: String, lines_count: usize) -> VoltResult<Vec<String>> {
    let log_path = VoltPath::log_path(&app, &id, &version);
    if !log_path.exists() {
        return Ok(vec![]);
    }

    let content = std::fs::read_to_string(&log_path)?;
    let lines: Vec<String> = content.lines().rev().take(lines_count).map(|s| s.to_string()).collect();
    Ok(lines.into_iter().rev().collect())
}

#[tauri::command]
pub async fn is_port_available(port: u16) -> bool {
    tokio::net::TcpListener::bind(("127.0.0.1", port)).await.is_ok()
}

#[tauri::command]
pub async fn save_custom_service(app: AppHandle, service: ServiceDefinition) -> VoltResult<()> {
    let custom_dir = VoltPath::custom_services_dir(&app);
    std::fs::create_dir_all(&custom_dir)?;

    let path = custom_dir.join(format!("{}.json", service.id));
    let json = serde_json::to_string_pretty(&service)?;
    std::fs::write(path, json)?;
    Ok(())
}

#[tauri::command]
pub async fn delete_custom_service(app: AppHandle, id: String) -> VoltResult<()> {
    let path = VoltPath::custom_services_dir(&app).join(format!("{}.json", id));
    if path.exists() {
        std::fs::remove_file(path)?;
    }
    Ok(())
}

#[tauri::command]
pub async fn run_composer_command(app: AppHandle, project_path: String, args: Vec<String>) -> VoltResult<String> {
    let settings = Settings::load(&app);
    let php_version = settings.active_versions.get("php").cloned().unwrap_or_else(|| "8.2.0".to_string());
    let php_bin = VoltPath::service_binary_path(&app, "php", &php_version);

    if !php_bin.exists() {
        return Err(VoltError::Service("PHP is not installed or not found. Please setup PHP first.".to_string()));
    }

    let composer_path = std::path::Path::new(&project_path).join("composer.phar");
    let mut cmd = tokio::process::Command::new(php_bin);

    if composer_path.exists() {
        cmd.arg(composer_path);
    } else {
        cmd = tokio::process::Command::new("composer");
    }

    let output = cmd
        .args(args)
        .current_dir(project_path)
        .output()
        .await
        .map_err(|e| VoltError::Custom(format!("Failed to execute composer: {}", e)))?;

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();

    if !output.status.success() {
        return Err(VoltError::Custom(format!("Composer failed: {}\n{}", stderr, stdout)));
    }

    Ok(stdout)
}

#[tauri::command]
pub async fn run_self_diagnostic(app: AppHandle) -> VoltResult<serde_json::Value> {
    let mut results = HashMap::new();

    // Check directories
    let bin_dir = VoltPath::bin_dir(&app);
    results.insert("bin_dir_exists", serde_json::json!(bin_dir.exists()));

    // Check ports
    let ports = vec![80, 443, 3306, 9000];
    let mut port_results = HashMap::new();
    for port in ports {
        port_results.insert(port.to_string(), serde_json::json!(is_port_available(port).await));
    }
    results.insert("ports", serde_json::json!(port_results));

    // Check SSL CA
    let ssl_dir = VoltPath::ssl_dir(&app);
    let ca_exists = ssl_dir.join("rootCA.pem").exists();
    results.insert("ca_installed", serde_json::json!(ca_exists));

    Ok(serde_json::to_value(results)?)
}
