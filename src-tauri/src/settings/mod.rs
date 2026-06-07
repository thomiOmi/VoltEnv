use std::collections::HashMap;
use std::path::PathBuf;
use tauri::AppHandle;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AutoStartGroup {
    pub name: String,
    pub services: Vec<String>,
    #[serde(default)]
    pub auto_start: bool,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Settings {
    #[serde(default)]
    pub preferred_ports: HashMap<String, u16>,
    #[serde(default)]
    pub resolved_ports: HashMap<String, u16>,
    #[serde(default)]
    pub auto_start_groups: Vec<AutoStartGroup>,
    #[serde(default)]
    pub active_versions: HashMap<String, String>,
}

impl Default for Settings {
    fn default() -> Self {
        let mut preferred = HashMap::new();
        preferred.insert("nginx".to_string(), 8080);
        preferred.insert("php".to_string(), 9000);
        preferred.insert("mysql".to_string(), 3306);

        Self {
            preferred_ports: preferred,
            resolved_ports: HashMap::new(),
            auto_start_groups: vec![AutoStartGroup {
                name: "Web Stack".to_string(),
                services: vec!["nginx".to_string(), "php".to_string(), "mysql".to_string()],
                auto_start: true,
            }],
            active_versions: HashMap::new(),
        }
    }
}

impl Settings {
    pub fn path(app: &AppHandle) -> PathBuf {
        crate::paths::VoltPath::config_dir(app).join("settings.json")
    }

    pub fn load(app: &AppHandle) -> Self {
        let path = Self::path(app);
        if path.exists() {
            match std::fs::read_to_string(&path) {
                Ok(content) => match serde_json::from_str::<Settings>(&content) {
                    Ok(settings) => return settings,
                    Err(e) => {
                        eprintln!("[voltenv] Corrupt settings.json: {}. Using defaults.", e);
                        let _ = std::fs::rename(&path, path.with_extension("json.bad"));
                    }
                },
                Err(e) => {
                    eprintln!("[voltenv] Failed to read settings: {}", e);
                }
            }
        }
        let settings = Settings::default();
        let _ = settings.save(app);
        settings
    }

    pub fn save(&self, app: &AppHandle) -> Result<(), String> {
        let path = Self::path(app);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create config dir: {}", e))?;
        }
        let content = serde_json::to_string_pretty(self)
            .map_err(|e| format!("Failed to serialize settings: {}", e))?;
        std::fs::write(&path, &content).map_err(|e| format!("Failed to write settings: {}", e))?;
        Ok(())
    }
}
