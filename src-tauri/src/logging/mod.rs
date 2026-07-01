pub mod system;

use crate::utils::VoltResult;
use std::path::{Path, PathBuf};
use tauri::AppHandle;
use tokio::fs::{self, OpenOptions};
use tokio::io::AsyncWriteExt;

pub struct LogManager;

impl LogManager {
    pub fn logs_dir(app: &AppHandle) -> PathBuf {
        crate::paths::VoltPath::logs_dir(app)
    }

    pub async fn write_service_log(
        app: &AppHandle,
        service_id: &str,
        message: &str,
    ) -> VoltResult<()> {
        let dir = Self::logs_dir(app);
        fs::create_dir_all(&dir).await?;

        let log_path = dir.join(format!("{}.log", service_id));

        if log_path.exists() {
            if let Ok(meta) = fs::metadata(&log_path).await {
                if meta.len() > 10 * 1024 * 1024 {
                    let _ = Self::rotate_logs(&dir, service_id).await;
                }
            }
        }

        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&log_path)
            .await?;

        let log_line = format!("{}\n", message);
        file.write_all(log_line.as_bytes()).await?;

        Ok(())
    }

    async fn rotate_logs(dir: &Path, service_id: &str) -> std::io::Result<()> {
        let log3 = dir.join(format!("{}.log.3", service_id));
        let _ = fs::remove_file(&log3).await;

        let log2 = dir.join(format!("{}.log.2", service_id));
        if log2.exists() {
            let _ = fs::rename(&log2, &log3).await;
        }

        let log1 = dir.join(format!("{}.log.1", service_id));
        if log1.exists() {
            let _ = fs::rename(&log1, &log2).await;
        }

        let log_base = dir.join(format!("{}.log", service_id));
        if log_base.exists() {
            let _ = fs::rename(&log_base, &log1).await;
        }

        Ok(())
    }
}

pub async fn get_service_logs(
    app: &tauri::AppHandle,
    id: &str,
    version: &str,
    lines_count: usize,
) -> VoltResult<Vec<String>> {
    let log_path = crate::paths::VoltPath::log_path(app, id, version);

    if !log_path.exists() {
        return Ok(vec![format!("-- No log entries for {} {} --", id, version)]);
    }

    let content = fs::read_to_string(&log_path).await?;

    let lines: Vec<String> = content
        .lines()
        .rev()
        .take(lines_count)
        .map(|s| s.to_string())
        .collect();

    let mut result = lines;
    result.reverse();
    Ok(result)
}
