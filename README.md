# CrossSave Cloud

CrossSave Cloud is a lightweight multi-platform tool for syncing emulator save files between **Linux** and **Android**.  
The app automatically detects changes in save directories, packages updated data, uploads them to the cloud, and restores them across devices — allowing players to continue their game seamlessly anywhere.

---

## ✨ Features

- **Auto-sync save data** between Linux and Android  
- **Real-time file watching** using inotify (Linux) and FileObserver (Android)  
- **Cloud backup & restore** with version history  
- **Supports multiple emulators** (RetroArch, PPSSPP, Dolphin, etc.)  
- **Custom save paths** for uncommon or modded emulator setups  
- **Lightweight & fast**, built with Tauri + Rust  
- **Secure transfer & storage** using encrypted packages  
- **Conflict-free syncing** using timestamp & hash detection  

---

## ⚙️ How It Works

1. CrossSave Cloud scans configured emulator save folders  
2. Any file change triggers a save package build (compressed + metadata)  
3. The package is uploaded to the user’s cloud storage  
4. Other devices pull the latest version and apply changes  
5. Users can continue the same game instantly on another platform  

---

## 📦 Technology

- **Rust** – core logic, file system access, hashing, compression, syncing  
- **Tauri** – lightweight UI wrapper for Linux & Android  
- **Web UI** (Svelte/React) – fast, responsive interface  
- **Cloud API** – minimal REST endpoint for upload/download & metadata  

---

## 🔒 Security

- Save packages are encrypted before upload  
- Only save data is synced — **no ROMs, BIOS, or copyrighted content**  
- User account and device keys are never shared  

---

## 🌐 Platforms

- **Linux Desktop / Steam Deck**  
- **Android phones & handheld devices**  

Additional platforms may be supported in the future.

---

## 🚀 Roadmap

- Emulator auto-detection  
- Peer-to-peer local sync (no cloud required)  
- Plugin system for community emulator profiles  
- UI theming + dark/light mode  
- iOS/macOS support (experimental later)  

---

## 📄 License

MIT License. Free to use and modify.

---

## 📨 Contact

For issues, suggestions, or emulator integration requests, open an issue or discussion in the repository.

CrossSave Cloud — *Play everywhere. Your progress follows you.*
