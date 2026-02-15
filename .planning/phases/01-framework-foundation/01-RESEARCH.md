# Phase 1: Framework Foundation - Research

**Researched:** 2026-02-15
**Domain:** Axum 0.8, tower-http 0.6, tracing-subscriber, figment configuration
**Confidence:** HIGH

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

#### Configuration style
- TOML config file as base, env vars override specific values
- Single `config.toml` for defaults; production overrides via env vars (no per-environment files)
- Load order: config.toml → env var overrides → dotenvy .env in dev
- Config file format: TOML (Rust ecosystem standard)

#### API error format
- Detailed response shape: `{code, message, details}`
- Error codes are uppercase snake_case strings (e.g., `VALIDATION_ERROR`, `NOT_FOUND`)
- Messages always in English from API; frontend handles localization if needed
- Validation errors include field-level detail: `details: [{field: "email", reason: "invalid_format"}]`
- 500 errors: include error chain in development, generic message + request ID in production

#### Logging behavior
- Pretty (human-readable, colored) format in development, JSON structured in production
- Per-request info: method, path, status code, duration, and unique request ID
- Default log level: INFO (overridable via env var)
- Request/response body logging available at TRACE level only — never in production

#### CORS policy
- Wildcard (`*`) allowed in development for easy frontend dev
- Explicit origin list from config in production
- Credentials allowed (`Access-Control-Allow-Credentials: true`) for JWT Authorization header

### Claude's Discretion
- Secrets handling: whether sensitive values (DB password, JWT secret) are env-only or allowed in config file — pick the safer approach
- CORS allowed methods: pick based on what the REST API will actually use
- Exact config crate choice and TOML parsing approach
- CORS preflight cache max-age: balance dev flexibility with production performance

### Deferred Ideas (OUT OF SCOPE)
None — discussion stayed within phase scope
</user_constraints>

---

## Summary

This phase replaces Rocket 0.5 with Axum 0.8 as the HTTP framework for the zeiterfassung backend. Axum 0.8 was released January 2025 and is the current stable version (0.8.8 on crates.io as of research date). The migration requires replacing Rocket-specific constructs — fairings, guards, catchers — with Axum equivalents: tower layers, `from_fn` middleware, and `IntoResponse` implementations. The existing tracing/logging setup, Diesel ORM, and schema are preserved.

The middleware stack is built on `tower-http` 0.6, which provides `TraceLayer` (per-request structured logging), `CorsLayer` (CORS headers), and `SetRequestIdLayer`/`PropagateRequestIdLayer` (unique x-request-id per request). Configuration is handled by `figment` 0.10, which natively supports the decided TOML-base + env-var-override pattern. Auth in this phase is a skeleton only: `middleware::from_fn` extracts the Bearer token, validates it is non-empty, and returns 401 if absent — no real JWT validation yet.

**Critical CORS finding:** The user decided wildcard origin (`*`) in development AND `allow_credentials(true)`. These are mutually exclusive in the CORS specification — browsers reject `Access-Control-Allow-Origin: *` combined with `Access-Control-Allow-Credentials: true`. The correct approach for development is to use an allow-any-origin function (echo the request's `Origin` header) rather than the literal `*` wildcard. This is documented in MDN and must be handled in the implementation.

**Primary recommendation:** Use Axum 0.8.8 + tower-http 0.6.8 + figment 0.10.19 + dotenvy 0.15.7. Build the CORS layer with `allow_origin` using a dynamic origin function in dev (not `Any`), and use `ServiceBuilderExt::set_x_request_id` + `PropagateRequestIdLayer` for request ID tracking.

---

## Standard Stack

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| axum | 0.8.8 | HTTP routing and request handling | Locked decision; active development, tight Tokio integration |
| tokio | 1.49.0 | Async runtime | Required by Axum; de facto Rust async standard |
| tower | 0.5.x | Middleware composition | Axum's middleware system IS Tower; `ServiceBuilder` required |
| tower-http | 0.6.8 | HTTP-specific middleware (CORS, tracing, request-id) | Official companion crate for Axum |
| tracing | 0.1.x | Structured logging instrumentation | Locked decision; already in codebase |
| tracing-subscriber | 0.3.x | Log output (pretty/JSON) | Locked decision; already in codebase |
| figment | 0.10.19 | TOML + env var configuration | Supports decided load order natively; better than config-rs for this use case |
| dotenvy | 0.15.7 | Load .env file in development | Locked decision; well-maintained dotenv fork |
| serde | 1.x | Serialization for config structs and JSON responses | Required by figment, axum Json extractor |
| thiserror | 1.x | Error type derivation | Already in codebase; use for new Error type |

### Supporting
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| uuid | 1.21.0 | Generate x-request-id values | Used with `tower_http::request_id::MakeRequestUuid` |
| axum-extra | 0.10.x | TypedHeader extractor, additional extractors | When typed header extraction is needed |
| tower-http (request-id feature) | 0.6.8 | Built-in x-request-id generation | Use `ServiceBuilderExt::set_x_request_id` |

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| figment | config-rs 0.15 | config-rs is more widely used but figment has better TOML semantics and cleaner merge/join distinction |
| figment | envy + custom TOML | More manual; figment already does this |
| tower-http CorsLayer | axum_cors | axum_cors is experimental; tower-http is official |
| `middleware::from_fn` auth | `FromRequestParts` custom extractor | `from_fn` is the right pattern for middleware that blocks entire router; `FromRequestParts` is for per-handler extraction |

**Installation:**
```toml
[dependencies]
axum = "0.8"
tokio = { version = "1", features = ["full"] }
tower = "0.5"
tower-http = { version = "0.6", features = ["cors", "trace", "request-id"] }
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter", "fmt", "json"] }
figment = { version = "0.10", features = ["toml", "env"] }
dotenvy = "0.15"
serde = { version = "1", features = ["derive"] }
thiserror = "1"
uuid = { version = "1", features = ["v4"] }
```

---

## Architecture Patterns

### Recommended Project Structure
```
backend/src/
├── bin/
│   └── backend.rs       # main() — tokio::main, loads config, starts server
├── config.rs            # Config struct, figment loading, AppConfig
├── state.rs             # AppState struct (shared across handlers)
├── error.rs             # Error enum, IntoResponse impl for error JSON
├── tracing.rs           # Subscriber init (pretty dev / JSON prod)
├── middleware/
│   └── auth.rs          # Bearer extraction skeleton, 401 response
├── routes/
│   ├── mod.rs           # Router assembly: public_router() + protected_router()
│   └── health.rs        # GET /health → 200 OK
└── lib.rs               # pub mod declarations, app() builder fn
```

### Pattern 1: AppState with Arc
**What:** Wrap shared application state in `Arc<AppState>` and attach to the router via `Router::with_state()`.
**When to use:** Any data that needs to be accessed from handlers — config, DB pool, tokenizer.
**Example:**
```rust
// Source: https://docs.rs/axum/latest/axum/ (AppState pattern)
#[derive(Clone)]
pub struct AppState {
    pub config: Arc<AppConfig>,
    // db pool and tokenizer added in later phases
}

pub fn app(state: AppState) -> Router {
    let public = Router::new()
        .route("/health", get(routes::health::handler));

    let protected = Router::new()
        .route("/api/ping", get(routes::ping::handler))
        .route_layer(middleware::from_fn_with_state(
            state.clone(),
            middleware::auth::require_auth,
        ));

    Router::new()
        .merge(public)
        .merge(protected)
        .layer(middleware_stack())
        .with_state(state)
}
```

### Pattern 2: Middleware Stack with ServiceBuilder
**What:** Apply tower middleware layers in order using `ServiceBuilder`. Ordering matters: middleware added later in the builder wraps earlier middleware.
**When to use:** Attaching global middleware (CORS, tracing, request-id) to the entire router.
**Example:**
```rust
// Source: axum/tower-http docs — ServiceBuilder applies outermost-last
use tower::ServiceBuilder;
use tower_http::{
    cors::CorsLayer,
    trace::TraceLayer,
    request_id::{MakeRequestUuid, PropagateRequestIdLayer, SetRequestIdLayer},
};

fn middleware_stack(cors: CorsLayer) -> impl Layer<Router> {
    ServiceBuilder::new()
        .layer(SetRequestIdLayer::x_request_id(MakeRequestUuid::default()))
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(|request: &Request<_>| {
                    let request_id = request
                        .headers()
                        .get("x-request-id")
                        .and_then(|v| v.to_str().ok())
                        .unwrap_or("unknown");
                    tracing::info_span!(
                        "http_request",
                        method = %request.method(),
                        uri = %request.uri(),
                        request_id = request_id,
                    )
                })
                .on_response(DefaultOnResponse::new().level(Level::INFO))
        )
        .layer(cors)
        .layer(PropagateRequestIdLayer::x_request_id())
}
```

### Pattern 3: CORS for Dev vs Production
**What:** In development, echo the request Origin header (not literal `*`) when credentials are required. In production, use an explicit allow-list from config.
**When to use:** Both environments require `allow_credentials(true)`, so `Any` wildcard is not usable.

**Critical constraint:** `allow_origin(Any)` + `allow_credentials(true)` is rejected by browsers (MDN spec). In development the correct approach is `allow_origin` with a function that echoes the request origin.

```rust
// Source: tower-http CorsLayer docs + MDN CORS spec
use tower_http::cors::{AllowOrigin, CorsLayer, Any};
use axum::http::{HeaderValue, Method, header};
use std::time::Duration;

fn cors_layer(config: &AppConfig) -> CorsLayer {
    let origins = if config.is_development() {
        // Echo the request origin — allows any origin WITH credentials
        AllowOrigin::mirror_request()
    } else {
        // Explicit list from config
        AllowOrigin::list(
            config.cors.allowed_origins.iter()
                .map(|o| o.parse::<HeaderValue>().expect("invalid origin"))
        )
    };

    CorsLayer::new()
        .allow_origin(origins)
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::PUT,
            Method::DELETE,
            Method::OPTIONS,
        ])
        .allow_headers([
            header::AUTHORIZATION,
            header::CONTENT_TYPE,
            header::ACCEPT,
        ])
        .allow_credentials(true)
        .max_age(if config.is_development() {
            Duration::from_secs(0)       // no caching in dev
        } else {
            Duration::from_secs(600)     // 10 minutes in production
        })
}
```

### Pattern 4: Auth Middleware Skeleton
**What:** `middleware::from_fn` that extracts the Bearer token and returns 401 if missing/malformed. No real JWT validation in this phase.
**When to use:** Applied with `route_layer()` on the protected router.
```rust
// Source: https://docs.rs/axum/latest/axum/middleware/fn.from_fn.html
use axum::{
    extract::Request,
    http::{header, StatusCode},
    middleware::Next,
    response::Response,
};

pub async fn require_auth(request: Request, next: Next) -> Result<Response, StatusCode> {
    let auth_header = request
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok());

    match auth_header {
        Some(h) if h.starts_with("Bearer ") => {
            let _token = h.trim_start_matches("Bearer ").trim();
            // Phase 1: presence check only — JWT validation in Phase 2
            if _token.is_empty() {
                return Err(StatusCode::UNAUTHORIZED);
            }
            Ok(next.run(request).await)
        }
        _ => Err(StatusCode::UNAUTHORIZED),
    }
}
```

### Pattern 5: IntoResponse Error Type
**What:** Implement `IntoResponse` on the app Error type to produce consistent `{code, message, details}` JSON.
**When to use:** All handler return types use `Result<T, AppError>`.
```rust
// Source: https://docs.rs/axum/latest/axum/response/trait.IntoResponse.html
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;

#[derive(Serialize)]
pub struct ErrorBody {
    pub code: &'static str,
    pub message: String,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub details: Vec<FieldError>,
}

#[derive(Serialize)]
pub struct FieldError {
    pub field: String,
    pub reason: String,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, body) = self.to_response_parts();
        (status, Json(body)).into_response()
    }
}
```

### Pattern 6: Figment Configuration
**What:** TOML file as base, env vars (prefixed `APP__`) override, dotenvy loads `.env` in dev.
**When to use:** Application startup in `main()`.
```rust
// Source: https://docs.rs/figment/latest/figment/
use figment::{Figment, providers::{Env, Format, Toml, Serialized}};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct AppConfig {
    pub server: ServerConfig,
    pub cors: CorsConfig,
    pub log_level: String,
    pub environment: String,
    // NO secrets here — DB password and JWT secret from env only
}

pub fn load_config() -> Result<AppConfig, figment::Error> {
    Figment::new()
        .merge(Toml::file("config.toml"))
        .merge(Env::prefixed("APP__").split("__"))
        .extract()
}
```

### Anti-Patterns to Avoid
- **`Router::layer` vs `Router::route_layer` confusion:** Use `route_layer` for auth middleware — it only runs when a route matches, preventing 404 responses from triggering auth checks.
- **Wildcard CORS + credentials:** Never use `allow_origin(Any)` with `allow_credentials(true)`. Use `AllowOrigin::mirror_request()` for development.
- **Attaching middleware in wrong order:** Middleware added first in `ServiceBuilder` wraps outermost. Request-ID must be set before TraceLayer creates its span to include it in span fields.
- **Putting secrets in config.toml:** Database passwords and JWT secret keys must ONLY come from env vars. `config.toml` should have placeholder/empty values for those fields that must be overridden.
- **Using `#[async_trait]` on extractors in Axum 0.8:** Axum 0.8 removed the need for `#[async_trait]` on `FromRequestParts`/`FromRequest` implementations.
- **Old path syntax `/:param`:** Axum 0.8 uses `/{param}` syntax. The old `/:param` syntax is a breaking change that silently fails routing.

---

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Unique request ID per request | Custom UUID middleware | `tower_http::request_id::SetRequestIdLayer` + `MakeRequestUuid` | Built into tower-http; propagates to response headers automatically |
| CORS header management | Manual header insertion | `tower_http::cors::CorsLayer` | Handles preflight, vary headers, credentials, max-age correctly |
| Per-request trace spans | Manual `tracing::info!` in each handler | `tower_http::trace::TraceLayer` | Automatically wraps each request in a span with timing |
| Config merging with env override | Custom config loading | `figment` | Handles TOML parse, env var override, type extraction, and error provenance |
| .env file loading | `std::env::set_var` calls | `dotenvy::dotenv()` | Standard approach; handles file-not-found gracefully |
| JSON error responses | Custom error serialization | `axum::Json` + `IntoResponse` | Axum's type system guarantees all errors produce responses |

**Key insight:** Tower's composable layer system handles all cross-cutting concerns. Resist the urge to implement any middleware manually — the tower-http crate has production-tested implementations for every HTTP middleware this phase needs.

---

## Common Pitfalls

### Pitfall 1: CORS Wildcard + Credentials Combination
**What goes wrong:** Setting `allow_origin(Any)` and `allow_credentials(true)` produces `Access-Control-Allow-Origin: *` with `Access-Control-Allow-Credentials: true`, which browsers reject with a CORS error. Frontend requests with cookies or Authorization headers fail silently.
**Why it happens:** The CONTEXT.md specifies both "wildcard in development" and "credentials allowed" — these are browser-spec incompatible.
**How to avoid:** Use `AllowOrigin::mirror_request()` instead of `Any` in development. This echoes the request's `Origin` header as `Access-Control-Allow-Origin`, which satisfies both the "allow any origin" goal and the credentials requirement.
**Warning signs:** Browser console shows `CORSNotSupportingCredentials` error; axios/fetch requests with `withCredentials: true` fail.

### Pitfall 2: Axum 0.8 Path Syntax Breaking Change
**What goes wrong:** Routes defined with old Rocket-style or Axum 0.7 syntax `/:id` don't match; handlers are never called and 404 is returned for all parameterized routes.
**Why it happens:** Axum 0.8 is a breaking change — path parameters changed from `/:single` to `/{single}`.
**How to avoid:** Use only `/{single}` and `/{*many}` syntax in all route definitions from the start.
**Warning signs:** Routes compile but 404 is returned for parameterized paths.

### Pitfall 3: Middleware Ordering (Request-ID Must Precede TraceLayer)
**What goes wrong:** The request ID is not included in trace spans because TraceLayer creates its span before SetRequestIdLayer sets the ID.
**Why it happens:** `ServiceBuilder` applies middleware in declaration order, but HTTP requests flow through them in the reverse order (outermost last in builder = innermost first for requests).
**How to avoid:** In `ServiceBuilder`, place `SetRequestIdLayer` before `TraceLayer`. The builder's `.layer()` calls wrap from outside in, so the first `.layer()` call processes the request first.
**Warning signs:** `request_id` field is missing from span fields or shows "unknown".

### Pitfall 4: `route_layer` vs `layer` for Auth Middleware
**What goes wrong:** Using `Router::layer` for auth means the auth middleware runs even for routes that don't exist, potentially returning 401 instead of 404 for unknown paths.
**Why it happens:** `Router::layer` wraps the entire router including the 404 fallback path, while `Router::route_layer` only runs when a route matches.
**How to avoid:** Apply auth middleware with `Router::route_layer` on the protected router.
**Warning signs:** Requests to non-existent protected routes return 401 instead of 404.

### Pitfall 5: Secrets in config.toml Committed to Git
**What goes wrong:** Database password or JWT secret appears in config.toml and gets committed to the repository.
**Why it happens:** It's convenient to put all config in one file.
**How to avoid:** Config struct fields for `db_password` and `jwt_secret` should have no defaults and must be set via env vars only. The `config.toml` should document these as "must be set via environment" with empty/placeholder values. Add `config.toml` to `.gitignore` or use a `config.example.toml`.
**Warning signs:** `figment` succeeds loading config with empty secrets — add startup validation.

### Pitfall 6: Removing `#[async_trait]` in Axum 0.8
**What goes wrong:** Old code uses `#[async_trait]` on `FromRequestParts` implementations. Axum 0.8 no longer uses this attribute.
**Why it happens:** Axum 0.7 required `#[async_trait]`; Axum 0.8 uses return-position `impl Trait` in traits (RPITIT).
**How to avoid:** When porting the existing `guard.rs` Rocket guard to Axum, implement `FromRequestParts` directly without `#[async_trait]`.
**Warning signs:** Compiler errors about trait implementations, or `async_trait` dependency being unused.

---

## Code Examples

Verified patterns from official sources:

### Health Check Handler
```rust
// Source: https://docs.rs/axum/latest/axum/ (basic handler)
use axum::http::StatusCode;

pub async fn health() -> StatusCode {
    StatusCode::OK
}
```

### Router Assembly (Public + Protected Split)
```rust
// Source: https://docs.rs/axum/latest/axum/struct.Router.html (merge pattern)
use axum::{routing::get, Router, middleware};

pub fn app(state: AppState) -> Router {
    let public_routes = Router::new()
        .route("/health", get(routes::health::health));

    let protected_routes = Router::new()
        .route("/api/users/me", get(routes::user::me))
        .route_layer(middleware::from_fn_with_state(
            state.clone(),
            middleware::auth::require_auth,
        ));

    Router::new()
        .merge(public_routes)
        .merge(protected_routes)
        .with_state(state)
}
```

### Tracing Subscriber — Dev Pretty / Prod JSON
```rust
// Source: https://docs.rs/tracing-subscriber/latest/tracing_subscriber/
// (registry + layered approach for conditional formatting)
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

pub fn init_tracing(is_production: bool) {
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info"));

    let registry = tracing_subscriber::registry().with(env_filter);

    if is_production {
        registry
            .with(tracing_subscriber::fmt::layer().json())
            .init();
    } else {
        registry
            .with(tracing_subscriber::fmt::layer().pretty())
            .init();
    }
}
```

### Figment Config Load Order
```rust
// Source: https://docs.rs/figment/latest/figment/
// TOML base → env var overrides → dotenvy for dev
use figment::{Figment, providers::{Env, Format, Toml}};

pub fn load() -> AppConfig {
    // dotenvy should be called BEFORE figment so .env vars are in std::env
    let _ = dotenvy::dotenv(); // silently ignore if .env missing

    Figment::new()
        .merge(Toml::file("config.toml"))
        .merge(Env::prefixed("APP__").split("__"))
        .extract()
        .expect("configuration error")
}
```

### Justfile — Complete Command Set
```makefile
# Based on existing justfile + adding missing commands for this phase
alias c := check
alias ct := compile-test
alias r := run
alias t := nextest
alias b := build

_default:
    @just --list

fmt *args='--check':
    cargo +nightly fmt {{ if args == "--write" { "" } else if args == "-w" { "" } else { args } }}

build:
    cargo build

run *args:
    cargo run {{ args }}

check *args:
    cargo clippy {{ if args == "-t" { "--all-targets" } else { args } }}

test *args:
    cargo test {{ args }}

nextest *args:
    cargo nextest run {{ args }}

compile-test *args:
    cargo test --no-run {{ args }}
```

---

## State of the Art

| Old Approach (Rocket 0.5) | New Approach (Axum 0.8) | Impact |
|--------------------------|-------------------------|--------|
| `Fairing` for middleware | `Tower Layer` via `ServiceBuilder` | Composable, standard Tower ecosystem |
| `Request Guard` (FromRequest) | `FromRequestParts` (no async_trait) | Simpler, no macro dependency |
| `rocket_cors::CorsOptions` | `tower_http::cors::CorsLayer` | More explicit configuration |
| `catchers!` for error handling | `impl IntoResponse` + fallback handler | Type-safe, no magic macros |
| `rocket_db_pools::Database` | State<Arc<Pool>> (Phase 2) | Explicit state sharing |
| `#[rocket::main]` | `#[tokio::main]` | Standard Tokio runtime |
| `Rocket::build().attach()` | `Router::new().layer()` | Tower-native composition |

**Deprecated/outdated from existing codebase:**
- `rocket`, `rocket_cors`, `rocket_db_pools`: All removed in this phase
- `anyhow`: Can be retained for internal error chains, but API errors use thiserror
- `jwt-simple 0.11.9`: Kept for Phase 2; not wired in Phase 1
- `tracing_appender` (file logging to `/var/log/`): Remove for this phase — stdout only, let the deployment environment handle log aggregation

---

## Open Questions

1. **Rust edition and minimum MSRV for Axum 0.8**
   - What we know: Axum 0.8 was released January 2025; uses RPITIT which requires Rust 1.75+
   - What's unclear: What exact Rust version is installed in this project's CI/dev environment
   - Recommendation: Add `rust-version = "1.75"` to Cargo.toml; verify with `rustup show`

2. **`config.toml` in git vs gitignored**
   - What we know: Config file contains non-secret values (server port, CORS origins, log level); secrets go to env vars
   - What's unclear: Whether the project wants to commit a `config.toml.example` or commit `config.toml` directly
   - Recommendation: Commit `config.toml` with only non-secret defaults; document that `APP__DB__PASSWORD` etc. must be env vars; add startup assertion that required secrets are non-empty

3. **Existing justfile `fmt` command uses nightly**
   - What we know: Current justfile uses `cargo +nightly fmt`
   - What's unclear: Whether nightly is available in all development environments
   - Recommendation: Preserve the nightly requirement (it's already in the justfile); document it as a prerequisite

---

## Sources

### Primary (HIGH confidence)
- `cargo search axum` — confirmed version 0.8.8
- `cargo search tower-http` — confirmed version 0.6.8
- `cargo search figment` — confirmed version 0.10.19
- `cargo search dotenvy` — confirmed version 0.15.7
- `cargo search uuid` — confirmed version 1.21.0
- https://tokio.rs/blog/2025-01-01-announcing-axum-0-8-0 — Axum 0.8 breaking changes (path syntax, async_trait removal)
- https://docs.rs/tower-http/latest/tower_http/ — TraceLayer, CorsLayer, request_id layer list and features
- https://docs.rs/tower-http/latest/tower_http/request_id/index.html — SetRequestIdLayer, MakeRequestUuid, PropagateRequestIdLayer, x-request-id header
- https://docs.rs/axum/latest/axum/middleware/fn.from_fn.html — from_fn signature, pattern for auth middleware
- https://docs.rs/axum/latest/axum/ — AppState pattern, Router::with_state, IntoResponse
- https://docs.rs/figment/latest/figment/ — merge vs join, Env::prefixed, TOML + env overlay
- https://developer.mozilla.org/en-US/docs/Web/HTTP/Guides/CORS/Errors/CORSNotSupportingCredentials — CORS credentials + wildcard spec restriction

### Secondary (MEDIUM confidence)
- WebSearch verified: `AllowOrigin::mirror_request()` as the correct tower-http solution for allowing all origins with credentials — confirmed browser spec via MDN
- WebSearch verified: `ServiceBuilderExt::set_x_request_id()` convenience method on ServiceBuilder

### Tertiary (LOW confidence)
- CORS `max_age` of 600 seconds for production: reasonable industry default but not sourced from a definitive authority; planner should validate

---

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — all versions verified via `cargo search` on crates.io
- Architecture: HIGH — patterns sourced from official Axum and tower-http docs
- CORS wildcard/credentials pitfall: HIGH — sourced from MDN spec documentation
- Pitfalls: HIGH for Axum 0.8 breaking changes (official announcement); MEDIUM for operational pitfalls
- Figment configuration: HIGH — verified against official figment docs

**Research date:** 2026-02-15
**Valid until:** 2026-03-15 (stable libraries; Axum 0.8.x patch releases unlikely to break patterns)
