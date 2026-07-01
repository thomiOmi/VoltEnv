use crate::http_client;
use crate::utils::{VoltError, VoltResult};
use futures_util::StreamExt;
use std::path::{Path, PathBuf};
use tauri::{AppHandle, Emitter};
use tokio::io::AsyncWriteExt;

pub struct DownloadManager;

impl DownloadManager {
    pub async fn download(
        app: &AppHandle,
        id: &str,
        url: &str,
        dest_path: &Path,
    ) -> VoltResult<()> {
        let response = http_client().get(url).send().await?;

        let total_size = response.content_length().unwrap_or(0);
        let part_path = part_path_for(dest_path);
        let mut temp_file = tokio::fs::File::create(&part_path).await?;

        let mut stream = response.bytes_stream();
        let mut downloaded: u64 = 0;
        let mut last_emitted_progress: u8 = 0;

        let loop_result: VoltResult<()> = async {
            while let Some(chunk_result) = stream.next().await {
                let chunk = chunk_result?;
                temp_file.write_all(&chunk).await?;
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

        if let Err(e) = loop_result {
            let _ = tokio::fs::remove_file(&part_path).await;
            return Err(e);
        }

        temp_file.flush().await?;

        tokio::fs::rename(&part_path, dest_path).await?;

        let _ = app.emit(
            "download-progress",
            serde_json::json!({ "id": id, "progress": 100 }),
        );

        Ok(())
    }
}

fn part_path_for(dest: &Path) -> PathBuf {
    let mut p = dest.to_path_buf();
    if let Some(ext) = p.extension() {
        let new_ext = format!("{}.part", ext.to_string_lossy());
        p.set_extension(new_ext);
    } else {
        p.set_extension("part");
    }
    p
}
