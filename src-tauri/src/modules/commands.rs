use crate::modules::services::{ProcessManager, ServiceStatus, ServiceInfo};
use tauri::{State, AppHandle};

#[tauri::command]
pub async fn get_services_status(
    process_manager: State<'_, ProcessManager>
) -> Result<Vec<ServiceInfo>, String> {
    let mut services = Vec::new();

    let items = vec![
        ("nginx", "Nginx", 80),
        ("php", "PHP-CGI", 9000),
        ("mysql", "MySQL", 3306),
    ];

    for (id, name, port) in items {
        let is_running = process_manager.is_running(id).await;
        services.push(ServiceInfo {
            id: id.to_string(),
            name: name.to_string(),
            status: if is_running { ServiceStatus::Running } else { ServiceStatus::Stopped },
            port,
        });
    }

    Ok(services)
}

#[tauri::command]
pub async fn start_service(
    id: String,
    process_manager: State<'_, ProcessManager>
) -> Result<(), String> {
    // Contoh argumen default, nantinya bisa diambil dari config
    let args = match id.as_str() {
        "nginx" => vec!["-c", "conf/nginx.conf"],
        _ => vec![],
    };

    process_manager.spawn_service(&id, args).await
}

#[tauri::command]
pub async fn stop_service(
    id: String,
    process_manager: State<'_, ProcessManager>
) -> Result<(), String> {
    process_manager.kill_service(&id).await
}
