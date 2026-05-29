use std::fs;
use std::path::Path;
use tauri::{AppHandle, Emitter};
use zip::ZipArchive;

/// Extracts downloaded service archives into versioned binary directories.
///
/// All zip I/O runs inside `tokio::task::spawn_blocking` so the async
/// runtime is never blocked by decompression or disk writes.
pub struct InstallerManager;

impl InstallerManager {
    /// Extracts `zip_path` into `bin_dir`, emitting `"install-progress"`
    /// events as each entry is written.
    ///
    /// **Idempotent:** the caller should check whether the target binary
    /// already exists before calling this method.
    ///
    /// **Security:** a Zip Slip guard rejects entries whose resolved path
    /// escapes the target directory.
    pub async fn install(
        app: &AppHandle,
        id: &str,
        bin_dir: &Path,
        zip_path: &Path,
    ) -> Result<(), String> {
        let app = app.clone();
        let id = id.to_string();
        let bin_dir = bin_dir.to_path_buf();
        let zip_path = zip_path.to_path_buf();

        tokio::task::spawn_blocking(move || extract_zip_sync(app, bin_dir, zip_path, id))
            .await
            .map_err(|e| format!("Extraction task panicked: {}", e))?
    }
}

/// Synchronous zip extraction — designed to run on the blocking thread pool.
///
/// Strips a common root directory if present (e.g. `nginx-1.26.2/` becomes
/// the extraction root), creates the target directory tree, and emits
/// `"install-progress"` events.
fn extract_zip_sync(
    app: AppHandle,
    bin_dir: std::path::PathBuf,
    zip_path: std::path::PathBuf,
    id: String,
) -> Result<(), String> {
    let zip_file = fs::File::open(&zip_path).map_err(|e| format!("Failed to open zip: {}", e))?;
    let mut archive =
        ZipArchive::new(zip_file).map_err(|e| format!("Failed to read zip: {}", e))?;

    // Determine the common root directory (if any) by checking the first
    // entry, then validating that ALL entries share that prefix.  Archives
    // without a wrapping directory (e.g. PHP Windows with a flat `ext/`
    // subfolder) get an empty zip_root so every entry is extracted relative
    // to bin_dir.
    let candidate_root = if !archive.is_empty() {
        archive
            .by_index(0)
            .ok()
            .and_then(|e| e.name().split('/').next().map(|s| s.to_string()))
    } else {
        None
    };

    let zip_root = if let Some(ref root) = candidate_root {
        let mut is_common_root = true;
        for i in 0..archive.len() {
            if let Ok(entry) = archive.by_index(i) {
                let name = entry.name();
                if !name.starts_with(&format!("{}/", root))
                    && name != root
                    && !name.starts_with(root)
                {
                    is_common_root = false;
                    break;
                }
            }
        }
        if is_common_root {
            root.clone()
        } else {
            String::new()
        }
    } else {
        String::new()
    };

    let total_count = archive.len();
    let mut last_emitted_progress: u8 = 0;

    let extraction_result: Result<(), String> = (|| {
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

            if !target.starts_with(&bin_dir) {
                return Err(format!(
                    "Zip entry '{}' attempts path traversal (target: {})",
                    full_name,
                    target.display()
                ));
            }

            if entry.is_dir() {
                fs::create_dir_all(&target)
                    .map_err(|e| format!("Failed to create dir {}: {}", target.display(), e))?;
            } else {
                if let Some(parent) = target.parent() {
                    fs::create_dir_all(parent).map_err(|e| {
                        format!("Failed to create parent: {}: {}", parent.display(), e)
                    })?;
                }
                let mut out = fs::File::create(&target)
                    .map_err(|e| format!("Failed to create file {}: {}", target.display(), e))?;
                std::io::copy(&mut entry, &mut out)
                    .map_err(|e| format!("Failed to write file {}: {}", target.display(), e))?;

                #[cfg(unix)]
                {
                    use std::os::unix::fs::PermissionsExt;
                    if let Some(mode) = entry.unix_mode() {
                        let _ = fs::set_permissions(&target, fs::Permissions::from_mode(mode));
                    }
                }
            }

            let install_progress = ((i + 1) as f64 / total_count as f64 * 100.0) as u8;
            if install_progress > last_emitted_progress {
                last_emitted_progress = install_progress;
                let _ = app.emit(
                    "install-progress",
                    serde_json::json!({ "id": id, "progress": install_progress }),
                );
            }
        }
        Ok(())
    })();

    // On any extraction failure, remove the corrupted target directory
    // so the system is never left in a half-installed state.
    if let Err(e) = extraction_result {
        let _ = fs::remove_dir_all(&bin_dir);
        return Err(e);
    }

    // Extraction succeeded — clean up the source zip.
    let _ = fs::remove_file(&zip_path);

    Ok(())
}
