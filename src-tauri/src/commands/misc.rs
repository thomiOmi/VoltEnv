use tauri::AppHandle;

use crate::errors::{VoltError, VoltResult, ResourceUsage};
use crate::paths::VoltPath;
use crate::service::ServiceDefinition;
use crate::settings::Settings;
use tauri::State;
use crate::process::ServiceProcesses;

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

#[tauri::command]
pub async fn check_service_id_available(app: AppHandle, id: String) -> VoltResult<bool> {
    let registry = crate::service::ServiceRegistry::load_all(&app);
    Ok(registry.get(&id).is_none())
}

#[tauri::command]
pub async fn get_resource_usage(
    state: State<'_, ServiceProcesses>,
    id: String,
) -> VoltResult<Option<ResourceUsage>> {
    let instances = state.instances.lock().await;
    let keys: Vec<String> = instances.keys()
        .filter(|k| k.starts_with(&format!("{}:", id)))
        .cloned()
        .collect();

    if keys.is_empty() {
        return Ok(None);
    }

    if let Some(instance) = instances.get(&keys[0]) {
        let pid = instance.pid;
        drop(instances);

        use sysinfo::{Pid, ProcessRefreshKind, ProcessesToUpdate};
        let mut sys = sysinfo::System::new();
        let pid_sys = Pid::from(pid as usize);

        sys.refresh_processes_specifics(
            ProcessesToUpdate::Some(&[pid_sys]),
            true,
            ProcessRefreshKind::everything()
        );

        if let Some(proc) = sys.process(pid_sys) {
            return Ok(Some(ResourceUsage {
                cpu: proc.cpu_usage(),
                memory: proc.memory(),
            }));
        }
    }

    Ok(None)
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportData {
    pub settings: Settings,
    pub custom_services: Vec<crate::service::ServiceDefinition>,
    pub vhosts: Vec<crate::vhost::VhostInfo>,
    pub export_date: String,
}

#[tauri::command]
pub async fn export_configuration(app: AppHandle) -> VoltResult<String> {
    let settings = Settings::load(&app);
    let mut custom_services = Vec::new();
    let custom_dir = VoltPath::custom_services_dir(&app);
    if custom_dir.exists() {
        if let Ok(entries) = std::fs::read_dir(custom_dir) {
            for entry in entries.flatten() {
                if entry.path().extension().is_none_or(|e| e != "json") { continue; }
                if let Ok(content) = std::fs::read_to_string(entry.path()) {
                    if let Ok(def) = serde_json::from_str::<crate::service::ServiceDefinition>(&content) {
                        custom_services.push(def);
                    }
                }
            }
        }
    }

    let vhosts_dir = VoltPath::vhosts_dir(&app);
    let vhosts = crate::vhost::VhostManager::list_vhosts(&vhosts_dir).unwrap_or_default();

    let data = ExportData {
        settings,
        custom_services,
        vhosts,
        export_date: chrono::Local::now().to_rfc3339(),
    };

    serde_json::to_string_pretty(&data).map_err(|e| VoltError::Generic(format!("Export failed: {}", e)))
}

#[tauri::command]
pub async fn import_configuration(app: AppHandle, json_content: String) -> VoltResult<()> {
    let data: ExportData = serde_json::from_str(&json_content)
        .map_err(|e| VoltError::Validation(format!("Invalid import file: {}", e)))?;

    data.settings.save(&app).map_err(VoltError::Generic)?;

    let custom_dir = VoltPath::custom_services_dir(&app);
    std::fs::create_dir_all(&custom_dir).map_err(VoltError::Io)?;
    for svc in data.custom_services {
        let path = custom_dir.join(format!("{}.json", svc.id));
        let content = serde_json::to_string_pretty(&svc).map_err(|e| VoltError::Generic(e.to_string()))?;
        std::fs::write(path, content).map_err(VoltError::Io)?;
    }

    let vhosts_dir = VoltPath::vhosts_dir(&app);
    for vh in data.vhosts {
        crate::vhost::VhostManager::save_vhost(&vhosts_dir, &vh.domain, &vh.root, vh.port, vh.php_port)
            .map_err(VoltError::Generic)?;
    }

    Ok(())
}
