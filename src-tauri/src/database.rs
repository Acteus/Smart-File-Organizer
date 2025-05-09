use anyhow::{Context, Result};
use rusqlite::{Connection, params};
use std::path::Path;
use std::fs;
use std::sync::{Arc, Mutex};
use tauri::{AppHandle, Manager};
use crate::commands::{Tag, FileInfo};

// Struct to hold the database connection
pub struct DatabaseConnection(pub Connection);

// Key for the database in the app state
const DB_KEY: &str = "db_connection";

// Initialize the database
pub fn init_database(app: &AppHandle) -> Result<()> {
    // Get app data directory for storing the database
    let app_dir = app.path()
        .app_data_dir()
        .context("Failed to get app data directory")?;

    // Create the directory if it doesn't exist
    fs::create_dir_all(&app_dir).context("Failed to create app data directory")?;

    // Create database path
    let db_path = app_dir.join("smart_file_organizer.db");
    
    // Connect to SQLite database
    let conn = Connection::open(&db_path)
        .context("Failed to open database connection")?;

    // Create tables
    create_tables(&conn)?;
    
    // Store the connection in the app state
    let mutex_conn = Arc::new(Mutex::new(DatabaseConnection(conn)));
    app.manage(mutex_conn);
    
    Ok(())
}

// Get the database connection from the app state
pub fn get_connection(app: &AppHandle) -> Result<Arc<Mutex<DatabaseConnection>>> {
    app.try_state::<Arc<Mutex<DatabaseConnection>>>()
        .map(|state| state.inner().clone())
        .ok_or_else(|| anyhow::anyhow!("Failed to get database connection from app state"))
}

// Create database tables
fn create_tables(conn: &Connection) -> Result<()> {
    // Create files table
    conn.execute(
        "CREATE TABLE IF NOT EXISTS files (
            id INTEGER PRIMARY KEY,
            path TEXT NOT NULL UNIQUE,
            name TEXT NOT NULL,
            extension TEXT NOT NULL,
            size INTEGER NOT NULL,
            created_at TEXT NOT NULL,
            modified_at TEXT NOT NULL
        )",
        [],
    ).context("Failed to create files table")?;

    // Create tags table
    conn.execute(
        "CREATE TABLE IF NOT EXISTS tags (
            id INTEGER PRIMARY KEY,
            name TEXT NOT NULL UNIQUE,
            color TEXT NOT NULL
        )",
        [],
    ).context("Failed to create tags table")?;

    // Create file_tags table for many-to-many relationship
    conn.execute(
        "CREATE TABLE IF NOT EXISTS file_tags (
            file_id INTEGER NOT NULL,
            tag_id INTEGER NOT NULL,
            PRIMARY KEY (file_id, tag_id),
            FOREIGN KEY (file_id) REFERENCES files (id) ON DELETE CASCADE,
            FOREIGN KEY (tag_id) REFERENCES tags (id) ON DELETE CASCADE
        )",
        [],
    ).context("Failed to create file_tags table")?;

    // Create rules table for auto-organization rules
    conn.execute(
        "CREATE TABLE IF NOT EXISTS rules (
            id INTEGER PRIMARY KEY,
            name TEXT NOT NULL,
            pattern TEXT NOT NULL,
            destination_folder TEXT NOT NULL,
            is_extension BOOLEAN NOT NULL DEFAULT 0,
            is_active BOOLEAN NOT NULL DEFAULT 1
        )",
        [],
    ).context("Failed to create rules table")?;

    // Create watched_folders table
    conn.execute(
        "CREATE TABLE IF NOT EXISTS watched_folders (
            id INTEGER PRIMARY KEY,
            path TEXT NOT NULL UNIQUE,
            is_active BOOLEAN NOT NULL DEFAULT 1
        )",
        [],
    ).context("Failed to create watched_folders table")?;

    // Create default tags if they don't exist
    let default_tags = [
        ("Documents", "#4287f5"),
        ("Images", "#42f54e"),
        ("Videos", "#f54242"),
        ("Music", "#f5a742"),
        ("Archives", "#8342f5"),
    ];

    for (name, color) in default_tags.iter() {
        conn.execute(
            "INSERT OR IGNORE INTO tags (name, color) VALUES (?, ?)",
            params![name, color],
        ).context("Failed to create default tag")?;
    }

    // Create default rules for common file types
    let default_rules = [
        ("Documents", "pdf,doc,docx,txt,rtf,odt", "Documents", true),
        ("Images", "jpg,jpeg,png,gif,bmp,webp,svg", "Images", true),
        ("Videos", "mp4,avi,mov,wmv,mkv,webm", "Videos", true),
        ("Music", "mp3,wav,flac,ogg,aac", "Music", true),
        ("Archives", "zip,rar,7z,tar,gz", "Archives", true),
    ];

    for (name, pattern, destination, is_extension) in default_rules.iter() {
        conn.execute(
            "INSERT OR IGNORE INTO rules (name, pattern, destination_folder, is_extension, is_active) 
             VALUES (?, ?, ?, ?, 1)",
            params![name, pattern, destination, is_extension],
        ).context("Failed to create default rule")?;
    }

    Ok(())
}

// Tag operations
pub fn get_all_tags(app: &AppHandle) -> Result<Vec<Tag>> {
    let conn = get_connection(app)?;
    let conn_guard = conn.lock().unwrap();
    
    let mut stmt = conn_guard.0.prepare("SELECT id, name, color FROM tags")?;
    let tag_iter = stmt.query_map([], |row| {
        Ok(Tag {
            id: row.get(0)?,
            name: row.get(1)?,
            color: row.get(2)?,
        })
    })?;

    let mut tags = vec![];
    for tag in tag_iter {
        tags.push(tag?);
    }

    Ok(tags)
}

pub fn add_tag(app: &AppHandle, name: String, color: String) -> Result<i64> {
    let conn = get_connection(app)?;
    let conn_guard = conn.lock().unwrap();
    
    conn_guard.0.execute(
        "INSERT INTO tags (name, color) VALUES (?, ?)",
        params![name, color],
    )?;

    Ok(conn_guard.0.last_insert_rowid())
}

pub fn remove_tag(app: &AppHandle, tag_id: i64) -> Result<()> {
    let conn = get_connection(app)?;
    let conn_guard = conn.lock().unwrap();
    
    conn_guard.0.execute(
        "DELETE FROM tags WHERE id = ?",
        params![tag_id],
    )?;

    Ok(())
}

// File operations
pub fn add_file(
    app: &AppHandle, 
    path: &Path, 
    name: &str, 
    extension: &str, 
    size: i64, 
    created_at: &str, 
    modified_at: &str
) -> Result<i64> {
    let conn = get_connection(app)?;
    let conn_guard = conn.lock().unwrap();
    
    let path_str = path.to_string_lossy().to_string();
    
    conn_guard.0.execute(
        "INSERT OR REPLACE INTO files (path, name, extension, size, created_at, modified_at) 
         VALUES (?, ?, ?, ?, ?, ?)",
        params![path_str, name, extension, size, created_at, modified_at],
    )?;

    Ok(conn_guard.0.last_insert_rowid())
}

pub fn add_tag_to_file(app: &AppHandle, file_id: i64, tag_id: i64) -> Result<()> {
    let conn = get_connection(app)?;
    let conn_guard = conn.lock().unwrap();
    
    conn_guard.0.execute(
        "INSERT OR IGNORE INTO file_tags (file_id, tag_id) VALUES (?, ?)",
        params![file_id, tag_id],
    )?;

    Ok(())
}

pub fn search_files(
    app: &AppHandle,
    query: Option<String>,
    tag_ids: Option<Vec<i64>>,
    extension: Option<String>,
) -> Result<Vec<FileInfo>> {
    let conn = get_connection(app)?;
    let conn_guard = conn.lock().unwrap();
    
    // Build the query
    let mut sql = String::from(
        "SELECT DISTINCT f.id, f.path, f.name, f.extension, f.size, f.created_at, f.modified_at 
         FROM files f"
    );
    
    let mut where_clauses = vec![];
    let mut params = vec![];
    
    // Join with file_tags if filtering by tags
    if tag_ids.is_some() {
        sql.push_str(" JOIN file_tags ft ON f.id = ft.file_id");
        
        if let Some(ids) = &tag_ids {
            let placeholders: Vec<String> = (0..ids.len()).map(|_| "?".to_string()).collect();
            let placeholders_str = placeholders.join(",");
            where_clauses.push(format!("ft.tag_id IN ({})", placeholders_str));
            
            for id in ids {
                params.push(id.to_string());
            }
        }
    }
    
    // Add search query
    if let Some(q) = &query {
        where_clauses.push("(f.name LIKE ? OR f.path LIKE ?)".to_string());
        let like_pattern = format!("%{}%", q);
        params.push(like_pattern.clone());
        params.push(like_pattern);
    }
    
    // Add extension filter
    if let Some(ext) = &extension {
        where_clauses.push("f.extension = ?".to_string());
        params.push(ext.to_string());
    }
    
    // Add WHERE clause if needed
    if !where_clauses.is_empty() {
        sql.push_str(" WHERE ");
        sql.push_str(&where_clauses.join(" AND "));
    }
    
    // Add ORDER BY
    sql.push_str(" ORDER BY f.name ASC");
    
    // Prepare and execute the query
    let mut stmt = conn_guard.0.prepare(&sql)?;
    let file_iter = stmt.query_map(rusqlite::params_from_iter(params.iter()), |row| {
        Ok(FileInfo {
            id: row.get(0)?,
            path: row.get(1)?,
            name: row.get(2)?,
            extension: row.get(3)?,
            size: row.get(4)?,
            created_at: row.get(5)?,
            modified_at: row.get(6)?,
            tags: vec![], // Will fill separately
        })
    })?;
    
    // Collect files
    let mut files = vec![];
    for file_result in file_iter {
        let mut file = file_result?;
        
        // Get tags for this file
        let mut tag_stmt = conn_guard.0.prepare(
            "SELECT t.id, t.name, t.color 
             FROM tags t
             JOIN file_tags ft ON t.id = ft.tag_id
             WHERE ft.file_id = ?"
        )?;
        
        let tag_iter = tag_stmt.query_map([file.id], |row| {
            Ok(Tag {
                id: row.get(0)?,
                name: row.get(1)?,
                color: row.get(2)?,
            })
        })?;
        
        for tag_result in tag_iter {
            file.tags.push(tag_result?);
        }
        
        files.push(file);
    }
    
    Ok(files)
} 