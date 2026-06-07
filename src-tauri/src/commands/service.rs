use std::collections::HashMap;
use std::path::Path;
use tauri::{AppHandle, Emitter, State};

use crate::config::ConfigGenerator;
use crate::download::manager::DownloadManager;
use crate::download::verifier::Verifier;
use crate::installer::manager::InstallerManager;
use crate::paths::VoltPath;
use crate::process::ServiceProcesses;
use crate::service::ServiceRegistry;
use crate::settings::Settings;

#[tauri::command]
pub async fn setup_service(app: AppHandle, id: String, version: String) -> Result<(), String> {
    let registry = ServiceRegistry::load_all(&app);
    let def = registry
        .get(&id)
        .ok_or_else(|| format!("Service '{}' not found", id))?;

    let version_info = def
        .versions
        .get(&version)
        .ok_or_else(|| format!("Version '{}' not found for '{}'", version, id))?;

    let bin_dir = VoltPath::service_dir(&app, &id, &version);
    let binary_path = bin_dir.join(&def.binary_name);

    if binary_path.exists() {
        app.emit(
            "service-status-changed",
            serde_json::json!({ "id": id, "status": "installed", "version": version }),
        )
        .ok();
        return Ok(());
    }

    if version_info.download_url.is_empty() {
        return Err(format!("No download URL for {} {}", id, version));
    }

    let temp_path =
        VoltPath::temporary_download_path(&app, &id, &version, &version_info.download_url);

    DownloadManager::download(&app, &id, &version_info.download_url, &temp_path).await?;

    if let Some(ref sha256) = version_info.sha256 {
        Verifier::sha256(&temp_path, sha256).await?;
    }

    InstallerManager::install(&app, &id, &bin_dir, &temp_path).await?;

    let root = VoltPath::root(&app);
    let config_path = VoltPath::config_path(&app, &id, &version);
    let data_dir = VoltPath::data_dir(&app, &id, &version);

    for cmd_str in &def.post_install_commands {
        let substituted =
            substitute_args(cmd_str, &root, &bin_dir, &config_path, &data_dir, def.port);
        run_command(&substituted).await?;
    }

    app.emit(
        "service-status-changed",
        serde_json::json!({ "id": id, "status": "installed", "version": version }),
    )
    .ok();

    Ok(())
}

#[tauri::command]
pub async fn start_service(
    app: AppHandle,
    state: State<'_, ServiceProcesses>,
    id: String,
) -> Result<u32, String> {
    let registry = ServiceRegistry::load_all(&app);
    let def = registry
        .get(&id)
        .ok_or_else(|| format!("Service '{}' not found", id))?;

    let settings = Settings::load(&app);
    let version = settings
        .active_versions
        .get(&id)
        .cloned()
        .unwrap_or(def.default_version.clone());

    let key = format!("{}:{}", id, version);
    if state.instances.lock().await.contains_key(&key) {
        return Err(format!("{} {} is already running", id, version));
    }

    let bin_dir = VoltPath::service_dir(&app, &id, &version);
    let binary_path = bin_dir.join(&def.binary_name);
    if !binary_path.exists() {
        return Err(format!(
            "{} {} is not installed. Run setup first.",
            id, version
        ));
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
        vars.insert(
            "bin_dir".to_string(),
            bin_dir.to_string_lossy().to_string(),
        );
        vars.insert(
            "www_dir".to_string(),
            VoltPath::www_dir(&app).to_string_lossy().to_string(),
        );
        let templates_dir = VoltPath::templates_dir(&app);
        ConfigGenerator::generate_to_file(tmpl, &vars, Some(&templates_dir), &config_path)?;
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
            &app,
            &id,
            &version,
            &binary_path.to_string_lossy(),
            &cwd,
            &substituted_args,
            port,
        )
        .await?;

    if let Some(ref hc) = def.health_check {
        let result = health_check(hc, port, def, &root, &bin_dir, &config_path, &data_dir).await;
        if let Err(e) = result {
            let _ = state.stop_all_by_id(&id).await;
            return Err(format!("Health check failed for {}: {}", id, e));
        }
    }

    app.emit(
        "service-status-changed",
        serde_json::json!({ "id": id, "status": "running", "version": version, "port": port }),
    )
    .ok();

    Ok(pid)
}

#[tauri::command]
pub async fn stop_service(state: State<'_, ServiceProcesses>, id: String) -> Result<(), String> {
    state.stop_all_by_id(&id).await
}

async fn resolve_port(app: &AppHandle, id: &str, preferred: u16) -> u16 {
    let settings = Settings::load(app);
    if let Some(resolved) = settings.resolved_ports.get(id) {
        if is_port_free(*resolved).await {
            return *resolved;
        }
    }
    for port in preferred..preferred.saturating_add(100) {
        if is_port_free(port).await {
            let mut new_settings = Settings::load(app);
            new_settings.resolved_ports.insert(id.to_string(), port);
            let _ = new_settings.save(app);
            return port;
        }
    }
    preferred
}

async fn is_port_free(port: u16) -> bool {
    tokio::net::TcpListener::bind(("127.0.0.1", port))
        .await
        .is_ok()
}

async fn health_check(
    hc: &crate::service::HealthCheckConfig,
    port: u16,
    _def: &crate::service::ServiceDefinition,
    root: &Path,
    bin_dir: &Path,
    config_path: &Path,
    data_dir: &Path,
) -> Result<(), String> {
    match hc.check_type.as_str() {
        "port" => {
            let deadline =
                tokio::time::Instant::now() + std::time::Duration::from_millis(hc.timeout_ms);
            while tokio::time::Instant::now() < deadline {
                if !is_port_free(port).await {
                    return Ok(());
                }
                tokio::time::sleep(std::time::Duration::from_millis(200)).await;
            }
            Err(format!(
                "Port {} not listening within {} ms",
                port, hc.timeout_ms
            ))
        }
        "command" => {
            if let Some(ref cmd) = hc.command {
                let substituted = substitute_args(cmd, root, bin_dir, config_path, data_dir, port);
                run_command(&substituted).await
            } else {
                Ok(())
            }
        }
        _ => Ok(()),
    }
}

fn substitute_args(
    cmd: &str,
    root: &Path,
    bin_dir: &Path,
    config_path: &Path,
    data_dir: &Path,
    port: u16,
) -> String {
    cmd.replace("{{root}}", &root.to_string_lossy())
        .replace("{{bin_dir}}", &bin_dir.to_string_lossy())
        .replace("{{config_path}}", &config_path.to_string_lossy())
        .replace("{{data_dir}}", &data_dir.to_string_lossy())
        .replace("{{port}}", &port.to_string())
}

async fn run_command(cmd_str: &str) -> Result<(), String> {
    let status = if cfg!(target_os = "windows") {
        tokio::process::Command::new("cmd")
            .args(["/C", cmd_str])
            .status()
            .await
            .map_err(|e| format!("Failed to run command: {}", e))?
    } else {
        tokio::process::Command::new("sh")
            .args(["-c", cmd_str])
            .status()
            .await
            .map_err(|e| format!("Failed to run command: {}", e))?
    };

    if !status.success() {
        return Err(format!(
            "Command failed with exit code: {:?}",
            status.code()
        ));
    }

    Ok(())
}

#[tauri::command]
pub async fn get_service_status(
    app: AppHandle,
    state: State<'_, ServiceProcesses>,
    id: String,
) -> Result<serde_json::Value, String> {
    let registry = ServiceRegistry::load_all(&app);
    let def = registry
        .get(&id)
        .ok_or_else(|| format!("Service '{}' not found", id))?;
    let settings = Settings::load(&app);
    let version = settings
        .active_versions
        .get(&id)
        .cloned()
        .unwrap_or(def.default_version.clone());
    let key = format!("{}:{}", id, version);

    let instances = state.instances.lock().await;
    if let Some(instance) = instances.get(&key) {
        Ok(serde_json::json!({
            "id": id,
            "version": instance.version,
            "status": "running",
            "port": instance.port,
        }))
    } else {
        Ok(serde_json::json!({
            "id": id,
            "version": version,
            "status": "stopped",
            "port": 0,
        }))
    }
}

#[tauri::command]
pub async fn get_services(app: AppHandle) -> Result<Vec<serde_json::Value>, String> {
    let registry = crate::service::ServiceRegistry::load_all(&app);
    let services: Vec<serde_json::Value> = registry
        .all()
        .into_iter()
        .map(|s| serde_json::to_value(s).unwrap_or_default())
        .collect();
    Ok(services)
}

#[tauri::command]
pub async fn switch_service_version(
    app: AppHandle,
    state: State<'_, ServiceProcesses>,
    id: String,
    version: String,
) -> Result<(), String> {
    let registry = ServiceRegistry::load_all(&app);
    let def = registry
        .get(&id)
        .ok_or_else(|| format!("Service '{}' not found", id))?;

    if !def.versions.contains_key(&version) {
        return Err(format!("Version '{}' not found for '{}'", version, id));
    }

    let key = format!("{}:{}", id, version);
    if state.instances.lock().await.contains_key(&key) {
        return Err(format!(
            "Version '{}' of '{}' is currently running. Stop it first.",
            version, id
        ));
    }

    let bin_dir = crate::paths::VoltPath::service_dir(&app, &id, &version);
    let binary_path = bin_dir.join(&def.binary_name);
    if !binary_path.exists() {
        return Err(format!(
            "Version '{}' of '{}' is not installed. Run setup first.",
            version, id
        ));
    }

    crate::paths::VoltPath::create_env_junction(&app, &id, &version)?;

    let mut settings = crate::settings::Settings::load(&app);
    settings.resolved_ports.remove(&id);
    settings.active_versions.insert(id.clone(), version.clone());
    let _ = settings.save(&app);

    app.emit(
        "service-status-changed",
        serde_json::json!({ "id": id, "status": "stopped", "version": version }),
    )
    .ok();

    Ok(())
}
