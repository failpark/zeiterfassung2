---
phase: 01-framework-foundation
plan: 01
subsystem: api
tags: [axum, tower-http, figment, tracing, cors, rust]

# Dependency graph
requires: []
provides:
  - Axum 0.8 HTTP server compiling and serving requests
  - AppConfig loaded from config.toml with APP__ env var overrides via figment
  - AppState with Arc<AppConfig> shared across handlers
  - Error type with IntoResponse producing {code, message, details} JSON
  - Structured tracing: pretty dev / JSON prod with per-request spans
  - CORS layer: mirror_request in dev, allowlist in prod, credentials enabled
  - SetRequestIdLayer generating UUID x-request-id per request
  - GET /health returning 200 OK
affects:
  - 01-framework-foundation
  - 02-database-auth
  - All subsequent phases building on app() function

# Tech tracking
tech-stack:
  added:
    - axum 0.8.8 (HTTP routing, handlers, IntoResponse)
    - tower-http 0.6.8 (CorsLayer, TraceLayer, SetRequestIdLayer, PropagateRequestIdLayer)
    - tower 0.5 (ServiceBuilder middleware composition)
    - tokio 1.49 with full features (async runtime)
    - figment 0.10.19 (TOML + env var config merging)
    - dotenvy 0.15.7 (.env loading in dev)
    - serde_json 1 (JSON serialization)
    - uuid 1 with v4 feature (request ID generation)
  patterns:
    - AppState with Arc<Config> shared across handlers via Router::with_state
    - ServiceBuilder middleware stack: SetRequestId -> TraceLayer -> CORS -> PropagateRequestId
    - Error enum implementing IntoResponse with {code, message, details} JSON shape
    - Config via figment: config.toml base merged with APP__-prefixed env var overrides
    - CORS dev: AllowOrigin::mirror_request() (not Any — compatible with credentials)
    - Module pub mod tracing shadows crate tracing; use ::tracing:: for crate-qualified access

key-files:
  created:
    - backend/src/config.rs
    - backend/src/state.rs
    - backend/src/routes/health.rs
    - config.toml
  modified:
    - backend/Cargo.toml
    - backend/src/lib.rs
    - backend/src/tracing.rs
    - backend/src/error.rs
    - backend/src/routes/mod.rs
    - backend/src/bin/backend.rs
    - .env.sample

key-decisions:
  - "Use AllowOrigin::mirror_request() not Any in dev — wildcard + credentials is browser-spec invalid"
  - "Module named 'tracing' shadows crate; use ::tracing:: prefix for crate-level macros in lib.rs"
  - "Secrets (DATABASE_URL, JWT_SECRET) are env-only, never in config.toml"
  - "Old Rocket route files remain on disk but commented out of routes/mod.rs for Phase 3+ porting"

patterns-established:
  - "Pattern: CORS dev uses mirror_request() — any origin allowed without wildcard restriction"
  - "Pattern: error.rs IntoResponse — 500 errors log actual error, return generic message to client"
  - "Pattern: config.toml committed to git with non-secret defaults; secrets via env vars only"
  - "Pattern: app() takes AppState by value, returns Router — all phases extend this function"

# Metrics
duration: 2min
completed: 2026-02-15
---

# Phase 1 Plan 1: Framework Foundation Summary

**Axum 0.8 server with figment config, structured tracing, CORS mirror-request dev policy, UUID request-id middleware, and {code, message, details} error JSON replacing Rocket 0.5**

## Performance

- **Duration:** 2 min
- **Started:** 2026-02-15T19:11:03Z
- **Completed:** 2026-02-15T19:13:56Z
- **Tasks:** 2
- **Files modified:** 11

## Accomplishments

- Replaced all Rocket/rocket_cors/rocket_db_pools dependencies with Axum 0.8 + tower-http 0.6 + figment 0.10
- Server compiles, starts on port 8000, responds to `/health` with 200, logs structured per-request traces with method/uri/request_id/latency/status
- CORS correctly configured: `AllowOrigin::mirror_request()` in dev (echoes origin, compatible with credentials), allowlist from config in production

## Task Commits

Each task was committed atomically:

1. **Task 1: Replace Cargo dependencies, create config, state, tracing, and error modules** - `b88463b` (feat)
2. **Task 2: Wire lib.rs app builder, middleware stack, health route, and main entry point** - `4ebf818` (feat)

## Files Created/Modified

- `backend/Cargo.toml` - Replaced rocket/rocket_cors/rocket_db_pools with axum, tower-http, figment, dotenvy, uuid, serde_json, tokio
- `backend/src/config.rs` - AppConfig with ServerConfig + CorsConfig, load_config() via figment TOML+env merge, is_development() method
- `backend/src/state.rs` - AppState with pub config: Arc<AppConfig>, Clone derive
- `backend/src/tracing.rs` - init_tracing(is_production: bool): pretty fmt in dev, JSON in prod, EnvFilter, no file appender
- `backend/src/error.rs` - Error enum with IntoResponse: {code, message, details} JSON; Validation(Vec<FieldError>) added; 500 errors log then return generic message
- `backend/src/lib.rs` - pub mod declarations; app(state: AppState) -> Router with cors_layer() helper and full middleware stack
- `backend/src/routes/mod.rs` - Only pub mod health; old route modules commented out with TODO
- `backend/src/routes/health.rs` - pub async fn health() -> StatusCode::OK
- `backend/src/bin/backend.rs` - tokio::main; load_config, init_tracing, AppState, TcpListener, axum::serve
- `config.toml` - Default server/cors/environment/log_level config (non-secret, git-committed)
- `.env.sample` - APP__ env var override examples + secrets documentation

## Decisions Made

- Used `AllowOrigin::mirror_request()` not `Any` for dev CORS — wildcard + `allow_credentials(true)` is browser-spec incompatible (CORS credentials error in browsers)
- Named module `pub mod tracing` shadows the `tracing` crate in lib.rs scope; resolved by using `::tracing::` for crate-qualified macro access
- Secrets (DATABASE_URL, JWT_SECRET) are env-only, never in config.toml
- Old Rocket route files (activity.rs, client.rs, login.rs, project.rs, tracking.rs, user.rs) remain on disk but commented out of mod.rs — Phase 3+ will port them to Axum

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Fixed `tracing` crate name collision in lib.rs**
- **Found during:** Task 2 (lib.rs rewrite)
- **Issue:** `pub mod tracing` declaration shadows the external `tracing` crate, causing `use tracing::Level` and `tracing::info_span!` to resolve to our module instead of the crate
- **Fix:** Removed `use tracing::Level` import; used `::tracing::info_span!` and `::tracing::Level::INFO` with crate-qualified paths
- **Files modified:** backend/src/lib.rs
- **Verification:** `cargo build` succeeds with no errors
- **Committed in:** 4ebf818 (Task 2 commit)

---

**Total deviations:** 1 auto-fixed (1 bug — name collision)
**Impact on plan:** Auto-fix required for correctness. No scope creep.

## Issues Encountered

None beyond the auto-fixed name collision above.

## User Setup Required

None - no external service configuration required for this plan. DATABASE_URL and JWT_SECRET will be needed in Phase 2.

## Next Phase Readiness

- Axum foundation complete: app() builder in lib.rs is ready to receive new routes in each subsequent phase
- AppState is ready to receive db pool (Phase 2) and tokenizer (Phase 2)
- Error type covers all variants from original Rocket code plus new Validation variant
- Config pattern established: extend AppConfig struct in config.rs for new config sections

---
*Phase: 01-framework-foundation*
*Completed: 2026-02-15*

## Self-Check: PASSED

All files exist: backend/Cargo.toml, backend/src/config.rs, backend/src/state.rs, backend/src/tracing.rs, backend/src/error.rs, backend/src/lib.rs, backend/src/routes/mod.rs, backend/src/routes/health.rs, backend/src/bin/backend.rs, config.toml, .env.sample, 01-01-SUMMARY.md
All commits exist: b88463b, 4ebf818
