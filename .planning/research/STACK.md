# Stack Research

**Domain:** Rust Axum web application with SPA frontend — time tracking rewrite
**Researched:** 2026-02-13
**Confidence:** MEDIUM-HIGH (training data cross-checked against Cargo.lock; web tools unavailable for live crate verification)

---

## Recommended Stack

### Core Technologies

| Technology | Version | Purpose | Why Recommended |
|------------|---------|---------|-----------------|
| axum | 0.8.x | HTTP web framework | Tokio-native, actively developed by the Tokio team; Rocket 0.5 is slower to evolve; Axum's extractor pattern maps directly to Rocket guards and is easier to test in isolation |
| tokio | 1.x (full) | Async runtime | Axum requires Tokio; already in lockfile at 1.42.0; `features = ["full"]` for dev, tune for prod |
| tower-http | 0.6.x | HTTP middleware (CORS, tracing, compression) | Tower's service model is the lingua franca of Axum middleware; tower-http bundles the most common layers (CorsLayer, TraceLayer, CompressionLayer) as first-class crates |
| diesel | 2.2.x | ORM and query builder | Already in use at 2.1.6; schema compatibility preserved; 2.2 added async-aware connection support |
| diesel-async | 0.5.x | Async Diesel connection adapter | The canonical async Diesel bridge for Axum; replaces `rocket_db_pools/diesel_mysql`; uses `AsyncMysqlConnection` backed by `tokio-mysql` |
| deadpool-diesel | 0.6.x | Async connection pool for diesel-async | Deadpool is deadpool-diesel's own first-party pool integration; simpler than bb8 for MariaDB/MySQL; tight integration with diesel-async |
| serde / serde_json | 1.x | Serialization | Axum JSON extractor/responder uses serde; already at 1.0.133 in lockfile |
| argon2 | 0.5.x | Password hashing | Already in use at 0.5.3; Argon2id is current best practice for password hashing |
| jsonwebtoken | 9.x | JWT generation and verification | More widely maintained than `jwt-simple` (used currently); `jsonwebtoken` 9.x has strong community adoption, active maintenance, and clearer HMAC/RSA API; replaces `jwt-simple 0.11.9` |
| tracing | 0.1.x | Structured logging | Already used; pairs with `tower-http TraceLayer` for per-request spans in Axum |
| tracing-subscriber | 0.3.x | Log subscriber setup | Already used; keep existing configuration pattern |
| thiserror | 2.x | Error type derivation | Standard for custom error enums; current stable is 2.x; upgrade from 1.x is non-breaking for derive usage |

### Supporting Libraries

| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| axum-extra | 0.10.x | Extended extractors (TypedHeader, CookieJar, etc.) | When you need typed header extraction (Authorization), multipart, or cookie support; ships with axum |
| tower | 0.5.x | Service/middleware composition | Direct tower primitives when tower-http layers aren't enough; rate limiting, timeout layers |
| diesel_migrations | 2.2.x | Schema migration management | Run migrations at startup via `MigrationHarness`; same version as diesel |
| chrono | 0.4.x | Date/time types | Already used at 0.4.31; Diesel's `chrono` feature maps NaiveDate/NaiveTime/NaiveDateTime to SQL Date/Time/Timestamp |
| anyhow | 1.x | Error context chaining in tests/bin | Keep for `bin/backend.rs` startup; don't use in library code (use thiserror there) |
| validator | 0.19.x | Request body validation | Derive-based validation for Axum JSON payloads; add `ValidationLayer` or validate manually in handlers |
| dotenvy | 0.15.x | `.env` file loading | Replaces the dotenv crate; load `DATABASE_URL`, `JWT_SECRET`, `CORS_ORIGIN` from environment |
| secrecy | 0.10.x | Secret string types | Wrap `JWT_SECRET` and database passwords so they don't appear in logs or Debug output |

### Frontend (Keep Existing Stack — Already Modern)

The existing frontend is already Tailwind v4 + React 19 + TanStack Router + Vite 6. This is the correct 2025/2026 frontend stack for a backend dev who wants something understandable. **Do not change frameworks.**

| Technology | Version | Purpose | Why Keep |
|------------|---------|---------|---------|
| React | 19.x | UI framework | React is the dominant choice for backend devs learning frontend; huge ecosystem; existing code already written |
| Vite | 6.x | Dev server + bundler | Fastest HMR; zero-config TypeScript; standard for React 2025 |
| TanStack Router | 1.x | Client-side routing | Type-safe routing with file-based route support; already in use; better TypeScript integration than React Router |
| Tailwind CSS | 4.x | Utility CSS | v4 in use; Tailwind Plus components require Tailwind; no alternative |
| React Hook Form + Zod | 7.x / 3.x | Form + validation | Already in use; industry standard combination for typed forms |
| @tanstack/react-query | 5.x | Server state / API cache | NOT yet in codebase but strongly recommended; replaces manual `useState + useEffect + axios` fetch patterns; reduces the 766-line TrackingPage problem by 60% |
| axios | 1.x | HTTP client | Already in use with interceptors for auth; keep |
| TypeScript | 5.7.x | Type safety | Already in use; strict mode enabled |

### Development Tools

| Tool | Purpose | Notes |
|------|---------|-------|
| just | Task runner | Already present; extend with `just migrate`, `just seed`, `just check-all` |
| cargo-nextest | Faster test runner | Already used via `just t`; keep |
| diesel CLI | Migration management | Required for `diesel migration run/generate`; install with `cargo install diesel_cli --no-default-features --features mysql` |
| cargo-watch | Auto-rebuild on file change | `cargo watch -x run` for dev feedback loop; pair with Vite dev server |
| typeshare | Rust→TypeScript type generation | Already in use at 1.0.1; critical for keeping frontend types in sync with backend structs; keep |

---

## Installation

### Rust backend changes (Cargo.toml replacements)

```toml
[dependencies]
# Replace Rocket stack
axum = { version = "0.8", features = ["macros", "json"] }
axum-extra = { version = "0.10", features = ["typed-header"] }
tower = { version = "0.5" }
tower-http = { version = "0.6", features = ["cors", "trace", "compression-gzip"] }

# Replace rocket_db_pools with diesel-async + deadpool
diesel = { version = "2.2", features = ["mysql", "chrono"] }
diesel-async = { version = "0.5", features = ["mysql", "deadpool"] }
diesel_migrations = "2.2"

# Replace jwt-simple with jsonwebtoken
jsonwebtoken = "9"

# Keep or upgrade
argon2 = "0.5"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
chrono = { version = "0.4", features = ["serde"] }
tokio = { version = "1", features = ["full"] }
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter", "fmt", "json"] }
tracing-appender = "0.2"
thiserror = "2"
anyhow = "1"
dotenvy = "0.15"
typeshare = "1"
validator = { version = "0.19", features = ["derive"] }
```

### Frontend additions (package.json)

```bash
# Add TanStack Query — strongly recommended to replace manual fetch patterns
npm install @tanstack/react-query @tanstack/react-query-devtools

# All other packages already present — do not change
```

---

## Alternatives Considered

| Recommended | Alternative | When to Use Alternative |
|-------------|-------------|-------------------------|
| axum 0.8 | Rocket 0.5/0.6 | Rocket if team already has muscle memory and no async Diesel requirement; for this project Axum's Tower ecosystem makes Diesel-async wiring cleaner |
| diesel-async | sqlx | sqlx if starting fresh without an existing schema; sqlx is compile-time query verified but requires rewriting all queries; diesel-async preserves existing schema/query code |
| deadpool-diesel | bb8 + diesel-async | bb8 if you want more fine-grained pool configuration; deadpool-diesel is simpler to set up and is diesel-async's documented first recommendation |
| jsonwebtoken | jwt-simple | jwt-simple if you need JWK/JWKS support or ES256/EdDSA without extra config; jsonwebtoken is more widely deployed, better maintained, and has more Stack Overflow coverage |
| React (keep) | Svelte / SolidJS | Svelte or SolidJS only if starting fresh; switching now requires rewriting all existing components with no functional benefit for a small internal tool |
| @tanstack/react-query | SWR | SWR if already using Next.js; TanStack Query is framework-agnostic and better TypeScript support |
| dotenvy | config / figment | Figment if you need multi-environment TOML config (like Rocket.toml); dotenvy is simpler and standard for Axum apps |

---

## What NOT to Use

| Avoid | Why | Use Instead |
|-------|-----|-------------|
| `rocket_db_pools` | Rocket-specific pool abstraction; not portable to Axum | `diesel-async` + `deadpool-diesel` |
| `jwt-simple` (keep using in Axum) | Works but smaller community, less documentation; switching to jsonwebtoken gives better error types and wider ecosystem | `jsonwebtoken = "9"` |
| `r2d2` (sync pool) | Blocking pool that requires `spawn_blocking` wrappers in async handlers; defeats the purpose of async Axum | `deadpool-diesel` (native async pool) |
| Rocket-style `#[get]` / `#[post]` macros | Rocket macros; Axum uses `Router::new().route(path, get(handler))` pattern | Axum Router API |
| `rocket_cors` | Rocket-specific CORS fairing | `tower-http CorsLayer` |
| `anyhow` in handler return types | Leaks internal error context to HTTP responses; bad for security and API stability | `thiserror` enum + `IntoResponse` impl |
| Redux / Zustand for frontend state | Overkill for a small internal tool; adds complexity for a backend dev | React Context + TanStack Query server state |
| Next.js / Remix / SvelteKit | Full-stack frameworks that duplicate the Axum backend; architectural mismatch | Vite SPA + Axum API (current approach) |
| localStorage for JWT (existing issue) | XSS-vulnerable token storage | httpOnly cookies with `SameSite=Strict`; or keep localStorage if XSS is not a concern for internal tool |

---

## Stack Patterns by Variant

**For the connection pool setup (Axum + diesel-async + deadpool):**
- Create `deadpool_diesel::mysql::Pool` in `main.rs`
- Add it to Axum app state via `.with_state(AppState { pool })`
- Extract in handlers via `State<AppState>` extractor
- Obtain connection with `pool.get().await?` — returns `AsyncMysqlConnection`

**For JWT authentication in Axum:**
- Implement `FromRequestParts` (not `FromRequest`) for `AuthUser` extractor
- Extract `Authorization: Bearer <token>` header via `TypedHeader<Authorization<Bearer>>` from axum-extra
- Validate token with `jsonwebtoken::decode`
- Store `DecodingKey` in `AppState` (loaded once at startup from env var)

**For CORS in Axum:**
- Use `tower_http::cors::CorsLayer` added as `.layer(CorsLayer::new()...)` on the Router
- Load allowed origins from environment variable at startup
- Replace hardcoded `http://localhost:5173`

**For request/response tracing:**
- Add `tower_http::trace::TraceLayer` to Router
- Existing tracing-subscriber config works unchanged

**If you add sub-projects (Teilprojekte) to the schema:**
- Add new Diesel migration with `diesel migration generate add_subproject`
- Update schema.rs by running `diesel print-schema`
- Add new db module; no framework changes needed

---

## Version Compatibility

| Package A | Compatible With | Notes |
|-----------|-----------------|-------|
| axum 0.8 | tokio 1.x | Axum 0.8 requires Tokio 1.x; already in lockfile at 1.42.0 |
| axum 0.8 | tower-http 0.6 | tower-http 0.6 matches axum 0.8's Tower dependency tree |
| diesel 2.2 | diesel-async 0.5 | diesel-async 0.5 requires diesel 2.x; versions must be kept in sync |
| diesel-async 0.5 | deadpool-diesel 0.6 | deadpool-diesel 0.6 provides the `deadpool` feature for diesel-async |
| chrono 0.4 | diesel 2.2 | diesel `chrono` feature requires chrono 0.4; already in use |
| jsonwebtoken 9 | no Rocket deps | Clean replacement; no version conflicts |
| thiserror 2 | existing error.rs | thiserror 2 is backward compatible for `#[derive(Error)]` usage; only internal API changes |
| @tanstack/react-query 5 | React 19 | TanStack Query 5 supports React 19; use `QueryClientProvider` wrapper |
| Tailwind CSS 4 | PostCSS / Vite 6 | Already working in current setup; no change needed |

---

## Sources

- Cargo.lock (verified): `axum` not yet present (rewrite), `diesel 2.1.6`, `tokio 1.42.0`, `argon2 0.5.3`, `jwt-simple 0.11.9`
- `.planning/codebase/STACK.md` (codebase audit): existing stack confirmed
- `backend/Cargo.toml` (verified): Rocket 0.5, diesel 2.1.4, diesel_migrations 2.1.0, jwt-simple 0.11.9
- `frontend/package.json` (verified): React 19, TanStack Router 1.115.3, Tailwind 4.1.3, Vite 6.2.0
- Training data (MEDIUM confidence): axum 0.8, tower-http 0.6, diesel-async 0.5, deadpool-diesel 0.6, jsonwebtoken 9 — version numbers reflect training knowledge through early 2025; verify exact patch versions when adding to Cargo.toml

**Confidence by area:**
- Axum as framework choice: HIGH (official Tokio project, active development confirmed by community pattern)
- diesel-async + deadpool-diesel for async Diesel: HIGH (this is the documented official async path for Diesel 2.x)
- jsonwebtoken over jwt-simple: MEDIUM (more popular/maintained but jwt-simple also works; verify maintenance status)
- Keep React/Vite/TanStack frontend: HIGH (already modern, working, TypeScript-friendly)
- TanStack Query addition: MEDIUM (strong community recommendation for replacing useEffect fetching, not yet in codebase)
- Version numbers: MEDIUM (pattern versions verified via training; patch versions need `cargo search` verification before use)

---

*Stack research for: Axum-based time tracking application rewrite (Zeiterfassung)*
*Researched: 2026-02-13*
