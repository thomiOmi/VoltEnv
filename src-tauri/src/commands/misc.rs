use std::collections::HashMap;
use tauri::{AppHandle, State, Manager};
use crate::paths::VoltPath;
use crate::settings::Settings;
use crate::service::ServiceDefinition;

#[tauri::command]
pub async fn get_settings(app: AppHandle) -> Result<Settings, String> {
    Ok(Settings::load(&app))
}

#[tauri::command]
pub async fn update_settings(app: AppHandle, settings: Settings) -> Result<(), String> {
    settings.save(&app)
}

#[tauri::command]
pub async fn get_service_logs(app: AppHandle, id: String, version: String, lines_count: usize) -> Result<Vec<String>, String> {
    let log_path = VoltPath::log_path(&app, &id, &version);
    if !log_path.exists() {
        return Ok(vec![]);
    }

    let content = std::fs::read_to_string(&log_path).map_err(|e| e.to_string())?;
    let lines: Vec<String> = content.lines().rev().take(lines_count).map(|s| s.to_string()).collect();
    Ok(lines.into_iter().rev().collect())
}

#[tauri::command]
pub async fn is_port_available(port: u16) -> bool {
    tokio::net::TcpListener::bind(("127.0.0.1", port)).await.is_ok()
}

#[tauri::command]
pub async fn save_custom_service(app: AppHandle, service: ServiceDefinition) -> Result<(), String> {
    let custom_dir = VoltPath::custom_services_dir(&app);
    std::fs::create_dir_all(&custom_dir).map_err(|e| e.to_string())?;

    let path = custom_dir.join(format!("{}.json", service.id));
    let json = serde_json::to_string_pretty(&service).map_err(|e| e.to_string())?;
    std::fs::write(path, json).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn delete_custom_service(app: AppHandle, id: String) -> Result<(), String> {
    let path = VoltPath::custom_services_dir(&app).join(format!("{}.json", id));
    if path.exists() {
        std::fs::remove_file(path).map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
pub async fn run_composer_command(app: AppHandle, project_path: String, args: Vec<String>) -> Result<String, String> {
    let settings = Settings::load(&app);
    let php_version = settings.active_versions.get("php").cloned().unwrap_or_else(|| "8.2.0".to_string());
    let php_bin = VoltPath::service_binary_path(&app, "php", &php_version);

    if !php_bin.exists() {
        return Err("PHP is not installed or not found. Please setup PHP first.".to_string());
    }

    // Try to find composer.phar in project or global
    let composer_path = std::path::Path::new(&project_path).join("composer.phar");
    let mut cmd = tokio::process::Command::new(php_bin);

    if composer_path.exists() {
        cmd.arg(composer_path);
    } else {
        // Fallback to 'composer' command assuming it's in PATH
        cmd = tokio::process::Command::new("composer");
    }

    let output = cmd
        .args(args)
        .current_dir(project_path)
        .output()
        .await
        .map_err(|e| format!("Failed to execute composer: {}", e))?;

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();

    if !output.status.success() {
        return Err(format!("Composer failed: {}\n{}", stderr, stdout));
    }

    Ok(stdout)
}
