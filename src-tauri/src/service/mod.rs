pub mod php_ini;
use crate::utils::{VoltError, VoltResult};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VersionInfo {
    #[serde(default)]
    pub download_url: String,
    #[serde(default)]
    pub sha256: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HealthCheckConfig {
    #[serde(rename = "type")]
    pub check_type: String,
    #[serde(default)]
    pub command: Option<String>,
    #[serde(default)]
    pub timeout_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ServiceDefinition {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub kind: String,
    #[serde(default = "default_version")]
    pub default_version: String,
    #[serde(default)]
    pub versions: HashMap<String, VersionInfo>,
    pub binary_name: String,
    #[serde(default)]
    pub start_args: Vec<String>,
    #[serde(default)]
    pub stop_args: Vec<String>,
    #[serde(default)]
    pub port: u16,
    #[serde(default)]
    pub config_template_name: Option<String>,
    #[serde(default)]
    pub health_check: Option<HealthCheckConfig>,
    #[serde(default)]
    pub post_install_commands: Vec<String>,
}

fn default_version() -> String {
    "0.0.0".to_string()
}

impl ServiceDefinition {
    pub fn download_url_for(&self, version: &str) -> Option<&str> {
        self.versions.get(version).map(|v| v.download_url.as_str())
    }

    pub fn sha256_for(&self, version: &str) -> Option<Option<&str>> {
        self.versions.get(version).map(|v| v.sha256.as_deref())
    }

    pub fn merge_from(&mut self, other: ServiceDefinition) {
        if other.id != self.id {
            return;
        }
        if !other.name.is_empty() {
            self.name = other.name;
        }
        if !other.kind.is_empty() {
            self.kind = other.kind;
        }
        if other.default_version != default_version() {
            self.default_version = other.default_version;
        }
        if !other.binary_name.is_empty() {
            self.binary_name = other.binary_name;
        }
        if !other.start_args.is_empty() {
            self.start_args = other.start_args;
        }
        if !other.stop_args.is_empty() {
            self.stop_args = other.stop_args;
        }
        if other.port != 0 {
            self.port = other.port;
        }
        if other.config_template_name.is_some() {
            self.config_template_name = other.config_template_name;
        }
        if other.health_check.is_some() {
            self.health_check = other.health_check;
        }
        if !other.post_install_commands.is_empty() {
            self.post_install_commands = other.post_install_commands;
        }
        self.versions.extend(other.versions);
    }
}

pub struct ServiceRegistry {
    pub services: HashMap<String, ServiceDefinition>,
}

impl ServiceRegistry {
    pub fn new() -> Self {
        Self {
            services: HashMap::new(),
        }
    }

    pub fn get(&self, id: &str) -> Option<&ServiceDefinition> {
        self.services.get(id)
    }

    pub fn all(&self) -> Vec<&ServiceDefinition> {
        self.services.values().collect()
    }

    pub fn load_embedded() -> Self {
        let mut registry = Self::new();

        let embedded = [
            include_str!("../../embedded/services/nginx.json"),
            include_str!("../../embedded/services/php.json"),
            include_str!("../../embedded/services/mysql.json"),
        ];

        for json in embedded {
            match serde_json::from_str::<ServiceDefinition>(json) {
                Ok(def) => {
                    registry.services.insert(def.id.clone(), def);
                }
                Err(e) => {
                    eprintln!("[voltenv] Failed to parse embedded service: {}", e);
                }
            }
        }

        registry
    }

    pub fn load_from_dir(&mut self, dir: &Path) {
        let dir_entries = match std::fs::read_dir(dir) {
            Ok(entries) => entries,
            Err(_) => return,
        };

        for entry in dir_entries.flatten() {
            let path = entry.path();
            if path.extension().is_none_or(|e| e != "json") {
                continue;
            }

            match std::fs::read_to_string(&path) {
                Ok(content) => match serde_json::from_str::<ServiceDefinition>(&content) {
                    Ok(def) => {
                        let id = def.id.clone();
                        if let Some(existing) = self.services.get_mut(&id) {
                            existing.merge_from(def);
                        } else {
                            self.services.insert(id, def);
                        }
                    }
                    Err(e) => {
                        eprintln!(
                            "[voltenv] Failed to parse service '{}': {}",
                            path.display(),
                            e
                        );
                    }
                },
                Err(e) => {
                    eprintln!("[voltenv] Failed to read '{}': {}", path.display(), e);
                }
            }
        }
    }

    pub fn load_all(app: &tauri::AppHandle) -> Self {
        use crate::paths::VoltPath;

        let mut registry = Self::load_embedded();
        registry.load_from_dir(&VoltPath::builtin_overrides_dir(app));
        registry.load_from_dir(&VoltPath::custom_services_dir(app));
        registry
    }
}

impl Default for ServiceRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_service_definition_merge() {
        let mut base = ServiceDefinition {
            id: "test".to_string(),
            name: "Test".to_string(),
            kind: "test".to_string(),
            default_version: "1.0.0".to_string(),
            versions: HashMap::new(),
            binary_name: "test.exe".to_string(),
            start_args: vec![],
            stop_args: vec![],
            port: 80,
            config_template_name: None,
            health_check: None,
            post_install_commands: vec![],
        };

        let other = ServiceDefinition {
            id: "test".to_string(),
            name: "Updated".to_string(),
            kind: "test".to_string(),
            default_version: "2.0.0".to_string(),
            versions: HashMap::new(),
            binary_name: "test2.exe".to_string(),
            start_args: vec![],
            stop_args: vec![],
            port: 8080,
            config_template_name: Some("test.conf".to_string()),
            health_check: None,
            post_install_commands: vec![],
        };

        base.merge_from(other);
        assert_eq!(base.name, "Updated");
        assert_eq!(base.default_version, "2.0.0");
        assert_eq!(base.port, 8080);
    }
}
