use crate::http_client;
use crate::utils::{VoltError, VoltResult};
use futures_util::StreamExt;
use std::path::{Path, PathBuf};
use tauri::{AppHandle, Emitter};
use tokio::io::AsyncWriteExt;

pub struct DownloadManager {
    app: AppHandle,
}

impl DownloadManager {
    pub fn new(app: &AppHandle) -> Self {
        Self {
            app: app.clone(),
        }
    }

    pub async fn download_and_install(
        &self,
        def: &crate::service::ServiceDefinition,
        version: &str,
    ) -> VoltResult<()> {
        let v_info = def.versions.get(version).ok_or_else(|| {
            VoltError::Service(format!("Version {} not found for {}", version, def.id))
        })?;

        let url = &v_info.download_url;

        let temp_archive = crate::paths::VoltPath::temporary_download_path(&self.app, &def.id, version, url);
        self.download(&def.id, url, &temp_archive).await?;

        let bin_dir = crate::paths::VoltPath::service_dir(&self.app, &def.id, version);
        crate::installer::manager::InstallerManager::install(&self.app, &def.id, &bin_dir, &temp_archive).await?;

        Ok(())
    }

    pub async fn download(
        &self,
        id: &str,
        url: &str,
        dest_path: &Path,
    ) -> VoltResult<()> {
        let response = http_client().get(url).send().await?;

        let total_size = response.content_length().unwrap_or(0);
        let part_path = part_path_for(dest_path);

        if let Some(parent) = part_path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }

        let mut temp_file = tokio::fs::File::create(&part_path).await?;

        let mut stream = response.bytes_stream();
        let mut downloaded: u64 = 0;
        let mut last_emitted_progress: u8 = 0;

        while let Some(chunk_result) = stream.next().await {
            let chunk = chunk_result?;
            temp_file.write_all(&chunk).await?;
            downloaded += chunk.len() as u64;

            if total_size > 0 {
                let progress = (downloaded as f64 / total_size as f64 * 100.0) as u8;
                if progress > last_emitted_progress {
                    last_emitted_progress = progress;
                    let _ = self.app.emit(
                        "download-progress",
                        serde_json::json!({ "id": id, "progress": progress }),
                    );
                }
            }
        }

        temp_file.flush().await?;
        tokio::fs::rename(&part_path, dest_path).await?;

        let _ = self.app.emit(
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
