use std::sync::{Arc, RwLock};

use tracing::{info, warn};

use crate::core::profile::{EmulatorProfile, ProfileError, ProfileManager};

fn map_profile_error(err: ProfileError) -> String {
    err.to_string()
}

#[tauri::command]
pub async fn list_profiles(
    state: tauri::State<'_, Arc<RwLock<ProfileManager>>>,
) -> Result<Vec<EmulatorProfile>, String> {
    let mgr = state.read().unwrap();
    let profiles = mgr.list_profiles().map_err(map_profile_error)?;
    info!("[PROFILE] Returning {} profiles", profiles.len());
    Ok(profiles)
}

#[tauri::command]
pub async fn get_profile(
    state: tauri::State<'_, Arc<RwLock<ProfileManager>>>,
    emulator_id: String,
) -> Result<Option<EmulatorProfile>, String> {
    let mgr = state.read().unwrap();
    let profile = mgr.get_profile(&emulator_id).map_err(map_profile_error)?;

    if profile.is_none() {
        warn!("[PROFILE] Requested unknown emulator_id: {emulator_id}");
    }

    Ok(profile)
}

#[tauri::command]
pub async fn save_profile(
    state: tauri::State<'_, Arc<RwLock<ProfileManager>>>,
    profile: EmulatorProfile,
) -> Result<EmulatorProfile, String> {
    let mut mgr = state.write().unwrap();
    mgr.save_profile(profile).map_err(map_profile_error)
}

#[tauri::command]
pub async fn delete_profile(
    state: tauri::State<'_, Arc<RwLock<ProfileManager>>>,
    emulator_id: String,
) -> Result<(), String> {
    let mut mgr = state.write().unwrap();
    mgr.delete_profile(&emulator_id).map_err(map_profile_error)
}
