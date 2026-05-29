use crate::modules::paths::VoltPath;
use tauri::AppHandle;
use tokio::fs;

// Automatic Environment Backup — safety net for OS PATH modifications

/// Creates a timestamped backup of the current PATH value **before** any
/// modification, stored at `$ROOT/config/backups/env_before_{service_id}_{timestamp}.bak`.
///
/// A maximum of **5 backup files** is retained per service.  When the limit
/// is exceeded the oldest file is removed (backup rotation).
///
/// ## Fail‑safe contract
///
/// Every OS‑level PATH write in the application **must** call this function
/// first.  If backup creation fails the write is aborted entirely —
/// the original PATH is never touched.
pub async fn create_env_backup(
    service_id: &str,
    current_path: &str,
    app: &AppHandle,
) -> Result<(), String> {
    let backups_dir = VoltPath::backups_dir(app);

    // Ensure the backups directory exists
    fs::create_dir_all(&backups_dir)
        .await
        .map_err(|e| format!("Failed to create backups dir: {}", e))?;

    // Generate unique backup filename with millisecond precision to prevent
    // collisions when this function is called multiple times within the same
    // wall-clock second.
    let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S%.3f").to_string();
    let timestamp_clean = timestamp.replace('.', "_");
    let filename = format!("env_before_{}_{}.bak", service_id, timestamp_clean);
    let filepath = backups_dir.join(&filename);

    fs::write(&filepath, current_path)
        .await
        .map_err(|e| format!("Failed to write backup {}: {}", filepath.display(), e))?;

    // Rotate old backups: keep at most 5 per service
    rotate_backups(&backups_dir, service_id, 5).await;

    Ok(())
}

/// Keeps at most `max_count` backup files matching the service prefix,
/// removing the oldest entries when the limit is exceeded.
///
/// All I/O is non-blocking (`tokio::fs`) and runs asynchronously.
async fn rotate_backups(backups_dir: &std::path::Path, service_id: &str, max_count: usize) {
    let prefix = format!("env_before_{}_", service_id);

    let mut rd = match tokio::fs::read_dir(backups_dir).await {
        Ok(dir) => dir,
        Err(_) => return,
    };

    let mut candidates = Vec::new();
    while let Some(entry) = rd.next_entry().await.unwrap_or(None) {
        if entry.file_name().to_string_lossy().starts_with(&prefix) {
            candidates.push(entry.path());
        }
    }

    if candidates.len() <= max_count {
        return;
    }

    // Fetch modification times asynchronously
    let mut with_time = Vec::new();
    for path in candidates {
        if let Ok(meta) = tokio::fs::metadata(&path).await {
            if let Ok(time) = meta.modified() {
                with_time.push((path, time));
            }
        }
    }

    // Sort by oldest modification time
    with_time.sort_by_key(|(_, time)| *time);

    let to_remove = with_time.len().saturating_sub(max_count);
    for (path, _) in with_time.iter().take(to_remove) {
        let _ = tokio::fs::remove_file(path).await;
    }
}
