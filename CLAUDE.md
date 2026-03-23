# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Common Commands

- **Build the project**:
  ```bash
  cargo build          # Debug build
  cargo build --release # Release build (for production)
  ```
- **Run the application**:
  ```bash
  cargo run            # Runs both HTTP (port 3000) and gRPC (port 2999) servers
  ```
- **Run tests**:
  ```bash
  cargo test           # Run all tests
  cargo test --package qcloud  # Run tests for specific package
  ```
- **Database operations**:
  ```bash
  # Run migrations (requires sea-orm-cli: `cargo install sea-orm-cli`)
  sea-orm-cli migrate up

  # Generate database entities from schema
  sea-orm-cli generate entity -l -o ./entity/src --with-serde both

  # Alternative migration runner (built-in)
  cargo run --bin migration  # From migration directory
  ```
- **Docker operations**:
  ```bash
  docker build -t sso-rs .  # Build Docker image
  # The Dockerfile expects the binary at ./target/release/sso-rs
  ```

## Required Environment Variables

Create a `.env` file in the root directory with:
```env
DATABASE_URL=mysql://user:password@localhost/database_name
REDIS_URL=redis://localhost:6379
RUST_LOG=info  # or debug for development
```

Additional environment variables for production:
- `FRONTEND_URL`: Frontend application URL for CORS
- `CDN_URL`: CDN endpoint for static assets

## Architecture Overview

This is a production-ready SSO (Single Sign-On) provider implementing OpenID Connect 1.0, built as a Cargo workspace in Rust.

### Workspace Structure
- **`sso-rs` (root crate)**: Main application with Axum web server (port 3000) and Volo gRPC server (port 2999)
- **`entity`**: SeaORM database models with serde serialization
- **`migration`**: Database migrations managed by sea-orm-cli
- **`volo-gen`**: Auto-generated gRPC code from protobuf definitions
- **`qcloud`**: Tencent Cloud integration module (optional)

### Core Technologies
- **Web Framework**: Axum with async/await, tower-http for CORS and tracing
- **Database**: MySQL with SeaORM and SQLx driver
- **Session Store**: Redis with async-session
- **Authentication**: PBKDF2 password hashing, RSA key pairs for JWT signing
- **File Storage**: S3-compatible storage (AWS S3 or Cloudflare R2)
- **gRPC**: Volo framework for internal microservice communication
- **OIDC**: Full OpenID Connect implementation with authorization code flow

### Application Structure
```
src/
├── extractor/     # Request extractors for authentication
├── model/        # Business logic and data structures
├── route/
│   ├── api/      # REST API endpoints (auth, user, oidc, application, etc.)
│   └── mod.rs    # Route definitions and middleware
├── rpc/          # gRPC service implementations
├── storage/      # Database, session, and S3 abstractions
└── util.rs       # Utility functions
```

### API Endpoints (see api.md for full documentation)
- **Authentication**: `/api/auth/{register,login,logout}`
- **User Management**: `/api/user` (GET, PATCH)
- **OIDC Provider**: `/api/oidc/authorize`, token endpoints, discovery
- **Applications**: CRUD operations with secrets management
- **Crypto**: RSA key endpoint (`/api/crypto/rsa`)
- **Images**: Upload and update with S3 storage

### Database Schema
Key entities:
- `users` - User accounts with PBKDF2 hashed passwords
- `applications` - OAuth2/OIDC client applications
- `secrets` - Application secrets with rotation support
- `images` - User profile images stored in S3
- `sessions` - Redis-based session management
- OIDC tables for authorization codes, tokens, and consents

### Development Workflow
1. Set up MySQL and Redis instances
2. Create `.env` file with connection strings
3. Run database migrations: `sea-orm-cli migrate up`
4. Start development: `cargo run`
5. Access API at http://localhost:3000, gRPC at localhost:2999

### Docker Deployment
- Multi-stage Dockerfile uses Ubuntu 24.04 base
- Exposes both HTTP (3000) and gRPC (2999) ports
- CI/CD via GitHub Actions builds and publishes to Docker Hub
- Binary expects release build with `cargo build --release`
