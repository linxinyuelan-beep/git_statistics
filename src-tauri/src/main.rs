#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

mod commands;
mod database;
mod git_analyzer;
mod models;

use commands::*;

fn main() {
    tauri::Builder::default()
        .manage(commands::AppState::default())
        .setup(|app| {
            // Initialize database
            let app_handle = app.handle();
            let handle_clone = app_handle.clone();
            tauri::async_runtime::spawn(async move {
                if let Err(e) = database::init_database(&handle_clone).await {
                    eprintln!("Failed to initialize database: {}", e);
                }
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            add_repository,
            remove_repository,
            get_repositories,
            scan_repository,
            force_scan_repository,
            scan_last_24_hours,
            get_statistics,
            get_commit_timeline,
            get_commit_detail
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}