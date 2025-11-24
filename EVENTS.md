# Event Bridge Reference

Standard event names for the CrossSave Cloud Tauri bridge.

## Sync queue
- `sync://status` – payload: `{ queue_length, active_job, last_sync, is_syncing }`. Emitted when queue changes or on manual sync calls.
- `sync://conflict-detected` – payload: `game_id` string when a conflict is identified.

## Downloads
- `sync://download-progress` – payload: `{ version_id, progress }` where `progress` is 0-100.
- `sync://download-complete` – payload: `{ version_id, path }` with the downloaded archive path.
- `sync://download-error` – payload: `{ version_id, message }` when download or extraction fails.

## Connectivity
- `sync://online` – payload: `"online"` when the periodic ping succeeds after being offline.
- `sync://offline` – payload: `"offline"` when ping/config validation fails.
