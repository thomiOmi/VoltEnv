use std::fs;
use std::path::PathBuf;
use tauri::{AppHandle, Manager};

/// VoltEnv path management — resolves data directories under the OS-native
/// local data root (e.g. `%LOCALAPPDATA%\voltenv` on Windows).
///
/// Layout:
/// ```text
/// $ROOT/
///   bin/{service_id}/{version}/{service_id}.exe      ← installed binaries
///   env/
///     {service_id}/  → junction → bin/{service_id}/{version}  ← active version
/// ```
///
/// Each `env/{service_id}` folder is a **junction** (Windows) or **symlink**
/// (Unix) pointing to the versioned binary directory inside `bin/`.  Only
/// the `env/` subtree is registered on the OS PATH — never `bin/` directly.
pub struct VoltPath;

impl VoltPath {
    /// Returns the root directory (`$APPDATA/voltenv`).
    fn root(app: &AppHandle) -> PathBuf {
        let mut path = app
            .path()
            .app_local_data_dir()
            .unwrap_or_else(|_| PathBuf::from("."));
        path.pop();
        path.push("voltenv");
        path
    }

    /// Returns `$ROOT/bin` — the parent directory for all service installations.
    pub fn bin_dir(app: &AppHandle) -> PathBuf {
        let mut path = Self::root(app);
        path.push("bin");
        path
    }

    /// Returns the versioned directory for a specific service
    /// (`$ROOT/bin/{service_id}/{version}`).
    pub fn service_dir(app: &AppHandle, service_id: &str, version: &str) -> PathBuf {
        let mut path = Self::bin_dir(app);
        path.push(service_id);
        path.push(version);
        path
    }

    /// Returns the path to the primary executable for a service
    /// (`$ROOT/bin/{service_id}/{version}/{service_id}.exe`).
    ///
    /// The binary name is derived from `service_id` with the platform
    /// executable suffix (`.exe` on Windows, empty on Unix).
    pub fn service_binary_path(app: &AppHandle, service_id: &str, version: &str) -> PathBuf {
        let mut path = Self::service_dir(app, service_id, version);
        path.push(format!("{}{}", service_id, std::env::consts::EXE_SUFFIX));
        path
    }

    /// Returns `$ROOT/env` — the parent directory of per-service junction
    /// / symlink folders.
    pub fn env_dir(app: &AppHandle) -> PathBuf {
        let mut path = Self::root(app);
        path.push("env");
        path
    }

    /// Returns the per-service junction / symlink folder
    /// (`$ROOT/env/{service_id}`).
    ///
    /// This folder is a **directory junction** (Windows) or **symlink**
    /// (Unix) pointing to the active version directory inside `bin/`.  When
    /// the user switches versions the junction is re-created to point to the
    /// new version.
    pub fn env_service_path(app: &AppHandle, service_id: &str) -> PathBuf {
        let mut path = Self::env_dir(app);
        path.push(service_id);
        path
    }

    /// Creates or replaces the junction / symlink at `$ROOT/env/{service_id}`
    /// so it points to `$ROOT/bin/{service_id}/{version}/`.
    ///
    /// ## Platform behaviour
    ///
    /// | Platform | Mechanism         | Elevation required |
    /// |----------|-------------------|-------------------|
    /// | Windows  | NTFS junction     | No                |
    /// | Unix     | Symlink           | No                |
    ///
    /// Any pre-existing entry at the link path is removed first.
    pub fn create_env_junction(
        app: &AppHandle,
        service_id: &str,
        version: &str,
    ) -> Result<(), String> {
        let target = Self::service_dir(app, service_id, version);
        let link = Self::env_service_path(app, service_id);

        if !target.exists() {
            return Err(format!("Version directory not found: {}", target.display()));
        }

        // Remove existing link if present
        if link.exists() {
            remove_existing_link(&link)?;
        }

        // Ensure parent $ROOT/env/ exists
        if let Some(parent) = link.parent() {
            fs::create_dir_all(parent).map_err(|e| format!("Failed to create env dir: {}", e))?;
        }

        create_link(&target, &link)
    }

    /// Returns `$ROOT/config` — the directory for configuration files
    /// (`catalog.json`, `current_versions.json`, and the `backups/` subdir).
    pub fn config_dir(app: &AppHandle) -> PathBuf {
        let mut path = Self::root(app);
        path.push("config");
        path
    }

    /// Returns `$ROOT/config/backups` — the directory for PATH backup files.
    pub fn backups_dir(app: &AppHandle) -> PathBuf {
        let mut path = Self::config_dir(app);
        path.push("backups");
        path
    }

    /// Creates `$ROOT/bin`, `$ROOT/env`, and `$ROOT/config/backups` if they
    /// do not exist.  Called once at application startup from `main.rs` setup.
    pub fn ensure_all_dirs(app: &AppHandle) -> std::io::Result<()> {
        let bin_path = Self::bin_dir(app);
        if !bin_path.exists() {
            fs::create_dir_all(&bin_path)?;
        }
        let env_path = Self::env_dir(app);
        if !env_path.exists() {
            fs::create_dir_all(&env_path)?;
        }
        let backups_path = Self::backups_dir(app);
        if !backups_path.exists() {
            fs::create_dir_all(&backups_path)?;
        }
        Ok(())
    }

    // -- Directory layout helpers -------------------------------------------

    /// Resolves the on-disk binary path and working directory for a service
    /// at a specific version.
    pub fn resolve_service_paths(app: &AppHandle, id: &str, version: &str) -> (String, String) {
        let bin = Self::service_binary_path(app, id, version);
        let cwd = Self::service_dir(app, id, version);
        (
            bin.to_string_lossy().to_string(),
            cwd.to_string_lossy().to_string(),
        )
    }

    /// Returns the download path for a temporary zip archive
    /// (`$ROOT/bin/tmp_{id}.zip`).
    pub fn temporary_download_path(app: &tauri::AppHandle, id: &str) -> std::path::PathBuf {
        let mut path = Self::bin_dir(app);
        path.push(format!("tmp_{}.zip", id));
        path
    }
}

// -- private helpers --------------------------------------------------------

/// Removes the existing junction / symlink at `link`.
#[cfg(target_os = "windows")]
fn remove_existing_link(link: &std::path::Path) -> Result<(), String> {
    // Junctions are removed as directories
    if link.is_dir() {
        fs::remove_dir(link).map_err(|e| format!("Failed to remove existing junction: {}", e))
    } else {
        fs::remove_file(link).map_err(|e| format!("Failed to remove existing entry: {}", e))
    }
}

#[cfg(not(target_os = "windows"))]
fn remove_existing_link(link: &std::path::Path) -> Result<(), String> {
    fs::remove_file(link).map_err(|e| format!("Failed to remove existing symlink: {}", e))
}

/// Creates a filesystem junction (Windows) or symlink (Unix) from `link` →
/// `target`.
///
/// ## Windows notes
///
/// Uses the `junction` crate which creates a true NTFS junction via
/// `FSCTL_SET_REPARSE_POINT`.  Junctions are created at user‑level
/// (no Developer Mode / Administrator elevation required), making them
/// suitable for non‑admin development environments.
#[cfg(target_os = "windows")]
fn create_link(target: &std::path::Path, link: &std::path::Path) -> Result<(), String> {
    junction::create(target, link).map_err(|e| format!("Failed to create junction: {}", e))
}

#[cfg(not(target_os = "windows"))]
fn create_link(target: &std::path::Path, link: &std::path::Path) -> Result<(), String> {
    std::os::unix::fs::symlink(target, link).map_err(|e| format!("Failed to create symlink: {}", e))
}
