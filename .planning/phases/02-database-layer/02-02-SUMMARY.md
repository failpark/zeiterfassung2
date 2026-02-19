---
phase: 02-database-layer
plan: 02
subsystem: database
tags: [diesel, mysql, sync, MysqlConnection, rocket-migration, db-layer]

# Dependency graph
requires:
  - phase: 02-database-layer
    plan: 01
    provides: deadpool-diesel pool, DbPool type, AppState.db_pool, diesel dependency already present
provides:
  - All db/ modules compile with sync &mut MysqlConnection signatures
  - schema.rs and db/ re-enabled in lib.rs
  - Synchronous CRUD for: user, client, activity, project, tracking, tracking_to_activity
  - run_migrations() sync function accepting &mut MysqlConnection
  - Tracking middle-layer (middlelayer.rs) with sync orchestration and paginate via grouped_by
affects: [02-03-routes, 03-route-handlers, all subsequent api handler plans]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - sync-diesel-crud: All db/ functions take &mut MysqlConnection (no async, no .await)
    - sync-transaction: db.transaction(|conn| { ... }) without Box::pin(async move)
    - interact-ready: All db/ functions can be called directly inside deadpool-diesel interact() closures

key-files:
  created: []
  modified:
    - backend/src/lib.rs
    - backend/src/db/mod.rs
    - backend/src/db/user.rs
    - backend/src/db/client.rs
    - backend/src/db/activity.rs
    - backend/src/db/project.rs
    - backend/src/db/helper.rs
    - backend/src/db/tracking/tracking.rs
    - backend/src/db/tracking/tracking_to_activity.rs
    - backend/src/db/tracking/middlelayer.rs

key-decisions:
  - "sync transactions use db.transaction(|conn| { ... }) — no Box::pin, no async move"
  - "helper.rs last_page<T> uses T: Table + SelectDsl<CountStar> + LoadQuery bound pattern"
  - "middlelayer.rs paginate uses diesel BelongingToDsl::belonging_to() + grouped_by() unchanged from async version"
  - "::tracing:: qualified path used in run_migrations because mod tracing in lib.rs shadows crate name"

patterns-established:
  - "Sync db call pattern: Entity::method(db, ...) where db: &mut MysqlConnection"
  - "Sync transaction: db.transaction(|conn| { insert_into(t).values(v).execute(conn)?; t.filter(...).first(conn) })"
  - "Interact usage (Phase 3+): pool.get().await?.interact(|conn| Entity::method(conn, ...)).await??"

# Metrics
duration: 5min
completed: 2026-02-19
---

# Phase 02 Plan 02: DB Module Port to Sync MysqlConnection Summary

**All 10 db/ source files ported from rocket_db_pools async Connection<DB> to synchronous &mut diesel::MysqlConnection; zero rocket_db_pools references remain; cargo build and clippy clean**

## Performance

- **Duration:** 5 min
- **Started:** 2026-02-19T16:02:42Z
- **Completed:** 2026-02-19T16:07:42Z
- **Tasks:** 2
- **Files modified:** 10

## Accomplishments

- Re-enabled `pub mod db` and `pub mod schema` in `lib.rs` (were commented out since Phase 1 Axum migration)
- Rewrote `db/mod.rs`: removed Rocket DB struct, replaced with `run_migrations(conn: &mut MysqlConnection)` using `diesel_migrations::embed_migrations!`
- Ported 5 simple entity modules (user, client, activity, project, helper): replaced `rocket_db_pools` imports with `diesel::prelude::*` + `diesel::MysqlConnection`, removed all `async`/`.await`, converted `Box::pin(async move { ... })` transactions to sync closures
- Ported 3 tracking submodules (tracking.rs, tracking_to_activity.rs, middlelayer.rs): same mechanical transformation plus preserved `belonging_to()` + `grouped_by()` association query in `paginate()`

## Task Commits

Each task was committed atomically:

1. **Task 1: Re-enable db and schema modules; port db/mod.rs and simple entity modules** - `6cc0627` (feat)
2. **Task 2: Port tracking module (tracking.rs, middlelayer.rs, tracking_to_activity.rs)** - `c6a4695` (feat)

**Plan metadata:** `(pending docs commit)` (docs: complete plan)

## Files Created/Modified

- `backend/src/lib.rs` - Uncommented `pub mod db` and `pub mod schema`
- `backend/src/db/mod.rs` - Removed DB struct + Rocket imports; added sync `run_migrations`; enabled all submodules
- `backend/src/db/user.rs` - Removed rocket_db_pools; all methods sync with `&mut MysqlConnection`
- `backend/src/db/client.rs` - Removed rocket_db_pools; all methods sync with `&mut MysqlConnection`
- `backend/src/db/activity.rs` - Removed rocket_db_pools; all methods sync with `&mut MysqlConnection`
- `backend/src/db/project.rs` - Removed rocket_db_pools; all methods sync with `&mut MysqlConnection`
- `backend/src/db/helper.rs` - Removed rocket_db_pools; sync `last_page` with proper LoadQuery bound
- `backend/src/db/tracking/tracking.rs` - Removed rocket_db_pools; all methods sync
- `backend/src/db/tracking/tracking_to_activity.rs` - Removed rocket_db_pools; all methods sync
- `backend/src/db/tracking/middlelayer.rs` - Removed rocket_db_pools; sync orchestration; preserved Diesel association query

## Decisions Made

- Sync transactions use `db.transaction(|conn| { ... })` without `Box::pin` — Diesel's sync transaction closure takes `&mut MysqlConnection` directly
- `helper.rs` `last_page<T>` generic bound uses `T: Table + SelectDsl<CountStar>` with `LoadQuery<'static, MysqlConnection, i64>` on the output type
- `middlelayer.rs` `paginate()` inline `use rocket_db_pools::diesel::prelude::*;` replaced by top-level `use diesel::prelude::*;` — no behavior change since `BelongingToDsl` and `grouped_by()` are in `diesel::prelude`
- `::tracing::` fully qualified prefix retained in `run_migrations` because `mod tracing` in `lib.rs` shadows the crate name

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Fixed helper.rs LoadQuery import**
- **Found during:** Task 1 (helper.rs port)
- **Issue:** `LoadQuery` trait not in scope — not re-exported by `diesel::prelude::*`
- **Fix:** Added explicit `use diesel::query_dsl::LoadQuery;` import
- **Files modified:** `backend/src/db/helper.rs`
- **Verification:** `cargo check` passed after fix
- **Committed in:** `6cc0627` (Task 1 commit)

**2. [Rule 1 - Bug] Fixed clippy multiple_bound_locations warning in helper.rs**
- **Found during:** Task 1 verification (clippy)
- **Issue:** `T: Table` in both generic parameter position and where clause triggered `multiple_bound_locations` warning
- **Fix:** Moved `Table` bound to where clause: `T: Table + SelectDsl<CountStar>`
- **Files modified:** `backend/src/db/helper.rs`
- **Verification:** `cargo clippy` clean after fix
- **Committed in:** `6cc0627` (Task 1 commit)

---

**Total deviations:** 2 auto-fixed (both Rule 1 - Bug)
**Impact on plan:** Both were minor compile/lint issues in the generic helper function. No scope creep.

## Issues Encountered

A pre-commit linter/hook was commenting out not-yet-compiled modules from `db/mod.rs` during each commit. This was handled by porting all modules (Task 1 + Task 2) before attempting any partial commit, then staging and committing with all modules present and compiling.

## User Setup Required

None - no external service configuration required. All changes are compile-time transformations.

## Next Phase Readiness

- All db/ CRUD functions ready to be called from within `pool.get().await?.interact(|conn| { ... }).await??` closures in Phase 3+ route handlers
- `run_migrations(conn)` ready to be called at startup from `main()` via the pool
- Zero rocket_db_pools dependencies remain; project compiles cleanly with just diesel + deadpool-diesel
- `cargo build` and `cargo clippy` both pass with zero errors and zero warnings (excluding third-party future-incompat note)

---
*Phase: 02-database-layer*
*Completed: 2026-02-19*

## Self-Check: PASSED

Files present and commits verified:
- `backend/src/lib.rs` — pub mod db + pub mod schema confirmed
- `backend/src/db/mod.rs` — all 6 submodules declared
- `backend/src/db/user.rs` — sync fn read(db: &mut MysqlConnection confirmed
- `backend/src/db/tracking/tracking.rs` — sync fn read confirmed
- Task 1 commit `6cc0627` — verified in git log
- Task 2 commit `c6a4695` — verified in git log
- `grep -r "rocket_db_pools" backend/src/db/` — zero matches
- `grep -r "async fn" backend/src/db/` — zero matches
- `grep -r "\.await" backend/src/db/` — zero matches
- `cargo build` — Finished (no errors)
- `cargo clippy` — Finished (no warnings)
