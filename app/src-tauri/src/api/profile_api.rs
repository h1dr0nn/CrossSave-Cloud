use tracing::{info, warn};

use crate::core::profile::{EmulatorProfile, ProfileError, ProfileManager};

fn map_profile_error(err: ProfileError) -> String {
    err.to_string()
}

#[tauri::command]
pub async fn list_profiles(
    state: tauri::State<'_, ProfileManager>,
) -> Result<Vec<EmulatorProfile>, String> {
    let profiles = state.list_profiles().map_err(map_profile_error)?;
    info!("[PROFILE] Returning {} profiles", profiles.len());
    Ok(profiles)
}

#[tauri::command]
pub async fn get_profile(
    state: tauri::State<'_, ProfileManager>,
    emulator_id: String,
) -> Result<Option<EmulatorProfile>, String> {
    let profile = state
        .get_profile(&emulator_id)
        .map_err(map_profile_error)?;

    if profile.is_none() {
        warn!("[PROFILE] Requested unknown emulator_id: {emulator_id}");
    }

    Ok(profile)
}

#[tauri::command]
pub async fn save_profile(
    state: tauri::State<'_, ProfileManager>,
    profile: EmulatorProfile,
) -> Result<EmulatorProfile, String> {
    state.save_profile(profile).map_err(map_profile_error)
}

#[tauri::command]
pub async fn delete_profile(
    state: tauri::State<'_, ProfileManager>,
    emulator_id: String,
) -> Result<(), String> {
    state.delete_profile(&emulator_id).map_err(map_profile_error)
}
