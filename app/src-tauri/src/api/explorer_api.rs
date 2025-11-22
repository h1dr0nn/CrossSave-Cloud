use std::path::PathBuf;
use std::sync::{Arc, RwLock};

use serde::Serialize;
use tracing::info;

use crate::core::packager::SavePackager;
use crate::core::profile::ProfileManager;

#[derive(Debug, Serialize)]
pub struct ScannedFile {
    pub path: String,
    pub name: String,
    pub size: u64,
    pub modified: u128,
}

#[tauri::command]
pub async fn scan_save_files(
    profile_state: tauri::State<'_, Arc<RwLock<ProfileManager>>>,
    emulator_id: String,
) -> Result<Vec<ScannedFile>, String> {
    // Clone data needed for the thread in a separate block to ensure lock is released
    let (default_save_paths, file_patterns) = {
        let manager = profile_state.read().map_err(|e| e.to_string())?;
        let profile = manager
            .get_profile(&emulator_id)
            .map_err(|e| e.to_string())?
            .ok_or_else(|| format!("Profile {} not found", emulator_id))?;
            
        (profile.default_save_paths.clone(), profile.file_patterns.clone())
    };

    let emulator_id_clone = emulator_id.clone();
    
    // Offload scanning to a blocking thread to avoid freezing the UI
    let scanned_files = tauri::async_runtime::spawn_blocking(move || {
        let packager = SavePackager::new("explorer".to_string(), emulator_id_clone.clone());
        
        let paths: Vec<PathBuf> = default_save_paths
            .iter()
            .map(PathBuf::from)
            .collect();

        let files = packager
            .collect_files(paths, file_patterns)
            .map_err(|e| e.to_string())?;

        let mut scanned_files = Vec::new();

        for path in files {
            if let Ok(metadata) = std::fs::metadata(&path) {
                let name = path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("unknown")
                    .to_string();

                let modified = metadata
                    .modified()
                    .unwrap_or(std::time::SystemTime::now())
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_millis();

                scanned_files.push(ScannedFile {
                    path: path.to_string_lossy().to_string(),
                    name,
                    size: metadata.len(),
                    modified,
                });
            }
        }
        
        info!("[EXPLORER] Found {} files for {}", scanned_files.len(), emulator_id_clone);
        Ok::<Vec<ScannedFile>, String>(scanned_files)
    }).await.map_err(|e| e.to_string())??;

    Ok(scanned_files)
}

#[derive(Debug, Serialize)]
pub struct PathStatus {
    pub path: String,
    pub exists: bool,
    pub is_dir: bool,
    pub error: Option<String>,
}

#[tauri::command]
pub async fn check_path_status(
    profile_state: tauri::State<'_, Arc<RwLock<ProfileManager>>>,
    emulator_id: String,
) -> Result<Vec<PathStatus>, String> {
    let manager = profile_state.read().map_err(|e| e.to_string())?;
    let profile = manager
        .get_profile(&emulator_id)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| format!("Profile {} not found", emulator_id))?;

    let mut statuses = Vec::new();

    for path_str in profile.default_save_paths {
        let path = PathBuf::from(&path_str);
        let exists = path.exists();
        let is_dir = path.is_dir();
        
        // Try to read dir to check permissions if it exists
        let error = if exists && is_dir {
            match std::fs::read_dir(&path) {
                Ok(_) => None,
                Err(e) => Some(e.to_string()),
            }
        } else {
            None
        };

        statuses.push(PathStatus {
            path: path_str,
            exists,
            is_dir,
            error,
        });
    }

    Ok(statuses)
}
