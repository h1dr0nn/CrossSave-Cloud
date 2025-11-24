# AGENTS.md

## Project: CrossSave Cloud

## Version: 0.1.0

---

# 1. Purpose

Guidelines for AI agents contributing to the CrossSave Cloud repository.

---

# 2. General Principles

* Produce clean, minimal, readable code.
* Follow existing architecture and folder structure.
* Do not introduce new patterns unless explicitly requested.
* Keep changes small and PR‑sized.
* Do not remove logs unless instructed.
* Avoid unnecessary dependencies.

---

# 3. Naming Rules

## Rust

* Modules: `snake_case`
* Files: `snake_case.rs`
* Structs / Enums: `PascalCase`
* Functions: `snake_case`
* Constants: `SCREAMING_SNAKE_CASE`
* Exposed commands: `#[tauri::command]`

## Svelte + TypeScript

* Components: `PascalCase.svelte`
* Variables: `camelCase`
* Functions: `camelCase`
* Types/Interfaces: `PascalCase`
* Stores: `camelCase + Store`

---

# 4. Code Placement

## Rust (`src-tauri/src/`)

* **core/**: business logic
* **api/**: Tauri command layer
* **utils/**: shared helpers
* **main.rs**: entry point

## Svelte (`src/`)

* **lib/components/**: UI components
* **lib/stores/**: global state
* **lib/helpers/**: utilities
* **routes/**: pages

---

# 5. Module Scope Rules

* **Watcher**: only emits file events.
* **Sync**: metadata, compression, version compare, upload/download.
* **Profiles**: load JSON, detect paths, list tracked folders.

---

# 6. Function Requirements

## Rust

* Return `Result<T, E>` unless trivial.
* No `unwrap()` in production.
* Avoid blocking I/O inside commands.
* Heavy work must run via `tauri::async_runtime`.

## TypeScript

* Use stores instead of globals.
* Use async/await.
* Catch and send errors to UI log panel.
* Avoid direct DOM manipulation.

---

# 7. Logging

## Rust

* Use `tracing` (`trace/debug/info/warn/error`).
* Log: file events, sync start/end, errors.

## Frontend

* Use `logStore`.
* Prefix logs: `[WATCHER]`, `[SYNC]`, `[PROFILE]`, `[UI]`.

---

# 8. Git Commit Rules

* Use English.
* Format: `<category>: <description>`

### Examples

```
watcher: initial linux watcher
profile: add retroarch default config
sync: implement zip packaging
ui: add sidebar layout
infra: setup tauri config
```

* Never commit build artifacts.

---

# 9. Security

* Never sync ROM/Bios.
* Only sync save data (`*.sav`, `*.srm`, `*.bin`, ...).
* Remove sensitive data before logging.
* Do not store credentials in plaintext.

---

**End of File**
