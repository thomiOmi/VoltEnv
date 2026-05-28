use directories::ProjectDirs;
use std::fs;
use std::path::PathBuf;

/// VoltEnv path management module.
/// Uses a dedicated folder under the user's local data directory
/// to store binaries on demand.
pub struct VoltPath;

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

    /// Path to the binary directory
    pub fn bin_dir() -> PathBuf {
        let mut path = Self::root();
        path.push("bin");
        path
    }

    /// Ensures the binary directory exists, creating it if necessary
    pub fn ensure_dirs() -> std::io::Result<()> {
        let bin_path = Self::bin_dir();
        if !bin_path.exists() {
            fs::create_dir_all(&bin_path)?;
        }
        Ok(())
    }

    /// Returns the full path for a given binary, appending the platform-specific extension
    pub fn get_binary_path(name: &str) -> PathBuf {
        let mut path = Self::bin_dir();

        #[cfg(target_os = "windows")]
        let filename = format!("{}.exe", name);

        #[cfg(not(target_os = "windows"))]
        let filename = name.to_string();

        path.push(filename);
        path
    }

    /// Checks whether a binary is already present on disk
    #[allow(dead_code)]
    pub fn is_binary_present(name: &str) -> bool {
        Self::get_binary_path(name).exists()
    }
}
