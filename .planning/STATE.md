# Project State

## Project Reference

See: .planning/PROJECT.md (updated 2026-02-13)

**Core value:** Employees can accurately record and retrieve time entries with performed vs billed hour tracking for client billing
**Current focus:** Phase 1 - Framework Foundation

## Current Position

Phase: 1 of 9 (Framework Foundation)
Plan: 3 of 3 in current phase
Status: Executing
Last activity: 2026-02-15 — Plan 01-03 complete: justfile build recipe, alias b, clippy clean, fmt passing, /health=200

Progress: [██░░░░░░░░] 11%

## Performance Metrics

**Velocity:**
- Total plans completed: 3
- Average duration: 3 min
- Total execution time: 0.05 hours

**By Phase:**

| Phase | Plans | Total | Avg/Plan |
|-------|-------|-------|----------|
| 01-framework-foundation | 3 | 9 min | 3 min |

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

### Pending Todos

None yet.

### Blockers/Concerns

- [Research]: deadpool-diesel exact API (interact() closure shape) is MEDIUM confidence — verify against docs.rs before Phase 2
- [Research]: TanStack Router v1.x migration from old object-based API needs design before Phase 3 frontend work
- [Research]: MariaDB target version unknown — affects whether RETURNING clause is available (affects Phase 7-8 migration design)
- [Research]: tower-governor rate limiting crate maintenance status unverified — decide before Phase 3

## Session Continuity

Last session: 2026-02-15
Stopped at: Completed 01-02-PLAN.md — Bearer auth middleware, protected/public router split, 6 integration tests all passing
Resume file: None
