# Pull from GitHub Container Registry

## Quick Start

Pull and run the pre-built Docker image:

```bash
# Pull latest image
docker pull ghcr.io/h1dr0nn/crosssave-cloud/crosssave-selfhost-server:latest

# Run with docker-compose
# Or use the image directly:
docker run -d \
  --name crosssave-server \
  -p 7373:7373 \
  -e JWT_SECRET="your-secret-here" \
  -e S3_ENDPOINT="http://your-minio:9373" \
  -e S3_BUCKET="crosssave" \
  -e S3_ACCESS_KEY="minioadmin" \
  -e S3_SECRET_KEY="minioadmin" \
  ghcr.io/h1dr0nn/crosssave-cloud/crosssave-selfhost-server:latest
```

## Use with docker-compose

Update your `docker-compose.yml` to use the pre-built image:

```yaml
services:
  server:
    image: ghcr.io/h1dr0nn/crosssave-cloud/crosssave-selfhost-server:latest
    # Remove the 'build' section
    container_name: crosssave-server
    ports:
      - "7373:7373"
    environment:
      # ... your environment variables
```

## Available Tags

- `latest` - Latest build from main branch
- `main` - Main branch builds
- `develop` - Development branch builds
- `v1.0.0` - Specific version tags
- `sha-abc1234` - Specific commit builds

## Platforms

The image is built for multiple platforms:

- `linux/amd64` - x86_64 / Intel / AMD
- `linux/arm64` - ARM64 / Apple Silicon / Raspberry Pi 4

Docker will automatically pull the correct platform for your system.

## Verify Image

Check image signature and provenance:

```bash
docker buildx imagetools inspect ghcr.io/h1dr0nn/crosssave-cloud/crosssave-selfhost-server:latest
```
