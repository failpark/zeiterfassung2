# Pitfalls Research

**Domain:** Rocket-to-Axum rewrite — time tracking app (Rust backend, MariaDB/MySQL, React frontend)
**Researched:** 2026-02-13
**Confidence:** MEDIUM (no web access; based on direct codebase inspection + training data for Axum/Diesel ecosystem; flagged items need field verification)

---

## Critical Pitfalls

### Pitfall 1: Diesel Async Is Not True Async — Blocking Calls on Tokio Executor

**What goes wrong:**
`diesel-async` (with `AsyncMysqlConnection` or `AsyncPgConnection`) wraps the underlying sync MySQL/MariaDB driver in async I/O, but the current `rocket_db_pools` integration uses `MysqlPool` which is backed by `deadpool` with Diesel's async MySQL support. When migrating to Axum, developers commonly swap to `bb8` or `deadpool` directly, but still call `.transaction()` or `.execute()` inside a `spawn_blocking` by habit, causing double-wrapping or blocking the Tokio thread pool. The inverse error — calling blocking I/O directly without `spawn_blocking` — stalls the async executor.

The existing codebase already uses `rocket_db_pools::diesel::MysqlPool` with async methods (`.execute().await`, `.first().await`). This pattern must be preserved exactly in the Axum migration; reverting to sync Diesel is a regression that blocks threads.

**Why it happens:**
Developers familiar with sync Diesel (used in `dev-dependencies` with `r2d2` in this repo) apply the same patterns to async Diesel. The `run_migrations` function in `db/mod.rs` already uses `spawn_blocking` correctly for sync migration code — this split (sync migrations, async queries) must be understood and replicated.

**How to avoid:**
- Use `diesel-async` with `AsyncMysqlConnection` + `deadpool-diesel` or `bb8-diesel` for the pool.
- Keep migrations in `spawn_blocking` (they use sync `MigrationHarness`).
- All query code (`.load()`, `.first()`, `.execute()`, `.get_result()`) must use `.await` and must NOT be wrapped in `spawn_blocking`.
- Never derive `Clone` on pool types and pass them by reference into async closures.

**Warning signs:**
- Tokio "blocking in async context" panics or warnings.
- Tests hanging indefinitely under concurrent load.
- `spawn_blocking` calls wrapping Diesel async query code.
- Using `r2d2` (sync pool) instead of `deadpool`/`bb8` in production code.

**Phase to address:** Foundation / DB Layer phase (first backend phase)

---

### Pitfall 2: Transaction Closure Ownership — `Box::pin(async move {})` Pattern Is Mandatory

**What goes wrong:**
The existing codebase correctly uses `db.transaction(|mut conn| { Box::pin(async move { ... }) })`. When developers translate this to Axum with `diesel-async`, they sometimes flatten the transaction into sequential awaited calls outside a transaction closure, thinking the connection keeps an implicit transaction. This silently removes ACID guarantees. The `create` and `update` operations in `tracking/tracking.rs` and `user.rs` both use `INSERT` followed by `SELECT last_insert_id()` — without a transaction, a concurrent insert between these two statements would return the wrong ID.

**Why it happens:**
`diesel-async`'s transaction API requires a specific closure shape that is non-obvious. The `Box::pin(async move { })` wrapper is required to produce a `Pin<Box<dyn Future>>` as the transaction callback. Developers unfamiliar with this pattern remove it to simplify code.

**How to avoid:**
```rust
// CORRECT
conn.transaction(|conn| {
    Box::pin(async move {
        insert_into(table).values(&item).execute(conn).await?;
        table.filter(id.eq(last_insert_id())).first(conn).await
    })
}).await
```
Every `INSERT` + subsequent `SELECT last_insert_id()` MUST be inside a transaction closure. Add a linting comment near all `transaction` calls noting this requirement.

**Warning signs:**
- Sequential `execute().await` + `first().await` calls not wrapped in `.transaction()`.
- IDs returned by create operations are occasionally wrong under concurrent test load.

**Phase to address:** Foundation / DB Layer phase

---

### Pitfall 3: Axum Extractors Consume Request Body — Order Matters

**What goes wrong:**
Axum's `Json<T>` extractor consumes the request body. If a handler has multiple extractors and `Json<T>` is not the last extractor parameter, or if middleware attempts to read the body before the handler, the body is consumed and subsequent deserialization fails with an opaque error. This is fundamentally different from Rocket, where `FromRequest` guards and `data = "<body>"` are clearly separated.

In the current codebase, Rocket handlers use `data = "<create_tracking>"` explicitly; Axum handles this implicitly by extractor position. The `User` guard (currently a `FromRequest` in `guard.rs`) becomes an Axum extractor. If `Json<T>` and `Extension<User>` (or a custom `FromRequestParts` impl) both extract from the same request, the extractor consuming the body must come last.

**Why it happens:**
Axum enforces that `FromRequest` (body-consuming) extractors must be the last parameter in a handler. `FromRequestParts` extractors (headers, path, state) can appear in any order. Developers unfamiliar with this distinction see confusing compile errors or runtime 400s.

**How to avoid:**
- Auth extractor (`User`) must implement `FromRequestParts`, not `FromRequest`. Extract JWT from headers only (no body access needed — this is already the case in the existing `guard.rs`).
- `Json<T>` or `Form<T>` must be the last parameter in handler signatures.
- Verify at compile time: if you get "body already consumed" 400 errors in tests, check extractor order.

**Warning signs:**
- Handlers with `Json<T>` not as the last parameter.
- Unexplained 400 Bad Request responses in integration tests.
- Compile errors mentioning "FromRequest vs FromRequestParts".

**Phase to address:** Foundation / Backend API phase

---

### Pitfall 4: Axum State vs Extension — Choosing the Wrong Injection Mechanism

**What goes wrong:**
Axum has two state injection mechanisms: `State<T>` (type-erased, recommended) and `Extension<T>` (axum middleware layer approach). Using `Extension` for application state (like the `Tokenizer` in this app) instead of `State` leads to runtime panics when the extension is missing from the router tree, rather than compile-time errors. Rocket's `.manage()` is equivalent to Axum's `.with_state()`, not `.layer(Extension(...))`.

The existing `auth::Tokenizer` is registered via `.manage()` in Rocket. In Axum, this must become `.with_state(tokenizer)` using a shared `AppState` struct, or it will be invisible to extractors.

**Why it happens:**
Axum's `Extension` middleware is often shown in examples for things like database pools and auth state. Developers copy these examples without understanding that `Extension` is for per-request middleware data, while `State` is for shared application configuration.

**How to avoid:**
- Define a single `AppState` struct containing the pool and `Tokenizer`.
- Use `Arc<AppState>` or derive `Clone` on `AppState` (the pool is already `Clone`).
- Register with `.with_state(state)` on the router.
- Extractors use `State<Arc<AppState>>` or `State<AppState>` (if `Clone`).
- Never use `Extension` for the db pool or tokenizer.

**Warning signs:**
- Runtime panics with "missing extension" messages.
- Having both `Extension` and `State` for the same type in different routes.

**Phase to address:** Foundation / Backend API phase

---

### Pitfall 5: CORS Configuration — Hardcoded Dev Origin Survives to Production

**What goes wrong:**
The current `lib.rs` hardcodes `allowed_origins = ["http://localhost:5173"]`. When migrating to Axum with `tower-http`'s `CorsLayer`, developers often copy this pattern. The risk is that the CORS origin is never made configurable from environment/config, so deploying to production serves CORS errors or — worse — if someone switches to `AllowAny` to "fix" it in dev, that gets deployed.

`tower-http`'s `CorsLayer` has a different API than `rocket_cors`. The `AllowOrigin` type is not a simple string list; it requires explicit use of `AllowOrigin::list()` or `AllowOrigin::predicate()`.

**How to avoid:**
- Read allowed origins from environment variables at startup.
- In `AppState` or startup config, make `CorsLayer` configurable.
- Use `AllowOrigin::list([origin])` with origins parsed from config.
- Fail fast at startup if `ALLOWED_ORIGINS` env var is not set in non-dev environments.

**Warning signs:**
- Hardcoded `localhost` origin in CORS config.
- `AllowAny` origin used in any non-test configuration.
- CORS config not tested in integration tests.

**Phase to address:** Foundation / Backend API phase

---

### Pitfall 6: `last_insert_id()` Is MySQL-Specific and Session-Scoped

**What goes wrong:**
The current codebase defines `diesel::sql_function!(fn last_insert_id() -> Integer)` and uses it after inserts to retrieve the new row ID. This works because `rocket_db_pools` with `deadpool` returns the same connection object for the duration of a transaction. In Axum, if the pool implementation changes or connection handling changes, `last_insert_id()` might return an ID from a different connection's insert in a concurrent scenario.

This is already partially guarded by using transactions (`db.transaction()`), but the underlying MySQL `LAST_INSERT_ID()` is session-scoped. As long as the `INSERT` and the `SELECT last_insert_id()` are in the same transaction on the same connection, it is safe. The danger arises if a developer refactors out of the transaction.

**How to avoid:**
- Never call `last_insert_id()` outside a `transaction()` closure.
- Document this constraint explicitly in a `// SAFETY:` comment near every use.
- Consider switching to `RETURNING id` syntax if MariaDB version supports it (MariaDB 10.5+ supports `RETURNING`). This is more robust but requires schema/query changes.
- Test with concurrent inserts in the test suite.

**Warning signs:**
- `last_insert_id()` called outside a `transaction()` block.
- Remove of transaction wrappers during "cleanup" refactors.

**Phase to address:** Foundation / DB Layer phase

---

### Pitfall 7: Schema Evolution — Adding `sub_project` and `travel_route` to Live Production Data

**What goes wrong:**
The existing schema has `tracking` with a direct `project_id` FK. Adding sub-projects means either:
(a) Adding a `sub_project_id` nullable FK to `tracking` — simple but creates NULL proliferation in historical data.
(b) Creating a `project_hierarchy` table — complex, requires data migration.

Adding `travel_route` to `tracking` similarly risks breaking the existing API contract if the frontend is updated before the backend (or vice versa).

The most common failure mode: running `ALTER TABLE tracking ADD COLUMN sub_project_id INTEGER NULL` in production while the Rust struct does NOT yet have this field — Diesel's type-checked queries will fail to compile if the schema and struct are out of sync. Since `schema.rs` is auto-generated by `diesel print-schema`, forgetting to regenerate it after a migration breaks compilation.

**Why it happens:**
Diesel's `schema.rs` is a snapshot at migration-run time. If `diesel print-schema` is not re-run after a migration, the schema file is stale. Brownfield migration makes this worse because the order of "migrate DB → regenerate schema → update Rust structs → update API" must be strictly followed.

**How to avoid:**
- Strict migration sequence: `diesel migration run` → `diesel print-schema > src/schema.rs` → update Rust structs → compile → test.
- Add `nullable` columns with explicit `DEFAULT NULL` so existing rows are unaffected.
- Use a migration checklist in the planning phase for every schema change.
- `travel_route` should be a separate table with an FK to `tracking` (not a column) to avoid wide-row proliferation if routes have multiple stops.
- For `sub_project`: make `sub_project_id` nullable on `tracking`, with the constraint that if null, it uses the parent project. Keep backward compatibility.

**Warning signs:**
- Compile errors after a new migration mentioning column count mismatches.
- `schema.rs` last-modified timestamp is older than the latest migration folder.
- API returning 500 on tracking endpoints after a migration.

**Phase to address:** Schema Evolution / New Features phase

---

## Technical Debt Patterns

| Shortcut | Immediate Benefit | Long-term Cost | When Acceptable |
|----------|-------------------|----------------|-----------------|
| Copy Rocket guards verbatim, wrap in `async` | Fast migration of auth logic | Wrong trait bounds; runtime panics when `FromRequest` used where `FromRequestParts` required | Never — get the trait right from the start |
| Skip re-generating `schema.rs` after migration | Faster iteration | Compile failure in CI, or worse, silent wrong queries if the schema is patched manually | Never |
| Use `Extension` instead of `State` for app state | Mirrors common Axum examples | Runtime panic if route tree is incomplete; no compile-time guarantee | Never for application state |
| Hardcode `unwrap()` on DB pool errors | Simpler error handling during development | Panics that kill the whole server process in production | Only in `main.rs` startup checks, never in request handlers |
| Keep `sys_role` as a raw `String` enum | No migration needed | Authorization bugs if role strings drift (e.g., `"Admin"` vs `"admin"`) | Acceptable for MVP, must be replaced with a proper enum type for production |
| Inline `argon2` hashing in the route handler | Less indirection | Business logic in HTTP layer; untestable without HTTP | Never — keep in auth module as it is now |

---

## Integration Gotchas

| Integration | Common Mistake | Correct Approach |
|-------------|----------------|------------------|
| `diesel-async` + MariaDB | Using `AsyncMysqlConnection` directly without a pool (connection per request) | Use `deadpool-diesel` or `bb8-diesel`; obtain a pooled connection per request handler |
| `diesel-async` + `diesel_migrations` | Calling `run_pending_migrations` from async context directly | Always use `spawn_blocking` for migrations; `MigrationHarness` is sync-only |
| `axum` + `tower-http` CORS | Placing `CorsLayer` after auth middleware in the layer stack | `CorsLayer` must be the outermost layer so preflight OPTIONS requests bypass auth |
| `jwt-simple` + Axum state | Re-generating the `Ed25519KeyPair` on every request instead of once at startup | Store `Tokenizer` in `AppState`, initialized once in `main` |
| `typeshare` + new API types | Forgetting to run `typeshare` after adding new structs | Add `typeshare` to CI / `just check` pipeline; TypeScript types will be stale |
| MariaDB `FLOAT(4,2)` | Rust's `f32` accumulates rounding errors in `performed`/`billed` calculations | Convert to `DECIMAL(10,2)` in a migration; map to `rust_decimal::Decimal` in Diesel |

---

## Performance Traps

| Trap | Symptoms | Prevention | When It Breaks |
|------|----------|------------|----------------|
| N+1 query in `tracking/middlelayer.rs` paginate | Slow paginate responses; DB load spikes on page loads | Already using `grouped_by` + single join query — preserve this pattern in rewrite | Breaks at ~100 trackings per page with individual activity lookups |
| No connection pool size limit | Under load, pool exhaustion causes 500 errors with unhelpful messages | Configure `max_connections` in pool config; expose as environment variable | Breaks with > default pool size concurrent requests (typically ~10 for small pools) |
| Fetching all tracking records without pagination | Frontend or export feature requests all data at once | Enforce server-side pagination; never expose a `GET /tracking/all` endpoint | Breaks at ~1000 tracking records in a single response |
| `f32` accumulation for billing calculations | Invoiced totals are off by ±0.01 EUR | Use `DECIMAL(10,2)` in DB and `rust_decimal::Decimal` in Rust | Breaks at any scale — correctness issue, not scale |

---

## Security Mistakes

| Mistake | Risk | Prevention |
|---------|------|------------|
| JWT key regenerated on restart invalidates all tokens (by design) but with no user warning | Users lose sessions silently after deploys | Document this behavior; show "session expired, please log in" on 401, not a generic error |
| `sys_role` stored as raw string; checked with `if user.sys_role != "admin"` | Typo or case mismatch bypasses authorization silently | Convert to a typed `enum Role { Admin, User }` with `FromStr`/`TryFrom<String>` |
| Password hash logged in error branch (`error!("Hash is invalid... hash: {}", rec.hash)`) | Hash exposed in log files | Remove the hash from the error log; log "invalid hash for user {username}" only |
| No rate limiting on `/login` endpoint | Brute-force password attacks | Add `tower-governor` or similar rate limiter as a layer on the login route |
| JWT token expiry is 5 days — long-lived tokens | Compromised token valid for 5 days | Consider refresh token pattern or reduce to 24h for production |

---

## UX Pitfalls

| Pitfall | User Impact | Better Approach |
|---------|-------------|-----------------|
| Time entry form requires manual calculation of `performed` and `billed` | Consulting team users make errors; resent the tool | Auto-calculate `performed` from `begin`/`end`/`pause`; let user override `billed` only |
| No validation that `end` > `begin` in time entries | Corrupt data; negative durations silently stored | Validate server-side AND client-side; return a 422 with a clear error message |
| Pagination with 0-based page index exposed in URL | `?page=0` is confusing to users; `?page=1` gives wrong first page | Keep 0-based internally; display and accept 1-based in the UI |
| Sub-project relationship not reflected in the tracking form | Users must know to select a sub-project separately | When a sub-project exists, make sub-project selection mandatory in the tracking form |
| No confirmation before deleting a tracking entry | Accidental deletions with no undo | Add a confirmation dialog; consider soft-delete (mark as deleted, recover within 30 days) |
| Travel route entry is separate from time tracking | Users must record the same trip twice | Integrate travel route into the tracking form as an optional section |

---

## "Looks Done But Isn't" Checklist

- [ ] **Auth migration:** JWT verify works in Axum extractor — verify 401 is returned (not 500) for expired/invalid tokens
- [ ] **CORS:** Preflight OPTIONS requests return 200 without hitting auth middleware — verify with `curl -X OPTIONS`
- [ ] **Migrations:** `schema.rs` is in sync with latest migration — verify by running `diesel print-schema` and diff against committed file
- [ ] **Transaction integrity:** `last_insert_id()` returns the correct ID under concurrent test load — verify with a concurrent integration test
- [ ] **Error responses:** All error variants return JSON, not plain text — verify every error path returns `Content-Type: application/json`
- [ ] **Role authorization:** `Admin`-only routes reject `user` role — verify with a test using a non-admin JWT
- [ ] **Password security:** Password hash is NOT logged in any error path — grep log output for `$argon2`
- [ ] **Schema evolution:** Existing data rows with NULL `sub_project_id` do not cause query errors — verify with a test using pre-migration fixtures
- [ ] **Frontend routing:** TanStack Router v1 API used in `router.tsx` uses old object API (`new Route`, `new RootRoute`) — this needs migration to the file-based or builder API in v1.x

---

## Recovery Strategies

| Pitfall | Recovery Cost | Recovery Steps |
|---------|---------------|----------------|
| Wrong trait used for auth extractor (panic at runtime) | LOW | Implement `FromRequestParts` instead of `FromRequest`; recompile; test |
| `schema.rs` out of sync after migration | LOW | Run `diesel print-schema > src/schema.rs`; fix Rust struct fields to match |
| Transaction removed, ID race condition in production | HIGH | Add transaction back; audit all `last_insert_id()` uses; consider switching to `RETURNING` clause |
| `sys_role` string mismatch in production | MEDIUM | Add a migration to normalize all role strings; add DB constraint `CHECK (sys_role IN ('admin', 'user'))` |
| CORS `AllowAny` deployed to production | MEDIUM | Hotfix `CorsLayer` with explicit origins from env; redeploy |
| Sub-project migration breaks tracking queries | HIGH | Write down migration in both rollback-safe steps; test against a copy of production data before running |

---

## Pitfall-to-Phase Mapping

| Pitfall | Prevention Phase | Verification |
|---------|------------------|--------------|
| Diesel async blocking calls | Phase 1: DB foundation | All query functions use `.await`; no `spawn_blocking` around query code |
| Transaction closure pattern | Phase 1: DB foundation | `create` + `update` ops wrapped in `transaction()`; concurrent test passes |
| Extractor ordering (FromRequestParts) | Phase 2: API foundation | Auth extractor compiles as `FromRequestParts`; handler tests pass |
| Axum State vs Extension | Phase 2: API foundation | `AppState` defined; no `Extension` used for pool or tokenizer |
| CORS configuration | Phase 2: API foundation | Preflight OPTIONS returns 200; origin is configurable via env |
| `last_insert_id()` session safety | Phase 1: DB foundation | Concurrent insert test; `SELECT last_insert_id()` only inside transaction |
| Schema evolution for sub_project | Phase 3: New features | Migration checklist followed; `schema.rs` regenerated; existing data tested |
| `sys_role` string enum | Phase 2: API foundation | Typed `Role` enum with `TryFrom<String>`; authorization test coverage |
| Password hash in logs | Phase 2: API foundation | `grep -r '\\$argon2'` finds no log statements |
| No rate limiting on login | Phase 2: API foundation (or Phase 4: Hardening) | `tower-governor` layer on `/login`; verified with rapid repeated requests |
| `f32` billing calculations | Phase 3: New features (or Phase 1 if addressed early) | `DECIMAL` in schema; `rust_decimal` in structs; rounding test |
| TanStack Router old API | Phase 2: Frontend | No `new Route` / `new RootRoute` object construction; file-based routing used |

---

## Sources

- Codebase inspection: `/Users/phedias/code/zeiterfassung/backend/src/` (Rocket 0.5 + rocket_db_pools + diesel-async patterns)
- Codebase inspection: `/Users/phedias/code/zeiterfassung/migrations/` (MariaDB schema structure)
- Codebase inspection: `/Users/phedias/code/zeiterfassung/frontend/package.json` (TanStack Router v1.115.3, React 19)
- Training data: Axum 0.7.x extractor rules (`FromRequest` vs `FromRequestParts`), `tower-http` `CorsLayer` API — MEDIUM confidence; verify against current docs
- Training data: `diesel-async` transaction closure shape, `deadpool-diesel` pool integration — MEDIUM confidence; verify version compatibility
- Training data: MariaDB 10.5+ `RETURNING` clause support — LOW confidence; verify against MariaDB release notes for target version
- Training data: `tower-governor` for rate limiting — LOW confidence; verify crate is still maintained and compatible with Axum 0.7

---
*Pitfalls research for: Rocket-to-Axum rewrite of zeiterfassung time tracking backend*
*Researched: 2026-02-13*
