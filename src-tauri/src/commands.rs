use crate::database;
use crate::file_ops;
use crate::cloud_sync;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tauri_plugin_dialog::DialogExt;
use tokio::sync::oneshot;

// Simple greeting for initial testing
#[tauri::command]
pub fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

// File and folder operations
#[tauri::command]
pub async fn select_folder(app: tauri::AppHandle) -> Result<String, String> {
    let dialog = app.dialog();
    let (tx, rx) = oneshot::channel();

    dialog.file().pick_folder(move |folder_path| {
        let _ = tx.send(folder_path);
    });

    match rx.await {
        Ok(Some(path)) => match path.as_path() {
            Some(p) => Ok(p.to_string_lossy().to_string()),
            None => Err("Invalid path".to_string()),
        },
        Ok(None) => Err("No folder selected".to_string()),
        Err(_) => Err("Failed to receive folder selection".to_string()),
    }
}

#[tauri::command]
pub async fn start_watching_folder(
    app: tauri::AppHandle,
    path: String,
) -> Result<(), String> {
    file_ops::start_watching(&app, path)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn stop_watching_folder(app: tauri::AppHandle) -> Result<(), String> {
    file_ops::stop_watching(&app)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn organize_file(
    app: tauri::AppHandle, 
    file_path: String,
    destination_folder: Option<String>,
) -> Result<(), String> {
    file_ops::organize_file(&app, PathBuf::from(file_path), destination_folder)
        .await
        .map_err(|e| e.to_string())
}

// Tag operations
#[derive(Debug, Serialize, Deserialize)]
pub struct Tag {
    pub id: i64,
    pub name: String,
    pub color: String,
}

#[tauri::command]
pub fn get_tags(app: tauri::AppHandle) -> Result<Vec<Tag>, String> {
    database::get_all_tags(&app)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn add_tag(app: tauri::AppHandle, name: String, color: String) -> Result<i64, String> {
    database::add_tag(&app, name, color)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn remove_tag(app: tauri::AppHandle, tag_id: i64) -> Result<(), String> {
    database::remove_tag(&app, tag_id)
        .map_err(|e| e.to_string())
}

// File search
#[derive(Debug, Serialize, Deserialize)]
pub struct FileInfo {
    pub id: i64,
    pub path: String,
    pub name: String,
    pub extension: String,
    pub size: i64,
    pub created_at: String,
    pub modified_at: String,
    pub tags: Vec<Tag>,
}

#[tauri::command]
pub fn search_files(
    app: tauri::AppHandle,
    query: Option<String>,
    tag_ids: Option<Vec<i64>>,
    extension: Option<String>,
) -> Result<Vec<FileInfo>, String> {
    database::search_files(&app, query, tag_ids, extension)
        .map_err(|e| e.to_string())
}

// Cloud backup
#[tauri::command]
pub async fn backup_to_cloud(
    app: tauri::AppHandle,
    folder_path: String,
    bucket_name: String,
) -> Result<(), String> {
    cloud_sync::backup_folder(&app, folder_path, bucket_name)
        .await
        .map_err(|e| e.to_string())
} 