// Module declarations
mod file_ops;
mod database;
mod cloud_sync;
mod utils;
mod commands;

// Re-exports for public API
pub use commands::*;

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_shell::init())
        .setup(|app| {
            // Initialize database
            let app_handle = app.handle();
            database::init_database(&app_handle).expect("Failed to initialize database");
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::greet,
            commands::select_folder,
            commands::start_watching_folder,
            commands::stop_watching_folder,
            commands::organize_file,
            commands::get_tags,
            commands::add_tag,
            commands::remove_tag,
            commands::search_files,
            commands::backup_to_cloud
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
