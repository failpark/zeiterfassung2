# Project Research Summary

**Project:** Zeiterfassung — professional time tracking for consulting teams
**Domain:** Brownfield Rust backend rewrite (Rocket → Axum) with React SPA frontend
**Researched:** 2026-02-13
**Confidence:** MEDIUM

## Executive Summary

Zeiterfassung is a brownfield rewrite of a working internal tool used by a German consulting firm. The Rocket 0.5 backend is being replaced with Axum 0.8 while the existing React 19 + Vite + TanStack Router frontend is kept and extended. This is not a greenfield project: the existing MariaDB schema, Diesel query layer, auth logic, and most frontend components carry over with targeted changes. The rewrite is motivated by the desire for the Tokio-native Axum ecosystem (better async DB integration via diesel-async + deadpool-diesel) rather than feature gaps in the current backend.

The recommended approach is an incremental, strictly-sequenced rewrite that ports the framework skeleton first (state, errors, middleware, routing), then the DB layer, then route handlers — in that dependency order. The most important risk is treating this as a clean-slate project: the existing DB schema, auth tokenizer, Diesel query code, and frontend components are assets that should be ported with minimal changes, not rewritten. Feature additions (sub-projects, travel routes) belong in a separate phase after the framework port is stable.

The three highest-priority risks are: (1) async Diesel misuse — deadpool-diesel's `interact()` closure must be kept synchronous and every existing transaction must be preserved exactly; (2) Axum extractor ordering — `Json<T>` must be the last handler parameter or requests silently return 400; and (3) schema evolution ordering — adding `sub_project_id` and `travel_route` requires the strict sequence of migrate → regenerate schema.rs → update structs → compile. All three risks cause hard-to-diagnose runtime failures rather than compile errors.

## Key Findings

### Recommended Stack

The backend migration replaces Rocket's ecosystem with Axum's: `axum 0.8` as the HTTP framework, `tower-http 0.6` for CORS/tracing/compression middleware, `diesel-async 0.5` + `deadpool-diesel 0.6` as the async Diesel bridge (replacing `rocket_db_pools`), and `jsonwebtoken 9` replacing `jwt-simple` for better community support and error types. The existing `argon2 0.5`, `serde 1.x`, `chrono 0.4`, `tracing 0.1`, and `thiserror` (upgrade to 2.x) all carry over unchanged.

The frontend stack is already modern and correct for 2025/2026 — React 19, Vite 6, TanStack Router 1.x, Tailwind 4, React Hook Form + Zod — and must not change frameworks. The only recommended addition is `@tanstack/react-query 5` to replace the manual `useState + useEffect + axios` fetch patterns that currently produce unmaintainable components like the 766-line `TrackingPage.tsx`.

**Core technologies:**
- axum 0.8: HTTP framework — Tokio-native, Tower ecosystem, cleaner async than Rocket 0.5
- diesel-async 0.5 + deadpool-diesel 0.6: async DB access — preserves existing Diesel queries, avoids rewriting schema/ORM
- jsonwebtoken 9: JWT — wider community adoption and better error types than jwt-simple
- tower-http 0.6: middleware — CorsLayer, TraceLayer, CompressionLayer as first-class Tower layers
- @tanstack/react-query 5: server state — replaces manual fetch patterns, dramatically reduces component complexity

### Expected Features

The team already has and depends on: full time entry CRUD (with begin/end/pause), client + project + activity + user management, JWT authentication, admin/user role separation, pagination. These must not regress.

There is one critical bug to fix early: the frontend hardcodes `billed = performed` on entry create and edit, but the schema supports independent values. This is a bug fix, not a feature, and is P1.

New features required for launch: user-scoped entry view (currently all users see all entries), admin role gating in UI, date-based filtering on the entry list, sub-projects (Teilprojekt — not in current schema), and travel routes (Fahrstrecken — listed in PROJECT.md as active requirement).

**Must have (table stakes, v1):**
- Billed hours independent of performed hours — fix current hardcoding bug
- User-scoped entry view — users must not see each other's entries
- JWT auth with configurable expiration — currently hardcoded 5-day tokens
- Admin role gating in UI — hide admin functions from regular users
- Date-based filtering on entry list — minimum weekly view
- Sub-projects (Teilprojekt) — schema migration + CRUD + selection on entry form
- Travel routes (Fahrstrecken) — new entity, separate from time entry activities

**Should have (v1.x):**
- Calendar week view (Day/Week/Month toggle) — legacy app had this; primary navigation mode for consultants
- Copy/Paste entry shortcut — visible in legacy UI; reduces daily entry friction
- Employee filter in admin view — admin reviewing team entries
- Activity short codes (token) in compact list views — already in schema, not surfaced in UI

**Defer (v2+):**
- Hierarchical summary reports (Client > Month > Project drill-down) — explicitly deferred in PROJECT.md
- PDF export and CSV data export
- Employee calendar overview (Mitarbeiterkalender)
- Daily total summary in calendar view

### Architecture Approach

The architecture is a two-layer separation: an Axum HTTP layer (middleware stack, router, handlers) sits above a framework-agnostic Diesel DB layer. The Axum layer uses `AppState` (with `Arc`) containing the deadpool-diesel pool and JWT tokenizer, distributed to handlers via `State<AppState>`. Auth is handled by a single `require_auth` middleware applied to a protected `Router` subtree — not per-handler. The DB layer (`db/` module) is almost entirely reusable from the Rocket version; only the connection type argument changes from `rocket_db_pools::Connection<DB>` to `&deadpool_diesel::mysql::Pool`.

**Major components:**
1. `state.rs` + `error.rs` — shared `AppState` struct and unified error type with `impl IntoResponse`; foundation for everything else
2. `middleware/auth.rs` — replaces Rocket's `guard.rs`; extracts Bearer JWT from headers, injects `User` into request extensions
3. `routes/` (per-domain modules) — Axum handlers replacing Rocket route functions; extractor-based inputs, db layer calls, JSON responses
4. `db/` (per-domain + middlelayer) — Diesel query functions ported from `rocket_db_pools::Connection` to `deadpool-diesel` pool; framework-agnostic logic preserved
5. `auth/tokenizer.rs` — Ed25519 JWT keypair; already framework-agnostic, only needs Rocket imports removed

### Critical Pitfalls

1. **Diesel async blocking misuse** — deadpool-diesel's `interact()` closure runs sync Diesel code in a spawn_blocking thread; do NOT wrap async Diesel calls in spawn_blocking or vice versa. Every existing `transaction(|mut conn| Box::pin(async move { ... }))` must be preserved exactly.

2. **Axum extractor ordering** — `Json<T>` (body-consuming, implements `FromRequest`) must be the last handler parameter. Auth extractor must implement `FromRequestParts` (headers only, not body). Wrong order produces opaque 400 errors, not compile errors.

3. **Axum State vs Extension** — use `State<AppState>` (registered with `.with_state()`) for the DB pool and tokenizer, never `Extension`. `Extension` for app state produces runtime panics instead of compile errors when a route is missing from the state tree.

4. **Schema evolution sequence** — adding `sub_project_id` to `tracking` and creating `travel_route` table requires strict ordering: `diesel migration run` → `diesel print-schema > src/schema.rs` → update Rust structs → compile → test. Skipping `print-schema` produces silent schema drift that fails compilation later.

5. **CORS layer placement** — `CorsLayer` must be the outermost Tower layer so that preflight OPTIONS requests bypass auth middleware. If placed inside the protected router, OPTIONS preflight returns 401 and the frontend cannot make any requests.

## Implications for Roadmap

Based on research, the natural phase structure follows the build-order dependency graph from ARCHITECTURE.md and the feature priority matrix from FEATURES.md.

### Phase 1: Framework Foundation (Backend)

**Rationale:** Everything else depends on the Axum app skeleton. Until `AppState`, `error.rs`, middleware, and the router compile and start, no route handlers can be developed. This phase is pure port work — no new features, no schema changes.

**Delivers:** A running Axum server that handles JWT authentication and returns correct HTTP status codes. All middleware (CORS, tracing, auth) wired up correctly. No business routes yet — placeholder 200 responses only.

**Addresses:** JWT authentication (table stakes), CORS configuration

**Avoids:** Pitfall 3 (extractor ordering — establish correct pattern from the start), Pitfall 4 (State vs Extension — wire AppState correctly once), Pitfall 5 (CORS placement — CorsLayer as outermost layer from day one)

**Research flag:** Standard patterns — Axum router composition, AppState, Tower layers are well-documented. No additional phase research needed.

### Phase 2: DB Layer Port

**Rationale:** The DB layer is the most reusable asset from the Rocket version but requires targeted changes to connection types. Porting and testing this in isolation (before adding route handlers) enables confident assertion that Diesel queries are correct before HTTP concerns are layered on.

**Delivers:** All existing Diesel query functions working under deadpool-diesel. Migrations running at startup. Connection pool configured and tested.

**Addresses:** Schema management, transaction integrity

**Avoids:** Pitfall 1 (async Diesel blocking — deadpool-diesel interact() pattern established), Pitfall 2 (transaction closure shape — Box::pin pattern preserved from existing code), Pitfall 6 (last_insert_id() session safety — all existing transaction wrappers preserved)

**Research flag:** Standard patterns for deadpool-diesel pool setup. The `interact()` closure API should be verified against current deadpool-diesel docs before implementation — training data is MEDIUM confidence for exact API surface.

### Phase 3: Route Handler Port + Bug Fixes

**Rationale:** With the foundation and DB layer working, route handlers can be ported one module at a time. This phase also addresses the critical bugs discovered in research: user-scoped entry view, billed/performed independence, and admin role gating. These are not new features — they are correctness fixes that belong in the port phase.

**Delivers:** All existing API endpoints working under Axum. Bug fixes: user-scoped entry view, independent billed/performed fields on create and edit, admin-only UI gating. Rate limiting on `/login`.

**Addresses:** User-scoped entry view (P1), billed hours bug fix (P1), admin role gating (P1), JWT expiration configuration (P1)

**Avoids:** `sys_role` string enum bug (replace raw string with typed `Role` enum), password hash in logs (remove from error branch), Pitfall 3 (extractor ordering validated per handler)

**Research flag:** Standard patterns for Axum handlers. TanStack Router frontend migration (old `new Route`/`new RootRoute` API must be updated to current v1.x API) needs verification against TanStack Router v1.115.3 docs — this is flagged in PITFALLS.md as a known issue.

### Phase 4: New Features (Schema Extensions)

**Rationale:** Schema changes (sub-projects, travel routes) must come after the core data model is stable. Adding new tables and nullable FKs to a working system is lower risk than doing it during the framework migration. Both sub-projects and travel routes require their own backend CRUD, frontend forms, and migration sequence.

**Delivers:** Sub-project entity (Teilprojekt) with migration, CRUD, and time entry form integration. Travel routes entity (Fahrstrecken) with CRUD as a separate entity from time entry activities. Date-based filtering on the entry list.

**Addresses:** Sub-projects (P1), travel routes (P1), date filtering (P1)

**Avoids:** Pitfall 7 (schema evolution sequence — migrate → print-schema → update structs → compile; nullable columns with DEFAULT NULL for backward compatibility), f32 billing rounding (consider DECIMAL migration alongside sub-project migration)

**Research flag:** Schema migration strategy for adding sub_project_id to existing tracking rows needs careful planning. Travel routes entity design (separate table vs column on tracking) should be decided before migration is written. This phase likely benefits from a `/gsd:research-phase` call focused on MariaDB migration patterns and DECIMAL type mapping in Diesel.

### Phase 5: Calendar View + UX Polish

**Rationale:** The calendar week view is the highest-value UX feature (visible in legacy screenshots, explicitly listed in differentiators) but requires all time entry data, date filtering, and user-scoped views to exist first. Building it last ensures the underlying data model is stable. UX polish items (copy/paste entry, employee filter, activity tokens) belong in this phase.

**Delivers:** Calendar week view (Day/Week/Month toggle). Copy/Paste entry shortcut. Employee filter in admin view. Activity short codes in compact list views. @tanstack/react-query integration to refactor TrackingPage.

**Addresses:** Calendar view (P2), copy/paste (P2), employee filter (P2), activity tokens (P2)

**Uses:** @tanstack/react-query 5 (addition to frontend stack)

**Research flag:** Calendar week rendering in React is moderately complex (grid layout, entry overlap, day boundary handling). This phase likely benefits from a `/gsd:research-phase` call to evaluate whether to use a React calendar library (react-big-calendar, @fullcalendar/react) or build a custom grid. The legacy app's calendar design (visible in docs screenshots) should be the reference.

### Phase 6: Reporting (Deferred)

**Rationale:** Explicitly deferred in PROJECT.md. Requires stable time entry data model from all prior phases. The hierarchical summary view (Client > Month > Project) and PDF/CSV export are high-value but high-complexity, and the team needs to validate the core workflow first.

**Delivers:** Hierarchical summary reports (Client > Month > Project drill-down). CSV data export. Employee calendar overview (Mitarbeiterkalender). PDF export.

**Addresses:** Reporting (P3), export (P3)

**Research flag:** Reporting aggregation queries (GROUP BY with multiple levels) in Diesel need design. PDF generation in Rust (printpdf, headless Chrome, or frontend print CSS) needs evaluation. This phase should use `/gsd:research-phase`.

### Phase Ordering Rationale

- **Foundation before features:** Pitfalls 3, 4, and 5 are all wiring problems that manifest as silent runtime failures. Establishing the correct patterns (State, extractors, CORS) in Phase 1 prevents them from contaminating every subsequent handler.
- **DB layer before handlers:** Diesel async patterns (Pitfalls 1, 2, 6) are the highest-risk technical area of this rewrite. Isolating and testing the DB layer before HTTP handlers are added means bugs can be caught without HTTP noise.
- **Bug fixes in Phase 3, not Phase 1:** The billed/performed bug and user-scoping bug are in the handler layer, not the framework or DB layer. Fixing them during handler port (not before) keeps concerns separated.
- **Schema changes after port:** Adding new tables to a partially-ported codebase risks breaking Diesel compilation mid-migration. Phase 4 schema changes belong after Phase 3 delivers a fully working Axum backend.
- **Calendar in Phase 5:** Calendar view depends on date-based filtering (Phase 4) and user-scoped views (Phase 3). Building it last avoids rework.

### Research Flags

Phases likely needing `/gsd:research-phase` during planning:
- **Phase 4 (Schema Extensions):** MariaDB nullable FK migration patterns, Diesel DECIMAL type mapping, deadpool-diesel version compatibility at time of implementation
- **Phase 5 (Calendar View):** React calendar library evaluation vs custom implementation; @tanstack/react-query 5 migration patterns for existing axios-based code
- **Phase 6 (Reporting):** Diesel GROUP BY aggregation query patterns, PDF generation approach for Rust

Phases with standard patterns (skip research-phase):
- **Phase 1 (Framework Foundation):** Axum Router, AppState, Tower middleware are well-documented with clear patterns
- **Phase 2 (DB Layer Port):** deadpool-diesel pool setup follows documented pattern; `interact()` API is stable
- **Phase 3 (Handler Port):** Rocket-to-Axum handler mapping is a direct translation (see ARCHITECTURE.md mapping table)

## Confidence Assessment

| Area | Confidence | Notes |
|------|------------|-------|
| Stack | MEDIUM-HIGH | Core choices (Axum, diesel-async, deadpool-diesel) are HIGH confidence; exact patch versions need `cargo search` verification before use; TanStack Query addition is MEDIUM |
| Features | HIGH | Primary evidence from codebase inspection, legacy screenshots, and active PROJECT.md requirements — not reliant on training data for domain knowledge |
| Architecture | MEDIUM | Axum Router, state, extractors patterns are stable and HIGH confidence; deadpool-diesel `interact()` API and tower-http `AllowOrigin` API should be verified against current docs |
| Pitfalls | MEDIUM | All critical pitfalls derived from direct codebase inspection; Axum-specific pitfalls (extractor ordering, State vs Extension) are MEDIUM from training data; verify with integration tests |

**Overall confidence:** MEDIUM

### Gaps to Address

- **deadpool-diesel exact API:** Training data for deadpool-diesel 0.6 `interact()` closure shape and `AsyncMysqlConnection` setup is MEDIUM confidence. Verify against https://docs.rs/deadpool-diesel/latest/deadpool_diesel/ before Phase 2 implementation begins.
- **TanStack Router v1.x migration:** The existing frontend uses the old object-based API (`new Route`, `new RootRoute`). Migration to the current v1.x file-based or builder API needs to be designed before Phase 3 frontend work. Verify against TanStack Router v1.115.3 changelog.
- **MariaDB RETURNING clause:** PITFALLS.md flags that MariaDB 10.5+ supports `RETURNING` as an alternative to `last_insert_id()`. The target MariaDB version is unknown — verify whether the production database supports this before designing the migration strategy.
- **tower-governor rate limiting:** Flagged as LOW confidence in PITFALLS.md — verify that `tower-governor` is maintained and compatible with Axum 0.8 before using it for login rate limiting.
- **f32 vs DECIMAL for billed/performed:** The current schema uses `FLOAT(4,2)` for `performed` and `billed`. Migrating to `DECIMAL(10,2)` + `rust_decimal::Decimal` in Rust is the correct long-term fix but requires a schema migration and Cargo dependency addition. Decide before Phase 4 whether to include this in the sub-project migration.

## Sources

### Primary (HIGH confidence)
- Codebase inspection: `backend/src/` (Rocket 0.5 code, guard.rs, db/ modules, schema.rs, error.rs) — ground truth for current architecture
- Codebase inspection: `frontend/src/pages/TrackingPage.tsx`, `frontend/src/types/index.ts` — frontend patterns and API types
- Legacy UI screenshots: `docs/zeiterfassung_overview.png`, `docs/zeiterfassung_form.png`, `docs/zeiterfassung_detail_list.png`, `docs/zeiterfassung_tracked_time.png`, `docs/zeiterfassung_tracked_time_order.png` — authoritative record of feature set the team relies on
- `.planning/PROJECT.md` — active requirements (sub-projects, travel routes, deferred reporting)
- `Cargo.lock` — verified current dependency versions (diesel 2.1.6, tokio 1.42.0, argon2 0.5.3, jwt-simple 0.11.9)
- `frontend/package.json` — verified frontend versions (React 19, TanStack Router 1.115.3, Tailwind 4.1.3, Vite 6.2.0)

### Secondary (MEDIUM confidence)
- Training data: Axum 0.7.x/0.8.x stable API patterns (Router, extractors, State, IntoResponse, tower-http CorsLayer)
- Training data: deadpool-diesel 0.5/0.6 integration patterns and `interact()` closure shape
- Training data: diesel-async 0.5 `AsyncMysqlConnection` + transaction closure shape (`Box::pin(async move {})`)
- Training data: Harvest, Toggl, Clockify, Kimai feature sets (for competitive comparison)

### Tertiary (LOW confidence)
- Training data: MariaDB 10.5+ `RETURNING` clause support — verify against MariaDB release notes for target version
- Training data: `tower-governor` rate limiting crate — verify maintenance status and Axum 0.8 compatibility

---
*Research completed: 2026-02-13*
*Ready for roadmap: yes*
