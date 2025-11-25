# CrossSave Official Cloud Worker (stub)

This folder contains the initial Cloudflare Worker scaffold for the Official Cloud backend.

- **R2 binding:** `CROSSSAVE_R2` (see `wrangler.toml`; update `bucket_name` and account settings when deploying).
- **Directory schema (Option B):**
  - `users/{userId}/metadata.json`
  - `users/{userId}/devices.json`
  - `users/{userId}/saves/{gameId}/{versionId}.zip`
- **Health endpoint:** `GET /health` creates the default metadata/devices files for a `health-check` user and returns `{ ok: true, r2: "ready" }`.

Future steps will add authentication, real save endpoints, and integration with the Tauri client.
