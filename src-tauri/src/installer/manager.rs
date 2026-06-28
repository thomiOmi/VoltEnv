use std::path::Path;
use tauri::{AppHandle, Emitter};
use crate::utils::{VoltResult, VoltError};

pub struct InstallerManager;

impl InstallerManager {
    pub async fn install(
        app: &AppHandle,
        id: &str,
        bin_dir: &Path,
        archive_path: &Path,
    ) -> VoltResult<()> {
        let data = tokio::fs::read(archive_path).await?;

        let format = detect_format(archive_path).map_err(VoltError::Custom)?;
        let extractor = archive::ArchiveExtractor::new()
            .with_max_file_size(100 * 1024 * 1024)
            .with_max_total_size(500 * 1024 * 1024);

        let files = extractor
            .extract(&data, format)
            .map_err(|e| VoltError::Custom(format!("Extraction failed: {}", e)))?;

        let common_root = detect_common_root(&files);

        let total = files.len();
        let app_clone = app.clone();
        let id_owned = id.to_string();
        let bin = bin_dir.to_path_buf();
        let bin_rollback = bin.clone();

        tokio::task::spawn_blocking(move || {
            for (i, file) in files.iter().enumerate() {
                let file_path = Path::new(&file.path);
                let relative = if let Some(ref root) = common_root {
                    file_path.strip_prefix(Path::new(root)).unwrap_or(file_path)
                } else {
                    file_path
                };

                if relative.as_os_str().is_empty() {
                    continue;
                }

                let target = bin.join(relative);

                if !target.starts_with(&bin) {
                    return Err(VoltError::Custom(format!("Path traversal: {}", file_path.display())));
                }

                if file.is_directory {
                    let _ = std::fs::create_dir_all(&target);
                } else {
                    if let Some(parent) = target.parent() {
                        let _ = std::fs::create_dir_all(parent);
                    }
                    if let Err(e) = std::fs::write(&target, &file.data) {
                        return Err(VoltError::Custom(format!("Failed to write {}: {}", target.display(), e)));
                    }
                }

                let progress = ((i + 1) as f64 / total as f64 * 100.0) as u8;
                let _ = app_clone.emit(
                    "install-progress",
                    serde_json::json!({ "id": id_owned, "progress": progress }),
                );
            }
            Ok(())
        })
        .await
        .map_err(|e| VoltError::Custom(format!("Extraction task panicked: {}", e)))??;

        let _ = tokio::fs::remove_file(archive_path).await;

        Ok(())
    }
}

fn detect_format(path: &Path) -> Result<archive::ArchiveFormat, String> {
    let name = path.to_string_lossy().to_lowercase();
    if name.ends_with(".zip") {
        Ok(archive::ArchiveFormat::Zip)
    } else if name.ends_with(".tar.gz") || name.ends_with(".tgz") {
        Ok(archive::ArchiveFormat::TarGz)
    } else if name.ends_with(".tar.bz2") || name.ends_with(".tbz2") {
        Ok(archive::ArchiveFormat::TarBz2)
    } else if name.ends_with(".tar.xz") || name.ends_with(".txz") {
        Ok(archive::ArchiveFormat::TarXz)
    } else if name.ends_with(".7z") {
        Ok(archive::ArchiveFormat::SevenZ)
    } else if name.ends_with(".tar") {
        Ok(archive::ArchiveFormat::Tar)
    } else if name.ends_with(".gz") {
        Ok(archive::ArchiveFormat::Gz)
    } else {
        Err(format!("Unknown archive format: {}", name))
    }
}

fn detect_common_root(files: &[archive::ExtractedFile]) -> Option<String> {
    let first = files.first()?;
    let first_path = Path::new(&first.path);
    let first_component = first_path.components().next()?;
    let root = first_component.as_os_str().to_str()?.to_string();

    if files.iter().all(|f| {
        let fp = Path::new(&f.path);
        fp.starts_with(&root) || fp == Path::new(&root)
    }) {
        Some(root)
    } else {
        None
    }
}
