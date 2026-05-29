use crate::modules::catalog::models::ServiceConfig;
use std::collections::HashMap;
use std::fs;
use std::sync::RwLock;
use tauri::AppHandle;
use tauri::Manager;

/// Manages the service catalog loaded from `$APPDATA/voltenv/config/catalog.json`.
///
/// On first launch the file is created automatically with sensible defaults
/// (Nginx, PHP, MySQL, Redis), so the app never crashes on a missing config.
///
/// Active versions are cached in an in-memory `RwLock<HashMap>` to avoid
/// repeated disk reads and to guarantee thread-safe concurrent access.
pub struct CatalogManager {
    services: Vec<ServiceConfig>,
    active_versions: RwLock<HashMap<String, String>>,
}

impl CatalogManager {
    /// Returns the config directory path.
    fn config_dir(app: &AppHandle) -> std::path::PathBuf {
        let mut path = app
            .path()
            .app_local_data_dir()
            .unwrap_or_else(|_| std::path::PathBuf::from("."));
        path.pop();
        path.push("voltenv");
        path.push("config");
        path
    }

    /// Returns the full path to `catalog.json`.
    fn catalog_path(app: &AppHandle) -> std::path::PathBuf {
        let mut path = Self::config_dir(app);
        path.push("catalog.json");
        path
    }

    /// Returns the default service catalog embedded in the binary.
    fn default_catalog() -> Vec<ServiceConfig> {
        vec![
            ServiceConfig {
                id: "nginx".into(),
                version: "nginx-1.26.2".into(),
                start_args: vec!["-c".into(), "conf/nginx.conf".into()],
                stop_args: vec!["-s".into(), "stop".into()],
                url_template: Some("https://nginx.org/download/nginx-{version}.zip".into()),
                sha256: None,
                pgp_signature_url_template: Some(
                    "https://nginx.org/download/nginx-{version}.zip.asc".into(),
                ),
            },
            ServiceConfig {
                id: "php".into(),
                version: "unknown".into(),
                start_args: vec![],
                stop_args: vec![],
                url_template: None,
                sha256: None,
                pgp_signature_url_template: None,
            },
            ServiceConfig {
                id: "mysql".into(),
                version: "unknown".into(),
                start_args: vec![],
                stop_args: vec![],
                url_template: None,
                sha256: None,
                pgp_signature_url_template: None,
            },
            ServiceConfig {
                id: "redis".into(),
                version: "redis-7.2.14".into(),
                start_args: vec!["--port".into(), "6379".into()],
                stop_args: vec![],
                url_template: Some(
                    "https://github.com/redis-windows/redis-windows/releases/download/{version}/Redis-{version}-Windows-x64-msys2.zip".into(),
                ),
                sha256: Some("B31D0F867608017F0B0962624D55A4C569A745587AD4B08F7FE9EEA59D6916C1".into()),
                pgp_signature_url_template: None,
            },
            ServiceConfig {
                id: "redis".into(),
                version: "redis-7.0.15".into(),
                start_args: vec!["--port".into(), "6379".into()],
                stop_args: vec![],
                url_template: Some(
                    "https://github.com/redis-windows/redis-windows/releases/download/7.0.15/Redis-7.0.15-Windows-x64-msys2.zip".into(),
                ),
                sha256: None,
                pgp_signature_url_template: None,
            },
        ]
    }

    /// Writes the embedded default catalog to disk, creating the config
    /// directory if necessary.
    fn write_default_catalog(
        app: &AppHandle,
        path: &std::path::Path,
    ) -> Result<Vec<ServiceConfig>, String> {
        let config_dir = Self::config_dir(app);
        fs::create_dir_all(&config_dir)
            .map_err(|e| format!("Failed to create config dir: {}", e))?;

        let defaults = Self::default_catalog();
        let json = serde_json::to_string_pretty(&defaults)
            .map_err(|e| format!("Failed to serialize default catalog: {}", e))?;

        fs::write(path, &json).map_err(|e| format!("Failed to write catalog.json: {}", e))?;

        eprintln!(
            "[voltenv] Created default catalog.json at {}",
            path.display()
        );

        Ok(defaults)
    }

    /// Loads the catalog from `catalog.json`, or creates the file with
    /// default entries when it does not exist.
    ///
    /// This function is called once at application startup.  The returned
    /// `CatalogManager` is registered as Tauri managed state so every
    /// command can access it via `State<'_, CatalogManager>`.
    pub fn load_or_create(app: &AppHandle) -> Result<Self, String> {
        let path = Self::catalog_path(app);

        let services: Vec<ServiceConfig> = if path.exists() {
            match fs::read_to_string(&path) {
                Ok(content) => match serde_json::from_str(&content) {
                    Ok(catalog) => catalog,
                    Err(e) => {
                        // Corrupt file — rename to .bad so user can inspect,
                        // then create fresh defaults on disk
                        let backup = path.with_extension("json.bad");
                        let _ = fs::rename(&path, &backup);
                        eprintln!(
                            "[voltenv] corrupt catalog.json — renamed to {}: {}",
                            backup.display(),
                            e
                        );
                        Self::write_default_catalog(app, &path)?
                    }
                },
                Err(e) => {
                    // Read error — same salvage strategy
                    let backup = path.with_extension("json.bad");
                    let _ = fs::rename(&path, &backup);
                    eprintln!(
                        "[voltenv] unreadable catalog.json — renamed to {}: {}",
                        backup.display(),
                        e
                    );
                    Self::write_default_catalog(app, &path)?
                }
            }
        } else {
            Self::write_default_catalog(app, &path)?
        };

        // Load persisted active versions into in-memory cache
        let versions = Self::load_current_versions(app);

        Ok(Self {
            services,
            active_versions: RwLock::new(versions),
        })
    }

    /// Returns the catalog entry for `id`, or `None` if not found.
    ///
    /// **WARNING:** Returns ONLY the first matching entry for the given ID.
    /// For multi-version inspection, use `all_versions_by_id` or
    /// `by_id_and_version`.
    pub fn by_id(&self, id: &str) -> Option<&ServiceConfig> {
        self.services.iter().find(|s| s.id == id)
    }

    /// Returns all catalog entries matching a specific service ID.
    ///
    /// Useful when a service (e.g. Redis) has multiple versions registered
    /// in the catalog — the caller can iterate over all matching entries
    /// instead of relying on `by_id` which only returns the first match.
    #[allow(dead_code)]
    pub fn all_versions_by_id(&self, id: &str) -> Vec<&ServiceConfig> {
        self.services.iter().filter(|s| s.id == id).collect()
    }

    /// Returns the catalog entry matching both `id` and `version`.
    pub fn by_id_and_version(&self, id: &str, version: &str) -> Option<&ServiceConfig> {
        self.services
            .iter()
            .find(|s| s.id == id && s.version == version)
    }

    /// Returns an empty catalog, used as a fallback when loading fails.
    pub fn empty() -> Self {
        Self {
            services: Vec::new(),
            active_versions: RwLock::new(HashMap::new()),
        }
    }

    // -- current_versions.json support --------------------------------------

    /// Path to the `current_versions.json` file.
    fn current_versions_path(app: &AppHandle) -> std::path::PathBuf {
        let mut path = Self::config_dir(app);
        path.push("current_versions.json");
        path
    }

    /// Loads `current_versions.json`, returning a map of `service_id →
    /// version` or an empty map if the file does not exist or fails to parse.
    ///
    /// ## Corruption handling
    ///
    /// If the file exists but cannot be read or parsed, it is renamed to
    /// `current_versions.json.bad` so the user can inspect the data (no data
    /// loss).  A warning is printed and an empty map is returned so the
    /// application continues normally.
    fn load_current_versions(app: &AppHandle) -> HashMap<String, String> {
        let path = Self::current_versions_path(app);
        if !path.exists() {
            return HashMap::new();
        }

        let content = match fs::read_to_string(&path) {
            Ok(c) => c,
            Err(e) => {
                let backup = path.with_extension("json.bad");
                let _ = fs::rename(&path, &backup);
                eprintln!(
                    "[voltenv] unreadable current_versions.json — renamed to {}: {}",
                    backup.display(),
                    e
                );
                return HashMap::new();
            }
        };

        match serde_json::from_str(&content) {
            Ok(map) => map,
            Err(e) => {
                let backup = path.with_extension("json.bad");
                let _ = fs::rename(&path, &backup);
                eprintln!(
                    "[voltenv] corrupt current_versions.json — renamed to {}: {}",
                    backup.display(),
                    e
                );
                HashMap::new()
            }
        }
    }

    /// Persists the map of `service_id → version` to `current_versions.json`.
    fn save_current_versions(
        app: &AppHandle,
        versions: &HashMap<String, String>,
    ) -> Result<(), String> {
        let path = Self::current_versions_path(app);
        let json = serde_json::to_string_pretty(versions)
            .map_err(|e| format!("Failed to serialize versions: {}", e))?;
        fs::write(&path, &json).map_err(|e| format!("Failed to write current_versions.json: {}", e))
    }

    /// Sets the active version for `service_id`, updates the in-memory cache,
    /// and persists to disk.
    pub fn set_active_version(
        &self,
        app: &AppHandle,
        service_id: &str,
        version: &str,
    ) -> Result<(), String> {
        {
            let mut map = self
                .active_versions
                .write()
                .map_err(|e| format!("RwLock poisoned: {}", e))?;
            map.insert(service_id.to_string(), version.to_string());
        }
        // Persist a snapshot — drop the write lock first to avoid contention
        // during the file write.
        let snapshot = self
            .active_versions
            .read()
            .map_err(|e| format!("RwLock poisoned: {}", e))?
            .clone();
        Self::save_current_versions(app, &snapshot)
    }

    /// Returns the full `HashMap<String, String>` of active versions from
    /// the in-memory cache — no disk reads.
    pub fn all_active_versions(&self) -> HashMap<String, String> {
        self.active_versions
            .read()
            .map(|map| map.clone())
            .unwrap_or_default()
    }
}
