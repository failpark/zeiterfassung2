# Phase 1: Framework Foundation - Context

**Gathered:** 2026-02-13
**Status:** Ready for planning

<domain>
## Phase Boundary

Axum server skeleton with AppState, middleware stack (CORS, tracing), auth middleware skeleton, and justfile tooling. This provides the wiring patterns every subsequent phase builds on. No business logic, no database, no real auth — just the framework foundation.

</domain>

<decisions>
## Implementation Decisions

### Configuration style
- TOML config file as base, env vars override specific values
- Single `config.toml` for defaults; production overrides via env vars (no per-environment files)
- Load order: config.toml → env var overrides → dotenvy .env in dev
- Config file format: TOML (Rust ecosystem standard)

### API error format
- Detailed response shape: `{code, message, details}`
- Error codes are uppercase snake_case strings (e.g., `VALIDATION_ERROR`, `NOT_FOUND`)
- Messages always in English from API; frontend handles localization if needed
- Validation errors include field-level detail: `details: [{field: "email", reason: "invalid_format"}]`
- 500 errors: include error chain in development, generic message + request ID in production

### Logging behavior
- Pretty (human-readable, colored) format in development, JSON structured in production
- Per-request info: method, path, status code, duration, and unique request ID
- Default log level: INFO (overridable via env var)
- Request/response body logging available at TRACE level only — never in production

### CORS policy
- Wildcard (`*`) allowed in development for easy frontend dev
- Explicit origin list from config in production
- Credentials allowed (`Access-Control-Allow-Credentials: true`) for JWT Authorization header

### Claude's Discretion
- Secrets handling: whether sensitive values (DB password, JWT secret) are env-only or allowed in config file — pick the safer approach
- CORS allowed methods: pick based on what the REST API will actually use
- CORS preflight cache max-age: balance dev flexibility with production performance
- Exact config crate choice and TOML parsing approach

</decisions>

<specifics>
## Specific Ideas

No specific requirements — open to standard approaches

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope

</deferred>

---

*Phase: 01-framework-foundation*
*Context gathered: 2026-02-13*
