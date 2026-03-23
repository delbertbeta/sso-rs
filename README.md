# sso-rs

An implement of SSO (OpenID Connect Provider) in Rust.

---

## Quick Deploy

This service depends on:

- MySQL
- Redis
- An S3-compatible object storage service

The HTTP server listens on `3000` and the RPC server listens on `2999`.

### 1. Prepare `.env`

Create a `.env` file in the project root for local runs, or use the same file with `docker run --env-file`.

```env
DATABASE_URL=mysql://username:password@mysql-host:3306/sso
REDIS_URL=redis://default:password@redis-host:6379/1
FRONT_END_URL=https://sso.example.com

# Optional. If set, cookies are treated as production cookies.
PROD=1

# Optional. Defaults to: sso_rs=debug,tower_http=trace
RUST_LOG=info

BUCKET_REGION=auto
BUCKET_NAME=sso-static
BUCKET_SECRET_ID=replace-me
BUCKET_SECRET_KEY=replace-me
BUCKET_ENDPOINT=https://example.r2.cloudflarestorage.com
CDN_BASE_URL=https://static.example.com
```

### 2. Run with Docker Hub image

```bash
docker pull delbertbeta/sso-rs:main

docker run -d \
  --name sso-rs \
  --restart unless-stopped \
  --env-file .env \
  -p 3000:3000 \
  -p 2999:2999 \
  delbertbeta/sso-rs:main
```

### 3. Build and run locally

```bash
cargo build --release
./target/release/sso-rs
```

### 4. Build Docker image locally

```bash
docker build -t sso-rs:local .

docker run -d \
  --name sso-rs \
  --restart unless-stopped \
  --env-file .env \
  -p 3000:3000 \
  -p 2999:2999 \
  sso-rs:local
```

### Notes

- `FRONT_END_URL` must be a valid public URL. The service derives the cookie root domain from it.
- `PROD` is treated as a boolean flag by presence. Any non-empty value enables production mode.
- `BUCKET_ENDPOINT` should be the R2 API endpoint root only, without the bucket name. The server uses path-style presigned URLs like `/BUCKET_NAME/object-key`.
- `BUCKET_*` and `CDN_BASE_URL` are required because image upload paths depend on them.
- The container does not start MySQL or Redis for you. Point the `.env` values at external services.
- The Docker image now builds the Rust binary inside the container, so it no longer depends on the host machine's glibc version.

#### Development code snippets

- Generate entity

```
sea-orm-cli generate entity -l -o ./entity/src --with-serde both
```
