// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod modules;
use modules::paths::VoltPath;
use modules::services::ServiceProcesses;
use tauri::Manager;

fn main() {
    // Workaround: Tauri CLI v2.11.2 on Windows does not forward TAURI_DEV/TAURI_DEV_URL
    // to child processes. Set manually so Tauri knows this is dev mode.
    #[cfg(debug_assertions)]
    {
        std::env::set_var("TAURI_DEV", "1");
        std::env::set_var("TAURI_DEV_URL", "http://localhost:3000");
    }

    // Ensure data directories exist at startup
    if let Err(e) = VoltPath::ensure_dirs() {
        eprintln!("[voltenv] Failed to initialize VoltEnv directories: {}", e);
    }

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .manage(ServiceProcesses::new())
        .invoke_handler(tauri::generate_handler![
            modules::commands::provision_service,
            modules::commands::start_service,
            modules::commands::stop_service,
        ])
        .setup(|app| {
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
