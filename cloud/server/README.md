# CrossSave Self-host Server

Self-hosted cloud sync server for CrossSave - a local-first save game synchronization application.

## Features

- 🔐 **Email/Password Authentication** with JWT tokens
- 💾 **S3-Compatible Storage** (MinIO or external S3/R2)
- 🔄 **Full API Compatibility** with CrossSave official cloud
- 🐳 **Docker Deployment** with docker-compose
- 🚀 **Presigned URLs** for efficient uploads/downloads
- 📝 **Metadata Management** for save versions

## Quick Start

### Prerequisites

- Docker and Docker Compose
- (Optional) Reverse proxy for production (nginx/Caddy)

### 1. Clone and Configure

```bash
cd cloud/server
cp .env.example .env
# Edit .env and change JWT_SECRET!
```

**Important:** Generate a secure JWT secret:

```bash
openssl rand -base64 32
```

### 2. Start Services

```bash
docker-compose up -d
```

This will start:

- **MinIO** (S3 storage) on ports 9373 (API) and 9374 (Console)
- **CrossSave Server** on port 7373

### 3. Verify

```bash
curl http://localhost:7373/health
# Should return: {"ok":true,"status":"healthy"}
```

### 4. Access MinIO Console (Optional)

- URL: http://localhost:9374
- Username: `minioadmin`
- Password: `minioadmin`

## Configuration

### Environment Variables

| Variable        | Description     | Default             |
| --------------- | --------------- | ------------------- |
| `SERVER_HOST`   | Bind address    | `0.0.0.0`           |
| `SERVER_PORT`   | Server port     | `7373`              |
| `JWT_SECRET`    | JWT signing key | **REQUIRED**        |
| `S3_ENDPOINT`   | S3 endpoint URL | `http://minio:9373` |
| `S3_BUCKET`     | S3 bucket name  | `crosssave`         |
| `S3_ACCESS_KEY` | S3 access key   | `minioadmin`        |
| `S3_SECRET_KEY` | S3 secret key   | `minioadmin`        |
| `S3_REGION`     | S3 region       | `us-east-1`         |

### Using External S3

Edit `.env` to use AWS S3, Cloudflare R2, or other S3-compatible storage:

```env
S3_ENDPOINT=https://your-s3-endpoint.com
S3_BUCKET=your-bucket-name
S3_ACCESS_KEY=your-access-key
S3_SECRET_KEY=your-secret-key
S3_REGION=auto
```

## API Endpoints

All endpoints match the official CrossSave cloud API.

### Authentication

| Endpoint  | Method | Auth | Description    |
| --------- | ------ | ---- | -------------- |
| `/signup` | POST   | -    | Create account |
| `/login`  | POST   | -    | Login          |

### Device Management

| Endpoint           | Method | Auth | Description     |
| ------------------ | ------ | ---- | --------------- |
| `/device/register` | POST   | ✓    | Register device |
| `/device/list`     | GET    | ✓    | List devices    |
| `/device/remove`   | POST   | ✓    | Remove device   |

### Save Management

| Endpoint              | Method | Auth | Description      |
| --------------------- | ------ | ---- | ---------------- |
| `/save/upload-url`    | POST   | ✓    | Get upload URL   |
| `/save/notify-upload` | POST   | ✓    | Confirm upload   |
| `/save/download-url`  | POST   | ✓    | Get download URL |
| `/save/list`          | POST   | ✓    | List saves       |
| `/save/games`         | POST   | ✓    | List games       |

### Health Check

| Endpoint  | Method | Auth | Description   |
| --------- | ------ | ---- | ------------- |
| `/health` | GET    | -    | Server health |

## Client Configuration

In CrossSave app:

1. Go to **Settings → Cloud Settings**
2. Select **Self-host** mode
3. Enter:
   - **API Server**: `http://your-server-ip:7373`
   - **Access Key**: (leave empty)
4. Click **Save & Connect**

## Production Deployment

### 1. Use Reverse Proxy

For production, use nginx or Caddy for SSL termination:

```nginx
server {
    listen 443 ssl http2;
    server_name crosssave.yourdomain.com;

    ssl_certificate /path/to/cert.pem;
    ssl_certificate_key /path/to/key.pem;

    location / {
        proxy_pass http://localhost:7373;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
    }
}
```

### 2. Security Checklist

- [ ] Change `JWT_SECRET` to a secure random value
- [ ] Enable HTTPS with valid SSL certificate
- [ ] Change MinIO default credentials
- [ ] Configure firewall (allow 7373, block 9373/9374 publicly)
- [ ] Set up S3 bucket lifecycle policies
- [ ] Enable logging and monitoring
- [ ] Configure backup strategy

### 3. Resource Limits

Edit `docker-compose.yml` to set resource limits:

```yaml
services:
  server:
    deploy:
      resources:
        limits:
          memory: 512M
          cpus: "0.5"
```

## Development

### Build from Source

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Build
cargo build --release

# Run
./target/release/crosssave-selfhost-server
```

### Run Tests

```bash
cargo test
```

## Troubleshooting

### Server won't start

Check logs:

```bash
docker-compose logs -f server
```

### MinIO connection fails

1. Ensure MinIO is healthy:

   ```bash
   docker-compose ps
   ```

2. Check MinIO logs:

   ```bash
   docker-compose logs -f minio
   ```

3. Verify bucket exists in MinIO console

### Client can't connect

1. Verify server is accessible:

   ```bash
   curl http://your-server-ip:7373/health
   ```

2. Check firewall rules
3. Verify CORS is enabled

## Storage Structure

Files are stored in S3 with this structure:

```
crosssave/
├── users/
│   └── {user_id}/
│       ├── metadata.json          # User account info
│       ├── devices.json           # Registered devices
│       ├── save_metadata.json     # Save versions index
│       └── saves/
│           └── {game_id}/
│               └── {version_id}.zip
└── email_lookup/
    └── {email}.json               # Email to user_id mapping
```

## License

See main CrossSave project for license information.

## Support

For issues and questions, please open an issue on the main CrossSave repository.
