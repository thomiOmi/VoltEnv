use crate::modules::download::DownloadManager;
use crate::modules::download::Verifier;
use crate::modules::installer::InstallerManager;
use crate::modules::paths::VoltPath;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use tauri::{AppHandle, Emitter};

/// A single downloadable asset in a provisioning manifest.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Asset {
    /// Human-readable name used in log messages, event payloads, and
    /// temporary file names.
    pub name: String,
    /// Remote URL to download the asset from.
    pub url: String,
    /// Relative subdirectory inside `target_root` to place or extract this
    /// asset into.  Use `""` to place files directly into `target_root`.
    #[serde(default)]
    pub destination_subdir: String,
    /// Optional SHA‑256 hex digest for integrity verification.
    /// When `None`, integrity checking is skipped.
    #[serde(default)]
    pub sha256: Option<String>,
    /// Optional PGP signature URL for authenticity verification.
    /// The signature is downloaded, then `gpg --verify` is run against the
    /// downloaded asset.  When `None`, authenticity checking is skipped.
    #[serde(default)]
    pub pgp_signature_url: Option<String>,
    /// When `true`, the downloaded file is treated as a zip archive and
    /// extracted into `target_root / destination_subdir`.  When `false`,
    /// the raw downloaded file is placed (renamed) at that location.
    #[serde(default)]
    pub extract: bool,
}

/// A manifest describing a set of assets to provision for a single service.
///
/// Each asset is processed sequentially in declaration order.  The caller
/// provides a `target_root` directory under which every asset is placed or
/// extracted, allowing a single manifest to populate multiple subdirectories
/// from different remote sources.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DownloadManifest {
    pub assets: Vec<Asset>,
}

/// Processes every asset in `manifest` sequentially through the
/// download → verify → install pipeline.
///
/// # Pipeline per asset
///
/// 1. **Download** — `DownloadManager::download` streams the asset from
///    `asset.url` to a temporary location inside `$ROOT/bin/`.  Progress
///    events (`"download-progress"`) are emitted automatically by the
///    download manager.
/// 2. **SHA‑256** — If `asset.sha256` is set, `Verifier::sha256` compares
///    the file content against the expected digest.
/// 3. **PGP** — If `asset.pgp_signature_url` is set, the signature is
///    downloaded and `Verifier::pgp_signature` runs `gpg --verify`.
/// 4. **Install** — If `asset.extract` is `true`, `InstallerManager::install`
///    extracts the zip archive into `target_root / destination_subdir`.
///    Otherwise the raw file is renamed to that location.
///
/// # Abort-on-failure semantics
///
/// If any step fails, the function immediately stops, emits an
/// `"orchestrator-error"` event with the asset name and error detail,
/// cleans up any temporary files left by the failed step, and propagates
/// the error to the caller.  Subsequent assets are never processed.
///
/// An `"orchestrator-step"` event is emitted at the start of each asset
/// so the frontend can display per-asset progress.
pub async fn process_download_manifest(
    app: &AppHandle,
    id: &str,
    manifest: DownloadManifest,
    target_root: &Path,
) -> Result<(), String> {
    for asset in &manifest.assets {
        let temp_path = asset_temp_path(app, id, &asset.name);
        let _ = app.emit(
            "orchestrator-step",
            serde_json::json!({ "id": id, "asset": asset.name, "step": "download" }),
        );

        // ── 1. Download ──────────────────────────────────────────────────
        DownloadManager::download(app, id, &asset.url, &temp_path)
            .await
            .map_err(|e| {
                let msg = format!("[{}] Download failed: {}", asset.name, e);
                let _ = app.emit(
                    "orchestrator-error",
                    serde_json::json!({ "id": id, "asset": asset.name, "error": msg }),
                );
                msg
            })?;

        // ── 2. SHA‑256 integrity verification ────────────────────────────
        if let Some(ref expected_hash) = asset.sha256 {
            let _ = app.emit(
                "orchestrator-step",
                serde_json::json!({ "id": id, "asset": asset.name, "step": "verify-sha256" }),
            );

            if let Err(e) = Verifier::sha256(&temp_path, expected_hash).await {
                let _ = std::fs::remove_file(&temp_path);
                let msg = format!("[{}] SHA‑256 mismatch: {}", asset.name, e);
                let _ = app.emit(
                    "orchestrator-error",
                    serde_json::json!({ "id": id, "asset": asset.name, "error": msg }),
                );
                return Err(msg);
            }
        }

        // ── 3. PGP authenticity verification ─────────────────────────────
        if let Some(ref sig_url) = asset.pgp_signature_url {
            let _ = app.emit(
                "orchestrator-step",
                serde_json::json!({ "id": id, "asset": asset.name, "step": "verify-pgp" }),
            );

            if let Err(e) = Verifier::pgp_signature(sig_url, &temp_path).await {
                let _ = std::fs::remove_file(&temp_path);
                let msg = format!("[{}] PGP verification failed: {}", asset.name, e);
                let _ = app.emit(
                    "orchestrator-error",
                    serde_json::json!({ "id": id, "asset": asset.name, "error": msg }),
                );
                return Err(msg);
            }
        }

        // ── 4. Install (extract or place) ────────────────────────────────
        let dest_dir = if asset.destination_subdir.is_empty() {
            target_root.to_path_buf()
        } else {
            target_root.join(&asset.destination_subdir)
        };

        if asset.extract {
            let _ = app.emit(
                "orchestrator-step",
                serde_json::json!({ "id": id, "asset": asset.name, "step": "extract" }),
            );

            InstallerManager::install(app, id, &dest_dir, &temp_path)
                .await
                .map_err(|e| {
                    // InstallerManager removes dest_dir on failure but leaves the
                    // zip — clean it up ourselves using sync I/O (we're inside a
                    // sync closure, not an async context).
                    let _ = std::fs::remove_file(&temp_path);
                    let msg = format!("[{}] Extraction failed: {}", asset.name, e);
                    let _ = app.emit(
                        "orchestrator-error",
                        serde_json::json!({ "id": id, "asset": asset.name, "error": msg }),
                    );
                    msg
                })?;
            // On success, InstallerManager already removed the zip.
        } else {
            // Non-extract asset: atomically move the downloaded file.
            let _ = app.emit(
                "orchestrator-step",
                serde_json::json!({ "id": id, "asset": asset.name, "step": "place" }),
            );

            tokio::fs::create_dir_all(&dest_dir).await.map_err(|e| {
                let msg = format!("[{}] Failed to create destination: {}", asset.name, e);
                let _ = app.emit(
                    "orchestrator-error",
                    serde_json::json!({ "id": id, "asset": asset.name, "error": msg }),
                );
                msg
            })?;

            let dest_path = dest_dir.join(&asset.name);
            tokio::fs::rename(&temp_path, &dest_path)
                .await
                .map_err(|e| {
                    let _ = std::fs::remove_file(&temp_path);
                    let msg = format!("[{}] Failed to place file: {}", asset.name, e);
                    let _ = app.emit(
                        "orchestrator-error",
                        serde_json::json!({ "id": id, "asset": asset.name, "error": msg }),
                    );
                    msg
                })?;
        }
    }

    Ok(())
}

/// Returns a unique temporary download path for an asset inside `$ROOT/bin/`.
///
/// The path includes both `id` and `asset_name` so that concurrent
/// orchestrations for different services or different assets of the same
/// service do not collide.
fn asset_temp_path(app: &AppHandle, id: &str, asset_name: &str) -> PathBuf {
    let mut path = VoltPath::bin_dir(app);
    path.push(format!("tmp_{id}_{name}", id = id, name = asset_name));
    path
}
