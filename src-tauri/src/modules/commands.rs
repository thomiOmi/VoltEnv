use crate::modules::paths::VoltPath;
use crate::modules::services::{ServiceDefinition, ServiceProcesses};
use std::fs;
use std::io::{Cursor, Read};
use tauri::{AppHandle, Emitter, State};
use tauri_plugin_shell::ShellExt;
use zip::ZipArchive;

#[tauri::command]
pub async fn provision_service(app: AppHandle, id: String) -> Result<(), String> {
    let def = ServiceDefinition::by_id(&id).ok_or_else(|| format!("Unknown service: {}", id))?;

    let bin_dir = VoltPath::service_dir(def.id, def.version);
    let bin_path = VoltPath::service_binary_path(def.id, def.version);

    if bin_path.exists() {
        return Ok(());
    }
    fs::create_dir_all(&bin_dir).map_err(|e| format!("Failed to create dir: {}", e))?;

    if def.download_url.is_empty() {
        return Ok(());
    }

    let _ = app.emit(
        "provision-progress",
        serde_json::json!({"id": id, "progress": 0}),
    );

    let response = reqwest::get(def.download_url)
        .await
        .map_err(|e| format!("Download failed: {}", e))?;

    let bytes = response
        .bytes()
        .await
        .map_err(|e| format!("Download body failed: {}", e))?;

    let _ = app.emit(
        "provision-progress",
        serde_json::json!({"id": id, "progress": 50}),
    );

    let reader = Cursor::new(&bytes);
    let mut archive = ZipArchive::new(reader).map_err(|e| format!("Failed to read zip: {}", e))?;

    let zip_root: String = {
        let names: Vec<String> = (0..archive.len())
            .filter_map(|i| archive.name_for_index(i).map(|s| s.to_string()))
            .collect();
        names
            .iter()
            .find(|n| n.contains('/'))
            .and_then(|n| n.split('/').next())
            .unwrap_or("")
            .to_string()
    };

    let total_count = archive.len();
    for i in 0..total_count {
        let mut entry = archive
            .by_index(i)
            .map_err(|e| format!("Zip entry error: {}", e))?;
        let full_name = entry.name().to_string();

        let relative = if zip_root.is_empty() {
            full_name.clone()
        } else {
            full_name
                .strip_prefix(&format!("{}/", zip_root))
                .unwrap_or(&full_name)
                .to_string()
        };

        if relative.is_empty() {
            continue;
        }

        let target = bin_dir.join(&relative);

        if entry.is_dir() {
            fs::create_dir_all(&target)
                .map_err(|e| format!("Failed to create dir {}: {}", target.display(), e))?;
        } else {
            if let Some(parent) = target.parent() {
                fs::create_dir_all(parent)
                    .map_err(|e| format!("Failed to create parent {}: {}", parent.display(), e))?;
            }
            let mut out = fs::File::create(&target)
                .map_err(|e| format!("Failed to create file {}: {}", target.display(), e))?;
            let mut data = Vec::new();
            entry
                .read_to_end(&mut data)
                .map_err(|e| format!("Failed to read entry: {}", e))?;
            std::io::copy(&mut Cursor::new(data), &mut out)
                .map_err(|e| format!("Failed to write file {}: {}", target.display(), e))?;
        }

        let _ = app.emit(
            "provision-progress",
            serde_json::json!({
                "id": id,
                "progress": 50 + ((i + 1) * 50 / total_count)
            }),
        );
    }

    Ok(())
}

#[tauri::command]
pub async fn start_service(
    app: AppHandle,
    state: State<'_, ServiceProcesses>,
    id: String,
) -> Result<u32, String> {
    let def = ServiceDefinition::by_id(&id).ok_or_else(|| format!("Unknown service: {}", id))?;

    let bin_path = VoltPath::service_binary_path(def.id, def.version);
    let cwd = VoltPath::service_dir(def.id, def.version);

    state
        .start(
            &app,
            def.id,
            &bin_path.to_string_lossy(),
            &cwd.to_string_lossy(),
            def.start_args,
        )
        .await
}

#[tauri::command]
pub async fn stop_service(
    app: AppHandle,
    state: State<'_, ServiceProcesses>,
    id: String,
) -> Result<(), String> {
    // Graceful stop via the service binary itself (fire-and-forget)
    if let Some(def) = ServiceDefinition::by_id(&id) {
        if !def.stop_args.is_empty() {
            let bin_path = VoltPath::service_binary_path(def.id, def.version);
            let cwd = VoltPath::service_dir(def.id, def.version);

            let _ = app
                .shell()
                .command(&bin_path)
                .args(def.stop_args)
                .current_dir(&cwd)
                .spawn();
        }
    }

    state.stop(&id)
}
