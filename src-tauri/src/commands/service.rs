use crate::service::ServiceRegistry;
use crate::process::ServiceProcesses;
use crate::paths::VoltPath;
use crate::utils::{VoltError, VoltResult};
use crate::config::ConfigGenerator;
use std::collections::HashMap;
use tauri::{AppHandle, State};

fn substitute_args(
    arg: &str,
    root: &std::path::Path,
    bin_dir: &std::path::Path,
    config: &std::path::Path,
    data: &std::path::Path,
    port: u16,
) -> String {
    arg.replace("{root}", &root.to_string_lossy())
        .replace("{bin_dir}", &bin_dir.to_string_lossy())
        .replace("{config}", &config.to_string_lossy())
        .replace("{data_dir}", &data.to_string_lossy())
        .replace("{port}", &port.to_string())
}

async fn resolve_port(app: &AppHandle, id: &str, default: u16) -> u16 {
    let settings = crate::commands::misc::get_settings(app.clone()).await.unwrap_or_default();
    settings.preferred_ports.get(id).copied().unwrap_or(default)
}

async fn health_check(
    hc: &crate::service::HealthCheckConfig,
    port: u16,
    _def: &crate::service::ServiceDefinition,
    _root: &std::path::Path,
    _bin: &std::path::Path,
    _config: &std::path::Path,
    _data: &std::path::Path,
) -> Result<(), String> {
    match hc.check_type.as_str() {
        "tcp" => {
            let addr = format!("127.0.0.1:{}", port);
            let mut attempts = 0;
            while attempts < 10 {
                if tokio::net::TcpStream::connect(&addr).await.is_ok() {
                    return Ok(());
                }
                tokio::time::sleep(std::time::Duration::from_millis(500)).await;
                attempts += 1;
            }
            Err(format!("TCP Health check failed on {}", addr))
        }
        "http" => {
            let path = hc.command.as_deref().unwrap_or("/");
            let url = format!("http://127.0.0.1:{}{}", port, path);
            let mut attempts = 0;
            while attempts < 10 {
                if let Ok(resp) = crate::http_client().get(&url).send().await {
                    if resp.status().is_success() {
                        return Ok(());
                    }
                }
                tokio::time::sleep(std::time::Duration::from_millis(500)).await;
                attempts += 1;
            }
            Err(format!("HTTP Health check failed on {}", url))
        }
        _ => Ok(()),
    }
}

#[tauri::command]
pub async fn setup_service(
    app: AppHandle,
    id: String,
    version: String,
) -> VoltResult<()> {
    let registry = ServiceRegistry::load_all(&app);
    let def = registry.get(&id).ok_or_else(|| VoltError::Service("Service not found".to_string()))?;

    crate::download::manager::DownloadManager::new(&app)
        .download_and_install(def, &version)
        .await
        .map_err(|e| VoltError::Custom(e.to_string()))?;

    Ok(())
}

#[tauri::command]
pub async fn start_service(
    app: AppHandle,
    state: State<'_, ServiceProcesses>,
    id: String,
    version: String,
) -> VoltResult<u32> {
    let registry = ServiceRegistry::load_all(&app);
    let def = registry.get(&id).ok_or_else(|| VoltError::Service("Service not found".to_string()))?;

    let bin_dir = VoltPath::service_dir(&app, &id, &version);
    let binary_path = bin_dir.join(&def.binary_name);
    if !binary_path.exists() {
        return Err(VoltError::Service(format!(
            "{} {} is not installed. Run setup first.",
            id, version
        )));
    }

    let port = resolve_port(&app, &id, def.port).await;

    if let Some(ref tmpl) = def.config_template_name {
        let log_dir = VoltPath::logs_dir(&app);
        let vhosts_dir = VoltPath::vhosts_dir(&app);
        let data_dir = VoltPath::data_dir(&app, &id, &version);
        let config_path = VoltPath::config_path(&app, &id, &version);
        let mut vars = HashMap::new();
        vars.insert("port".to_string(), port.to_string());
        vars.insert("log_dir".to_string(), log_dir.to_string_lossy().to_string());
        vars.insert(
            "vhosts_dir".to_string(),
            vhosts_dir.to_string_lossy().to_string(),
        );
        vars.insert(
            "data_dir".to_string(),
            data_dir.to_string_lossy().to_string(),
        );
        vars.insert("bin_dir".to_string(), bin_dir.to_string_lossy().to_string());
        vars.insert(
            "www_dir".to_string(),
            VoltPath::www_dir(&app).to_string_lossy().to_string(),
        );
        let templates_dir = VoltPath::templates_dir(&app);
        ConfigGenerator::generate_to_file(tmpl, &vars, Some(&templates_dir), &config_path)
            .map_err(|e| VoltError::Custom(e.to_string()))?;
    }

    let root = VoltPath::root(&app);
    let config_path = VoltPath::config_path(&app, &id, &version);
    let data_dir = VoltPath::data_dir(&app, &id, &version);

    let substituted_args: Vec<String> = def
        .start_args
        .iter()
        .map(|arg| substitute_args(arg, &root, &bin_dir, &config_path, &data_dir, port))
        .collect();

    let cwd = bin_dir.to_string_lossy().to_string();

    let pid = state
        .start(
            app.clone(),
            id.clone(),
            version.clone(),
            binary_path.to_string_lossy().to_string(),
            cwd.clone(),
            substituted_args.clone(),
            port,
        )
        .await
        .map_err(|e| VoltError::Process(e.to_string()))?;

    if let Some(ref hc) = def.health_check {
        let result = health_check(hc, port, def, &root, &bin_dir, &config_path, &data_dir).await;
        if let Err(e) = result {
            let _ = state.stop(&id, &version).await;
            return Err(VoltError::Service(format!("Health check failed: {}", e)));
        }
    }

    Ok(pid)
}

#[tauri::command]
pub async fn stop_service(
    state: State<'_, ServiceProcesses>,
    id: String,
    version: String,
) -> VoltResult<()> {
    state.stop(&id, &version).await
}

#[tauri::command]
pub async fn get_service_status(
    state: State<'_, ServiceProcesses>,
    id: String,
    version: String,
) -> VoltResult<String> {
    Ok(state.get_status(&id, &version).await)
}

#[tauri::command]
pub async fn get_services(app: AppHandle) -> VoltResult<Vec<crate::service::ServiceDefinition>> {
    Ok(ServiceRegistry::load_all(&app).all().into_iter().cloned().collect())
}

#[tauri::command]
pub async fn switch_service_version(
    app: AppHandle,
    state: State<'_, ServiceProcesses>,
    id: String,
    version: String,
) -> VoltResult<()> {
    let _ = state.stop_all_by_id(&id).await.map_err(|e| VoltError::Process(e.to_string()));

    VoltPath::create_env_junction(&app, &id, &version).map_err(|e| VoltError::Custom(e.to_string()))?;

    let mut settings = crate::commands::misc::get_settings(app.clone()).await.unwrap_or_default();
    settings.active_versions.insert(id, version);
    settings.save(&app).map_err(|e| VoltError::Custom(e.to_string()))?;

    Ok(())
}

#[tauri::command]
pub async fn get_php_extensions(app: AppHandle, version: String) -> VoltResult<Vec<crate::service::php_ini::PhpExtension>> {
    let config_path = VoltPath::config_path(&app, "php", &version);
    if !config_path.exists() {
        return Err(VoltError::Config("php.ini not found".to_string()));
    }
    crate::service::php_ini::PhpIniManager::get_extensions(&config_path).map_err(|e| VoltError::Custom(e.to_string()))
}

#[tauri::command]
pub async fn toggle_php_extension(
    app: AppHandle,
    version: String,
    extension: String,
    enabled: bool,
) -> VoltResult<()> {
    let config_path = VoltPath::config_path(&app, "php", &version);
    crate::service::php_ini::PhpIniManager::toggle_extension(&config_path, &extension, enabled)
        .map_err(|e| VoltError::Custom(e.to_string()))
}
