use std::path::Path;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VhostInfo {
    pub domain: String,
    pub root: String,
    pub port: u16,
    pub php_port: Option<u16>,
}

pub struct VhostManager;

impl VhostManager {
    pub fn is_valid_domain(domain: &str) -> bool {
        if domain.is_empty() || domain.len() > 253 {
            return false;
        }
        // Basic domain validation (alphanumeric, dots, dashes)
        domain.chars().all(|c| c.is_alphanumeric() || c == '.' || c == '-')
            && !domain.starts_with('.')
            && !domain.ends_with('.')
            && !domain.contains("..")
    }

    pub fn sanitize_path(path: &str) -> String {
        // Remove characters that could break Nginx config or be used for injection
        path.replace('"', "").replace(';', "").replace('{', "").replace('}', "")
    }

    pub fn generate_server_block(
        domain: &str,
        root: &str,
        port: u16,
        php_port: Option<u16>,
    ) -> String {
        let safe_root = Self::sanitize_path(root);
        let mut block = format!(
            r#"server {{
    listen {};
    server_name {};
    root "{}";
    index index.html index.php;

    location / {{
        try_files $uri $uri/ =404;
    }}
"#,
            port, domain, safe_root
        );

        if let Some(php) = php_port {
            block.push_str(&format!(
                r#"
    location ~ \.php$ {{
        fastcgi_pass 127.0.0.1:{};
        fastcgi_param SCRIPT_FILENAME $document_root$fastcgi_script_name;
        include fastcgi_params;
    }}
"#,
                php
            ));
        }

        block.push_str("}\n");
        block
    }

    pub fn save_vhost(
        vhosts_dir: &Path,
        domain: &str,
        root: &str,
        port: u16,
        php_port: Option<u16>,
    ) -> Result<VhostInfo, String> {
        if !Self::is_valid_domain(domain) {
            return Err(format!("Invalid domain name: {}", domain));
        }

        std::fs::create_dir_all(vhosts_dir)
            .map_err(|e| format!("Failed to create vhosts dir: {}", e))?;

        let content = Self::generate_server_block(domain, root, port, php_port);

        let conf_path = vhosts_dir.join(format!("{}.conf", domain));
        std::fs::write(&conf_path, &content)
            .map_err(|e| format!("Failed to write vhost conf: {}", e))?;

        let info = VhostInfo {
            domain: domain.to_string(),
            root: root.to_string(),
            port,
            php_port,
        };

        let meta_path = vhosts_dir.join(format!("{}.json", domain));
        let meta_json = serde_json::to_string_pretty(&info)
            .map_err(|e| format!("Failed to serialize vhost metadata: {}", e))?;
        std::fs::write(&meta_path, &meta_json)
            .map_err(|e| format!("Failed to write vhost metadata: {}", e))?;

        Ok(info)
    }

    pub fn delete_vhost(vhosts_dir: &Path, domain: &str) -> Result<(), String> {
        if !Self::is_valid_domain(domain) {
            return Err(format!("Invalid domain name: {}", domain));
        }

        let conf_path = vhosts_dir.join(format!("{}.conf", domain));
        if conf_path.exists() {
            std::fs::remove_file(&conf_path)
                .map_err(|e| format!("Failed to delete vhost conf: {}", e))?;
        }

        let meta_path = vhosts_dir.join(format!("{}.json", domain));
        if meta_path.exists() {
            let _ = std::fs::remove_file(&meta_path);
        }

        Ok(())
    }

    pub fn list_vhosts(vhosts_dir: &Path) -> Result<Vec<VhostInfo>, String> {
        let mut vhosts = Vec::new();
        if !vhosts_dir.exists() {
            return Ok(vhosts);
        }

        let mut entries: Vec<_> = std::fs::read_dir(vhosts_dir)
            .map_err(|e| format!("Failed to read vhosts dir: {}", e))?
            .flatten()
            .collect();
        entries.sort_by_key(|e| e.file_name());

        for entry in entries {
            let path = entry.path();
            if path.extension().is_none_or(|e| e != "json") {
                continue;
            }

            if let Ok(content) = std::fs::read_to_string(&path) {
                if let Ok(info) = serde_json::from_str::<VhostInfo>(&content) {
                    vhosts.push(info);
                }
            }
        }

        Ok(vhosts)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_valid_domain() {
        assert!(VhostManager::is_valid_domain("example.com"));
        assert!(VhostManager::is_valid_domain("test-site.local"));
        assert!(!VhostManager::is_valid_domain("site;rm -rf /"));
        assert!(!VhostManager::is_valid_domain(".."));
        assert!(!VhostManager::is_valid_domain(".start"));
    }

    #[test]
    fn test_sanitize_path() {
        assert_eq!(VhostManager::sanitize_path("/var/www/site"), "/var/www/site");
        assert_eq!(VhostManager::sanitize_path("/var/www/site\";"), "/var/www/site");
    }
}
