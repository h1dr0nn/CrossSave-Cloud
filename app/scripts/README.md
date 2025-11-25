# Build Scripts

CrossSave Cloud build scripts for all supported platforms with multi-architecture support.

## Quick Start

```bash
# Make scripts executable (already done)
chmod +x scripts/build-*.sh

# Build for your platform
./scripts/build-macos.sh      # macOS (x86_64 + Apple Silicon)
./scripts/build-windows.sh    # Windows (x86_64 + ARM64)
./scripts/build-linux.sh      # Linux (x86_64 + aarch64)
./scripts/build-ios.sh        # iOS (Simulator + Device)
./scripts/build-android.sh    # Android (aarch64)
```

---

## Platform-Specific Builds

### 🍎 macOS (`build-macos.sh`)

**Requirements:** macOS, Xcode Command Line Tools

```bash
# Build both architectures
./scripts/build-macos.sh

# Build specific architecture
./scripts/build-macos.sh --arch x86_64      # Intel
./scripts/build-macos.sh --arch aarch64     # Apple Silicon
```

**Output:**

- Binary: `builds/macos/{arch}/crosssave-cloud-{date}-{version}-{arch}`
- Tarball: `builds/macos/{arch}/crosssave-cloud-{date}-{version}-{arch}.tar.gz`
- DMG: `builds/macos/{arch}/crosssave-cloud-{date}-{version}-{arch}.dmg`

---

### 🪟 Windows (`build-windows.sh`)

**Requirements:**

- Windows: MSVC toolchain
- Linux/macOS: mingw-w64 for cross-compilation

```bash
# Build both architectures
./scripts/build-windows.sh

# Build specific architecture
./scripts/build-windows.sh --arch x86_64
./scripts/build-windows.sh --arch aarch64   # ARM64
```

**Cross-compilation setup:**

```bash
# Linux
sudo apt-get install mingw-w64

# macOS
brew install mingw-w64
```

**Output:**

- Binary: `builds/windows/{arch}/crosssave-cloud-{date}-{version}-{arch}.exe`
- Zip: `builds/windows/{arch}/crosssave-cloud-{date}-{version}-{arch}.zip`
- MSI: `builds/windows/{arch}/crosssave-cloud-{date}-{version}-{arch}.msi` (Windows only)

---

### 🐧 Linux (`build-linux.sh`)

**Requirements:** Linux, gcc-aarch64-linux-gnu (for cross-compilation)

```bash
# Build both architectures
./scripts/build-linux.sh

# Build specific architecture
./scripts/build-linux.sh --arch x86_64
./scripts/build-linux.sh --arch aarch64
```

**Cross-compilation setup:**

```bash
sudo apt-get install gcc-aarch64-linux-gnu
```

**Output:**

- Binary: `builds/linux/{arch}/crosssave-cloud-{date}-{version}-{arch}`
- Tarball: `builds/linux/{arch}/crosssave-cloud-{date}-{version}-{arch}.tar.gz`

---

### 📱 iOS (`build-ios.sh`)

**Requirements:** macOS, Xcode

```bash
# Build both architectures
./scripts/build-ios.sh

# Build for simulator
./scripts/build-ios.sh --arch x86_64

# Build for device
./scripts/build-ios.sh --arch aarch64
```

**Output:**

- Simulator: `builds/ios/x86_64/crosssave-cloud-{date}-{version}-x86_64.app`
- Device: `builds/ios/aarch64/crosssave-cloud-{date}-{version}-aarch64.ipa`
- Tarball: `builds/ios/{arch}/crosssave-cloud-{date}-{version}-{arch}.tar.gz`

**Testing:**

```bash
# Install on simulator
xcrun simctl install booted builds/ios/x86_64/crosssave-cloud-*.app

# Install on device
ios-deploy --bundle builds/ios/aarch64/crosssave-cloud-*.ipa
```

---

### 🤖 Android (`build-android.sh`)

**Requirements:** Android SDK, NDK, Java

```bash
./scripts/build-android.sh
```

**Output:**

- APK: `builds/android/crosssave-cloud-{date}-{version}.apk`

---

## Common Features

All build scripts include:

✅ **Multi-architecture support** - Build for both x86_64 and ARM64/aarch64  
✅ **Automatic versioning** - Date-based versioning (YYYYMMDD-0.0.X)  
✅ **Build artifacts** - Binaries, archives, and platform-specific packages  
✅ **Dependency checking** - Validates required tools before building  
✅ **Color output** - Easy-to-read build progress  
✅ **Help documentation** - Use `--help` flag for usage info

---

## Versioning

All builds use automatic date-based versioning:

**Format:** `crosssave-cloud-YYYYMMDD-0.0.X`

**Examples:**

- First build today: `crosssave-cloud-20251125-0.0.1`
- Second build today: `crosssave-cloud-20251125-0.0.2`
- Tomorrow's first build: `crosssave-cloud-20251126-0.0.1`

Version number (`X`) automatically increments for each build on the same day.

---

## Build Directory Structure

```
builds/
├── macos/
│   ├── x86_64/
│   │   ├── crosssave-cloud-20251125-0.0.1-x86_64
│   │   ├── crosssave-cloud-20251125-0.0.1-x86_64.tar.gz
│   │   └── crosssave-cloud-20251125-0.0.1-x86_64.dmg
│   └── aarch64/
│       ├── crosssave-cloud-20251125-0.0.1-aarch64
│       ├── crosssave-cloud-20251125-0.0.1-aarch64.tar.gz
│       └── crosssave-cloud-20251125-0.0.1-aarch64.dmg
├── windows/
│   ├── x86_64/
│   │   ├── crosssave-cloud-20251125-0.0.1-x86_64.exe
│   │   ├── crosssave-cloud-20251125-0.0.1-x86_64.zip
│   │   └── crosssave-cloud-20251125-0.0.1-x86_64.msi
│   └── aarch64/
├── linux/
│   ├── x86_64/
│   └── aarch64/
├── ios/
│   ├── x86_64/
│   └── aarch64/
└── android/
    └── crosssave-cloud-20251125-0.0.1.apk
```

---

## Help

Each script has built-in help:

```bash
./scripts/build-macos.sh --help
./scripts/build-windows.sh --help
./scripts/build-linux.sh --help
./scripts/build-ios.sh --help
```

---

## Platform Requirements Summary

| Platform | Host OS | Cross-compile | Additional Tools      |
| -------- | ------- | ------------- | --------------------- |
| macOS    | macOS   | ✅ Both archs | Xcode CLI Tools       |
| Windows  | Any     | ✅ Yes        | mingw-w64 (cross)     |
| Linux    | Linux   | ✅ Yes        | gcc-aarch64-linux-gnu |
| iOS      | macOS   | ✅ Both archs | Xcode                 |
| Android  | Any     | N/A           | Android SDK/NDK       |

---

## Troubleshooting

### Missing dependencies

Each script checks for required dependencies and provides installation instructions if missing.

### Cross-compilation issues

**Linux aarch64:**

```bash
sudo apt-get install gcc-aarch64-linux-gnu
rustup target add aarch64-unknown-linux-gnu
```

**Windows from Linux/macOS:**

```bash
# Linux
sudo apt-get install mingw-w64

# macOS
brew install mingw-w64

rustup target add x86_64-pc-windows-gnu
```

### Build failures

1. Ensure all dependencies are installed
2. Check Rust toolchain: `rustup update`
3. Clean build: `cargo clean` in `src-tauri/`
4. Rebuild frontend: `pnpm install && pnpm build`
