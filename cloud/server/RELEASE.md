# Release Process

## Creating a New Release

To build and publish a Docker image to GitHub Container Registry:

### 1. Tag your release

```bash
# Create and push a version tag
git tag v1.0.0
git push origin v1.0.0
```

### 2. GitHub Actions will automatically:

- Build Docker image for `linux/amd64` and `linux/arm64`
- Push to `ghcr.io/h1dr0nn/crosssave-cloud/crosssave-selfhost-server`
- Create tags:
  - `v1.0.0` (exact version)
  - `1.0` (major.minor)
  - `1` (major)
  - `latest`

### 3. Check build status

Go to: https://github.com/h1dr0nn/crosssave-cloud/actions

### 4. Pull and test

```bash
docker pull ghcr.io/h1dr0nn/crosssave-cloud/crosssave-selfhost-server:v1.0.0
# or
docker pull ghcr.io/h1dr0nn/crosssave-cloud/crosssave-selfhost-server:latest
```

## Manual Build (Optional)

You can also manually trigger a build from GitHub:

1. Go to Actions → Build and Push Docker Image
2. Click "Run workflow"
3. Select branch/tag
4. Click "Run workflow"

## Version Naming

Follow semantic versioning:

- `v1.0.0` - Major release (breaking changes)
- `v1.1.0` - Minor release (new features)
- `v1.0.1` - Patch release (bug fixes)

## Available Images

After release, images are available at:

```
ghcr.io/h1dr0nn/crosssave-cloud/crosssave-selfhost-server:latest
ghcr.io/h1dr0nn/crosssave-cloud/crosssave-selfhost-server:v1.0.0
ghcr.io/h1dr0nn/crosssave-cloud/crosssave-selfhost-server:1.0
ghcr.io/h1dr0nn/crosssave-cloud/crosssave-selfhost-server:1
```
