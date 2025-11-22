# Build Scripts

## Android Build

Build Android APK with automatic date-based versioning:

```bash
./scripts/build-android.sh
```

This will:

1. Build the Android APK
2. Sign it with debug keystore
3. Name it as `crosssave-cloud-YYYYMMDD-0.0.X.apk`
4. Save to `builds/android/` directory

Version number (`X`) automatically increments for each build on the same day.

### Output Location

All builds are saved to: `builds/android/`

### Example

First build today: `crosssave-cloud-20251122-0.0.1.apk`
Second build today: `crosssave-cloud-20251122-0.0.2.apk`
Tomorrow's first build: `crosssave-cloud-20251123-0.0.1.apk`
