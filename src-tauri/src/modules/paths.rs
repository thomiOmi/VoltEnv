use directories::ProjectDirs;
use std::fs;
use std::path::PathBuf;

/// VoltEnv path management module.
/// Uses a dedicated folder under the user's local data directory
/// to store binaries on demand.
pub struct VoltPath;

#[allow(dead_code)]
impl VoltPath {
    /// Returns the VoltEnv root path (e.g. ~/.voltenv or OS equivalent)
    pub fn root() -> PathBuf {
        ProjectDirs::from("com", "voltenv", "app")
            .map(|proj| proj.data_local_dir().to_path_buf())
            .unwrap_or_else(|| {
                // Fallback if ProjectDirs fails
                let mut path = dirs::home_dir().unwrap_or_default();
                path.push(".voltenv");
                path
            })
    }

    /// Path to the binary root directory
    pub fn bin_dir() -> PathBuf {
        let mut path = Self::root();
        path.push("bin");
        path
    }

    /// Ensures the binary root directory exists, creating it if necessary
    pub fn ensure_dirs() -> std::io::Result<()> {
        let bin_path = Self::bin_dir();
        if !bin_path.exists() {
            fs::create_dir_all(&bin_path)?;
        }
        Ok(())
    }

    /// Returns the versioned service directory: bin/{service_id}/{version}/
    pub fn service_dir(service_id: &str, version: &str) -> PathBuf {
        let mut path = Self::bin_dir();
        path.push(service_id);
        path.push(version);
        path
    }

    /// Returns the full path to a service binary within its versioned directory
    pub fn service_binary_path(service_id: &str, version: &str) -> PathBuf {
        let mut path = Self::service_dir(service_id, version);
        #[cfg(target_os = "windows")]
        path.push(format!("{}.exe", service_id));
        #[cfg(not(target_os = "windows"))]
        path.push(service_id.to_string());
        path
    }

    /// Path to the temp directory (for downloads, extraction, etc.)
    pub fn temp_dir() -> PathBuf {
        let mut path = Self::root();
        path.push("temp");
        path
    }
}
