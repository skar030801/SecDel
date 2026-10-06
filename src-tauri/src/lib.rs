#![cfg_attr(mobile, tauri::mobile_entry_point)]

pub mod commands;
pub mod core;
pub mod error;
pub mod platform;
pub mod reporting;
pub mod scanner;
pub mod utils;

use std::sync::atomic::AtomicBool;

pub struct AppState {
    pub cancelled: AtomicBool,
}

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_shell::init())
        .manage(AppState {
            cancelled: AtomicBool::new(false),
        })
        .setup(|app| {
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::delete::delete_files,
            commands::delete::cancel_operation,
            commands::delete::verify_deletion,
            commands::drive::get_devices,
            commands::drive::get_device_info,
            commands::drive::erase_drive,
            commands::scanner::scan_artifacts,
            commands::report::generate_report,
            commands::settings::get_settings,
            commands::settings::save_settings,
            commands::settings::get_app_status,
            commands::utility::get_file_size,
            commands::utility::get_recent_logs,
            commands::utility::get_default_settings,
            commands::utility::export_scan_results,
            commands::utility::download_report,
            commands::utility::open_report,
        ])
        .run(tauri::generate_context!())
        .expect("error while running SecDel");
}
