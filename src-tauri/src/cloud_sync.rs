use anyhow::{Result};
use aws_config::BehaviorVersion;
use aws_config::meta::region::RegionProviderChain;
use aws_sdk_s3::Client;
use aws_sdk_s3::config::{Builder, Region};
use aws_sdk_s3::primitives::ByteStream;
use chrono::Utc;
use std::fs;
use std::path::{Path, PathBuf};
use tauri::AppHandle;
use tokio::sync::Semaphore;

// Maximum concurrent uploads
const MAX_CONCURRENT_UPLOADS: usize = 5;

// AWS S3 client configuration
async fn get_s3_client() -> Result<Client> {
    let region_provider = RegionProviderChain::default_provider().or_else(Region::new("us-east-1"));
    
    let config = aws_config::defaults(BehaviorVersion::latest())
        .region(region_provider)
        .load()
        .await;
    
    let s3_config = Builder::from(&config)
        .force_path_style(true)
        .build();
    
    Ok(Client::from_conf(s3_config))
}

// Backup a folder to S3
pub async fn backup_folder(_app: &AppHandle, folder_path: String, bucket_name: String) -> Result<()> {
    // Check if folder exists
    let folder = Path::new(&folder_path);
    if !folder.exists() || !folder.is_dir() {
        return Err(anyhow::anyhow!("Invalid folder path"));
    }
    
    // Get S3 client
    let client = get_s3_client().await?;
    
    // Create bucket if it doesn't exist
    let buckets = client.list_buckets().send().await?;
    let bucket_exists = if let Some(bucket_list) = buckets.buckets {
        bucket_list.iter().any(|b| {
            if let Some(name) = &b.name {
                name == &bucket_name
            } else {
                false
            }
        })
    } else {
        false
    };
    
    if !bucket_exists {
        client.create_bucket()
            .bucket(&bucket_name)
            .send()
            .await?;
    }
    
    // Find all files in the folder (recursively)
    let files = collect_files(folder)?;
    
    // Create a timestamp for the backup
    let timestamp = Utc::now().format("%Y%m%d%H%M%S").to_string();
    
    // Upload files concurrently using a semaphore inside each task
    let mut tasks = vec![];
    
    for file_path in files {
        let client = client.clone();
        let bucket = bucket_name.clone();
        let folder_base = folder.to_path_buf();
        let timestamp = timestamp.clone(); // Clone timestamp for each task
        
        let task = tokio::spawn(async move {
            // Create a local semaphore inside the task
            let semaphore = Semaphore::new(1);
            let _permit = semaphore.acquire().await?;
            
            // Create the S3 key with the timestamp
            let relative_path = file_path.strip_prefix(&folder_base).unwrap_or(&file_path);
            let key = format!(
                "backup_{}/{}",
                timestamp,
                relative_path.to_string_lossy().replace("\\", "/")
            );
            
            // Get file content
            let body = ByteStream::from_path(&file_path).await?;
            
            // Upload to S3
            client.put_object()
                .bucket(&bucket)
                .key(&key)
                .body(body)
                .send()
                .await?;
            
            Ok::<_, anyhow::Error>(())
        });
        
        tasks.push(task);
    }
    
    // Wait for all uploads to complete
    for task in tasks {
        task.await??;
    }
    
    Ok(())
}

// Helper function to recursively collect all files in a directory
fn collect_files(dir: &Path) -> Result<Vec<PathBuf>> {
    let mut files = vec![];
    
    if dir.is_dir() {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            
            if path.is_dir() {
                files.extend(collect_files(&path)?);
            } else {
                files.push(path);
            }
        }
    }
    
    Ok(files)
}

// Download a file from S3
pub async fn download_file(bucket: &str, key: &str, destination: &Path) -> Result<()> {
    let client = get_s3_client().await?;
    
    // Get the object from S3
    let resp = client.get_object()
        .bucket(bucket)
        .key(key)
        .send()
        .await?;
    
    // Create destination directory if it doesn't exist
    if let Some(parent) = destination.parent() {
        fs::create_dir_all(parent)?;
    }
    
    // Save the file
    let body = resp.body.collect().await?;
    fs::write(destination, body.into_bytes())?;
    
    Ok(())
}

// List all backups for a bucket
pub async fn list_backups(bucket: &str) -> Result<Vec<String>> {
    let client = get_s3_client().await?;
    
    let resp = client.list_objects_v2()
        .bucket(bucket)
        .delimiter("/")
        .send()
        .await?;
    
    let mut backups = vec![];
    
    if let Some(prefixes) = resp.common_prefixes {
        for prefix in prefixes {
            if let Some(prefix_str) = prefix.prefix {
                backups.push(prefix_str);
            }
        }
    }
    
    Ok(backups)
}

// Restore a backup to local folder
pub async fn restore_backup(bucket: &str, backup_prefix: &str, destination: &Path) -> Result<()> {
    let client = get_s3_client().await?;
    
    // List all objects in the backup
    let resp = client.list_objects_v2()
        .bucket(bucket)
        .prefix(backup_prefix)
        .send()
        .await?;
    
    // Create destination directory if it doesn't exist
    fs::create_dir_all(destination)?;
    
    let mut tasks = vec![];
    
    if let Some(objects) = resp.contents {
        for obj in objects {
            if let Some(key) = obj.key {
                let client = client.clone();
                let bucket = bucket.to_string();
                let key_str = key;
                let dest_path = destination.join(
                    key_str.strip_prefix(backup_prefix).unwrap_or(&key_str)
                );
                
                let task = tokio::spawn(async move {
                    // Create a local semaphore inside the task
                    let semaphore = Semaphore::new(1);
                    let _permit = semaphore.acquire().await?;
                    
                    // Download the file
                    let resp = client.get_object()
                        .bucket(&bucket)
                        .key(&key_str)
                        .send()
                        .await?;
                    
                    // Create parent directories if needed
                    if let Some(parent) = dest_path.parent() {
                        fs::create_dir_all(parent)?;
                    }
                    
                    // Save the file
                    let body = resp.body.collect().await?;
                    fs::write(&dest_path, body.into_bytes())?;
                    
                    Ok::<_, anyhow::Error>(())
                });
                
                tasks.push(task);
            }
        }
    }
    
    // Wait for all downloads to complete
    for task in tasks {
        task.await??;
    }
    
    Ok(())
} 