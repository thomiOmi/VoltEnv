use tauri::AppHandle;

use crate::paths::VoltPath;
use crate::service::ServiceDefinition;
use crate::settings::Settings;

#[tauri::command]
pub async fn get_settings(app: AppHandle) -> Result<Settings, String> {
    Ok(Settings::load(&app))
}

#[tauri::command]
pub async fn update_settings(app: AppHandle, settings: Settings) -> Result<(), String> {
    settings.save(&app)
}

#[tauri::command]
pub async fn get_service_logs(
    app: AppHandle,
    id: String,
    version: String,
    lines_count: usize,
) -> Result<Vec<String>, String> {
    crate::logging::get_service_logs(&app, &id, &version, lines_count).await
}

#[tauri::command]
pub async fn is_port_available(port: u16) -> bool {
    tokio::net::TcpListener::bind(("127.0.0.1", port))
        .await
        .is_ok()
}

#[tauri::command]
pub async fn save_custom_service(
    app: AppHandle,
    mut service: ServiceDefinition,
) -> Result<(), String> {
    if service.id.is_empty() {
        return Err("Service ID is required".to_string());
    }
    if service.name.is_empty() {
        return Err("Service name is required".to_string());
    }
    if service.binary_name.is_empty() {
        return Err("Binary name is required".to_string());
    }
    if service.default_version.is_empty() {
        service.default_version = "0.0.0".to_string();
    }
    service.kind = "custom".to_string();

    let custom_dir = VoltPath::custom_services_dir(&app);
    std::fs::create_dir_all(&custom_dir)
        .map_err(|e| format!("Failed to create custom services dir: {}", e))?;

    let path = custom_dir.join(format!("{}.json", service.id));
    let json = serde_json::to_string_pretty(&service)
        .map_err(|e| format!("Failed to serialize service: {}", e))?;
    std::fs::write(&path, &json).map_err(|e| format!("Failed to write custom service: {}", e))?;

    Ok(())
}

#[tauri::command]
pub async fn delete_custom_service(app: AppHandle, id: String) -> Result<(), String> {
    let path = VoltPath::custom_services_dir(&app).join(format!("{}.json", id));
    if path.exists() {
        std::fs::remove_file(&path)
            .map_err(|e| format!("Failed to delete custom service: {}", e))?;
    }
    Ok(())
}
