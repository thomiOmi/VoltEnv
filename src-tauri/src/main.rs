// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod modules;
mod utils;
use modules::catalog::CatalogManager;
use modules::paths::VoltPath;
use modules::services::ServiceProcesses;
use tauri::Manager;

/// Patches Tauri dev environment variables that the CLI v2.11.2
/// on Windows fails to forward to child processes.
#[cfg(debug_assertions)]
fn patch_dev_env() {
    std::env::set_var("TAURI_DEV", "1");
    std::env::set_var("TAURI_DEV_URL", "http://localhost:3000");
}

fn main() {
    #[cfg(debug_assertions)]
    patch_dev_env();

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_http::init())
        .plugin(tauri_plugin_os::init())
        .plugin(tauri_plugin_log::Builder::new().build())
        .plugin(tauri_plugin_store::Builder::new().build())
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.set_focus();
            }
        }))
        .invoke_handler(tauri::generate_handler![
            modules::commands::download_service,
            modules::commands::install_service,
            modules::commands::start_service,
            modules::commands::stop_service,
            modules::commands::soft_stop_service,
            modules::commands::force_stop_service,
            modules::commands::switch_service_version,
            modules::commands::get_active_versions,
            modules::logger::get_service_logs,
            modules::env::register_to_os_env,
            modules::env::register_service_environment,
            modules::env::unregister_service_environment,
            modules::env::restore_os_path_backup,
            crate::utils::is_port_available,
        ])
        .manage(ServiceProcesses::default())
        .setup(|app| {
            // Load dynamic catalog from catalog.json (auto-creates defaults)
            match CatalogManager::load_or_create(app.handle()) {
                Ok(catalog) => {
                    app.manage(catalog);
                }
                Err(e) => {
                    eprintln!("[voltenv] Failed to load service catalog: {}", e);
                    // Fallback: manage an empty catalog so commands do not panic
                    app.manage(CatalogManager::empty());
                }
            }

            if let Err(e) = VoltPath::ensure_all_dirs(app.handle()) {
                eprintln!("[voltenv] Failed to initialize VoltEnv directories: {}", e);
            }

            // Register central env/bin/ folder to OS PATH (idempotent)
            if let Err(e) = modules::env::init_central_env_bin(app.handle()) {
                eprintln!("[voltenv] Failed to init central env bin: {}", e);
            }

            #[cfg(debug_assertions)]
            if let Some(window) = app.get_webview_window("main") {
                window.open_devtools();
            }

            Ok(())
        })
        .run(tauri::generate_context!())
        .unwrap_or_else(|e| {
            eprintln!("FATAL: {}", e);
            std::process::exit(1);
        });
}
