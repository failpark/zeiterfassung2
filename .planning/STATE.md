# Project State

## Project Reference

See: .planning/PROJECT.md (updated 2026-02-13)

**Core value:** Employees can accurately record and retrieve time entries with performed vs billed hour tracking for client billing
**Current focus:** Phase 2 - Database Layer

## Current Position

Phase: 2 of 9 (Database Layer)
Plan: 3 of 3 in current phase
Status: Executing
Last activity: 2026-02-19 — Plan 02-02 complete: all db/ modules ported from rocket_db_pools to sync &mut MysqlConnection; cargo build and clippy clean

Progress: [████░░░░░░] 20%

## Performance Metrics

**Velocity:**
- Total plans completed: 5
- Average duration: 3 min
- Total execution time: 0.08 hours

**By Phase:**

| Phase | Plans | Total | Avg/Plan |
|-------|-------|-------|----------|
| 01-framework-foundation | 3 | 9 min | 3 min |
| 02-database-layer | 3 | 10 min | 3.3 min |

**Recent Trend:**
- Last 5 plans: 3 min
- Trend: —

*Updated after each plan completion*

## Accumulated Context

### Decisions

Decisions are logged in PROJECT.md Key Decisions table.
Recent decisions affecting current work:

- [Init]: Axum over Rocket — more active development, tighter Tokio/Diesel async integration
- [Init]: Keep Diesel ORM — existing async support, schema compatibility preserved
- [Init]: Evolve schema (not fresh start) — existing MariaDB data must be preserved
- [Init]: Sub-projects and travel routes included in v1 — features still needed by team
- [Init]: Reporting and PDF export deferred to v2
- [01-01]: AllowOrigin::mirror_request() not Any in dev — wildcard + credentials is browser-spec invalid
- [01-01]: Module named 'tracing' shadows crate; use ::tracing:: prefix for crate-qualified access in lib.rs
- [01-01]: Secrets (DATABASE_URL, JWT_SECRET) env-only, never in config.toml
- [01-01]: Old Rocket route files commented out of mod.rs, remain on disk for Phase 3+ porting
- [01-02]: Use route_layer (not layer) on protected router so unknown paths return 404 not 401
- [01-02]: Phase 1 auth is presence-only check; JWT validation deferred to Phase 3 require_auth replacement
- [01-02]: Default impls on AppConfig/ServerConfig/CorsConfig enable test state without figment overhead
- [01-03]: Added build recipe + alias b := build (was absent from justfile)
- [01-03]: Suppressed result_large_err on load_config() — figment::Error size is not controllable (third-party type)
- [01-03]: Applied nightly fmt to fix formatting diffs in backend.rs and tracing.rs
- [02-01]: deadpool_diesel::Runtime::Tokio1 — matches tokio runtime used by axum
- [02-01]: Pool max_size=10 hardcoded default; not yet exposed through AppConfig
- [02-01]: DATABASE_URL read via std::env::var directly (not figment) — locked decision, secrets env-only
- [02-01]: redact_password() strips between last ':' and '@' to handle mysql://user:pass@host patterns
- [Phase 02-03]: Self-contained migration block in binary using embed_migrations — avoids wave-2 cross-plan dependency on db::run_migrations
- [Phase 02-03]: Lazy pool in test_state() uses placeholder URL — deadpool never connects at build time, so tests not exercising DB routes work without live DB
- [02-02]: sync transactions use db.transaction(|conn| { ... }) — no Box::pin, no async move required for Diesel sync connections
- [02-02]: All db/ functions now &mut MysqlConnection — ready for deadpool-diesel interact() closures in Phase 3+ route handlers
- [02-02]: ::tracing:: qualified path used in run_migrations — mod tracing in lib.rs shadows crate name

### Pending Todos

None yet.

### Blockers/Concerns

- [Research]: TanStack Router v1.x migration from old object-based API needs design before Phase 3 frontend work
- [Research]: MariaDB target version unknown — affects whether RETURNING clause is available (affects Phase 7-8 migration design)
- [Research]: tower-governor rate limiting crate maintenance status unverified — decide before Phase 3

## Session Continuity

Last session: 2026-02-19
Stopped at: Completed 02-02-PLAN.md — all db/ modules ported to sync MysqlConnection, rocket_db_pools removed, cargo build and clippy clean
Resume file: None
