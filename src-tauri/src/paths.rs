use std::fs;
use std::path::PathBuf;
use tauri::{AppHandle, Manager};

pub struct VoltPath;

impl VoltPath {
    pub fn root(app: &AppHandle) -> PathBuf {
        let mut path = app
            .path()
            .app_local_data_dir()
            .unwrap_or_else(|_| PathBuf::from("."));
        path.pop();
        path.push("voltenv");
        path
    }

    pub fn bin_dir(app: &AppHandle) -> PathBuf {
        let mut path = Self::root(app);
        path.push("bin");
        path
    }

    pub fn service_dir(app: &AppHandle, id: &str, version: &str) -> PathBuf {
        let mut path = Self::bin_dir(app);
        path.push(id);
        path.push(version);
        path
    }

    pub fn service_binary_path(app: &AppHandle, id: &str, version: &str) -> PathBuf {
        let mut path = Self::service_dir(app, id, version);
        path.push(format!("{}{}", id, std::env::consts::EXE_SUFFIX));
        path
    }

    pub fn env_dir(app: &AppHandle) -> PathBuf {
        let mut path = Self::root(app);
        path.push("env");
        path
    }

    pub fn env_service_path(app: &AppHandle, service_id: &str) -> PathBuf {
        let mut path = Self::env_dir(app);
        path.push(service_id);
        path
    }

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

        if link.exists() {
            remove_existing_link(&link)?;
        }

        if let Some(parent) = link.parent() {
            fs::create_dir_all(parent).map_err(|e| format!("Failed to create env dir: {}", e))?;
        }

        create_link(&target, &link)
    }

    pub fn config_dir(app: &AppHandle) -> PathBuf {
        let mut path = Self::root(app);
        path.push("config");
        path
    }

    pub fn logs_dir(app: &AppHandle) -> PathBuf {
        let mut path = Self::root(app);
        path.push("logs");
        path
    }

    pub fn log_path(app: &AppHandle, id: &str, version: &str) -> PathBuf {
        let mut path = Self::logs_dir(app);
        path.push(format!("{}-{}.log", id, version));
        path
    }

    pub fn data_dir(app: &AppHandle, id: &str, version: &str) -> PathBuf {
        let mut path = Self::root(app);
        path.push("data");
        path.push(format!("{}-{}", id, version));
        path
    }

    pub fn config_path(app: &AppHandle, id: &str, version: &str) -> PathBuf {
        let mut path = Self::config_dir(app);
        path.push(format!("{}-{}.conf", id, version));
        path
    }

    pub fn vhosts_dir(app: &AppHandle) -> PathBuf {
    pub fn ssl_dir(app: &AppHandle) -> PathBuf {
        let mut path = Self::root(app);
        path.push("ssl");
        path
    }
        let mut path = Self::root(app);
        path.push("vhosts");
        path
    }

    pub fn www_dir(app: &AppHandle) -> PathBuf {
        let mut path = Self::root(app);
        path.push("www");
        path
    }

    pub fn backups_dir(app: &AppHandle) -> PathBuf {
        let mut path = Self::config_dir(app);
        path.push("backups");
        path
    }

    pub fn custom_services_dir(app: &AppHandle) -> PathBuf {
        let mut path = Self::config_dir(app);
        path.push("services");
        path.push("custom");
        path
    }

    pub fn builtin_overrides_dir(app: &AppHandle) -> PathBuf {
        let mut path = Self::config_dir(app);
        path.push("services");
        path.push("builtin");
        path
    }

    pub fn templates_dir(app: &AppHandle) -> PathBuf {
        let mut path = Self::config_dir(app);
        path.push("templates");
        path
    }

    pub fn temporary_download_path(app: &AppHandle, id: &str, version: &str, url: &str) -> PathBuf {
        let mut path = Self::bin_dir(app);
        let filename = url
            .split('/')
            .next_back()
            .filter(|f| f.contains('.'))
            .map(|f| format!("tmp_{}_{}_{}", id, version, f))
            .unwrap_or_else(|| format!("tmp_{}_{}.zip", id, version));
        path.push(filename);
        path
    }

    pub fn ensure_all_dirs(app: &AppHandle) -> std::io::Result<()> {
        for dir in [
            Self::bin_dir(app),
            Self::env_dir(app),
            Self::logs_dir(app),
            Self::config_dir(app),
            Self::backups_dir(app),
            Self::vhosts_dir(app),
    pub fn ssl_dir(app: &AppHandle) -> PathBuf {
        let mut path = Self::root(app);
        path.push("ssl");
        path
    }
            Self::www_dir(app),
            Self::custom_services_dir(app),
            Self::builtin_overrides_dir(app),
            Self::templates_dir(app),
        ] {
            if !dir.exists() {
                fs::create_dir_all(&dir)?;
            }
        }
        Ok(())
    }
}

#[cfg(target_os = "windows")]
fn remove_existing_link(link: &std::path::Path) -> Result<(), String> {
    if link.is_dir() {
        fs::remove_dir(link).map_err(|e| format!("Failed to remove junction: {}", e))
    } else {
        fs::remove_file(link).map_err(|e| format!("Failed to remove entry: {}", e))
    }
}

#[cfg(not(target_os = "windows"))]
fn remove_existing_link(link: &std::path::Path) -> Result<(), String> {
    fs::remove_file(link).map_err(|e| format!("Failed to remove symlink: {}", e))
}

#[cfg(target_os = "windows")]
fn create_link(target: &std::path::Path, link: &std::path::Path) -> Result<(), String> {
    junction::create(target, link).map_err(|e| format!("Failed to create junction: {}", e))
}

#[cfg(not(target_os = "windows"))]
fn create_link(target: &std::path::Path, link: &std::path::Path) -> Result<(), String> {
    std::os::unix::fs::symlink(target, link).map_err(|e| format!("Failed to create symlink: {}", e))
}
