---
phase: 01-framework-foundation
plan: 02
subsystem: api
tags: [axum, tower, middleware, auth, bearer-token, integration-tests, rust]

# Dependency graph
requires:
  - phase: 01-01
    provides: Axum app() builder, AppState with Arc<AppConfig>, Error enum with IntoResponse, /health route
provides:
  - Bearer token presence-check middleware (require_auth) returning 401 JSON on missing/empty/wrong scheme
  - Protected router split using route_layer (unknown paths get 404 not 401)
  - GET /api/ping protected test endpoint returning "pong"
  - Integration tests covering all auth middleware scenarios using tower::ServiceExt::oneshot
  - AppConfig/ServerConfig/CorsConfig Default impls for test construction without figment
affects:
  - 01-framework-foundation
  - 02-database-auth (Phase 3 will replace presence check with real JWT validation)
  - All subsequent phases using the public/protected router pattern

# Tech tracking
tech-stack:
  added:
    - http-body-util 0.1 (dev-dependency for reading response bodies in tests)
    - tower util feature enabled for ServiceExt::oneshot in integration tests
  patterns:
    - "Protected/public router split: separate Router per access level, merged into app()"
    - "route_layer (not layer) on protected router: middleware only runs when route matches"
    - "Integration tests via tower::ServiceExt::oneshot: no real server needed"
    - "Default impls on config structs for test AppState construction"

key-files:
  created:
    - backend/src/middleware/mod.rs
    - backend/src/middleware/auth.rs
    - backend/src/routes/ping.rs
    - backend/tests/health.rs
    - backend/tests/auth.rs
  modified:
    - backend/src/lib.rs
    - backend/src/routes/mod.rs
    - backend/src/config.rs
    - backend/Cargo.toml

key-decisions:
  - "Use route_layer (not layer) on protected router so unknown paths return 404 not 401"
  - "Phase 1 auth is presence-only check; JWT validation deferred to Phase 3"
  - "Default impls on AppConfig/ServerConfig/CorsConfig enable test state without figment overhead"

patterns-established:
  - "Pattern: protected router uses route_layer with middleware::from_fn — keeps 404 semantics for unknown paths"
  - "Pattern: integration tests use tower::ServiceExt::oneshot with app(test_state()) — no server binding"
  - "Pattern: require_auth returns Err(Error::Unauthorized) — produces {code, message} JSON via IntoResponse"

# Metrics
duration: 3min
completed: 2026-02-15
---

# Phase 1 Plan 2: Auth Middleware and Router Split Summary

**Bearer token presence-check middleware with protected/public router split using route_layer; 6 integration tests via tower::ServiceExt::oneshot confirm auth behavior without starting a real server**

## Performance

- **Duration:** 3 min
- **Started:** 2026-02-15T19:16:29Z
- **Completed:** 2026-02-15T19:19:38Z
- **Tasks:** 2
- **Files modified:** 9

## Accomplishments

- Auth middleware rejects all requests without valid Bearer token, returning `{"code":"UNAUTHORIZED","message":"Unauthorized"}` JSON — consistent with app error format
- Protected and public routers merged cleanly: `/health` serves without auth, `/api/ping` requires Bearer, unknown paths get 404 (not 401) proving `route_layer` correctness
- 6 integration tests (5 auth scenarios + 1 health baseline) run in-process via `tower::ServiceExt::oneshot` — no port binding, no flakiness

## Task Commits

Each task was committed atomically:

1. **Task 1: Create auth middleware and wire protected/public router split** - `136a23b` (chore — included in prior justfile commit)
2. **Task 2: Add integration tests for auth middleware and health endpoint** - `76b819a` (feat)

**Plan metadata:** pending (docs commit)

## Files Created/Modified

- `backend/src/middleware/mod.rs` - Module declaration: `pub mod auth;`
- `backend/src/middleware/auth.rs` - `require_auth` async middleware: extracts Authorization header, checks Bearer prefix, rejects empty token, TODO(Phase 3) JWT validation comment
- `backend/src/routes/ping.rs` - Protected test handler: `pub async fn ping() -> &'static str { "pong" }`
- `backend/src/routes/mod.rs` - Added `pub mod ping;` alongside health
- `backend/src/lib.rs` - Added `mod middleware;`, split app() into public/protected routers using `route_layer`
- `backend/src/config.rs` - Added `Default` impls for `AppConfig`, `ServerConfig`, `CorsConfig` (derives where possible)
- `backend/Cargo.toml` - Added `http-body-util 0.1` to dev-deps; enabled `tower = { features = ["util"] }`
- `backend/tests/health.rs` - Integration test: GET /health returns 200 without auth token
- `backend/tests/auth.rs` - 5 integration tests covering: no auth→401+UNAUTHORIZED body, valid Bearer→200+"pong", empty Bearer→401, Basic scheme→401, /health without auth→200

## Decisions Made

- Used `route_layer` (not `layer`) on the protected router: `route_layer` only executes middleware when a matching route exists, so `GET /api/unknown` returns 404 not 401 — correct REST semantics
- Phase 1 auth is presence-only: checks that a non-empty Bearer token exists, no cryptographic validation. Phase 3 will replace the body of `require_auth` with JWT claim extraction
- Added `Default` impls for all config structs so integration tests can build `AppState` without loading config.toml or environment variables

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Fixed CorsConfig manual Default impl to derived Default**
- **Found during:** Task 2 (after running clippy during test setup)
- **Issue:** Manual `impl Default for CorsConfig` was identical to what `#[derive(Default)]` would produce — clippy flagged `derivable_impls` warning
- **Fix:** Replaced manual impl with `#[derive(Default)]` on the struct
- **Files modified:** backend/src/config.rs
- **Verification:** `just check` passes with no warnings from our code
- **Committed in:** 76b819a (Task 2 commit)

---

**Total deviations:** 1 auto-fixed (1 clippy quality fix)
**Impact on plan:** Minor code quality fix. No scope creep.

## Issues Encountered

Task 1 files (`middleware/auth.rs`, `middleware/mod.rs`, `routes/ping.rs`, `lib.rs` router split) were already included in commit `136a23b chore(01-03)` from a prior agent run that also fixed clippy/fmt issues across those files. The task 1 implementation was correct and all verification curl tests passed, so no re-commit was needed — Task 2 (integration tests) proceeded directly.

## User Setup Required

None - no external service configuration required. All tests run in-process.

## Next Phase Readiness

- Protected/public router pattern established: Phase 2 (database/auth) adds more protected routes by extending the protected Router in app()
- `require_auth` is a stub ready for Phase 3 JWT validation: replace the presence check body with claim extraction and store user ID in request extensions
- Integration test pattern established: new phases add their own `backend/tests/{feature}.rs` using the same `test_state()` + `oneshot` pattern

---
*Phase: 01-framework-foundation*
*Completed: 2026-02-15*

## Self-Check: PASSED

All files exist: backend/src/middleware/mod.rs, backend/src/middleware/auth.rs, backend/src/routes/ping.rs, backend/tests/health.rs, backend/tests/auth.rs, .planning/phases/01-framework-foundation/01-02-SUMMARY.md
All commits exist: 136a23b (middleware+router split), 76b819a (integration tests)
