pub mod hosts;
pub mod ssl;
use std::path::Path;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VhostInfo {
    pub domain: String,
    pub root: String,
    pub port: u16,
    pub php_port: Option<u16>,
    pub ssl: bool,
}

pub struct VhostManager;

impl VhostManager {
    pub fn generate_server_block(
        domain: &str,
        root: &str,
        port: u16,
        php_port: Option<u16>,
        ssl_config: Option<(&Path, &Path)>,
    ) -> String {
        let mut block = String::new();

        block.push_str(&format!(
            r#"server {{
    listen {};
    server_name {};
    root "{}";
    index index.html index.php;

    location / {{
        try_files $uri $uri/ =404;
    }}
"#,
            port,
            domain,
            root.replace("\\", "/")
        ));

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

        if let Some((cert, key)) = ssl_config {
            block.push_str(&format!(
                r#"
server {{
    listen 443 ssl;
    server_name {};
    root "{}";
    index index.html index.php;

    ssl_certificate "{}";
    ssl_certificate_key "{}";

    location / {{
        try_files $uri $uri/ =404;
    }}
"#,
                domain,
                root.replace("\\", "/"),
                cert.to_string_lossy().replace("\\", "/"),
                key.to_string_lossy().replace("\\", "/")
            ));

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
        }

        block
    }

    pub fn save_vhost(
        vhosts_dir: &Path,
        domain: &str,
        root: &str,
        port: u16,
        php_port: Option<u16>,
        ssl_paths: Option<(&Path, &Path)>,
    ) -> Result<VhostInfo, String> {
        std::fs::create_dir_all(vhosts_dir)
            .map_err(|e| format!("Failed to create vhosts dir: {}", e))?;

        let content = Self::generate_server_block(domain, root, port, php_port, ssl_paths);

        let conf_path = vhosts_dir.join(format!("{}.conf", domain));
        let tmp_conf = conf_path.with_extension("conf.tmp");
        std::fs::write(&tmp_conf, &content).map_err(|e| e.to_string())?;
        std::fs::rename(&tmp_conf, &conf_path).map_err(|e| e.to_string())?;

        let info = VhostInfo {
            domain: domain.to_string(),
            root: root.to_string(),
            port,
            php_port,
            ssl: ssl_paths.is_some(),
        };

        let meta_path = vhosts_dir.join(format!("{}.json", domain));
        let meta_json = serde_json::to_string_pretty(&info)
            .map_err(|e| format!("Failed to serialize vhost metadata: {}", e))?;
        let tmp_meta = meta_path.with_extension("json.tmp");
        std::fs::write(&tmp_meta, &meta_json).map_err(|e| e.to_string())?;
        std::fs::rename(&tmp_meta, &meta_path).map_err(|e| e.to_string())?;

        Ok(info)
    }

    pub fn delete_vhost(vhosts_dir: &Path, domain: &str) -> Result<(), String> {
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
