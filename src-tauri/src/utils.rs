use anyhow::Result;
use std::path::{Path, PathBuf};
use std::collections::HashMap;
use std::fs;

// Get the file extension from a path
pub fn get_file_extension(path: &Path) -> Option<String> {
    path.extension()
        .and_then(|ext| ext.to_str())
        .map(|s| s.to_lowercase())
}

// Get the file name without extension
pub fn get_file_stem(path: &Path) -> Option<String> {
    path.file_stem()
        .and_then(|s| s.to_str())
        .map(|s| s.to_string())
}

// Get the file name with extension
pub fn get_file_name(path: &Path) -> Option<String> {
    path.file_name()
        .and_then(|s| s.to_str())
        .map(|s| s.to_string())
}

// Get the file size in bytes
pub fn get_file_size(path: &Path) -> Result<u64> {
    let metadata = fs::metadata(path)?;
    Ok(metadata.len())
}

// Check if a path is a directory
pub fn is_directory(path: &Path) -> bool {
    path.is_dir()
}

// Check if a path is a file
pub fn is_file(path: &Path) -> bool {
    path.is_file()
}

// Create directory if it doesn't exist
pub fn ensure_dir_exists(path: &Path) -> Result<()> {
    if !path.exists() {
        fs::create_dir_all(path)?;
    }
    Ok(())
}

// Get MIME type from file extension
pub fn get_mime_type(extension: &str) -> String {
    let ext = extension.to_lowercase();
    match ext.as_str() {
        // Images
        "jpg" | "jpeg" => "image/jpeg",
        "png" => "image/png",
        "gif" => "image/gif",
        "webp" => "image/webp",
        "svg" => "image/svg+xml",
        "bmp" => "image/bmp",
        
        // Documents
        "pdf" => "application/pdf",
        "doc" | "docx" => "application/msword",
        "xls" | "xlsx" => "application/vnd.ms-excel",
        "ppt" | "pptx" => "application/vnd.ms-powerpoint",
        "txt" => "text/plain",
        "rtf" => "application/rtf",
        "odt" => "application/vnd.oasis.opendocument.text",
        
        // Audio
        "mp3" => "audio/mpeg",
        "wav" => "audio/wav",
        "ogg" => "audio/ogg",
        "flac" => "audio/flac",
        "aac" => "audio/aac",
        
        // Video
        "mp4" => "video/mp4",
        "avi" => "video/x-msvideo",
        "mov" => "video/quicktime",
        "mkv" => "video/x-matroska",
        "webm" => "video/webm",
        "wmv" => "video/x-ms-wmv",
        
        // Archives
        "zip" => "application/zip",
        "rar" => "application/x-rar-compressed",
        "7z" => "application/x-7z-compressed",
        "tar" => "application/x-tar",
        "gz" => "application/gzip",
        
        // Other
        _ => "application/octet-stream",
    }.to_string()
}

// Group files by extension
pub fn group_files_by_extension(files: &[PathBuf]) -> HashMap<String, Vec<PathBuf>> {
    let mut groups: HashMap<String, Vec<PathBuf>> = HashMap::new();
    
    for file in files {
        if let Some(ext) = get_file_extension(file) {
            groups.entry(ext).or_default().push(file.clone());
        } else {
            // Files without extension
            groups.entry("".to_string()).or_default().push(file.clone());
        }
    }
    
    groups
}

// Categorize files by type
pub fn categorize_files(files: &[PathBuf]) -> HashMap<String, Vec<PathBuf>> {
    let mut categories: HashMap<String, Vec<PathBuf>> = HashMap::new();
    
    for file in files {
        if let Some(ext) = get_file_extension(file) {
            let category = match ext.as_str() {
                // Documents
                "pdf" | "doc" | "docx" | "txt" | "rtf" | "odt" | "xls" | "xlsx" | "ppt" | "pptx" => "Documents",
                
                // Images
                "jpg" | "jpeg" | "png" | "gif" | "bmp" | "webp" | "svg" => "Images",
                
                // Videos
                "mp4" | "avi" | "mov" | "wmv" | "mkv" | "webm" => "Videos",
                
                // Audio
                "mp3" | "wav" | "flac" | "ogg" | "aac" => "Music",
                
                // Archives
                "zip" | "rar" | "7z" | "tar" | "gz" => "Archives",
                
                // Code
                "js" | "ts" | "html" | "css" | "rs" | "py" | "java" | "cpp" | "c" | "h" => "Code",
                
                // Other
                _ => "Other",
            };
            
            categories.entry(category.to_string()).or_default().push(file.clone());
        } else {
            // Files without extension
            categories.entry("Other".to_string()).or_default().push(file.clone());
        }
    }
    
    categories
}

// Format file size to human-readable string
pub fn format_file_size(size: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;
    const TB: u64 = GB * 1024;
    
    if size < KB {
        format!("{} B", size)
    } else if size < MB {
        format!("{:.2} KB", size as f64 / KB as f64)
    } else if size < GB {
        format!("{:.2} MB", size as f64 / MB as f64)
    } else if size < TB {
        format!("{:.2} GB", size as f64 / GB as f64)
    } else {
        format!("{:.2} TB", size as f64 / TB as f64)
    }
} 