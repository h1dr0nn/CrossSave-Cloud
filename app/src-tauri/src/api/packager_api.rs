use std::path::PathBuf;

use tracing::{error, info, warn};

use crate::core::packager::{PackagedSave, SavePackager};

#[tauri::command]
pub async fn package_save(
    game_id: String,
    emulator_id: String,
    paths: Vec<String>,
    patterns: Vec<String>,
) -> Result<PackagedSave, String> {
    let sanitized_paths: Vec<PathBuf> = paths
        .into_iter()
        .filter(|path| !path.trim().is_empty())
        .map(PathBuf::from)
        .collect();

    if sanitized_paths.is_empty() {
        warn!("[PACKAGER] No valid paths provided to API");
        return Err("no valid paths provided".into());
    }

    let sanitized_patterns: Vec<String> = patterns
        .into_iter()
        .filter(|pattern| !pattern.trim().is_empty())
        .collect();

    let packager = SavePackager::new(game_id, emulator_id);

    let join_result = tauri::async_runtime::spawn_blocking(move || {
        let mut packager = packager;
        match packager.package_save(sanitized_paths, sanitized_patterns) {
            Ok(result) => {
                info!("[PACKAGER] Packaging completed via API");
                Ok(result)
            }
            Err(err) => {
                error!("[PACKAGER] Packaging failed: {err}");
                Err(err.to_string())
            }
        }
    })
    .await
    .map_err(|err| err.to_string())?;

    join_result
}

#[tauri::command]
pub async fn validate_paths(paths: Vec<String>) -> Result<Vec<String>, String> {
    let cleaned: Vec<PathBuf> = paths
        .into_iter()
        .filter(|path| !path.trim().is_empty())
        .map(PathBuf::from)
        .collect();

    let join_result = tauri::async_runtime::spawn_blocking(move || {
        let mut valid: Vec<String> = Vec::new();

        for path in cleaned {
            match path.canonicalize() {
                Ok(resolved) => {
                    if resolved.exists() {
                        valid.push(resolved.to_string_lossy().to_string());
                    }
                }
                Err(err) => warn!("[PACKAGER] Invalid path {:?}: {}", path, err),
            }
        }

        if valid.is_empty() {
            Err("no valid paths found".into())
        } else {
            Ok(valid)
        }
    })
    .await
    .map_err(|err| err.to_string())?;

    join_result
}
