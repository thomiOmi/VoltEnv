use tauri::AppHandle;

use crate::errors::{VoltError, VoltResult};
use crate::paths::VoltPath;
use crate::service::ServiceDefinition;
use crate::settings::Settings;

#[tauri::command]
pub async fn get_settings(app: AppHandle) -> VoltResult<Settings> {
    Ok(Settings::load(&app))
}

#[tauri::command]
pub async fn update_settings(app: AppHandle, settings: Settings) -> VoltResult<()> {
    settings.save(&app).map_err(VoltError::Generic)
}

#[tauri::command]
pub async fn get_service_logs(
    app: AppHandle,
    id: String,
    version: String,
    lines_count: usize,
) -> VoltResult<Vec<String>> {
    crate::logging::get_service_logs(&app, &id, &version, lines_count).await.map_err(VoltError::Generic)
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
) -> VoltResult<()> {
    if service.id.is_empty() {
        return Err(VoltError::Validation("Service ID is required".to_string()));
    }
    if service.name.is_empty() {
        return Err(VoltError::Validation("Service name is required".to_string()));
    }
    if service.binary_name.is_empty() {
        return Err(VoltError::Validation("Binary name is required".to_string()));
    }
    if service.default_version.is_empty() {
        service.default_version = "0.0.0".to_string();
    }
    service.kind = "custom".to_string();

    let custom_dir = VoltPath::custom_services_dir(&app);
    std::fs::create_dir_all(&custom_dir)
        .map_err(VoltError::Io)?;

    let path = custom_dir.join(format!("{}.json", service.id));
    let json = serde_json::to_string_pretty(&service)
        .map_err(|e| VoltError::Generic(format!("Failed to serialize service: {}", e)))?;
    std::fs::write(&path, &json).map_err(VoltError::Io)?;

    Ok(())
}

#[tauri::command]
pub async fn delete_custom_service(app: AppHandle, id: String) -> VoltResult<()> {
    let path = VoltPath::custom_services_dir(&app).join(format!("{}.json", id));
    if path.exists() {
        std::fs::remove_file(&path)
            .map_err(VoltError::Io)?;
    }
    Ok(())
}
