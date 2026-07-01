use std::fs;
use std::path::Path;
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhpExtension {
    pub name: String,
    pub enabled: bool,
}

pub struct PhpIniManager;

impl PhpIniManager {
    pub fn get_extensions(ini_path: &Path) -> Result<Vec<PhpExtension>, String> {
        if !ini_path.exists() {
            return Ok(vec![]);
        }

        let content = fs::read_to_string(ini_path).map_err(|e| e.to_string())?;
        let mut extensions = Vec::new();

        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.contains("extension=") || trimmed.contains("zend_extension=") {
                let is_enabled = !trimmed.starts_with(';');
                let clean_line = trimmed.trim_start_matches(';');
                let parts: Vec<&str> = clean_line.split('=').collect();
                if parts.len() >= 2 {
                    let name = parts[1].trim();
                    if !name.is_empty() {
                        extensions.push(PhpExtension {
                            name: name.to_string(),
                            enabled: is_enabled,
                        });
                    }
                }
            }
        }

        Ok(extensions)
    }

    pub fn toggle_extension(ini_path: &Path, extension: &str, enable: bool) -> Result<(), String> {
        let content = fs::read_to_string(ini_path).map_err(|e| e.to_string())?;
        let mut new_lines = Vec::new();
        let mut found = false;

        for line in content.lines() {
            let trimmed = line.trim();
            let clean_line = trimmed.trim_start_matches(';');
            if clean_line.contains(&format!("extension={}", extension))
                || clean_line.contains(&format!("zend_extension={}", extension))
            {
                if enable {
                    new_lines.push(clean_line.to_string());
                } else {
                    new_lines.push(format!(";{}", clean_line));
                }
                found = true;
            } else {
                new_lines.push(line.to_string());
            }
        }

        if !found && enable {
            new_lines.push(format!("extension={}", extension));
        }

        fs::write(ini_path, new_lines.join("\n")).map_err(|e| e.to_string())?;
        Ok(())
    }
}
