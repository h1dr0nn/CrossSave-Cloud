# CrossSave Official Cloud Worker (stub)

This folder contains the initial Cloudflare Worker scaffold for the Official Cloud backend.

- **R2 binding:** `CROSSSAVE_R2` (see `wrangler.toml`; update `bucket_name` and account settings when deploying).
- **Directory schema (Option B):**
  - `users/{userId}/metadata.json`
  - `users/{userId}/devices.json`
  - `users/{userId}/saves/{gameId}/{versionId}.zip`
- **Health endpoint:** `GET /health` creates the default metadata/devices files for a `health-check` user and returns `{ ok: true, r2: "ready" }`.

Future steps will add authentication, real save endpoints, and integration with the Tauri client.

## Manual test plan (auth)

- `curl -X POST https://<worker>/signup -H "content-type: application/json" -d '{"email":"user@example.com","password":"hunter2"}'` → confirm `{ ok: true, token, user_id }` and new `users/{userId}/metadata.json` + `devices.json` in R2.
- `curl -X POST https://<worker>/login -H "content-type: application/json" -d '{"email":"user@example.com","password":"hunter2"}'` → expect `{ ok: true, token }` and verify `verifyJwt` accepts the token using the configured `JWT_SECRET`.
- Inspect stored files to ensure `metadata.json` matches the `UserMetadata` schema and `devices.json` contains device entries.
