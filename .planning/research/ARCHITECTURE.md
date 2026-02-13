# Architecture Research

**Domain:** Axum + Diesel + MariaDB REST API (Rocket → Axum rewrite)
**Researched:** 2026-02-13
**Confidence:** MEDIUM (training data, external sources unavailable; Axum stable since 0.6/0.7, patterns well-established)

---

## Standard Architecture

### System Overview

```
┌──────────────────────────────────────────────────────────────────┐
│                       React SPA (Frontend)                        │
│                     (Vite + TypeScript)                          │
└─────────────────────────┬────────────────────────────────────────┘
                          │  HTTP/JSON (REST)
                          │  Bearer JWT in Authorization header
┌─────────────────────────▼────────────────────────────────────────┐
│                     Axum HTTP Layer                               │
│  ┌────────────┐  ┌────────────┐  ┌──────────┐  ┌─────────────┐  │
│  │  Middleware │  │   Router   │  │   CORS   │  │   Tracing   │  │
│  │  (auth JWT)│  │  (routes/) │  │ (tower)  │  │  (tracing/) │  │
│  └────────┬───┘  └─────┬──────┘  └──────────┘  └─────────────┘  │
│           │             │                                         │
├───────────▼─────────────▼─────────────────────────────────────── ┤
│                    Handler Layer (routes/)                         │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────┐          │
│  │ tracking │  │  client  │  │ project  │  │   user   │          │
│  └────┬─────┘  └────┬─────┘  └────┬─────┘  └────┬─────┘          │
├───────▼──────────────▼────────────▼───────────────▼───────────── ┤
│                    Service / DB Layer (db/)                        │
│  ┌──────────────────────────────────────────────────────────┐     │
│  │   Domain functions (create/read/update/delete/paginate)  │     │
│  │   + middle layer for multi-table operations (tracking)   │     │
│  └──────────────────────────┬───────────────────────────────┘     │
├──────────────────────────────▼─────────────────────────────────── ┤
│                  Diesel ORM + Connection Pool                      │
│  ┌───────────────────────────────────────────────────────────┐    │
│  │   deadpool-diesel (async pool wrapping Diesel sync conn)  │    │
│  └──────────────────────────┬────────────────────────────────┘    │
└──────────────────────────────▼─────────────────────────────────── ┘
                               │
                    ┌──────────▼──────────┐
                    │      MariaDB        │
                    │  (6 tables, Diesel  │
                    │   schema.rs)        │
                    └─────────────────────┘
```

### Component Responsibilities

| Component | Responsibility | Typical Implementation |
|-----------|----------------|------------------------|
| `main.rs` | Entry point: build router, attach state, bind listener | `axum::serve()` with `TcpListener` |
| `router` | Compose all route modules, apply middleware layers | `Router::new().merge()` + `axum::middleware::from_fn_with_state()` |
| `routes/` | HTTP handlers: extract inputs, call db, return responses | Axum handler functions with extractor params |
| `auth/` | JWT generation + verification (Tokenizer singleton) | `jwt-simple` Ed25519 keypair, same logic as Rocket version |
| `middleware/auth.rs` | JWT bearer token extraction + user injection | `axum::middleware::from_fn_with_state()` |
| `db/` | Domain CRUD + middle layers (e.g., tracking + activities join) | Diesel queries, deadpool-diesel pool |
| `error.rs` | App-wide error type implementing `IntoResponse` | `thiserror` + `impl IntoResponse for Error` |
| `state.rs` | `AppState` struct shared across all handlers | `Arc<AppState>` with pool + tokenizer |
| `schema.rs` | Diesel-generated table definitions (unchanged) | Keep as-is from existing codebase |

---

## Recommended Project Structure

```
backend/
├── src/
│   ├── bin/
│   │   └── backend.rs          # Entry point: build router, bind port, run migrations
│   ├── lib.rs                  # pub fn app() -> Router — builds router (testable without binding)
│   ├── state.rs                # AppState { pool: Pool<AsyncDieselConnectionManager<MysqlConnection>>, tokenizer: Arc<Tokenizer> }
│   ├── error.rs                # Error enum + impl IntoResponse (replaces Rocket Responder)
│   ├── schema.rs               # Diesel-generated (unchanged)
│   ├── auth/
│   │   ├── mod.rs
│   │   └── tokenizer.rs        # Tokenizer (minimal changes — remove rocket deps)
│   ├── middleware/
│   │   ├── mod.rs
│   │   └── auth.rs             # JWT extractor middleware (replaces guard.rs)
│   ├── routes/
│   │   ├── mod.rs              # pub fn routes() -> Router — merge all route modules
│   │   ├── login.rs
│   │   ├── user.rs
│   │   ├── client.rs
│   │   ├── project.rs
│   │   ├── activity.rs
│   │   └── tracking.rs
│   ├── db/
│   │   ├── mod.rs              # Pool type alias, run_migrations, PaginationResult
│   │   ├── user.rs
│   │   ├── client.rs
│   │   ├── project.rs
│   │   ├── activity.rs
│   │   └── tracking/
│   │       ├── mod.rs
│   │       ├── tracking.rs
│   │       ├── tracking_to_activity.rs
│   │       └── middlelayer.rs  # Multi-table business logic (unchanged logic)
│   └── tracing.rs              # tracing-subscriber setup (unchanged)
```

### Structure Rationale

- **`lib.rs` exports `app()`:** Makes the application testable — tests call `app()` without binding a port, just like the existing `rocket()` function pattern. This mirrors the existing architecture directly.
- **`state.rs` centralizes shared state:** Axum uses `Arc<AppState>` instead of Rocket's `State<T>` managed types. One struct, one `Arc`, all handlers use the same clone.
- **`middleware/` replaces `guard.rs`:** Rocket guards are per-handler request guards. Axum middleware is applied to a `Router` subtree. Auth middleware injects a `User` via request extensions; handlers extract it.
- **`routes/mod.rs` composes routing:** Each route module returns a `Router` (not `AdHoc` fairings). `mod.rs` merges them and applies auth middleware to protected routes.
- **`db/` structure unchanged:** The Diesel query layer and middle layer are framework-agnostic. The only change is the connection type (from `rocket_db_pools::Connection<DB>` to a `deadpool-diesel` pooled connection).
- **`error.rs` implements `IntoResponse`:** Axum requires errors to implement `IntoResponse` instead of Rocket's `Responder`. The error variants and HTTP status mapping are identical.

---

## Architectural Patterns

### Pattern 1: AppState with Arc

**What:** All shared dependencies (DB pool, JWT tokenizer) wrapped in a single `AppState` struct, cloned cheaply via `Arc`.
**When to use:** Always. Axum requires `Clone` for state; `Arc` provides cheap cloning with shared ownership.
**Trade-offs:** All handlers receive the same state type — no per-dependency injection. Fine for this app's scale.

**Example:**
```rust
// state.rs
use std::sync::Arc;
use deadpool_diesel::mysql::Pool;
use crate::auth::Tokenizer;

#[derive(Clone)]
pub struct AppState {
    pub pool: Pool,
    pub tokenizer: Arc<Tokenizer>,
}

// lib.rs
pub fn app(state: AppState) -> Router {
    Router::new()
        .merge(routes::routes())
        .with_state(state)
}
```

### Pattern 2: Typed Extractors for Auth (replacing Rocket guards)

**What:** Axum extractors implement `FromRequestParts`. A `CurrentUser` extractor validates the JWT Bearer token from the `Authorization` header, returning `User` or 401.
**When to use:** Any handler that requires authentication. Apply it as the last parameter (before `State`) by convention.
**Trade-offs:** Extractor approach is explicit per-handler; alternatively use middleware + request extensions for blanket auth on a router subtree. Both work. For this app, middleware on a protected subrouter is cleaner.

**Example (middleware approach — recommended):**
```rust
// middleware/auth.rs
use axum::{
    extract::{Request, State},
    middleware::Next,
    response::Response,
};
use crate::{error::Error, state::AppState, db::user::User};

pub async fn require_auth(
    State(state): State<AppState>,
    mut req: Request,
    next: Next,
) -> Result<Response, Error> {
    let token = req
        .headers()
        .get("Authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .ok_or(Error::UnauthenticatedUser)?;

    let user = state.tokenizer.verify(token)?;
    req.extensions_mut().insert(user);
    Ok(next.run(req).await)
}

// routes/mod.rs — apply auth to all non-login routes
pub fn routes() -> Router<AppState> {
    let public = Router::new()
        .merge(login::router());

    let protected = Router::new()
        .merge(user::router())
        .merge(client::router())
        .merge(project::router())
        .merge(activity::router())
        .merge(tracking::router())
        .layer(axum::middleware::from_fn_with_state(
            // state provided at router build time
            AppState::placeholder(), // actual state injected via .with_state()
            require_auth,
        ));

    Router::new().merge(public).merge(protected)
}

// Handler: extract injected user from extensions
async fn create_tracking(
    Extension(user): Extension<User>,
    State(state): State<AppState>,
    Json(payload): Json<CreateTracking>,
) -> Result<Json<Tracking>, Error> {
    // ...
}
```

### Pattern 3: Diesel + deadpool-diesel (sync Diesel in async context)

**What:** Diesel's MySQL driver is synchronous. In an async Axum handler, all Diesel calls must be dispatched via `spawn_blocking` or wrapped by `deadpool-diesel` which does this automatically.
**When to use:** Every database access in every handler.
**Trade-offs:** `deadpool-diesel` is the idiomatic solution — it provides a connection pool and handles `spawn_blocking` internally. Alternative: `diesel-async` with `async-diesel` glue, but this requires patching Diesel's internals and is harder to maintain. Prefer `deadpool-diesel`.

**Example:**
```rust
// Cargo.toml additions
// deadpool-diesel = { version = "0.5", features = ["mysql"] }

// db/mod.rs
use deadpool_diesel::mysql::{Manager, Pool};
use diesel::MysqlConnection;

pub fn build_pool(database_url: &str) -> Pool {
    let manager = Manager::new(database_url, deadpool_diesel::Runtime::Tokio1);
    Pool::builder(manager).max_size(8).build().unwrap()
}

// db/tracking/middlelayer.rs (adapted)
pub async fn create(pool: &Pool, payload: &CreateTracking) -> Result<Tracking, Error> {
    let conn = pool.get().await.map_err(|_| Error::Internal)?;
    conn.interact(move |conn| {
        // Diesel sync call here — runs in spawn_blocking thread
        TrackingDB::create(conn, &payload_clone)
    })
    .await
    .map_err(|_| Error::Internal)?
    .map_err(Error::Database)
}
```

### Pattern 4: IntoResponse for Error Handling

**What:** The app-wide `Error` type implements `axum::response::IntoResponse`, producing JSON error bodies with appropriate HTTP status codes. This replaces Rocket's `Responder` implementation.
**When to use:** Single error type for the entire application; all handlers return `Result<T, Error>`.
**Trade-offs:** Identical logic to the existing Rocket `Responder` impl — only the trait name changes.

**Example:**
```rust
// error.rs
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        let status = self.to_status();
        let body = Json(json!({
            "error": self.to_string(),
            "code": status.as_u16(),
        }));
        (status, body).into_response()
    }

    fn to_status(&self) -> StatusCode {
        match self {
            Self::NotFound => StatusCode::NOT_FOUND,
            Self::UnauthenticatedUser | Self::WrongCredentials | Self::Unauthorized =>
                StatusCode::UNAUTHORIZED,
            Self::ForbiddenAccess => StatusCode::FORBIDDEN,
            Self::BadRequest(_) | Self::JWT(_) => StatusCode::BAD_REQUEST,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}
```

### Pattern 5: Router-per-module with Router::merge

**What:** Each route module (`routes/tracking.rs`, etc.) returns a `Router<AppState>` that defines its own paths. The top-level `routes/mod.rs` merges them. This replaces Rocket's `AdHoc::on_ignite` mount pattern.
**When to use:** Always for this app size — keeps route registration parallel to the existing module structure.
**Trade-offs:** No macro magic; explicit path composition. Easy to trace routes.

**Example:**
```rust
// routes/tracking.rs
pub fn router() -> Router<AppState> {
    Router::new()
        .route("/tracking", post(create).get(get_page))
        .route("/tracking/:id", get(get_one).patch(update).delete(delete))
        .route("/tracking/page/:page_size/:page", get(get_page_numbered))
        .route("/tracking/page/:page_size/last", get(get_last_page))
}

// routes/mod.rs
pub fn routes() -> Router<AppState> {
    Router::new()
        .merge(login::router())
        .merge(protected_routes())
}

fn protected_routes() -> Router<AppState> {
    Router::new()
        .merge(user::router())
        .merge(client::router())
        .merge(project::router())
        .merge(activity::router())
        .merge(tracking::router())
        // auth middleware applied to all protected routes
        .layer(axum::middleware::from_fn_with_state(
            AppState::default(),   // placeholder; state injected later
            middleware::auth::require_auth,
        ))
}
```

---

## Data Flow

### Request Flow (Protected Endpoint)

```
React SPA
    │  POST /tracking  { Authorization: Bearer <jwt> }
    ▼
TcpListener (tokio)
    ▼
Tower middleware stack:
    │  CorsLayer (tower-http)
    │  TraceLayer (tower-http)
    ▼
require_auth middleware
    │  Extract Bearer token → Tokenizer::verify() → User
    │  Inject User into request extensions
    ▼
Axum Router → tracking::create handler
    │  Extract: Extension(user), State(state), Json(payload)
    │  Authorization check: user.sys_role == "admin"
    ▼
db::tracking::middlelayer::Tracking::create(&pool, &payload)
    │  pool.get() → deadpool connection
    │  conn.interact(|conn| { Diesel insert }) → spawn_blocking
    ▼
MariaDB (INSERT INTO tracking ... + INSERT INTO tracking_to_activity ...)
    ▼
Result<Tracking, Error>
    ▼
Handler returns Ok(Json(tracking))   OR   Err(Error::ForbiddenAccess)
    ▼
IntoResponse: (StatusCode::OK, Json body)   OR   (StatusCode::FORBIDDEN, Json error)
    ▼
HTTP Response to React SPA
```

### State Flow

```
AppState (built once at startup)
    ├── pool: deadpool_diesel::mysql::Pool
    │       └── cloned cheaply (Arc internally)
    └── tokenizer: Arc<Tokenizer>
            └── Ed25519 keypair (regenerated on each restart — intentional)

Each handler receives: State(state): State<AppState>
    → state.pool.get().await  →  Diesel connection
    → state.tokenizer.verify() / .generate()
```

### Key Data Flows

1. **Login:** `POST /login` → `User::check_credentials(pool, email, password)` → argon2 verify → `Tokenizer::generate(user)` → `{ token: "..." }`
2. **Protected CRUD:** Bearer JWT → `require_auth` middleware → `User` in extensions → handler checks `user.sys_role` if needed → db layer → JSON response
3. **Tracking with activities:** `db::tracking::middlelayer` orchestrates two tables (`tracking` + `tracking_to_activity`) — this logic is framework-agnostic and carries over unchanged from Rocket version

---

## Component Boundaries

| From | To | Communication | Notes |
|------|-----|---------------|-------|
| React SPA | Axum routes | HTTP JSON REST | CORS must allow `http://localhost:5173` (dev); configure for prod origin |
| Axum handler | `db/` layer | Direct Rust function call | Pass `&pool` not a DB connection; pool manages connections |
| `db/` layer | MariaDB | Diesel + deadpool-diesel | sync Diesel wrapped in `spawn_blocking` via deadpool |
| `middleware/auth` | `auth::Tokenizer` | Direct Rust call | Tokenizer accessed via `AppState` |
| `routes/` | `middleware/auth` | Tower layer on protected `Router` | Applied once at router composition, not per-handler |
| `error.rs` | Axum response system | `impl IntoResponse` | Axum calls this automatically when handler returns `Err(...)` |

---

## Scaling Considerations

| Scale | Architecture Adjustments |
|-------|--------------------------|
| 0-1k users | Current monolith is fine; default pool size of 5-10 sufficient |
| 1k-100k users | Tune `deadpool-diesel` pool size; add connection-level rate limiting; deploy behind reverse proxy (nginx/caddy) |
| 100k+ users | Read replica for paginated queries; consider caching layer for activity/client lookups; no structural changes needed |

### Scaling Priorities

1. **First bottleneck:** DB connection pool exhaustion under concurrent load. Fix: increase `max_size` in deadpool config and tune MariaDB `max_connections`.
2. **Second bottleneck:** `spawn_blocking` thread pool saturation (deadpool-diesel uses tokio's blocking thread pool). Fix: tuning `TOKIO_WORKER_THREADS` and blocking thread limits.

---

## Anti-Patterns

### Anti-Pattern 1: Acquiring DB Connection in Middleware

**What people do:** Call `pool.get()` inside the auth middleware to look up the user by token.
**Why it's wrong:** This app uses stateless JWT — the user is encoded in the token itself. Getting a DB connection in middleware wastes pool connections on every request. The existing `Tokenizer::verify()` approach is correct and must be preserved.
**Do this instead:** Decode the JWT in middleware (no DB call), inject `User` struct into request extensions. Only hit the DB when business logic requires it (e.g., checking if a user still exists).

### Anti-Pattern 2: Holding Diesel Connection Across Await Points

**What people do:** Get a `deadpool-diesel` connection, call `.await` for something unrelated, then use the connection again.
**Why it's wrong:** Diesel connections are sync and not `Send + Sync` across `await` points. The deadpool `interact()` closure must be entirely synchronous.
**Do this instead:** Acquire the connection, do all Diesel work inside `conn.interact(|conn| { ... })`, drop the connection before any `await`.

### Anti-Pattern 3: Using Rocket's `Connection<DB>` Pattern Directly

**What people do:** Try to port the `Connection<DB>` extractor from `rocket_db_pools` by creating a similar Axum extractor.
**Why it's wrong:** Axum extractors consume the request; a per-request DB connection extractor that holds a pool connection across the entire handler lifetime reduces pool efficiency. deadpool-diesel's `pool.get()` called at the point of use, inside `interact()`, is the correct pattern.
**Do this instead:** Pass `&state.pool` to db functions; acquire connections lazily inside `interact()` closures.

### Anti-Pattern 4: Flattening Middleware into Every Handler

**What people do:** Check JWT in every handler function instead of using middleware.
**Why it's wrong:** Duplicates auth logic, easy to forget on a new route.
**Do this instead:** Apply `require_auth` middleware to a protected `Router` subtree. Login route lives on a separate public `Router`. Identical to how Rocket's `User` guard worked, but explicit at the routing layer.

### Anti-Pattern 5: Sharing Mutable AppState Without Arc

**What people do:** Wrap `AppState` in `Mutex` unnecessarily, or store mutable state directly.
**Why it's wrong:** `deadpool-diesel::Pool` and `Tokenizer` are already internally synchronized. Wrapping them in an outer `Mutex` creates unnecessary contention.
**Do this instead:** `AppState` fields are either `Clone` + internally synchronized (pool) or `Arc<T>` (tokenizer). No outer `Mutex` needed.

---

## Integration Points

### External Services

| Service | Integration Pattern | Notes |
|---------|---------------------|-------|
| MariaDB | `deadpool-diesel` connection pool | `DATABASE_URL` env var; same as existing Diesel setup |
| React SPA | `tower-http::CorsLayer` | Replace `rocket_cors`; configure `AllowOrigin::exact("http://localhost:5173")` in dev |

### Internal Boundaries

| Boundary | Communication | Notes |
|----------|---------------|-------|
| `routes/` ↔ `db/` | Direct function calls, `&Pool` passed via `AppState` | db functions are async (deadpool-diesel) |
| `routes/` ↔ `auth/` | Via `AppState.tokenizer` | `Arc<Tokenizer>` — no framework coupling |
| `middleware/auth` ↔ handler | `axum::Extension<User>` in request extensions | Handler uses `Extension(user): Extension<User>` extractor |
| `error.rs` ↔ all layers | `Error` type propagated via `?`, converted at response boundary | `impl IntoResponse` invoked by Axum automatically |

---

## Build Order

Building the Axum rewrite should follow this dependency order. Each step unblocks the next:

1. **Foundation: `state.rs`, `error.rs`, `db/mod.rs`**
   - `AppState` struct with `deadpool-diesel` pool
   - `Error` enum with `impl IntoResponse` (port from Rocket Responder)
   - `build_pool()` and `run_migrations()` (minimal framework changes)
   - **Unblocks:** everything else; state and errors are used everywhere

2. **Auth: `auth/tokenizer.rs`, `middleware/auth.rs`**
   - `Tokenizer` is already framework-agnostic — just remove `rocket` imports
   - `require_auth` middleware replaces `guard.rs` FromRequest impl
   - **Unblocks:** protected routes; can test middleware in isolation

3. **Route skeleton: `lib.rs`, `routes/mod.rs`, `bin/backend.rs`**
   - Build the router with placeholder handlers returning `StatusCode::OK`
   - Verify CORS, tracing middleware, and auth middleware wire up correctly
   - **Unblocks:** parallel development of individual route modules

4. **DB layer: `db/*` modules**
   - Port each db module from `rocket_db_pools::Connection<DB>` to `&deadpool_diesel::mysql::Pool`
   - `middlelayer.rs` for tracking has complex multi-table logic — port and test independently
   - **Unblocks:** route handlers that call db functions

5. **Route handlers: `routes/*`**
   - Port each handler: replace `Connection<DB>` extractor with `State(state): State<AppState>`, replace `User` guard with `Extension(user): Extension<User>`
   - Login route has no auth middleware
   - Admin-only operations (create/delete tracking) check `user.sys_role` inline
   - **Unblocks:** integration tests

6. **Integration tests**
   - Port existing tests: replace `rocket::local::blocking::Client` with `axum-test` or `tower::ServiceExt` + `hyper`
   - Existing test structure (generate_* helpers, methods helpers) is reusable

---

## Rocket → Axum Mapping Reference

| Rocket Concept | Axum Equivalent | Migration Notes |
|----------------|-----------------|-----------------|
| `Rocket::build()` + `.attach()` | `Router::new()` + `.layer()` + `.with_state()` | Fairings become tower `Layer`s |
| `rocket_db_pools::Connection<DB>` | `State<AppState>` + `pool.get()` inside handler | Pool accessed via state, not injected per-request |
| `FromRequest` guard (`User`) | `middleware::from_fn_with_state` + `Extension<User>` | Middleware injects, extractor consumes |
| `#[get("/<id>")]` proc macro | `.route("/resource/:id", get(handler))` | Path params via `Path<(i32,)>` extractor |
| `Json<T>` request extractor | `Json<T>` from `axum::extract::Json` | Identical usage |
| `State<T>` managed state | `State<AppState>` | One struct vs many individual managed types |
| `impl Responder for Error` | `impl IntoResponse for Error` | Different trait, same logic |
| `rocket_cors` crate | `tower-http::CorsLayer` | `tower-http` is the standard; widely used |
| `AdHoc::on_ignite("Mount X")` | `router.merge(module::router())` | Direct Router composition |
| `#[cfg(test)]` + blocking `Client` | `axum-test` crate or `tower::ServiceExt::oneshot()` | `axum-test` is most ergonomic |
| `tracing-subscriber` setup | Identical — `tracing.rs` needs no changes | `tracing` is framework-agnostic |

---

## Sources

- Training data: Axum 0.7.x documentation and patterns (HIGH confidence for stable patterns: Router, extractors, state, IntoResponse)
- Training data: deadpool-diesel integration patterns (MEDIUM confidence — verify `interact()` API against current deadpool-diesel docs before implementation)
- Training data: tower-http CorsLayer configuration (MEDIUM confidence — verify `AllowOrigin` API)
- Training data: axum middleware patterns with `from_fn_with_state` (MEDIUM confidence — stable API since 0.6)
- Existing codebase analysis: `/Users/phedias/code/zeiterfassung/backend/src/` (HIGH confidence — ground truth for current architecture)

**Note:** WebSearch and WebFetch were unavailable during this research session. All Axum/deadpool-diesel patterns are from training data (pre-January 2025). Before implementation, verify against:
- https://docs.rs/axum/latest/axum/
- https://docs.rs/deadpool-diesel/latest/deadpool_diesel/
- https://docs.rs/tower-http/latest/tower_http/cors/

---
*Architecture research for: Axum + Diesel + MariaDB time tracking API (Rocket rewrite)*
*Researched: 2026-02-13*
