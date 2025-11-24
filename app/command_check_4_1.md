# Phase 4.1 Verification (Cloud Mode Switcher)

## command_1 — Backend: CloudMode Foundation
**Fully implemented**
- `CloudMode` enum carries `Official`, `SelfHost`, and `Off`, with defaults wired into `AppSettings` so persisted settings deserialize with sensible defaults. 【F:src-tauri/src/core/settings.rs†L10-L86】
- `DisabledCloudBackend` exists and returns `CloudError::Disabled` for every trait method, ensuring the off state has a concrete backend. 【F:src-tauri/src/core/cloud.rs†L122-L175】
- Tauri setup builds the cloud backend as an `Arc<Mutex<dyn CloudBackend>>`, swaps implementations via `switch_cloud_backend`, and emits `cloud://backend-switched` after replacement. 【F:src-tauri/src/lib.rs†L193-L236】

**Partially implemented**
- Most cloud commands guard against `cloud_mode == Off`, but the validation commands (`validate_official_cloud_settings`, `validate_self_host_settings`) bypass `ensure_cloud_mode_enabled`, so they still run while the mode is Off instead of returning `CloudError::Disabled`. 【F:src-tauri/src/api/cloud_api.rs†L468-L519】

## command_2 — Backend: Mode Switching + Validation
**Fully implemented**
- `update_cloud_mode` saves the new mode, switches the backend, runs mode-specific validation, and emits `cloud://mode-changed` when successful. 【F:src-tauri/src/api/cloud_api.rs†L292-L338】
- Both validation commands emit `cloud://config-valid` / `cloud://config-invalid`, and download failures emit the standard `sync://download-error` payload. 【F:src-tauri/src/api/cloud_api.rs†L490-L519】【F:src-tauri/src/api/cloud_api.rs†L186-L238】

**Partially implemented**
- Because validation commands are callable while the mode is Off (see command_1), mode-off rejection is not consistent across the API surface. 【F:src-tauri/src/api/cloud_api.rs†L468-L519】

## command_3 — Frontend: Cloud Settings UI
**Fully implemented**
- Cloud Mode tab uses an iOS-style segmented control with the exact labels “Sync Off”, “Sync On”, and “Self-host”, and the page listens for all required cloud and sync events. 【F:src/components/pages/CloudSettingsPage.svelte†L21-L139】【F:src/components/pages/CloudSettingsPage.svelte†L451-L466】
- Selecting “Sync On” shows “Cloud Sync Enabled” and automatically triggers a connection test; statuses render as Online/Ready/Failed/Offline with matching colored chips. 【F:src/components/pages/CloudSettingsPage.svelte†L493-L513】【F:src/components/pages/CloudSettingsPage.svelte†L184-L219】【F:src/components/pages/CloudSettingsPage.svelte†L468-L481】
- “Sync Off” disables cloud features, while “Self-host” presents the ID/Relay header with copy/paste controls, padded separator, horizontal label/input rows, and a “Save & Connect” action that tests connectivity and reports Ready/Failed. 【F:src/components/pages/CloudSettingsPage.svelte†L486-L520】【F:src/components/pages/CloudSettingsPage.svelte†L515-L620】

**Partially implemented / Missing**
- The Cloud Mode store is updated only after the backend call resolves (`updateCloudMode` awaits `update_cloud_mode` before mutating the store), so the global store does not reflect the optimistic UI switch immediately as required. 【F:src/lib/stores/cloudStore.ts†L381-L385】【F:src/components/pages/CloudSettingsPage.svelte†L221-L299】
- Validation commands still execute when cloud mode is Off, so a rejected backend configuration can still be exercised rather than short-circuited. 【F:src-tauri/src/api/cloud_api.rs†L468-L519】

## Summary
- Backend foundations and event wiring for cloud mode switching largely exist, including disabled backend handling and validation events.
- Gaps remain around enforcing `CloudError::Disabled` for validation commands and reflecting optimistic mode changes in the shared store immediately, leaving the “Cloud Mode store updates instantly” requirement unmet.
