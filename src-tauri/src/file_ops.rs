use anyhow::Result;
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use notify_debouncer_mini::{new_debouncer, DebouncedEvent};
use tokio::sync::mpsc;
use tauri::{AppHandle, Manager, Emitter};
use crate::database;

// State used to hold file watchers
#[derive(Default)]
pub struct WatcherState {
    watchers: HashMap<String, notify_debouncer_mini::Debouncer<notify::RecommendedWatcher>>,
}

// Event struct for frontend
#[derive(Clone, serde::Serialize)]
pub struct FileEvent {
    pub path: String,
    pub file_name: String,
    pub event_type: String,
    pub extension: String,
    pub size: u64,
}

// Start watching a folder
pub async fn start_watching(app: &AppHandle, path: String) -> Result<()> {
    // Create state if it doesn't exist
    let state = app.state::<Arc<Mutex<WatcherState>>>();
    
    // Create channel for events
    let (tx, mut rx) = mpsc::channel::<FileEvent>(100);
    
    // Create debouncer - properly implemented for notify-debouncer-mini 0.4
    let tx_clone = tx.clone();
    let event_handler = move |res: notify::Result<Vec<DebouncedEvent>>| {
        if let Ok(events) = res {
            for e in events {
                let path = e.path.clone();
                
                // Skip directories, hidden files
                if path.is_dir() || path.file_name()
                    .and_then(|s| s.to_str())
                    .map(|s| s.starts_with("."))
                    .unwrap_or(false) 
                {
                    continue;
                }
                
                // Get file extension and name
                let extension = path
                    .extension()
                    .and_then(|s| s.to_str())
                    .unwrap_or("")
                    .to_lowercase();
                
                let file_name = path
                    .file_name()
                    .and_then(|s| s.to_str())
                    .unwrap_or("")
                    .to_string();
                
                // Get file size
                let size = fs::metadata(&path)
                    .map(|m| m.len())
                    .unwrap_or(0);
                
                // Create event
                let file_event = FileEvent {
                    path: path.to_string_lossy().to_string(),
                    file_name,
                    extension,
                    size,
                    event_type: "created".into(),
                };
                
                // Send to channel
                let _ = tx_clone.try_send(file_event);
            }
        }
    };
    
    let mut debouncer = new_debouncer(Duration::from_secs(2), event_handler)?;
    
    // Start watcher
    match debouncer.watcher().watch(Path::new(&path), notify::RecursiveMode::Recursive) {
        Ok(_) => {
            let mut state_guard = state.lock().unwrap();
            state_guard.watchers.insert(path.clone(), debouncer);
        }
        Err(e) => {
            return Err(anyhow::anyhow!("Failed to watch path: {}", e));
        }
    }
    
    // Create task to process file events
    let app_handle = app.clone();
    tokio::spawn(async move {
        while let Some(event) = rx.recv().await {
            // Process the file - this will auto-organize based on rules
            let _ = organize_file_by_rules(&app_handle, &PathBuf::from(&event.path)).await;
            
            // Emit the event to the frontend
            let _ = app_handle.emit("file_event", event);
        }
    });
    
    // Store the watched folder in the database
    // This is async, so we don't await it to avoid blocking
    let app_handle = app.clone();
    let watch_path_clone = path.clone();
    tokio::spawn(async move {
        if let Ok(conn) = database::get_connection(&app_handle) {
            let conn_guard = conn.lock().unwrap();
            let _ = conn_guard.0.execute(
                "INSERT OR REPLACE INTO watched_folders (path, is_active) VALUES (?, 1)",
                rusqlite::params![watch_path_clone],
            );
        }
    });
    
    Ok(())
}

// Stop watching a folder
pub async fn stop_watching(app: &AppHandle) -> Result<()> {
    // Get the watcher state if it exists
    if let Some(state) = app.try_state::<Arc<Mutex<WatcherState>>>() {
        let mut state_guard = state.lock().unwrap();
        
        // Clear all watchers
        state_guard.watchers.clear();
        
        // Update database
        let app_handle = app.clone();
        tokio::spawn(async move {
            if let Ok(conn) = database::get_connection(&app_handle) {
                let conn_guard = conn.lock().unwrap();
                let _ = conn_guard.0.execute(
                    "UPDATE watched_folders SET is_active = 0",
                    [],
                );
            }
        });
    }
    
    Ok(())
}

// Organize a file based on rules
pub async fn organize_file_by_rules(app: &AppHandle, file_path: &Path) -> Result<()> {
    // Check if file exists and is a file
    if !file_path.exists() || !file_path.is_file() {
        return Ok(());
    }
    
    // Get file extension
    let extension = file_path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();
    
    // Skip if no extension
    if extension.is_empty() {
        return Ok(());
    }
    
    // Get file metadata
    let metadata = fs::metadata(file_path)?;
    let size = metadata.len() as i64;
    
    // Get creation and modification times
    let created = metadata.created().unwrap_or_else(|_| std::time::SystemTime::now());
    let modified = metadata.modified().unwrap_or_else(|_| std::time::SystemTime::now());
    
    let created_dt: DateTime<Utc> = created.into();
    let modified_dt: DateTime<Utc> = modified.into();
    
    let created_str = created_dt.format("%Y-%m-%d %H:%M:%S").to_string();
    let modified_str = modified_dt.format("%Y-%m-%d %H:%M:%S").to_string();
    
    // Get file name
    let file_name = file_path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("")
        .to_string();
    
    // Get the rules for this extension
    let conn = database::get_connection(&app)?;
    let conn_guard = conn.lock().unwrap();
    
    let mut stmt = conn_guard.0.prepare(
        "SELECT destination_folder FROM rules 
         WHERE is_active = 1 AND is_extension = 1 
         AND ? IN (SELECT value FROM json_each(REPLACE(pattern, ',', '\",\"')))"
    )?;
    
    let mut destination = None;
    let rows = stmt.query_map([extension.clone()], |row| {
        Ok(row.get::<_, String>(0)?)
    })?;
    
    for row in rows {
        destination = Some(row?);
        break;
    }
    
    // If we have a destination, move the file
    if let Some(dest_folder) = destination {
        // Get home directory
        let home_dir = dirs::home_dir().ok_or_else(|| anyhow::anyhow!("Failed to get home directory"))?;
        
        // Create destination path
        let dest_path = home_dir.join(&dest_folder);
        if !dest_path.exists() {
            fs::create_dir_all(&dest_path)?;
        }
        
        // Create new file path
        let new_path = dest_path.join(&file_name);
        
        // Check if destination file already exists
        if new_path.exists() {
            // Create a unique filename by adding timestamp
            let now = Utc::now();
            let timestamp = now.format("%Y%m%d%H%M%S").to_string();
            
            let file_stem = file_path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("")
                .to_string();
            
            let new_filename = format!("{}_{}.{}", file_stem, timestamp, extension);
            let new_path = dest_path.join(new_filename);
            
            // Move the file
            fs::copy(file_path, &new_path)?;
            fs::remove_file(file_path)?;
        } else {
            // Move the file
            fs::copy(file_path, &new_path)?;
            fs::remove_file(file_path)?;
        }
        
        // Add file to database
        database::add_file(
            &app,
            &new_path,
            &file_name,
            &extension,
            size,
            &created_str,
            &modified_str,
        )?;
        
        // Auto-tag by extension
        let file_id = conn_guard.0.query_row(
            "SELECT id FROM files WHERE path = ?",
            [new_path.to_string_lossy().to_string()],
            |row| row.get::<_, i64>(0),
        )?;
        
        // Get tag ID based on extension type
        let tag_name = match extension.as_str() {
            "pdf" | "doc" | "docx" | "txt" | "rtf" | "odt" => "Documents",
            "jpg" | "jpeg" | "png" | "gif" | "bmp" | "webp" | "svg" => "Images",
            "mp4" | "avi" | "mov" | "wmv" | "mkv" | "webm" => "Videos",
            "mp3" | "wav" | "flac" | "ogg" | "aac" => "Music",
            "zip" | "rar" | "7z" | "tar" | "gz" => "Archives",
            _ => "",
        };
        
        if !tag_name.is_empty() {
            let tag_id = conn_guard.0.query_row(
                "SELECT id FROM tags WHERE name = ?",
                [tag_name],
                |row| row.get::<_, i64>(0),
            )?;
            
            database::add_tag_to_file(&app, file_id, tag_id)?;
        }
    }
    
    Ok(())
}

// Manually organize a file
pub async fn organize_file(
    app: &AppHandle, 
    file_path: PathBuf,
    destination_folder: Option<String>,
) -> Result<()> {
    if let Some(dest) = destination_folder {
        // User specified a destination folder
        let dest_path = PathBuf::from(dest);
        
        // Ensure destination folder exists
        if !dest_path.exists() {
            fs::create_dir_all(&dest_path)?;
        }
        
        // Get file name
        let file_name = file_path
            .file_name()
            .ok_or_else(|| anyhow::anyhow!("Invalid file path"))?;
        
        // Create new path
        let new_path = dest_path.join(file_name);
        
        // Move the file
        fs::copy(&file_path, &new_path)?;
        fs::remove_file(&file_path)?;
        
        // Add to database
        let extension = file_path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase();
        
        let metadata = fs::metadata(&new_path)?;
        let size = metadata.len() as i64;
        
        let created = metadata.created().unwrap_or_else(|_| std::time::SystemTime::now());
        let modified = metadata.modified().unwrap_or_else(|_| std::time::SystemTime::now());
        
        let created_dt: DateTime<Utc> = created.into();
        let modified_dt: DateTime<Utc> = modified.into();
        
        let created_str = created_dt.format("%Y-%m-%d %H:%M:%S").to_string();
        let modified_str = modified_dt.format("%Y-%m-%d %H:%M:%S").to_string();
        
        database::add_file(
            &app,
            &new_path,
            file_name.to_str().unwrap_or(""),
            &extension,
            size,
            &created_str,
            &modified_str,
        )?;
        
        Ok(())
    } else {
        // Use rule-based organization
        organize_file_by_rules(app, &file_path).await
    }
} 