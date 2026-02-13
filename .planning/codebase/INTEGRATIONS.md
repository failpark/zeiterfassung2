# External Integrations

**Analysis Date:** 2026-02-13

## APIs & External Services

**None Detected** - This is a self-contained application without third-party API integrations (Stripe, Slack, SendGrid, etc.)

## Data Storage

**Databases:**
- MySQL 5.7+
  - Connection: Configured in `Rocket.toml` via `databases.zeiterfassung.url` entry
  - Environment: `DATABASE_URL` env var (from `.env.sample`)
  - Client: Diesel 2.1.4 ORM with `rocket_db_pools` for pooling
  - Pool Type: `MysqlPool` via `rocket_db_pools::diesel::MysqlPool`
  - Definition: `backend/src/db/mod.rs` defines `DB` struct using `#[database("zeiterfassung")]` macro

**Migrations:**
- Diesel migrations system
  - Location: `migrations/` directory with numbered, timestamped migrations
  - Execution: Automatic on application startup via `backend/src/db/mod.rs::run_migrations`
  - Format: SQL files (up.sql/down.sql pairs)
  - Current migrations:
    - User and authentication schema
    - Activity tracking schema
    - Client and project management
    - Tracking-to-activity relationships

**File Storage:**
- Local filesystem only - No cloud storage integration (S3, Firebase Storage, etc.)

**Caching:**
- None detected - No Redis, Memcached, or in-memory caching layers

## Authentication & Identity

**Auth Provider:**
- Custom JWT-based implementation
  - Location: `backend/src/auth/` (tokenizer module)
  - Token generation: JWT-Simple 0.11.9 library
  - Token validation: `backend/src/guard.rs` for Rocket request guards
  - Token expiration: 5 days (set in `backend/src/lib.rs` line 46-48)
  - Storage: Client-side in localStorage with key `auth_token`

**Password Management:**
- Argon2 0.5.2 for password hashing
- Location: `backend/src/routes/login.rs` handles user authentication
- No OAuth/OIDC providers - Manual email/password login only

**Authorization:**
- User roles: Role-based access control via `User` struct in `backend/src/db/user.rs`
- JWT payload: Embedded user claims in token, decoded client-side via `jwt-decode`
- Frontend auth context: `frontend/src/contexts/AuthContext.tsx` manages login state

## Monitoring & Observability

**Error Tracking:**
- None detected - No Sentry, Datadog, or error tracking service

**Logging:**
- Structured logging via Tracing crate
  - Framework: `tracing`, `tracing-appender`, `tracing-subscriber`
  - Output locations:
    - Console: Pretty-formatted logs from zeiterfassung crate
    - File: `/var/log/zeiterfassung/server.log` with daily rolling files
  - Format: JSON structured logs to file, human-readable to stdout
  - Level: Configurable via `RUST_LOG` environment variable (tracing-subscriber::EnvFilter)
  - Implementation: `backend/src/tracing.rs` initializes subscriber

**Frontend Logging:**
- Console-only via standard JavaScript console methods
- No structured logging or APM integration

## CI/CD & Deployment

**Hosting:**
- Deployment target: Linux x86_64 (configured in `Cargo.toml` cargo-dist)
- Runtime: Self-hosted Rocket server (no serverless/container frameworks)

**CI Pipeline:**
- GitHub Actions support configured (ci = ["github"] in `Cargo.toml`)
- Cargo-dist version 0.5.0 for binary distribution
- Installer generation: Shell script installer available

**Build Artifacts:**
- Backend: Compiled Rust binary (executable name: `backend`)
- Frontend: Static HTML/CSS/JS bundle to `frontend/dist/`

## Environment Configuration

**Backend Environment Variables:**
- `DATABASE_URL` - MySQL connection string (required)
- `RUST_LOG` - Tracing filter level (optional, defaults to TRACE)
- Configuration also via `Rocket.toml`:
  - `secret_key` - JWT signing key
  - `databases.zeiterfassung.url` - Database connection string

**Frontend Environment Variables:**
- `VITE_API_URL` - Backend API endpoint
  - Development: `http://localhost:8000`
  - Production: `/api` (same-origin proxy)

**Secrets Location:**
- Backend: `Rocket.toml` for local development (note: not committed in production)
- `.env` files in frontend for Vite configuration
- Note: Neither frontend nor backend loads from .env files in standard way; Vite uses `VITE_*` prefix for public vars

## Webhooks & Callbacks

**Incoming:**
- None detected - API accepts only standard REST requests

**Outgoing:**
- None detected - No outbound webhook calls to external services

## CORS & Origin Configuration

**Frontend to Backend:**
- Allowed origins: `http://localhost:5173` (hardcoded in `backend/src/lib.rs`)
- Framework: Rocket CORS 0.6.0 with `CorsOptions`
- For production: CORS origin must be updated via code change to match production frontend URL

**Backend HTTP Client:**
- Axios configured in `frontend/src/api/axiosClient.ts`
  - Base URL: Configurable via `VITE_API_URL` environment variable
  - Request interceptor: Adds `Authorization: Bearer {token}` header with JWT from localStorage
  - Response interceptor: Handles 401 errors by clearing token and redirecting to `/login`
  - Default header: `Content-Type: application/json`

## Internal Data APIs

**REST Endpoints (Backend Routes):**
- Location: `backend/src/routes/` directory
- Endpoints organized by domain:
  - `login.rs` - Authentication (POST /login)
  - `user.rs` - User management (CRUD operations)
  - `activity.rs` - Activity tracking
  - `project.rs` - Project management
  - `client.rs` - Client/company management
  - `tracking.rs` - Time tracking entries
- Response format: JSON with custom error handling in `backend/src/error.rs`
- Error responses: Custom JSON structure via `Error` type implementing Rocket's `Responder`

**Frontend API Layer:**
- Location: `frontend/src/api/` directory
- API client wrappers:
  - `axiosClient.ts` - Core HTTP client with interceptors
  - `authApi.ts` - Authentication endpoints
  - `userApi.ts` - User management
  - `activityApi.ts` - Activity data
  - `projectApi.ts` - Project management
  - `clientApi.ts` - Client/company data
  - `trackingApi.ts` - Tracking entries
- Shared types: TypeScript interfaces in `frontend/src/types/`

---

*Integration audit: 2026-02-13*
