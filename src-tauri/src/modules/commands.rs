use crate::modules::catalog::CatalogManager;
use crate::modules::download::DownloadManager;
use crate::modules::download::Verifier;
use crate::modules::installer::InstallerManager;
use crate::modules::paths::VoltPath;
use crate::modules::services::{KillStrategy, ServiceProcesses};
use std::collections::HashMap;
use std::fs;
use tauri::{AppHandle, State};

/// Removes a downloaded temp file. Errors are silently ignored since we
/// are already in an error path.
fn remove_download(path: &std::path::Path) {
    let _ = fs::remove_file(path);
}

/// Substitutes `{version}` placeholders in a template string.
fn substitute_version(template: &str, version: &str) -> String {
    template.replace("{version}", version)
}

// Provisioning: download → verify → install

/// Downloads a service archive (zip).  The download URL is resolved from
/// the catalog's `url_template` field by substituting `{version}` with the
/// requested version.  SHA‑256 and PGP signature URLs are resolved the same
/// way when present.
///
/// **Idempotent:** returns early if the target binary already exists.
#[tauri::command]
pub async fn download_service(
    app: AppHandle,
    catalog: State<'_, CatalogManager>,
    id: String,
    version: String,
) -> Result<(), String> {
    let def = catalog
        .by_id_and_version(&id, &version)
        .ok_or_else(|| format!("Unknown service: {} (version {})", id, version))?;

    let bin_dir = VoltPath::service_dir(&app, &id, &version);
    let bin_path = VoltPath::service_binary_path(&app, &id, &version);

    if bin_path.exists() {
        return Ok(());
    }

    tokio::fs::create_dir_all(&bin_dir)
        .await
        .map_err(|e| format!("Failed to create directory: {}", e))?;

    // Resolve download URL from catalog template
    let download_url = def
        .url_template
        .as_deref()
        .map(|t| substitute_version(t, &version))
        .ok_or_else(|| format!("No url_template for service {}", id))?;

    // Derive a version-specific filename so concurrent downloads of
    // different versions don't clobber each other.
    let base_temp_path = VoltPath::temporary_download_path(&app, &id);
    let temp_filename = format!("{}_{}.zip", id, version);
    let temp_path = base_temp_path.with_file_name(temp_filename);

    DownloadManager::download(&app, &id, &download_url, &temp_path).await?;

    // SHA‑256 verification
    if let Some(hash_template) = &def.sha256 {
        let expected = substitute_version(hash_template, &version);
        if !expected.is_empty() {
            if let Err(e) = Verifier::sha256(&temp_path, &expected).await {
                remove_download(&temp_path);
                let _ = tokio::fs::remove_dir(&bin_dir).await;
                return Err(format!("{}: {}", id, e));
            }
        }
    }

    // PGP signature verification
    if let Some(sig_template) = &def.pgp_signature_url_template {
        let sig_url = substitute_version(sig_template, &version);
        if !sig_url.is_empty() {
            if let Err(e) = Verifier::pgp_signature(&sig_url, &temp_path).await {
                remove_download(&temp_path);
                let _ = tokio::fs::remove_dir(&bin_dir).await;
                return Err(format!("{}: {}", id, e));
            }
        }
    }

    Ok(())
}

/// Extracts a previously downloaded zip archive into the service's
/// versioned directory.
///
/// **Idempotent:** returns early if the target binary already exists.
#[tauri::command]
pub async fn install_service(
    app: AppHandle,
    catalog: State<'_, CatalogManager>,
    id: String,
    version: String,
) -> Result<(), String> {
    let _ = catalog
        .by_id(&id)
        .ok_or_else(|| format!("Unknown service: {}", id))?;

    let bin_dir = VoltPath::service_dir(&app, &id, &version);
    let bin_path = VoltPath::service_binary_path(&app, &id, &version);
    // Must match the version-specific filename written by download_service.
    let base_temp = VoltPath::temporary_download_path(&app, &id);
    let temp_filename = format!("{}_{}.zip", id, version);
    let temp_path = base_temp.with_file_name(temp_filename);

    if bin_path.exists() {
        return Ok(());
    }

    if !temp_path.exists() {
        return Err(format!(
            "No downloaded archive found for {} version {}",
            id, version
        ));
    }

    InstallerManager::install(&app, &id, &bin_dir, &temp_path).await
}

// Version switching — junction / symlink management

/// Switches the active OS version for a service by re-creating the
/// `env/{service_id}` junction / symlink to point to the new versioned
/// binary directory.
///
/// The active selection is persisted in `current_versions.json` so the
/// frontend can display the current state across restarts.
///
/// Disk I/O (junction creation) and Registry / profile writes are
/// offloaded to `spawn_blocking` to keep the Tauri main thread responsive.
#[tauri::command]
pub async fn switch_service_version(
    app: AppHandle,
    catalog: State<'_, CatalogManager>,
    state: State<'_, ServiceProcesses>,
    id: String,
    version: String,
) -> Result<(), String> {
    // Refuse to switch while any version of this service is still running —
    // swapping the junction out from under a running binary would strand it.
    if state.is_any_version_running(&id)? {
        return Err(format!(
            "Cannot switch {} version while the service is still running. Stop it first.",
            id
        ));
    }

    // Verify the target version is installed (quick `stat`, no blocking needed)
    let bin_path = VoltPath::service_binary_path(&app, &id, &version);
    if !bin_path.exists() {
        return Err(format!(
            "Binary not found for {} version {} — install it first",
            id, version
        ));
    }

    let app2 = app.clone();
    let id2 = id.clone();
    let version2 = version.clone();

    // Junction creation is filesystem I/O that blocks; keep it in
    // spawn_blocking.
    tokio::task::spawn_blocking(move || {
        VoltPath::create_env_junction(&app2, &id2, &version2)?;
        Ok::<_, String>(())
    })
    .await
    .map_err(|e| format!("switch_service_version panicked: {}", e))??;

    // PATH registration is async (uses tokio::fs for backup writes);
    // run outside spawn_blocking so the async runtime can manage it.
    crate::modules::env::register_service_os_path(&app, &id).await?;

    // Fast operation — active version is cached in memory.
    catalog.set_active_version(&app, &id, &version)?;

    Ok(())
}

/// Returns the currently active version for every service known in the
/// catalog.  Services that have never been switched will not appear in the
/// map.
///
/// Reads from the in-memory `RwLock` cache — no disk access.
#[tauri::command]
pub fn get_active_versions(catalog: State<'_, CatalogManager>) -> HashMap<String, String> {
    catalog.all_active_versions()
}

// Lifecycle: start / hybrid stop / soft stop / force stop

/// Starts a service: spawns the binary with catalog-configured arguments,
/// injects the central bin directory into the child process PATH (isolated),
/// pipes stdout / stderr for real-time log streaming, and begins watching
/// for process exit.
#[tauri::command]
pub async fn start_service(
    app: AppHandle,
    catalog: State<'_, CatalogManager>,
    state: State<'_, ServiceProcesses>,
    id: String,
    version: String,
) -> Result<u32, String> {
    let def = catalog
        .by_id_and_version(&id, &version)
        .ok_or_else(|| format!("Unknown service: {} (version {})", id, version))?;

    let (bin_path, cwd) = VoltPath::resolve_service_paths(&app, &id, &version);

    let args: Vec<&str> = def.start_args.iter().map(|s| s.as_str()).collect();

    state
        .start(&app, &id, &version, &bin_path, &cwd, &args)
        .await
}

/// Hybrid stop: service-specific graceful shutdown (phase 1), then
/// soft‑kill with a 3 s grace window (phase 2), and finally force‑kill
/// the entire process tree (phase 3).
#[tauri::command]
pub async fn stop_service(
    app: AppHandle,
    catalog: State<'_, CatalogManager>,
    state: State<'_, ServiceProcesses>,
    id: String,
    version: String,
) -> Result<(), String> {
    let def = catalog
        .by_id_and_version(&id, &version)
        .ok_or_else(|| format!("Unknown service version: {} {}", id, version))?;

    if !def.stop_args.is_empty() {
        let (bin_path, cwd) = VoltPath::resolve_service_paths(&app, &id, &version);
        let args: Vec<&str> = def.stop_args.iter().map(|s| s.as_str()).collect();

        let mut cmd = tokio::process::Command::new(&bin_path);
        cmd.args(&args).current_dir(&cwd);

        // Inject the isolated VoltEnv PATH so the stop binary can find its
        // runtime dependencies.
        crate::modules::env::prepare_command_env(&mut cmd, &app);

        let _ = cmd.spawn();
    }

    state.stop(&id, &version, KillStrategy::Hybrid).await
}

/// Sends a soft termination signal (SIGTERM / `taskkill /PID`).
#[tauri::command]
pub async fn soft_stop_service(
    state: State<'_, ServiceProcesses>,
    id: String,
    version: String,
) -> Result<(), String> {
    state.stop(&id, &version, KillStrategy::Soft).await
}

/// Immediately terminates the entire process tree (SIGKILL / `taskkill /F /T`).
#[tauri::command]
pub async fn force_stop_service(
    state: State<'_, ServiceProcesses>,
    id: String,
    version: String,
) -> Result<(), String> {
    state.stop(&id, &version, KillStrategy::Force).await
}
