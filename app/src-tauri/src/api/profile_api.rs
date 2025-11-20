use tracing::{info, warn};

use crate::core::profile::{get_profile_by_id, get_profiles, EmulatorProfile, ProfileError};

#[tauri::command]
pub async fn list_profiles() -> Result<Vec<EmulatorProfile>, String> {
    let profiles = get_profiles().map_err(|err| err.to_string())?;
    info!("[PROFILE] Returning {} profiles", profiles.len());
    Ok(profiles)
}

#[tauri::command]
pub async fn get_profile(emulator_id: String) -> Result<Option<EmulatorProfile>, String> {
    let profile = get_profile_by_id(&emulator_id).map_err(|err| err.to_string())?;

    if profile.is_none() {
        warn!("[PROFILE] Requested unknown emulator_id: {emulator_id}");
    }

    Ok(profile)
}
