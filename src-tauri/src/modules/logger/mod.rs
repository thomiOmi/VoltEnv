use std::path::{Path, PathBuf};
use tauri::{AppHandle, Manager};
use tokio::fs::{self, OpenOptions};
use tokio::io::AsyncWriteExt;

pub struct LogManager;

impl LogManager {
    /// Returns the log directory path at $ROOT/logs
    fn logs_dir(app: &AppHandle) -> PathBuf {
        let mut path = app
            .path()
            .app_local_data_dir()
            .unwrap_or_else(|_| PathBuf::from("."));
        path.pop();
        path.push("voltenv");
        path.push("logs");
        path
    }

    /// Writes a log line to $ROOT/logs/{service_id}.log with automatic rotation
    pub async fn write_service_log(
        app: &AppHandle,
        service_id: &str,
        message: &str,
    ) -> Result<(), String> {
        let dir = Self::logs_dir(app);
        if let Err(e) = fs::create_dir_all(&dir).await {
            return Err(format!("Failed to create log directory: {}", e));
        }

        let log_path = dir.join(format!("{}.log", service_id));

        // Trigger rotation when the file exceeds 10 MB
        if log_path.exists() {
            if let Ok(meta) = fs::metadata(&log_path).await {
                if meta.len() > 10 * 1024 * 1024 {
                    if let Err(e) = Self::rotate_logs(&dir, service_id).await {
                        eprintln!("[VoltEnv Logger] Failed to rotate log: {}", e);
                    }
                }
            }
        }

        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&log_path)
            .await
            .map_err(|e| format!("Failed to open log file: {}", e))?;

        let log_line = format!("{}\n", message);
        file.write_all(log_line.as_bytes())
            .await
            .map_err(|e| format!("Failed to write log: {}", e))?;

        Ok(())
    }

    async fn rotate_logs(dir: &Path, service_id: &str) -> std::io::Result<()> {
        // Remove the oldest backup (.3) if present
        let log3 = dir.join(format!("{}.log.3", service_id));
        let _ = fs::remove_file(&log3).await;

        // Shift .2 -> .3
        let log2 = dir.join(format!("{}.log.2", service_id));
        if log2.exists() {
            let _ = fs::rename(&log2, &log3).await;
        }

        // Shift .1 -> .2
        let log1 = dir.join(format!("{}.log.1", service_id));
        if log1.exists() {
            let _ = fs::rename(&log1, &log2).await;
        }

        // Shift .log -> .1
        let log_base = dir.join(format!("{}.log", service_id));
        if log_base.exists() {
            let _ = fs::rename(&log_base, &log1).await;
        }

        Ok(())
    }
}

/// Tauri command to read the last N lines from a service log file efficiently.
#[tauri::command]
pub async fn get_service_logs(
    app: tauri::AppHandle,
    id: String,
    lines_count: usize,
) -> Result<Vec<String>, String> {
    let dir = LogManager::logs_dir(&app);
    let log_path = dir.join(format!("{}.log", id));

    if !log_path.exists() {
        return Ok(vec![format!("-- No log entries yet for service {} --", id)]);
    }

    // Read the file asynchronously
    let content = fs::read_to_string(&log_path)
        .await
        .map_err(|e| format!("Failed to read log history: {}", e))?;

    let lines: Vec<String> = content
        .lines()
        .rev() // Reverse so we take from the end
        .take(lines_count)
        .map(|s| s.to_string())
        .collect();

    // Reverse again so the result is chronological (oldest first)
    let mut final_lines = lines;
    final_lines.reverse();

    Ok(final_lines)
}
