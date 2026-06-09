use tauri::AppHandle;

use crate::errors::VoltResult;
use crate::paths::VoltPath;
use crate::settings::Settings;
use crate::vhost::VhostManager;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QuickCreateResult {
    pub project_name: String,
    pub domain: String,
    pub root_path: String,
    pub created_vhost: bool,
    pub created_database: bool,
}

#[tauri::command]
pub async fn quick_create(
    app: AppHandle,
    project_name: String,
    create_database: bool,
) -> VoltResult<QuickCreateResult> {
    let slug: String = project_name
        .chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '-' || c == '_' {
                c
            } else {
                '-'
            }
        })
        .collect();
    let slug_lower = slug.to_lowercase();

    let domain = format!("{}.test", slug_lower);
    let www_dir = VoltPath::www_dir(&app);
    let project_dir = www_dir.join(&slug_lower);
    let root_path = project_dir.to_string_lossy().to_string();

    std::fs::create_dir_all(&project_dir)
        .map_err(|e| format!("Failed to create project directory: {}", e))?;

    let index_html = format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{}</title>
</head>
<body>
    <h1>{} — Ready</h1>
    <p>Project created by VoltEnv.</p>
</body>
</html>
"#,
        project_name, project_name
    );
    std::fs::write(project_dir.join("index.html"), &index_html)
        .map_err(|e| format!("Failed to write index.html: {}", e))?;

    let settings = Settings::load(&app);
    let nginx_port = settings
        .resolved_ports
        .get("nginx")
        .copied()
        .or_else(|| settings.preferred_ports.get("nginx").copied())
        .unwrap_or(8080);

    let php_port = settings
        .resolved_ports
        .get("php")
        .copied()
        .or_else(|| settings.preferred_ports.get("php").copied());

    let vhosts_dir = VoltPath::vhosts_dir(&app);
    VhostManager::save_vhost(&vhosts_dir, &domain, &root_path, nginx_port, php_port).map_err(|e| e.into())?;

    let mut created_db = false;
    if create_database {
        let sanitized: String = slug_lower
            .chars()
            .filter(|c| c.is_alphanumeric() || *c == '_')
            .collect();
        if !sanitized.is_empty() {
            match crate::commands::database::create_database_inner(&app, &sanitized).await {
                Ok(()) => created_db = true,
                Err(e) => eprintln!("[voltenv] Quick create database failed: {}", e),
            }
        }
    }

    Ok(QuickCreateResult {
        project_name,
        domain,
        root_path,
        created_vhost: true,
        created_database: created_db,
    })
}
