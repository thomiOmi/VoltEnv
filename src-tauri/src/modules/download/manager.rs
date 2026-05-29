use crate::utils::http_client;
use futures_util::StreamExt;
use std::path::{Path, PathBuf};
use tauri::{AppHandle, Emitter};
use tokio::io::AsyncWriteExt;

/// Handles HTTP download of service archives (zip) with real-time progress
/// events streamed to the Tauri frontend.
///
/// All file I/O uses `tokio::fs` so the async runtime is never blocked by
/// disk writes.
pub struct DownloadManager;

impl DownloadManager {
    /// Downloads a file from `url` and writes it to `dest_path`.
    ///
    /// Emits `"download-progress"` events on the `app` handle as chunks
    /// are received, so the frontend can display a progress bar.
    ///
    /// Returns `Ok(())` when the entire file has been written and flushed
    /// to disk.  The caller is responsible for directory creation.
    pub async fn download(
        app: &AppHandle,
        id: &str,
        url: &str,
        dest_path: &Path,
    ) -> Result<(), String> {
        let response = http_client()
            .get(url)
            .send()
            .await
            .map_err(|e| format!("Download failed: {}", e))?;

        let total_size = response.content_length().unwrap_or(0);

        // Write to a .part file so a failed download doesn't leave a corrupt
        // archive at the final destination path.
        let part_path = part_path_for(dest_path);
        let mut temp_file = tokio::fs::File::create(&part_path)
            .await
            .map_err(|e| format!("Failed to create temp file: {}", e))?;

        let mut stream = response.bytes_stream();
        let mut downloaded: u64 = 0;
        let mut last_emitted_progress: u8 = 0;

        let loop_result: Result<(), String> = async {
            while let Some(chunk_result) = stream.next().await {
                let chunk = chunk_result.map_err(|e| format!("Stream error: {}", e))?;
                temp_file
                    .write_all(&chunk)
                    .await
                    .map_err(|e| format!("Write error: {}", e))?;
                downloaded += chunk.len() as u64;

                if total_size > 0 {
                    let progress = (downloaded as f64 / total_size as f64 * 100.0) as u8;
                    if progress > last_emitted_progress {
                        last_emitted_progress = progress;
                        let _ = app.emit(
                            "download-progress",
                            serde_json::json!({ "id": id, "progress": progress }),
                        );
                    }
                }
            }
            Ok(())
        }
        .await;

        // If the stream loop failed, remove the partial file before returning.
        if let Err(e) = loop_result {
            let _ = tokio::fs::remove_file(&part_path).await;
            return Err(e);
        }

        temp_file
            .flush()
            .await
            .map_err(|e| format!("Failed to flush temp file: {}", e))?;

        // Atomically move the completed .part file to the final destination.
        tokio::fs::rename(&part_path, dest_path)
            .await
            .map_err(|e| format!("Failed to finalize download file: {}", e))?;

        // Ensure the frontend receives a definitive 100% completion event
        // even if the last chunk was too small to cross the threshold.
        let _ = app.emit(
            "download-progress",
            serde_json::json!({ "id": id, "progress": 100 }),
        );

        Ok(())
    }
}

/// Returns a sibling path with `.part` appended to the extension
/// (e.g. `nginx-1.24.0.zip` → `nginx-1.24.0.zip.part`).
fn part_path_for(dest: &Path) -> PathBuf {
    let mut p = dest.to_path_buf();
    if let Some(ext) = p.extension() {
        let mut new_ext = ext.to_os_string();
        new_ext.push(".part");
        p.set_extension(new_ext);
    } else {
        p.set_extension("part");
    }
    p
}
