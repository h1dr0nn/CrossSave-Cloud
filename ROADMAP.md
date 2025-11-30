# 🚀 CrossSave Cloud – Project Roadmap

Modern cross-platform save game synchronization using **Tauri + Svelte + Rust** with **local-first architecture**.

---

## 1. Goals

- Xây dựng hệ thống đồng bộ save game **local-first** với khả năng sync cloud tùy chọn.
- Hỗ trợ **3 chế độ cloud**: Official Cloud, Self-host, và Sync Off (local-only).
- Giao diện hiện đại, responsive, dễ sử dụng trên cả Linux và Android.
- Theo dõi thay đổi file realtime với **watcher engine** tối ưu.
- Đảm bảo app **nhẹ**, **ổn định**, **bảo mật** theo tiêu chuẩn Tauri.
- Thiết lập CI/CD tự động build và publish release đa nền tảng.

---

## 2. Architecture Overview

```
Tauri (App Shell)
├── Svelte + Tailwind (UI Layer)
├── Rust Backend (Watcher, Sync Engine, Cloud API)
├── Emulator Profile System
└── Multi-mode Cloud Backends (Official / Self-host / Disabled)
```

---

## 3. Milestones

### 🧩 **Phase 1 — Local Core System (Linux)**

**Status: ✅ Complete**

#### **1.1 — Watcher Engine**

- [x] Inotify-based watcher.
- [x] Debounce mechanism.
- [x] Event streaming to UI.
- [x] Multi-path support.

#### **1.2 — Emulator Profile System**

- [x] Load JSON profiles.
- [x] Validate paths & patterns.
- [x] Provide usable default profiles.
- [x] Manual profile testing in UI.

#### **1.3 — Save Packaging Engine**

- [x] Collect files by rules/patterns.
- [x] Create archive.
- [x] Metadata builder.
- [x] Version-id hashing.

#### **1.4 — Local Save History**

- [x] Store up to 10 versions.
- [x] Rollback function.
- [x] Delete version.
- [x] Version metadata view.

#### **1.5 — Local Test UI**

- [x] Watcher panel.
- [x] Packager panel.
- [x] History panel.
- [x] Error toast + logging.
- [x] Minimal layout for testing backend.

---

### 🎨 **Phase 2 — Local Full UI/UX**

**Status: ✅ Complete**

#### **2.1 — Main Application UI**

- [x] Sidebar: Emulator list.
- [x] Auto-load profiles.
- [x] Game list per emulator.
- [x] Save folder + pattern preview.

#### **2.2 — Game Detail View**

- [x] Current save info.
- [x] Manual save packaging.
- [x] Recent history panel.
- [x] Compare versions (hash/size/file count).

#### **2.3 — Integrated Watcher UI**

- [x] Live file-change feed.
- [x] Save-changed indicator.
- [x] Auto-package toggle.

#### **2.4 — Settings UI**

- [x] Add/edit emulator profiles.
- [x] Override save paths.
- [x] Backup retention settings.
- [x] App storage settings.

---

### ☁️ **Phase 3 — Cloud Sync System**

**Status: ✅ Complete**

#### **3.1 — Cloud API**

- [x] HTTP-based backend abstraction (CloudBackend).
- [x] Upload archive.
- [x] List versions.
- [x] Download version.
- [x] Device_id assignment & persistence.
- [x] Cloud settings persisted in AppSettings.

#### **3.2 — Sync Logic**

- [x] Latest-wins + conflict detection (hash + timestamp + device_id).
- [x] Tracking upload queue with persistence.
- [x] Throttled sync + backoff retry.
- [x] Online/offline detection and sync pausing.
- [x] Download path: fetch, extract, record to local history.

#### **3.3 — Cloud Account UI**

- [x] Login page (real backend).
- [x] Device management (list/remove).
- [x] Manual Sync button wired to sync engine.
- [x] Cloud version history with real metadata.
- [x] Sync/download progress & error indicators.
- [x] Cloud settings validation (base URL, API key).

---

### 🌐 **Phase 4 — Cloud Modes & Deployment**

**Status: 🚧 In Progress**

**Goal**: Support 3 fully independent cloud modes:

- **Official Cloud** (Cloudflare Worker + R2 + presigned URLs)
- **Self-host** (Docker backend user controls)
- **Sync Off** (local-only)

#### **4.1 — Cloud Mode Switcher**

- [x] Add global selector: OFFICIAL / SELF_HOST / SYNC_OFF.
- [x] Automatically switch cloud backend implementation.
- [x] Disable all cloud actions when SYNC_OFF.
- [x] UI indicator for current cloud mode.

#### **4.2 — Official Cloud (Cloudflare Worker + R2)**

- [x] Worker endpoints:
  - [x] `/login`, `/device/register`, `/device/list`.
  - [x] `/save/upload-url` (presigned PUT).
  - [x] `/save/download-url` (presigned GET).
  - [x] `/save/list`.
- [x] Upload pipeline:
  1. Request presigned URL.
  2. PUT archive directly to R2.
  3. Notify Worker → update metadata.
- [x] Full metadata: sha256, size, file list, timestamp, device_id.
- [x] Security: Cloudflare Access / Signed tokens / Zero trust rules.

#### **4.3 — Self-host Cloud (RustDesk-style Config)**

**Client App (GUI Settings):**

- [x] Self-host mode selector in Cloud Settings.
- [x] Input fields:
  - [x] ID Server (e.g., `id.server.com`).
  - [x] Relay Server (e.g., `relay.server.com`).
  - [x] API Server (e.g., `api.server.com`).
  - [x] Access Key/Password.
- [x] Copy/Paste config (format: `id|relay|api|key`).
- [x] Save & Connect button with status feedback.
- [x] Connection status indicators (Ready/Failed/Offline).

**Server Implementation (Docker Backend):**

- [ ] Docker image for easy self-hosting.
- [ ] API endpoints mirroring official cloud:
  - [ ] Authentication (access key / email+password).
  - [ ] Device registration & management.
  - [ ] Save upload/download (presigned or direct).
  - [ ] Version listing & metadata.
- [ ] File storage: local disk or S3-compatible.
- [ ] Deployment documentation & templates.
- [ ] Docker Compose example with reverse proxy.

#### **4.4 — Cloud Security & Permissions**

- [ ] Validate cloud configuration before enabling sync.
- [ ] Unified error mapping (Official & Self-host).
- [ ] Online/offline detection & UI indicators.
- [ ] Logging tags: `[CLOUD_OFFICIAL]`, `[CLOUD_SELF_HOST]`.

---

### 📱 **Phase 5 — Android Platform & Packaging**

**Status: Pending**

#### **5.1 — Android Storage Access**

- [ ] SAF directory selector.
- [ ] Persisted URI permissions per emulator.
- [ ] UI guidance for selecting correct folders.

#### **5.2 — Android Watcher**

- [ ] FileObserver wrapper per SAF folder.
- [ ] Fallback polling scan.
- [ ] Bridge watcher events into sync engine.

#### **5.3 — Android Sync UX**

- [ ] Manual sync button.
- [ ] Auto sync on app open/resume.
- [ ] "WiFi only" mode.
- [ ] Mobile-friendly progress indicators.

#### **5.4 — Android Packaging**

- [ ] Debug APK.
- [ ] Release APK + AAB.
- [ ] Keystore signing.
- [ ] Multi-device smoke tests.

---

### 🌟 **Phase 6 — Web Dashboard & Final Polish**

**Status: Pending**

#### **6.1 — Web Dashboard (Official & Self-host)**

- [ ] Login & auth UI.
- [ ] Device list + revoke.
- [ ] Game list + version history.
- [ ] Version metadata viewer (size, hash, device, timestamp).
- [ ] Rollback / mark preferred version (self-host only).
- [ ] Built with Cloudflare Pages or SvelteKit.

#### **6.2 — Developer & Plugin Ecosystem**

- [ ] Import/export emulator profiles.
- [ ] Community profile database.
- [ ] Self-host extensions & reverse proxy templates.

#### **6.3 — Performance & Optimizations**

- [ ] Reduce memory usage.
- [ ] Faster packaging pipeline.
- [ ] Optimize watcher + sync intervals.
- [ ] Better production logs.

#### **6.4 — Multi-platform Packaging & Release**

- [ ] Linux AppImage.
- [ ] Linux .deb.
- [ ] Linux Flatpak.
- [ ] Android APK/AAB.
- [ ] Full documentation (Official Cloud + Self-host + FAQ).

---

## 4. Success Criteria

- App chạy ổn trên Linux (x86_64, ARM64).
- Hỗ trợ Android native với SAF permissions.
- UI responsive, mượt mà, dễ sử dụng.
- 3 chế độ cloud hoạt động độc lập, ổn định.
- Sync engine đáng tin cậy với conflict detection.
- Build tự động, release tự động đa nền tảng.
- Documentation đầy đủ cho end users và self-hosters.

---

## 5. Long-term Vision

**Trở thành giải pháp sync save game local-first tốt nhất cho Linux gaming & retro emulation**:

- Hỗ trợ mọi emulator phổ biến với profile system linh hoạt.
- Cloud tùy chọn: Official managed service hoặc self-host đơn giản.
- Mở rộng sang Android để sync cross-device seamless.
- Community-driven profile database.
- Tích hợp web dashboard để quản lý từ mọi nơi.

---
