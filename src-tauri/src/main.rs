// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod modules;
use modules::paths::VoltPath;
use modules::services::ProcessManager;
use modules::commands::*;

fn main() {
    // Pastikan folder data tersedia saat startup
    if let Err(e) = VoltPath::ensure_dirs() {
        eprintln!("Gagal menginisialisasi direktori VoltEnv: {}", e);
    }

    tauri::Builder::default()
        .manage(ProcessManager::new())
        .invoke_handler(tauri::generate_handler![
            get_services_status,
            start_service,
            stop_service
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
