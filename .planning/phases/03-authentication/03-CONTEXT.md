# Phase 3: Authentication - Context

**Gathered:** 2026-02-22
**Status:** Ready for planning

<domain>
## Phase Boundary

JWT login/logout endpoints with configurable expiration and typed role enum. This is a brownfield port — existing Rocket-based auth code (tokenizer, password hashing, login route, frontend login page) is ported to Axum with specific improvements (typed Role enum, proper extractors). No new auth capabilities (refresh tokens, OAuth, MFA) are in scope.

</domain>

<decisions>
## Implementation Decisions

### JWT token strategy
- Keep ephemeral Ed25519 keypair — new keypair generated on each server restart, all tokens invalidate on deploy
- Keep Ed25519 signing algorithm (no switch to HMAC)
- Token expiration: Claude's discretion on config mechanism (env var vs config file)
- JWT claims content: Claude's discretion (full user struct vs minimal claims)

### Login/Logout behavior
- Login endpoint path: `/auth/login` (namespaced, not old `/login`)
- Logout: server endpoint exists (`POST /auth/logout`) — returns 200, currently no blacklisting but endpoint is there for future use
- Failed login: include a basic artificial delay to slow brute force attempts
- Login response shape: Claude's discretion (token-only vs token+user)

### Role enum design
- Extensible enum — designed to allow adding roles later (not just User/Admin forever)
- Unknown role values in DB cause error on deserialize (strict — forces code update before DB update)
- Role enum used everywhere: JWT claims, API JSON responses, DB layer — single source of truth
- JSON serialization as lowercase strings: "admin", "user" — matches current DB values

### Auth middleware wiring
- Custom Axum extractor pattern: `AuthUser(user)` declared as handler parameter
- Separate `AdminUser` extractor that returns 403 if not admin — type-level admin guard
- Route prefix and middleware scope: Claude's discretion

### Claude's Discretion
- Token expiration config mechanism (env var vs config file)
- JWT claims content (full user struct vs minimal id+role)
- Login response shape (token-only vs token+user data)
- Login endpoint under `/api` prefix or at root
- Auth middleware scope (router-level vs per-route-group)

</decisions>

<specifics>
## Specific Ideas

- Existing Rocket-based auth code provides the implementation reference: `auth/tokenizer.rs`, `routes/login.rs`, `guard.rs`, `db/user.rs`
- Frontend already has working login page, AuthContext, axios interceptor — needs endpoint URL update and potentially response shape adjustment
- Password hashing already uses Argon2 — keep as-is

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope

</deferred>

---

*Phase: 03-authentication*
*Context gathered: 2026-02-22*
