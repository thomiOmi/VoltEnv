use std::path::PathBuf;
use tauri::AppHandle;

use crate::errors::{VoltError, VoltResult};
use crate::paths::VoltPath;
use crate::settings::Settings;

fn find_mysql_bin(app: &AppHandle) -> Option<PathBuf> {
    let settings = Settings::load(app);
    let version = settings.preferred_ports.get("mysql").and_then(|_| {
        let registry = crate::service::ServiceRegistry::load_all(app);
        registry.get("mysql").map(|def| def.default_version.clone())
    });

    if let Some(ver) = version {
        let service_bin = VoltPath::service_dir(app, "mysql", &ver)
            .join("bin")
            .join("mysql.exe");
        if service_bin.exists() {
            return Some(service_bin);
        }
        let service_bin_exe = VoltPath::service_dir(app, "mysql", &ver)
            .join("bin")
            .join("mysql");
        if service_bin_exe.exists() {
            return Some(service_bin_exe);
        }
    }

    #[cfg(target_os = "windows")]
    {
        if let Ok(path) = std::env::var("PATH") {
            for dir in std::env::split_paths(&path) {
                let candidate = dir.join("mysql.exe");
                if candidate.exists() {
                    return Some(candidate);
                }
            }
        }
    }

    None
}

fn get_mysql_port(app: &AppHandle) -> u16 {
    let settings = Settings::load(app);
    settings
        .resolved_ports
        .get("mysql")
        .copied()
        .or_else(|| settings.preferred_ports.get("mysql").copied())
        .unwrap_or(3306)
}

async fn run_mysql(app: &AppHandle, sql: &str) -> VoltResult<String> {
    let mysql_bin = find_mysql_bin(app)
        .ok_or_else(|| VoltError::Database("MySQL CLI not found. Is MySQL installed?".to_string()))?;

    let port = get_mysql_port(app);

    let output = tokio::process::Command::new(&mysql_bin)
        .args([
            "-u",
            "root",
            "-h",
            "127.0.0.1",
            "-P",
            &port.to_string(),
            "-e",
            sql,
            "--batch",
            "--skip-column-names",
        ])
        .output()
        .await
        .map_err(VoltError::Io)?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(VoltError::Database(format!("MySQL command failed: {}", stderr.trim())));
    }

    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

fn sanitize_identifier(id: &str) -> VoltResult<String> {
    let sanitized: String = id
        .chars()
        .filter(|c| c.is_alphanumeric() || *c == '_')
        .collect();
    if sanitized.is_empty() || sanitized != id {
        return Err(VoltError::Validation(format!("Invalid MySQL identifier: {}", id)));
    }
    Ok(sanitized)
}

pub async fn create_database_inner(app: &AppHandle, name: &str) -> VoltResult<()> {
    let name = sanitize_identifier(name)?;
    run_mysql(
        app,
        &format!("CREATE DATABASE IF NOT EXISTS `{}`", name),
    )
    .await?;
    Ok(())
}

#[tauri::command]
pub async fn create_database(app: AppHandle, name: String) -> VoltResult<()> {
    create_database_inner(&app, &name).await
}

#[tauri::command]
pub async fn drop_database(app: AppHandle, name: String) -> VoltResult<()> {
    let name = sanitize_identifier(&name)?;
    run_mysql(&app, &format!("DROP DATABASE IF EXISTS `{}`", name)).await?;
    Ok(())
}

#[tauri::command]
pub async fn create_db_user(
    app: AppHandle,
    username: String,
    password: String,
    database: String,
) -> VoltResult<()> {
    let user = sanitize_identifier(&username)?;
    let db = sanitize_identifier(&database)?;

    let safe_password = password.replace('\'', "''");

    run_mysql(
        &app,
        &format!(
            "CREATE USER IF NOT EXISTS '{}'@'localhost' IDENTIFIED BY '{}'",
            user, safe_password
        ),
    )
    .await?;

    run_mysql(
        &app,
        &format!(
            "GRANT ALL PRIVILEGES ON `{}`.* TO '{}'@'localhost'",
            db, user
        ),
    )
    .await?;

    run_mysql(&app, "FLUSH PRIVILEGES").await?;

    Ok(())
}

#[tauri::command]
pub async fn list_databases(app: AppHandle) -> VoltResult<Vec<String>> {
    let output = run_mysql(&app, "SHOW DATABASES").await?;
    let databases: Vec<String> = output
        .lines()
        .map(|s| s.trim().to_string())
        .filter(|s| {
            !s.is_empty()
                && s != "information_schema"
                && s != "performance_schema"
                && s != "mysql"
                && s != "sys"
        })
        .collect();
    Ok(databases)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize_identifier() {
        assert!(sanitize_identifier("my_db").is_ok());
        assert!(sanitize_identifier("db123").is_ok());
        assert!(sanitize_identifier("db; drop table").is_err());
        assert!(sanitize_identifier("db name").is_err());
    }
}

#[tauri::command]
pub async fn test_mysql_connection(
    app: AppHandle,
    username: Option<String>,
    password: Option<String>,
) -> VoltResult<String> {
    let user = username.unwrap_or_else(|| "root".to_string());
    let pass = password.unwrap_or_default();

    // Sanitize user for the CLI command
    let safe_user = sanitize_identifier(&user)?;
    let safe_pass = pass.replace('\'', "''");

    let mysql_bin = find_mysql_bin(&app)
        .ok_or_else(|| VoltError::Database("MySQL CLI not found".to_string()))?;
    let port = get_mysql_port(&app);

    let mut cmd = tokio::process::Command::new(&mysql_bin);
    cmd.args([
        "-u", &safe_user,
        "-h", "127.0.0.1",
        "-P", &port.to_string(),
        "-e", "SELECT 1",
    ]);

    if !safe_pass.is_empty() {
        cmd.arg(format!("--password={}", pass)); // Use raw pass for the CLI arg
    }

    let output = cmd.output().await.map_err(VoltError::Io)?;

    if output.status.success() {
        Ok("Connection successful".to_string())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(VoltError::Database(format!("Connection failed: {}", stderr.trim())))
    }
}
