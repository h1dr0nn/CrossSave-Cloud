use tracing::{info, warn};

use crate::core::profile::{get_profile_by_id, get_profiles, EmulatorProfile, ProfileError};

#[tauri::command]
pub async fn list_profiles() -> Result<Vec<EmulatorProfile>, ProfileError> {
    let profiles = get_profiles()?;
    info!("[PROFILE] Returning {} profiles", profiles.len());
    Ok(profiles)
}

#[tauri::command]
pub async fn get_profile(emulator_id: String) -> Result<Option<EmulatorProfile>, ProfileError> {
    let profile = get_profile_by_id(&emulator_id)?;

    if profile.is_none() {
        warn!("[PROFILE] Requested unknown emulator_id: {emulator_id}");
    }

    Ok(profile)
}
