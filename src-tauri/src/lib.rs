#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::sync::OnceLock;
use tauri::Manager;
use tauri::Emitter;

pub mod commands;
pub mod config;
pub mod download;
pub mod installer;
pub mod logging;
pub mod paths;
pub mod process;
pub mod service;
pub mod settings;
pub mod vhost;
pub mod watcher;
pub mod utils;

pub fn http_client() -> &'static reqwest::Client {
    static CLIENT: OnceLock<reqwest::Client> = OnceLock::new();
    CLIENT.get_or_init(|| {
        reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(300))
            .build()
            .expect("Failed to build reqwest Client")
    })
}

pub fn path_sep() -> &'static str {
    if cfg!(target_os = "windows") {
        ";"
    } else {
        ":"
    }
}

#[cfg(debug_assertions)]
fn patch_dev_env() {
    std::env::set_var("TAURI_DEV", "1");
    std::env::set_var("TAURI_DEV_URL", "http://localhost:3000");
}

pub fn run() {
    #[cfg(debug_assertions)]
    patch_dev_env();

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_http::init())
        .plugin(tauri_plugin_os::init())
        .plugin(
            tauri_plugin_log::Builder::new()
                .level(log::LevelFilter::Info)
                .build(),
        )
        .plugin(tauri_plugin_store::Builder::new().build())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.set_focus();
            }
        }))
        .invoke_handler(tauri::generate_handler![
            commands::service::setup_service,
            commands::service::start_service,
            commands::service::stop_service,
            commands::service::get_service_status,
            commands::service::get_services,
            commands::service::switch_service_version,
            commands::vhost::list_vhosts,
            commands::vhost::create_vhost,
            commands::vhost::delete_vhost,
            commands::database::create_database,
            commands::database::drop_database,
            commands::database::create_db_user,
            commands::database::list_databases,
            commands::quick_create::quick_create,
            commands::misc::get_settings,
            commands::misc::update_settings,
            commands::misc::get_service_logs,
            commands::misc::is_port_available,
            commands::misc::save_custom_service,
            commands::misc::delete_custom_service,
            commands::service::get_php_extensions,
            commands::service::toggle_php_extension,
            commands::misc::run_composer_command,
            commands::misc::run_self_diagnostic,
        ])
        .manage(process::ServiceProcesses::new())
        .setup(|app| {
            let _ = paths::VoltPath::ensure_all_dirs(app.handle());

            // Restore env junctions for installed services
            let registry = service::ServiceRegistry::load_all(app.handle());
            for def in registry.all() {
                let version = def.default_version.clone();
                let bin_dir = paths::VoltPath::service_dir(app.handle(), &def.id, &version);
                if bin_dir.join(&def.binary_name).exists() {
                    if let Err(e) =
                        paths::VoltPath::create_env_junction(app.handle(), &def.id, &version)
                    {
                        eprintln!(
                            "[voltenv] Failed to restore env junction for {}: {}",
                            def.id, e
                        );
                    }
                }
            }

            match watcher::FsWatcher::new(
                app.handle().clone(),
                paths::VoltPath::bin_dir(app.handle()),
            ) {
                Ok(w) => {
                    app.manage(w);
                }
                Err(e) => eprintln!("[voltenv] Watcher init failed: {}", e),
            }

            #[cfg(debug_assertions)]
            if let Some(window) = app.get_webview_window("main") {
                window.open_devtools();
            }

            Ok(())
        })
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                // By default hide to tray on close
                let _ = window.hide();
                api.prevent_close();
            }
        })
        .run(tauri::generate_context!())
        .unwrap_or_else(|e| {
            eprintln!("FATAL: {}", e);
            std::process::exit(1);
        });
}
