# Phase 3: Authentication - Research

**Researched:** 2026-02-22
**Domain:** Axum JWT authentication, typed Role enum (Diesel + Serde), frontend auth wiring (TanStack Router v1)
**Confidence:** HIGH

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

- **JWT token strategy:** Keep ephemeral Ed25519 keypair — new keypair generated on each server restart, all tokens invalidate on deploy
- **Algorithm:** Keep Ed25519 signing (no switch to HMAC)
- **Login endpoint path:** `/auth/login` (namespaced, not old `/login`)
- **Logout:** `POST /auth/logout` exists — returns 200, no server-side blacklisting; endpoint placeholder for future use
- **Failed login:** Include a basic artificial delay to slow brute force attempts
- **Role enum design:**
  - Extensible enum (designed to allow adding roles later)
  - Unknown role values in DB cause error on deserialize (strict — forces code update before DB update)
  - Role enum used everywhere: JWT claims, API JSON responses, DB layer — single source of truth
  - JSON serialization as lowercase strings: `"admin"`, `"user"` — matches current DB values
- **Auth extractor pattern:** Custom Axum extractor: `AuthUser(user)` declared as handler parameter; separate `AdminUser` extractor that returns 403 if not admin

### Claude's Discretion

- Token expiration config mechanism (env var vs config file)
- JWT claims content (full user struct vs minimal id+role)
- Login response shape (token-only vs token+user data)
- Login endpoint under `/api` prefix or at root
- Auth middleware scope (router-level vs per-route-group)

### Deferred Ideas (OUT OF SCOPE)

None — discussion stayed within phase scope

</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|-----------------|
| AUTH-01 | User can log in with email and password and receive a JWT token | Existing `Tokenizer` struct (jwt-simple 0.11, Ed25519) is directly portable to Axum; `User::check_credentials()` already works with `&mut MysqlConnection`; Axum handler pattern is straightforward POST handler with `State<AppState>` and `Json<LoginRequest>` |
| AUTH-02 | JWT token expiration is configurable via environment/config (not hardcoded) | Current `Tokenizer::new(exp)` already accepts any `Into<Duration>` — adding `auth.token_expiration_secs` to `AppConfig` and reading via figment config.toml + `APP__AUTH__TOKEN_EXPIRATION_SECS` env var is the natural extension |
| AUTH-03 | User can log out (token invalidation on client side) | Frontend `authApi.removeToken()` and `AuthContext.logout()` already exist and work correctly; backend only needs the `POST /auth/logout` stub that returns 200; frontend URL update required |
| AUTH-04 | System role is a typed enum (user/admin) not a raw string | `sys_role: String` in `User` struct must become `Role` enum with manual `FromSql`/`ToSql` for MySQL Varchar; Serde serialize as lowercase string; Diesel schema stays as `Varchar` (no DB migration needed) |

</phase_requirements>

## Summary

This is a brownfield port of an existing working auth system (Rocket → Axum). The hard problems (Ed25519 JWT signing, Argon2 password verification, token-in-header extraction, frontend login page) are already solved. The three new technical challenges introduced in this phase are: (1) replacing the Rocket `FromRequest` guard with an Axum `FromRequestParts` extractor that reads `AppState` to access the `Tokenizer`, (2) implementing a typed `Role` enum that round-trips through Diesel's MySQL layer and Serde JSON serialization without a DB migration, and (3) updating the TanStack Router frontend to use the modern v1 API (current `router.tsx` uses deprecated class-based `RootRoute`/`Route`/`createReactRouter`) while pointing at `/auth/login`.

The existing codebase provides all the building blocks. The `auth/tokenizer.rs` is framework-agnostic and can be re-used as-is. The `db/user.rs` `check_credentials()` function already uses `&mut MysqlConnection`. The frontend `AuthContext`, `authApi`, and `LoginPage` components are fully implemented — they only need the endpoint URL changed from `/login` to `/auth/login`, and the router file needs modernization.

**Primary recommendation:** Port the auth system incrementally: (1) enable `auth` mod and wire Tokenizer into AppState, (2) add `Role` enum to replace `sys_role: String`, (3) build the `FromRequestParts` extractor replacing the `require_auth` middleware stub, (4) add `/auth/login` and `/auth/logout` routes, (5) update frontend URL and router API.

## Standard Stack

### Core

| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| jwt-simple | 0.11.9 (already in Cargo.toml) | Ed25519 JWT signing/verification | Already used; `Tokenizer` struct built on it; `Claims::with_custom_claims` handles custom payload |
| argon2 | 0.5.2 (already in Cargo.toml) | Password hashing and verification | Already used; `check_credentials()` already calls `Argon2::default().verify_password()` |
| axum | 0.8 (already in Cargo.toml) | HTTP framework + `FromRequestParts` trait | Extractor pattern replaces Rocket's `FromRequest` guard |
| serde | 1.0 (already in Cargo.toml) | Serialize/Deserialize for JWT claims and JSON responses | Standard |
| figment | 0.10 (already in Cargo.toml) | Config loading (token expiration via config.toml or env) | Already used for all config; `APP__AUTH__TOKEN_EXPIRATION_SECS` env var path naturally follows existing `APP__` prefix convention |

### Supporting

| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| axum-extra | must add (~0.10) | `TypedHeader<Authorization<Bearer>>` extractor | Use inside `FromRequestParts` impl to extract the Bearer token from headers in a type-safe way |
| tokio | 1 (already in Cargo.toml) | `tokio::time::sleep` for login delay | One-liner in login handler on failed credential check |

### Alternatives Considered

| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| Manual `FromSql`/`ToSql` for Role enum | diesel-derive-enum crate | diesel-derive-enum adds a dependency but reduces boilerplate; manual impl is ~30 lines and keeps deps minimal — prefer manual given the enum is small (2 variants + future) |
| axum-extra TypedHeader | Manual header parse (split "Bearer ") | Manual parsing is what the old Rocket guard.rs did and it worked, but TypedHeader is cleaner and already idiomatic for axum; adds axum-extra dep |
| figment config for expiration | Pure env var via `std::env::var` | Secrets are env-only (locked decision), but token expiration is not a secret — using figment config.toml with env override is the existing pattern for all non-secret config |

**Installation additions needed:**

```toml
# backend/Cargo.toml additions:
axum-extra = { version = "0.10", features = ["typed-header"] }
```

No other new dependencies — all auth logic uses already-present crates.

## Architecture Patterns

### Recommended Project Structure

```
backend/src/
├── auth/
│   ├── mod.rs          # pub use tokenizer::Tokenizer; pub use extractor::{AuthUser, AdminUser};
│   ├── tokenizer.rs    # existing — no changes needed
│   └── extractor.rs    # NEW: FromRequestParts impls for AuthUser and AdminUser
├── db/
│   └── user.rs         # Change sys_role: String -> sys_role: Role; add Role enum
├── routes/
│   ├── mod.rs          # uncomment login, add auth module
│   └── auth.rs         # NEW: POST /auth/login, POST /auth/logout handlers
├── config.rs           # Add AuthConfig struct with token_expiration_secs
├── state.rs            # Add tokenizer: Arc<Tokenizer> to AppState
└── lib.rs              # Uncomment auth mod, add /auth routes, replace require_auth stub
```

### Pattern 1: Axum FromRequestParts for JWT Extractor

**What:** Custom extractor that reads `AppState` from router state, extracts Bearer token from Authorization header, and calls `Tokenizer::verify()` to get `User`. `AdminUser` is a thin wrapper that additionally checks `Role::Admin`.

**When to use:** Any handler that needs an authenticated user declares `AuthUser(user): AuthUser` as a parameter. Admin-only handlers use `AdminUser(user): AdminUser`.

**Key mechanic — accessing AppState in extractor:** The extractor must be generic over `S` with a `FromRef<S, Target = AppState>` bound to access the tokenizer from the router state.

```rust
// Source: https://docs.rs/axum/latest/axum/extract/trait.FromRequestParts.html
// and https://github.com/tokio-rs/axum/discussions/1895 (AppState in extractor)

use axum::{
    async_trait,
    extract::{FromRef, FromRequestParts, State},
    http::{request::Parts, StatusCode},
};
use axum_extra::{
    headers::{authorization::Bearer, Authorization},
    TypedHeader,
};
use crate::{error::Error, state::AppState, db::user::User};

pub struct AuthUser(pub User);

#[async_trait]
impl<S> FromRequestParts<S> for AuthUser
where
    AppState: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = Error;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        // Extract app state (contains tokenizer)
        let State(app_state) = State::<AppState>::from_request_parts(parts, state)
            .await
            .map_err(|_| Error::Internal)?;

        // Extract Authorization: Bearer <token>
        let TypedHeader(Authorization(bearer)) =
            TypedHeader::<Authorization<Bearer>>::from_request_parts(parts, state)
                .await
                .map_err(|_| Error::Unauthorized)?;

        // Verify JWT — Tokenizer::verify() returns Result<User>
        let user = app_state.tokenizer.verify(bearer.token())?;
        Ok(AuthUser(user))
    }
}

pub struct AdminUser(pub User);

#[async_trait]
impl<S> FromRequestParts<S> for AdminUser
where
    AppState: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = Error;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let AuthUser(user) = AuthUser::from_request_parts(parts, state).await?;
        match user.sys_role {
            Role::Admin => Ok(AdminUser(user)),
            _ => Err(Error::ForbiddenAccess),
        }
    }
}
```

### Pattern 2: Login Handler with Artificial Delay

**What:** The login handler calls `check_credentials()` inside `deadpool-diesel`'s `interact()` closure. On failure, it awaits a short sleep before returning 401 to slow brute-force attempts. The delay should apply to both wrong-password and unknown-email cases (to prevent user enumeration).

```rust
// Source: existing db/user.rs pattern + tokio docs
use axum::{extract::State, Json};
use std::time::Duration;
use tokio::time::sleep;
use crate::{error::Error, state::AppState};

async fn post_login(
    State(state): State<AppState>,
    Json(login): Json<LoginRequest>,
) -> Result<Json<LoginResponse>, Error> {
    let email = login.email.clone();
    let password = login.password.clone();

    let result = state
        .db_pool
        .get()
        .await
        .map_err(|_| Error::Internal)?
        .interact(move |conn| {
            User::check_credentials(conn, &email, &password)
        })
        .await
        .map_err(|_| Error::Internal)?;

    match result {
        Ok(user) => {
            let token = state.tokenizer.generate(user)?;
            Ok(Json(LoginResponse { token }))
        }
        Err(_) => {
            // Artificial delay — do NOT leak which specific error occurred
            sleep(Duration::from_millis(500)).await;
            Err(Error::WrongCredentials)
        }
    }
}
```

**Note:** `sleep` inside an async Axum handler does NOT block other requests — Tokio's runtime handles concurrent requests on the executor while this future is parked.

### Pattern 3: Role Enum with Diesel MySQL and Serde

**What:** Replace `sys_role: String` in `User` struct with `sys_role: Role` enum. The DB column stays `Varchar(255)` — no migration needed. `FromSql`/`ToSql` handle the string ↔ enum round-trip. Unknown strings become an error (strict, as decided).

```rust
// Source: https://docs.diesel.rs/2.3.x/diesel/serialize/trait.ToSql.html
// and https://users.rust-lang.org/t/enum-in-diesel-orm-with-implemented-fromsql-and-tosql/117956

use diesel::{
    backend::Backend,
    deserialize::{self, FromSql, FromSqlRow},
    expression::AsExpression,
    mysql::{Mysql, MysqlValue},
    serialize::{self, IsNull, Output, ToSql},
    sql_types::Text,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, FromSqlRow, AsExpression)]
#[diesel(sql_type = Text)]
#[serde(rename_all = "lowercase")]  // "admin", "user" — matches DB values
pub enum Role {
    User,
    Admin,
    // Future roles added here — forces code update before DB can store new value
}

impl ToSql<Text, Mysql> for Role {
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, Mysql>) -> serialize::Result {
        let s = match self {
            Role::User => "user",
            Role::Admin => "admin",
        };
        <str as ToSql<Text, Mysql>>::to_sql(s, out)
    }
}

impl FromSql<Text, Mysql> for Role {
    fn from_sql(bytes: MysqlValue<'_>) -> deserialize::Result<Self> {
        let s = <String as FromSql<Text, Mysql>>::from_sql(bytes)?;
        match s.as_str() {
            "user" => Ok(Role::User),
            "admin" => Ok(Role::Admin),
            other => Err(format!("Unknown role: {other}").into()),
        }
    }
}
```

**Key insight:** `#[diesel(sql_type = Text)]` on the enum + `AsExpression` derive makes the enum work in Diesel queries directly. The `Text` SQL type maps to MySQL's `Varchar` without any schema change. The `MysqlValue` type is MySQL-specific — use `MysqlValue` not `PgValue`.

**JWT claims note:** The `User` struct (which is put into JWT custom claims) already derives `Serialize`/`Deserialize` — the `Role` enum with `#[serde(rename_all = "lowercase")]` will serialize as `"user"` or `"admin"` in the JWT payload automatically. The frontend `AuthContext` decodes `decodedToken.custom.sys_role` from the JWT — it will receive a lowercase string, which matches existing TypeScript types (though `types/index.ts` `sys_role: string` should be narrowed to a union type in this phase).

### Pattern 4: AppState with Tokenizer

**What:** `Tokenizer` needs to be in `AppState` so extractors can access it. `Tokenizer` is not `Clone` (contains private keys), so wrap in `Arc`.

```rust
// backend/src/state.rs
use std::sync::Arc;
use crate::{auth::Tokenizer, config::AppConfig};

pub type DbPool = deadpool_diesel::mysql::Pool;

#[derive(Clone)]
pub struct AppState {
    pub config: Arc<AppConfig>,
    pub db_pool: DbPool,
    pub tokenizer: Arc<Tokenizer>,  // Arc for Clone across handlers
}
```

### Pattern 5: Token Expiration in AppConfig

**What:** Add `AuthConfig` to `AppConfig` with `token_expiration_secs: u64`. Figment reads it from `config.toml` and/or `APP__AUTH__TOKEN_EXPIRATION_SECS` env var. Secrets (private keys, DB URL) stay env-only — token expiration is not a secret.

```rust
// backend/src/config.rs additions

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct AuthConfig {
    #[serde(default = "default_token_expiration_secs")]
    pub token_expiration_secs: u64,
}

fn default_token_expiration_secs() -> u64 {
    5 * 24 * 60 * 60  // 5 days — same as old Rocket default
}

impl Default for AuthConfig {
    fn default() -> Self {
        Self { token_expiration_secs: default_token_expiration_secs() }
    }
}

// In AppConfig:
pub struct AppConfig {
    pub server: ServerConfig,
    pub cors: CorsConfig,
    pub auth: AuthConfig,      // NEW
    pub log_level: String,
    pub environment: String,
}
```

**In binary startup:**
```rust
let exp = Duration::from_secs(state.config.auth.token_expiration_secs);
let tokenizer = Arc::new(Tokenizer::new(exp));
```

**config.toml addition:**
```toml
[auth]
token_expiration_secs = 432000  # 5 days
```

**Env var override:** `APP__AUTH__TOKEN_EXPIRATION_SECS=3600` (figment `Env::prefixed("APP__").split("__")` already handles nested config).

### Pattern 6: TanStack Router Migration

**What:** Current `router.tsx` uses deprecated class-based APIs (`new RootRoute`, `new Route`, `createReactRouter`). These work but will be removed in the next major version. This phase's frontend work is an opportunity to migrate to `createRootRoute`, `createRoute`, and `createRouter`.

**Auth URL change:** `authApi.ts` line `url: '/login'` → `url: '/auth/login'`. The logout stub: add `url: '/auth/logout'` call in `authApi.logout()` (currently only clears localStorage; should also fire the POST for future blacklisting).

**Modern router pattern (verified from TanStack Router v1 docs):**

```typescript
// Source: https://tanstack.com/router/v1/docs/framework/react/api/router/createRootRouteFunction
import { createRootRoute, createRoute, createRouter, redirect, Outlet } from '@tanstack/react-router';

const rootRoute = createRootRoute({
  component: () => <Outlet />,
});

const loginRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: '/login',
  component: LoginPage,
  beforeLoad: async () => {
    if (authApi.isAuthenticated()) {
      throw redirect({ to: '/' });
    }
  },
});

const authenticatedRoute = createRoute({
  getParentRoute: () => rootRoute,
  id: 'authenticated',
  beforeLoad: async () => {
    if (!authApi.isAuthenticated()) {
      throw redirect({ to: '/login' });
    }
  },
});

const router = createRouter({
  routeTree: rootRoute.addChildren([
    loginRoute,
    authenticatedRoute.addChildren([...]),
  ]),
});
```

**`createReactRouter` is replaced by `createRouter`** — the import changes but the API is identical.

### Anti-Patterns to Avoid

- **Don't wrap `require_auth` middleware AND use AuthUser extractor simultaneously** — the phase replaces `require_auth` with the extractor. The middleware stub in `middleware/auth.rs` is deleted in this phase; protected routes use `AuthUser` extractor instead.
- **Don't put JWT secret into `Tokenizer` for HMAC** — Ed25519 keypair is ephemeral (generated at startup), no secret needed in env. The locked decision keeps Ed25519.
- **Don't call `tokio::time::sleep` for successful logins** — delay only on failure. The sleep is unconditional-on-error; the success path returns immediately.
- **Don't define Role as a ENUM column in MySQL schema** — DB column is `Varchar(255)`, Role enum lives in Rust only. No migration needed.
- **Don't pass `&Tokenizer` directly into handler** — `Tokenizer` holds private key and is `!Clone`; must be `Arc<Tokenizer>` in `AppState`.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Bearer token header parsing | Manual `split("Bearer ")` | `axum_extra::TypedHeader<Authorization<Bearer>>` | Handles edge cases (missing header, malformed values, charset issues); idiomatic Axum |
| JWT signing/verification | HMAC or custom crypto | jwt-simple Ed25519 (already in Cargo.toml) | Already implemented in `tokenizer.rs`; no code change needed |
| Password verification | Re-implement hash comparison | `argon2::Argon2::default().verify_password()` (already in `db/user.rs`) | Already implemented; `check_credentials()` is correct |
| Brute-force protection | Rate limiting, IP tracking | `tokio::time::sleep(500ms)` on failed login | Sufficient for team-internal app; rate limiting (tower-governor) is v2 scope |

**Key insight:** The entire backend auth stack is already written in Rocket form. Phase 3 is a translation, not an implementation.

## Common Pitfalls

### Pitfall 1: Extractor Ordering and State Access

**What goes wrong:** `FromRequestParts` impl panics or fails to compile because `AppState` is not accessible from the generic state parameter `S`.

**Why it happens:** Axum extractors are generic over the router state type `S`. To access a concrete `AppState` from a generic `S`, you must bound `AppState: FromRef<S>` — not just `S: Send + Sync`.

**How to avoid:** Use the pattern `where AppState: FromRef<S>, S: Send + Sync` and extract state with `State::<AppState>::from_request_parts(parts, state).await`.

**Warning signs:** Compile error "the trait `FromRef<S>` is not implemented for `AppState`" or runtime panic extracting state.

### Pitfall 2: Role Enum Diesel SQL Type Mismatch

**What goes wrong:** `FromSql`/`ToSql` compile but Diesel query panics at runtime with deserialization error.

**Why it happens:** Using `#[diesel(sql_type = Text)]` requires that `Text` maps to the actual column type in the schema. The schema has `sys_role -> Varchar` which maps to `Text` in Diesel's MySQL backend — this is correct. The error would come from accidentally implementing `FromSql<SomeOtherType, Mysql>` instead of `FromSql<Text, Mysql>`.

**How to avoid:** Always match the Diesel SQL type in `FromSql`/`ToSql` impls to the type in `schema.rs`. The `AsExpression` derive with `#[diesel(sql_type = Text)]` ensures the expression type is correct for query composition.

**Warning signs:** "No implementation for `FromSql<Varchar, Mysql>` for Role" — note Diesel uses `Text` internally for varchar in MySQL.

### Pitfall 3: JWT Claims with Role Enum Deserialization

**What goes wrong:** An old JWT token (with `sys_role: "user"` string) fails to deserialize when the `User` struct's `sys_role` field type changes to `Role` enum.

**Why it happens:** jwt-simple stores custom claims as JSON. When `User` struct changes `sys_role: String` to `sys_role: Role`, old tokens that were signed when `sys_role` was a `String` field will still deserialize correctly because the JWT JSON contains `"user"` or `"admin"` and the `Role` enum's `#[serde(rename_all = "lowercase")]` accepts those strings. **This is not actually a problem** — but if an old token contained a role value that's not in the enum, verification returns an error (which is the intended strict behavior).

**How to avoid:** No action needed — the strict deserialization is the desired behavior. On server restart (new keypair), all old tokens are invalidated anyway.

**Warning signs:** Not a real pitfall — document it as a non-issue to prevent over-engineering.

### Pitfall 4: Frontend — `jwt_decode` Claim Structure

**What goes wrong:** `AuthContext.tsx` decodes the token as `jwt_decode<{ custom: User }>(token)` — the `custom` field name must match how jwt-simple structures the JWT payload.

**Why it happens:** jwt-simple wraps custom claims under the `custom` key in the JWT JSON payload (it's jwt-simple's convention, not standard JWT). The existing `AuthContext` already uses `decodedToken.custom`, so this is already correct.

**How to avoid:** No change needed to `AuthContext.tsx`. Just ensure that when `sys_role` becomes a `Role` enum on the backend, it serializes as `"user"` or `"admin"` string in the JWT (confirmed — `#[serde(rename_all = "lowercase")]` handles this). The frontend `User` type's `sys_role: string` may be narrowed to `'user' | 'admin'` optionally.

**Warning signs:** Frontend shows `decodedToken.custom` as `undefined` after migration — would indicate a jwt-simple version mismatch in payload structure.

### Pitfall 5: TanStack Router — `createReactRouter` Does Not Exist in v1.115

**What goes wrong:** The current `router.tsx` imports `createReactRouter` from `@tanstack/react-router` — this function no longer exists in the installed version (1.115.x).

**Why it happens:** The router was set up with an older API. Version 1.x replaced `createReactRouter` with `createRouter` and class-based `RootRoute`/`Route` with `createRootRoute`/`createRoute` functions.

**How to avoid:** Rewrite `router.tsx` using `createRouter`, `createRootRoute`, and `createRoute`. The `beforeLoad` / `redirect` / `Outlet` patterns are identical — only the route/router creation functions change.

**Warning signs:** TypeScript error "Module '@tanstack/react-router' has no exported member 'createReactRouter'" — already present in the codebase, confirms migration is needed.

### Pitfall 6: Axum Route Ordering — Public vs Protected

**What goes wrong:** Adding `/auth/login` to the protected router (with `route_layer(auth middleware)`) instead of the public router causes a circular dependency where login itself requires a token.

**How to avoid:** Mount `/auth/login` and `/auth/logout` on the **public** router (or a separate `auth_router` that does not have the auth route_layer). The `AuthUser` extractor handles authorization at the handler level for routes that need it.

**Warning signs:** `POST /auth/login` returns 401 before the handler even runs.

## Code Examples

### App Router Wiring with Tokenizer in State

```rust
// backend/src/lib.rs — updated app() function
pub fn app(state: AppState) -> Router {
    let cors = cors_layer(&state.config);

    let public = Router::new()
        .route("/health", get(routes::health::health))
        .route("/auth/login", post(routes::auth::login))
        .route("/auth/logout", post(routes::auth::logout));

    let protected = Router::new()
        .route("/api/ping", get(routes::ping::ping));
        // future routes added here — AuthUser extractor handles per-handler auth

    Router::new()
        .merge(public)
        .merge(protected)
        .layer(middleware_stack)
        .with_state(state)
}
```

**Note:** The `route_layer(require_auth)` on the protected router is **removed**. Authentication is now handled by the `AuthUser` extractor on individual handlers that need it. Handlers that don't use `AuthUser` are effectively public. This is a more granular and idiomatic Axum approach.

### Integration Test Pattern for Login

```rust
// backend/tests/auth.rs — additions
#[tokio::test]
async fn login_with_valid_credentials_returns_token() {
    // Requires live DB — use #[ignore] or separate integration test binary
    // Pattern from existing tests/auth.rs using axum::body::Body + tower::ServiceExt
}

#[tokio::test]
async fn login_with_invalid_credentials_returns_401() {
    // Can test without DB by verifying the error response shape
}
```

### Frontend Auth API Update

```typescript
// frontend/src/api/authApi.ts — url change
login: async (credentials: LoginRequest): Promise<LoginResponse> => {
    return await apiRequest<LoginResponse>({
        method: 'POST',
        url: '/auth/login',  // was '/login'
        data: credentials,
    });
},

logout: async (): Promise<void> => {
    try {
        await apiRequest({ method: 'POST', url: '/auth/logout' });
    } finally {
        localStorage.removeItem('auth_token');
    }
},
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Rocket `FromRequest` guard for auth | Axum `FromRequestParts` extractor | This phase | Replaces `guard.rs`; `require_auth` middleware stub removed |
| `sys_role: String` raw string | `sys_role: Role` typed enum | This phase | AUTH-04; Diesel + Serde round-trip without DB migration |
| `createReactRouter` / `new RootRoute` | `createRouter` / `createRootRoute` | TanStack Router ~v1.0 (2023) | Frontend router.tsx must be updated |
| `/login` endpoint | `/auth/login` + `/auth/logout` | This phase | URL namespace change; frontend authApi.ts must update |
| Presence-only auth check in `require_auth` | JWT signature + expiry verification in extractor | This phase | Phase 1 stub is replaced by real verification |

**Deprecated/outdated:**
- `backend/src/middleware/auth.rs` `require_auth()`: replaced by `AuthUser` extractor — file remains but `require_auth` is no longer used as a `route_layer`
- `backend/src/guard.rs`: Rocket-specific, stays commented out / can be deleted after port
- `router.tsx` `createReactRouter`, `new RootRoute`, `new Route`: deprecated in TanStack Router v1; must migrate to function-based API

## Open Questions

1. **Login response shape: token-only vs token+user**
   - What we know: Current frontend `AuthContext` decodes user from the JWT itself (`jwt_decode`), so the response today is just `{ token: string }` and that works
   - What's unclear: Token-only is simpler but requires a decode step on the client; token+user is slightly larger response but avoids the decode dependency
   - Recommendation: Keep `{ token: string }` response (matches current frontend types, `LoginResponse` in `types/index.ts` already defined this way) — Claude's discretion to decide during planning

2. **Middleware scope: Remove `route_layer(require_auth)` entirely or keep for belt-and-suspenders?**
   - What we know: The `AuthUser` extractor performs full JWT validation per-handler; the old `route_layer` was presence-only check
   - What's unclear: Whether to keep a lightweight route-level check as a defense-in-depth layer in addition to the extractor
   - Recommendation: Remove the `route_layer` approach entirely — the `AuthUser` extractor IS the auth layer, and having both would be redundant and confusing. Handlers that don't declare `AuthUser` are intentionally public.

3. **`/api` prefix for login endpoint or not?**
   - What we know: Decided path is `/auth/login` (not `/login`); unclear if it should be `/api/auth/login`
   - What's unclear: Whether auth routes should live under `/api` prefix for consistency
   - Recommendation: Keep `/auth/login` at root (not `/api/auth/login`) — auth is not a resource API, it's a separate concern. Frontend already uses root-level login redirect path `/login`.

## Sources

### Primary (HIGH confidence)

- Codebase direct read — `backend/src/auth/tokenizer.rs`, `backend/src/guard.rs`, `backend/src/routes/login.rs`, `backend/src/db/user.rs`, `backend/src/middleware/auth.rs`, `backend/src/state.rs`, `backend/src/config.rs`, `backend/src/lib.rs` — authoritative source for existing implementation
- `frontend/src/contexts/AuthContext.tsx`, `frontend/src/api/authApi.ts`, `frontend/src/router.tsx`, `frontend/src/types/index.ts` — authoritative source for frontend auth state
- https://docs.rs/jwt-simple/latest/jwt_simple/ — jwt-simple API, `Claims::with_custom_claims`, Ed25519 verification pattern
- https://github.com/tokio-rs/axum/blob/main/examples/jwt/src/main.rs — official Axum JWT example showing `FromRequestParts` pattern
- https://docs.rs/axum-extra/latest/axum_extra/struct.TypedHeader.html — TypedHeader API and typed-header feature requirement

### Secondary (MEDIUM confidence)

- https://users.rust-lang.org/t/enum-in-diesel-orm-with-implemented-fromsql-and-tosql/117956 — Diesel enum FromSql/ToSql pattern (Pg-focused but pattern applies to MySQL with `MysqlValue`)
- https://docs.diesel.rs/2.3.x/diesel/serialize/trait.ToSql.html — ToSql trait for MySQL (verified in official Diesel docs)
- TanStack Router v1 docs (authenticated routes) — `createRootRoute`/`createRoute`/`createRouter` migration from deprecated class-based API; `beforeLoad`/`redirect` pattern confirmed current

### Tertiary (LOW confidence)

- https://github.com/tokio-rs/axum/discussions/1895 — AppState in extractor using `FromRef` pattern (community discussion, not official docs — but pattern is standard Axum idiom verified by docs.rs)

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — all libraries already in Cargo.toml; no new dependencies except axum-extra
- Architecture patterns: HIGH — Tokenizer is framework-agnostic and directly portable; extractor pattern verified from axum official example; Role enum pattern verified from Diesel docs
- Pitfalls: HIGH for backend (based on direct codebase analysis); MEDIUM for frontend (TanStack Router API migration based on official docs)
- Frontend router migration: MEDIUM — confirmed deprecated API and replacement functions from docs, but full rewrite needs care with route tree structure

**Research date:** 2026-02-22
**Valid until:** 2026-03-22 (30 days — stable libraries, no fast-moving dependencies)
