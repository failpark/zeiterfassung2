# Phase 2: Database Layer - Research

**Researched:** 2026-02-18
**Domain:** deadpool-diesel 0.6.1, diesel 2.1.6, diesel_migrations 2.1.0, MariaDB/MySQL async pool
**Confidence:** HIGH

---

## Summary

Phase 2 wires the existing Diesel ORM code (currently commented out and referencing `rocket_db_pools`) into an Axum-compatible async database pool. The standard approach is `deadpool-diesel` 0.6.1 with the `mysql` feature, which wraps synchronous `MysqlConnection` objects in a `SyncWrapper` that spawns blocking tasks via `interact()`. This is the pattern used in the official axum `examples/diesel-postgres` example.

The existing `db/` module contains five entity modules (user, activity, client, project, tracking) all using `Connection<DB>` from `rocket_db_pools`. These must be re-typed to accept `&mut MysqlConnection` (a plain sync connection from the pool's `interact()` closure). All async `.await` calls on queries remain valid because the `interact()` closure runs on a blocking thread — diesel query methods are synchronous inside the closure.

Migration handling is simpler than the blocker note suggested: `run_pending_migrations(MIGRATIONS)` can be called inside an `interact()` closure at startup, using the same sync connection that `deadpool-diesel` provides. No separate sync connection is needed.

**Primary recommendation:** Use `deadpool-diesel 0.6.1` with feature `mysql`. Add pool to `AppState`. Port db/ function signatures from `&mut Connection<DB>` to `&mut MysqlConnection`. Run migrations via `interact()` at startup.

---

## Standard Stack

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| deadpool-diesel | 0.6.1 | Async pool for sync Diesel connections | Official axum example; wraps MysqlConnection in spawn_blocking via interact() |
| diesel | 2.1.6 | ORM and query DSL | Already in project; keep as-is |
| diesel_migrations | 2.1.0 | Embedded migration runner | Already in project; keep as-is |

### Supporting
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| deadpool | 0.12.x | Pool internals (transitive dep) | Pulled in automatically by deadpool-diesel |
| deadpool-sync | 0.1.1 | SyncWrapper providing interact() | Pulled in automatically |
| tokio | 1.x | Async runtime (already present) | Already in project |

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| deadpool-diesel | diesel-async + deadpool feature | diesel-async uses truly async MySQL (mysql_async crate) — better throughput at scale, but requires rewriting all Diesel query calls to use `AsyncMysqlConnection` and `async fn` directly; overkill for this project and incompatible with existing sync query patterns |
| deadpool-diesel | r2d2 + spawn_blocking | r2d2 is sync-only; requires manual spawn_blocking in every handler; deadpool-diesel does this automatically via interact() |

**Installation:**
```bash
cargo add deadpool-diesel --features mysql
```

Cargo.toml entry:
```toml
deadpool-diesel = { version = "0.6.1", features = ["mysql"] }
```

Remove from dev-dependencies (no longer needed):
```toml
# diesel = { version = "2.1.4", features = ["chrono", "mysql", "r2d2"] }
```

The `r2d2` feature was only needed for the old test infrastructure. deadpool-diesel replaces it.

---

## Architecture Patterns

### Recommended Project Structure
```
backend/src/
├── db/
│   ├── mod.rs           # Pool type alias, PaginationResult, last_insert_id, run_migrations
│   ├── user.rs          # fn signatures: (&mut MysqlConnection, ...) -> QueryResult<T>
│   ├── activity.rs      # same pattern
│   ├── client.rs        # same pattern
│   ├── project.rs       # same pattern
│   └── tracking/        # same pattern
├── state.rs             # AppState with db_pool: deadpool_diesel::mysql::Pool
├── bin/backend.rs       # Initialize pool, run migrations, build AppState
└── config.rs            # Add database_url to config (env-only, no config.toml)
```

### Pattern 1: Pool Type Alias and AppState

Define a type alias to avoid verbose generic signatures throughout the codebase.

**AppState extension (state.rs):**
```rust
// Source: official axum examples/diesel-postgres + docs.rs/deadpool-diesel
use std::sync::Arc;
use crate::config::AppConfig;

pub type DbPool = deadpool_diesel::mysql::Pool;

#[derive(Clone)]
pub struct AppState {
    pub config: Arc<AppConfig>,
    pub db_pool: DbPool,
}
```

### Pattern 2: Pool Initialization at Startup

The pool is created in `main()` and stored in `AppState`. DATABASE_URL comes from environment only (never config.toml — locked decision from phase 1).

```rust
// Source: axum examples/diesel-postgres/src/main.rs (official example)
use deadpool_diesel::mysql::{Manager, Pool, Runtime};

// In main():
let database_url = std::env::var("DATABASE_URL")
    .expect("DATABASE_URL must be set");

let manager = Manager::new(database_url.clone(), Runtime::Tokio1);
let db_pool = Pool::builder(manager)
    .max_size(10)
    .build()
    .expect("Failed to build database pool");

tracing::info!(
    pool_size = 10,
    // Log connection string with password redacted
    connection = %redact_password(&database_url),
    "Database pool initialized"
);
```

Password redaction helper (simple, no external dep):
```rust
fn redact_password(url: &str) -> String {
    // Replace password in mysql://user:password@host/db with mysql://user:***@host/db
    if let Some(at_pos) = url.find('@') {
        if let Some(colon_pos) = url[..at_pos].rfind(':') {
            let mut redacted = url.to_string();
            redacted.replace_range((colon_pos + 1)..at_pos, "***");
            return redacted;
        }
    }
    url.to_string()
}
```

### Pattern 3: Migration Runner at Startup

Migrations run inside `interact()` at startup before the server starts accepting requests. This is the canonical pattern from the official axum diesel-postgres example.

```rust
// Source: axum examples/diesel-postgres/src/main.rs (official axum example)
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("../migrations");
// Note: path is relative to CARGO_MANIFEST_DIR (backend/), so ../migrations
// reaches the workspace-level migrations/ directory

// In main(), after pool creation:
{
    let conn = db_pool.get().await
        .expect("Could not get DB connection for migrations");
    conn.interact(|conn| {
        conn.run_pending_migrations(MIGRATIONS).map(|_| ())
    })
    .await
    .expect("Migration interact() failed")
    .expect("Migration execution failed");
}
tracing::info!("Database migrations applied successfully");
```

**Critical detail about embed_migrations! path:** The macro argument is relative to the crate's `CARGO_MANIFEST_DIR`. With `backend/Cargo.toml` as the crate manifest, use `embed_migrations!("../migrations")` to reach the workspace-level `migrations/` directory. The existing `db/mod.rs` already uses this path — preserve it.

### Pattern 4: db/ Function Signatures After Port

All db/ functions currently accept `&mut Connection<DB>` from `rocket_db_pools`. After porting, they accept `&mut MysqlConnection` from the `deadpool-diesel` connection object passed by `interact()`.

**Before (rocket_db_pools):**
```rust
use rocket_db_pools::{diesel::prelude::*, Connection};
use crate::DB;

pub async fn read(db: &mut Connection<DB>, param_id: i32) -> QueryResult<Self> {
    user.filter(id.eq(param_id)).first::<Self>(db).await
}
```

**After (deadpool-diesel, inside interact closure):**
```rust
use diesel::prelude::*;
use diesel::MysqlConnection;

// Function is now SYNCHRONOUS (no async, no .await on queries)
pub fn read(db: &mut MysqlConnection, param_id: i32) -> QueryResult<Self> {
    use crate::schema::user::dsl::*;
    user.filter(id.eq(param_id)).first::<Self>(db)
}
```

**Handler call site:**
```rust
async fn get_user(
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> crate::Result<Json<User>> {
    let conn = state.db_pool.get().await
        .map_err(|_| Error::Internal)?;
    let user = conn.interact(move |conn| {
        User::read(conn, id)
    })
    .await
    .map_err(|_| Error::Internal)?  // InteractError (thread panic, etc.)
    .map_err(Error::Database)?;     // diesel::result::Error
    Ok(Json(user))
}
```

### Pattern 5: Transaction Pattern

The existing code uses `db.transaction(|mut conn| Box::pin(async move { ... }))` — async transactions from `rocket_db_pools`. After porting, transactions are synchronous inside `interact()`.

**Before:**
```rust
db.transaction(|mut conn| {
    Box::pin(async move {
        insert_into(user).values(item).execute(&mut conn).await?;
        user.filter(id.eq(last_insert_id())).first::<Self>(&mut conn).await
    })
})
.await
```

**After (inside interact closure):**
```rust
db.transaction(|conn| {
    insert_into(user).values(item).execute(conn)?;
    user.select(User::as_select())
        .filter(id.eq(last_insert_id()))
        .first::<Self>(conn)
})
```

The `last_insert_id()` sql function remains identical — it's a Diesel DSL construct that the existing code already uses.

### Anti-Patterns to Avoid

- **Async inside interact():** `interact()` closures must be synchronous (`FnOnce(&mut T) -> R`). Do NOT use `.await` or `async` inside the closure — it will not compile. All `diesel` query methods are synchronous when called directly on `MysqlConnection`.
- **Nesting spawn_blocking inside interact():** `interact()` already calls `spawn_blocking` internally. Nesting another `spawn_blocking` inside the closure causes lifetime errors and is redundant.
- **Storing MysqlConnection in AppState:** Only store the `Pool`, not individual connections. Connections are checked out per-request via `pool.get().await`.
- **DATABASE_URL in config.toml:** Locked decision — env-only. Read it with `std::env::var("DATABASE_URL")`.
- **Using .await? on both interact errors:** The interact method returns `Result<R, InteractError>`. The inner closure returns `QueryResult<T>`. Two separate `.map_err` calls (or `??`) are needed to handle both layers.

---

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Async pool for sync Diesel | Custom Arc<Mutex<MysqlConnection>> | deadpool-diesel Pool | Connection multiplexing, timeouts, pool sizing, health checks, proper spawn_blocking integration |
| Migration runner | Custom SQL migration system | diesel_migrations + embed_migrations! | Idempotent tracking, checksum validation, up/down support |
| Password redaction in logs | Custom URL parser | Simple string manipulation (see Pattern 2) | Sufficient for a single URL format; full URL parsing is overkill |
| Error wrapping for interact() | Custom error enum with complex mapping | Direct .map_err chain | Two .map_err calls handle both layers cleanly |

**Key insight:** The `interact()` mechanism handles all thread-safety concerns. Never try to share a `MysqlConnection` across async boundaries — always check out from the pool per operation and use `interact()`.

---

## Common Pitfalls

### Pitfall 1: Wrong embed_migrations! Path

**What goes wrong:** `embed_migrations!("migrations")` fails to find the migrations directory at compile time because the path is resolved relative to `CARGO_MANIFEST_DIR` (the `backend/` directory), not the workspace root.

**Why it happens:** The `backend/Cargo.toml` is at `backend/`, but migrations live at the workspace root in `migrations/`. The existing code already uses `embed_migrations!("../migrations")` — this must be preserved.

**How to avoid:** Use `embed_migrations!("../migrations")` exactly as in the existing `db/mod.rs`.

**Warning signs:** Build error like `no such directory 'migrations'` during `cargo build`.

### Pitfall 2: Double-Await Error on interact()

**What goes wrong:** `conn.interact(|conn| some_query(conn)).await?` — the `?` desugars to `From<InteractError> for Error` which may not be implemented. Additionally, the inner `QueryResult` is not unwrapped.

**Why it happens:** `interact()` returns `Result<R, InteractError>`. If `R = QueryResult<T>`, the result is `Result<QueryResult<T>, InteractError>` — two error layers.

**How to avoid:**
```rust
conn.interact(|conn| User::read(conn, id))
    .await
    .map_err(|_| Error::Internal)?   // InteractError
    .map_err(Error::Database)?        // diesel::result::Error
```

Or, add `impl From<deadpool_diesel::InteractError> for Error` to `error.rs`.

**Warning signs:** Compiler error about type mismatch or missing `From` impl for `InteractError`.

### Pitfall 3: Async Transaction Pattern Mismatch

**What goes wrong:** Copying the existing `Box::pin(async move { ... })` transaction pattern into a `deadpool-diesel interact()` closure causes compile failure because async is not allowed inside the sync closure.

**Why it happens:** `rocket_db_pools` provided an async-native Diesel connection. `deadpool-diesel` wraps a sync connection — transactions must use the sync `connection.transaction(|conn| { ... })` API.

**How to avoid:** Remove `Box::pin(async move { ... })` wrappers. The closure passed to `transaction()` must be a sync closure returning `QueryResult<T>`.

**Warning signs:** Compiler error about `async` or `Future` in a non-async context inside `interact()`.

### Pitfall 4: re-importing diesel Prelude Conflicts

**What goes wrong:** `rocket_db_pools::diesel::prelude::*` and `diesel::prelude::*` provide overlapping symbols. After removing `rocket_db_pools`, some imports may resolve to wrong paths or disappear.

**Why it happens:** `rocket_db_pools` re-exported a modified diesel prelude. The standalone `diesel::prelude::*` is the canonical source.

**How to avoid:** Replace all `use rocket_db_pools::diesel::...` with `use diesel::...`. Replace `use rocket_db_pools::Connection` with `use diesel::MysqlConnection`.

**Warning signs:** Unresolved import errors on `RunQueryDsl`, `Connection`, `prelude`.

### Pitfall 5: last_insert_id() Type Mismatch

**What goes wrong:** `diesel::sql_function!(fn last_insert_id() -> Integer)` in `db/mod.rs` uses Diesel's `Integer` (i32). If the function is redeclared elsewhere or the return type doesn't match the column type, query failures occur at runtime.

**Why it happens:** MySQL `LAST_INSERT_ID()` returns a 64-bit integer but the existing declaration uses `Integer` (32-bit). This works as long as IDs stay within i32 range, but is technically incorrect.

**How to avoid:** Keep the existing declaration (`-> Integer`) to match the existing `id: i32` field types across all entities. No change needed in this phase.

**Warning signs:** QueryResult deserialization errors if IDs grow large.

### Pitfall 6: Pool Not in AppState Causes Handler Compile Errors

**What goes wrong:** Adding routes that need the DB pool but passing only `AppState { config }` causes handlers to fail compilation because `state.db_pool` does not exist.

**Why it happens:** AppState needs the pool field added before any handlers can use it.

**How to avoid:** Add `pub db_pool: DbPool` to `AppState` in `state.rs` as part of plan 02-01. The `DbPool` type alias hides the verbose generic type.

---

## Code Examples

Verified patterns from official sources:

### Pool Creation (MySQL)
```rust
// Source: deadpool-diesel docs.rs 0.6.1 + axum official example adapted for MySQL
use deadpool_diesel::mysql::{Manager, Pool, Runtime};

let manager = Manager::new(database_url, Runtime::Tokio1);
let pool = Pool::builder(manager)
    .max_size(10)
    .build()
    .expect("Failed to create pool");
```

### Running Migrations via interact()
```rust
// Source: axum examples/diesel-postgres/src/main.rs (exact pattern, adapted for mysql)
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("../migrations");

let conn = pool.get().await.expect("Migration: could not get connection");
conn.interact(|conn| conn.run_pending_migrations(MIGRATIONS).map(|_| ()))
    .await
    .expect("Migration interact failed")
    .expect("Migrations failed");
```

### Handler Pattern with Pool
```rust
// Source: docs.rs/deadpool-diesel + axum examples/diesel-postgres pattern
async fn some_handler(
    State(state): State<AppState>,
) -> crate::Result<Json<User>> {
    let conn = state.db_pool.get().await
        .map_err(|_| Error::Internal)?;
    let user = conn.interact(|conn| {
        User::read(conn, 1)
    })
    .await
    .map_err(|_| Error::Internal)?
    .map_err(Error::Database)?;
    Ok(Json(user))
}
```

### Ported db/ Function (Synchronous)
```rust
// Pattern derived from existing db/user.rs after removing rocket_db_pools
use diesel::prelude::*;
use diesel::MysqlConnection;

impl User {
    pub fn read(db: &mut MysqlConnection, param_id: i32) -> QueryResult<Self> {
        use crate::schema::user::dsl::*;
        user.filter(id.eq(param_id)).first::<Self>(db)
    }

    pub fn create(db: &mut MysqlConnection, item: &CreateUser) -> QueryResult<Self> {
        use crate::schema::user::dsl::*;
        db.transaction(|conn| {
            diesel::insert_into(user).values(item).execute(conn)?;
            user.select(User::as_select())
                .filter(id.eq(super::last_insert_id()))
                .first::<Self>(conn)
        })
    }
}
```

### InteractError Integration with Error Type
```rust
// Add to error.rs to allow ? on interact() errors directly
use deadpool_diesel::InteractError;

impl From<InteractError> for Error {
    fn from(_: InteractError) -> Self {
        Error::Internal
    }
}
```

With this `From` impl, handlers can write:
```rust
conn.interact(|conn| User::read(conn, id))
    .await?           // InteractError -> Error::Internal
    .map_err(Error::Database)?  // diesel::result::Error -> Error::Database
```

### Pool Connectivity Test
```rust
// Integration test verifying pool can acquire and execute
#[tokio::test]
async fn pool_executes_select_1() {
    let database_url = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL required for this test");
    let manager = deadpool_diesel::mysql::Manager::new(database_url, deadpool_diesel::mysql::Runtime::Tokio1);
    let pool = deadpool_diesel::mysql::Pool::builder(manager)
        .max_size(2)
        .build()
        .expect("pool build failed");

    let conn = pool.get().await.expect("pool.get() failed");
    let result: i32 = conn.interact(|conn| {
        diesel::select(diesel::dsl::sql::<diesel::sql_types::Integer>("1"))
            .get_result(conn)
    })
    .await
    .expect("interact failed")
    .expect("query failed");

    assert_eq!(result, 1);
}
```

---

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| rocket_db_pools with async MysqlPool | deadpool-diesel with MysqlConnection + interact() | Phase 2 migration | db/ functions become sync; handlers use interact() wrapper |
| Async query methods (.await on load/first/etc.) | Sync query methods inside interact() closure | Phase 2 migration | Remove all .await from db/ query calls |
| Box::pin(async move { }) transactions | Sync transaction closures | Phase 2 migration | Simpler transaction code |
| r2d2 in dev-dependencies for tests | deadpool-diesel pool for all | Phase 2 migration | Unified pool type; remove r2d2 from Cargo.toml |

**Deprecated/outdated:**
- `rocket_db_pools::Connection<DB>`: Rocket-specific; replaced by `deadpool_diesel::mysql::Connection` (type alias for `deadpool_diesel::Object<MysqlConnection>`) — but this is only used at the call site in handlers. db/ functions see only `&mut MysqlConnection`.
- `rocket_db_pools::diesel::prelude::*`: Replace with `diesel::prelude::*`
- `rocket_db_pools::diesel::RunQueryDsl`: Replace with `diesel::RunQueryDsl`

---

## Scope Clarification: db/ Module Status

The `db/` module is currently **commented out** in `lib.rs`:
```rust
// TODO: Port to Axum in Phase 3+
// mod db;
// mod schema;
```

Phase 2's goal is to:
1. Add the pool to AppState and wire it at startup
2. Re-enable and port the `db/` module so it compiles (changing function signatures)
3. Write a connectivity integration test

The db/ functions do NOT need to be called by any routes in Phase 2 — they just need to compile. Routes that actually use these functions come in Phase 3+.

**schema.rs** is also commented out. It must be re-enabled as `pub mod schema` (or kept in the backend root) because db/ modules reference `crate::schema::*`.

---

## Open Questions

1. **DATABASE_URL configuration source**
   - What we know: Locked decision from Phase 1 — secrets env-only, never config.toml
   - What's unclear: Does `load_config()` via figment need to read DATABASE_URL, or should main() read it directly with `std::env::var`?
   - Recommendation: Read with `std::env::var("DATABASE_URL")` directly in main() before pool creation. This keeps secrets out of the figment/config system entirely. Dotenvy already loads `.env` early in main().

2. **Pool max_size**
   - What we know: The official axum example uses `max_size(8)`, deadpool default varies
   - What's unclear: Right default for development vs production MariaDB
   - Recommendation: Use 10 as default; make it a named constant or config value later. For this phase, hardcode 10.

3. **MariaDB RETURNING clause**
   - What we know: Diesel's mysql backend does NOT support RETURNING. MariaDB 10.5+ supports `DELETE ... RETURNING` but not `INSERT ... RETURNING`. Diesel uses `last_insert_id()` workaround for MySQL/MariaDB.
   - What's unclear: Target MariaDB version on the development machine
   - Recommendation: Keep the existing `last_insert_id()` pattern — it works on all MariaDB versions and avoids the unsupported RETURNING clause. No change needed.

4. **db/helper.rs port**
   - What we know: `helper.rs` contains a generic `last_page` function using `Connection<DB>` and rocket_db_pools types; it's currently commented out (`// pub mod helper`)
   - What's unclear: Whether helper.rs should be ported in Phase 2 or left commented until Phase 3
   - Recommendation: Port it in Phase 2 alongside the other db/ modules for completeness. The generic bounds will need updating to use diesel traits directly.

---

## Sources

### Primary (HIGH confidence)
- Official axum example `examples/diesel-postgres/src/main.rs` — exact interact() + migration pattern verified
- docs.rs/deadpool-diesel/0.6.1 — version, feature flags, interact() signature
- docs.rs/deadpool-sync (SyncWrapper) — `interact()` exact type: `async fn interact<F, R>(&self, f: F) -> Result<R, InteractError> where F: FnOnce(&mut T) -> R + Send + 'static, R: Send + 'static`
- Existing codebase (`backend/src/db/*.rs`) — all entity patterns, exact function signatures to port

### Secondary (MEDIUM confidence)
- GitHub weiznich/diesel_async issue #17 — confirms diesel_migrations is sync; interact() + run_pending_migrations is the workaround
- docs.rs/diesel-async 0.7.4 — feature flags, MySQL async alternative (diesel-async not chosen per phase requirements)
- diesel-rs/diesel discussion #3605 — confirms RETURNING not available on MySQL backend; last_insert_id() is the standard workaround

### Tertiary (LOW confidence)
- WebSearch results on MariaDB version compatibility — RETURNING clause availability varies by MariaDB version; unverified target version

---

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — official axum example uses deadpool-diesel 0.6.1 exactly; version confirmed from Cargo.toml source
- Architecture: HIGH — interact() signature verified from docs.rs/deadpool-sync; migration pattern from official axum source
- Pitfalls: HIGH — derived from codebase inspection (actual code that must change) + official docs confirmation
- db/ port scope: HIGH — all five entity files inspected; patterns are uniform and mechanical

**Research date:** 2026-02-18
**Valid until:** 2026-03-18 (deadpool-diesel is stable; diesel 2.x API is stable)
