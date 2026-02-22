# Roadmap: Zeiterfassung

## Overview

This roadmap covers the full brownfield rewrite of the Zeiterfassung time tracking application from Rocket 0.5 to Axum 0.8, with schema extensions for sub-projects and travel routes. Phases follow strict build-order dependency: the Axum framework skeleton and async database layer must exist before any route handlers can be written, and schema additions (sub-projects, travel routes) land only after the core data model is stable and fully ported.

## Phases

**Phase Numbering:**
- Integer phases (1, 2, 3): Planned milestone work
- Decimal phases (2.1, 2.2): Urgent insertions (marked with INSERTED)

Decimal phases appear between their surrounding integers in numeric order.

- [ ] **Phase 1: Framework Foundation** - Axum app skeleton with AppState, middleware stack, CORS, structured logging, and justfile tooling
- [ ] **Phase 2: Database Layer** - deadpool-diesel async connection pool, existing migrations running, Diesel query layer ported to async
- [ ] **Phase 3: Authentication** - JWT login/logout endpoints with configurable expiration and typed role enum
- [ ] **Phase 4: User Management** - Admin CRUD for user accounts with role-based access control
- [ ] **Phase 5: Reference Data** - Client, project, and activity CRUD with pagination and admin/user permission separation
- [ ] **Phase 6: Time Entries** - Complete time entry CRUD with independent performed/billed hours, user scoping, date filtering, and copy-paste
- [ ] **Phase 7: Sub-Projects** - Teilprojekt schema migration, CRUD, and optional selection on time entry form
- [ ] **Phase 8: Travel Routes** - Fahrstrecken entity with migration, CRUD, and admin/user access
- [ ] **Phase 9: Frontend Polish** - TanStack Query integration, Tailwind Plus component refinement, and UI consistency pass

## Phase Details

### Phase 1: Framework Foundation

**Goal**: A running Axum server compiles, starts, handles requests, and correctly applies all middleware layers — providing the correct wiring patterns for all subsequent phases.

**Depends on**: Nothing (first phase)

**Requirements**: INFR-01, INFR-03, INFR-04, INFR-06

**Success Criteria** (what must be TRUE):
  1. `just run` starts the Axum server and it responds to HTTP requests on the configured port
  2. CORS origin is read from an environment variable; the server rejects origins not on the list and allows configured origins with correct preflight responses
  3. Structured tracing logs appear at startup and per-request with request method, path, and status code
  4. `just check`, `just fmt`, `just test`, and `just build` all function correctly from the justfile
  5. The unprotected health check endpoint returns 200; all protected routes return 401 without a valid Bearer token

**Plans:** 3 plans

Plans:
- [ ] 01-01-PLAN.md — Axum crate with AppState, config, error type, tracing, CORS, health endpoint
- [ ] 01-02-PLAN.md — Auth middleware skeleton (Bearer extraction, 401) and protected/public router split with tests
- [ ] 01-03-PLAN.md — Justfile verification and updates for all build/run/test/lint/format commands

---

### Phase 2: Database Layer

**Goal**: All existing Diesel query functions work under the deadpool-diesel async pool, and existing database migrations run cleanly at application startup.

**Depends on**: Phase 1

**Requirements**: INFR-02, INFR-05

**Success Criteria** (what must be TRUE):
  1. The deadpool-diesel connection pool initializes at startup and appears in tracing logs with pool size and connection string (password redacted)
  2. All existing Diesel migrations run via `just migrate` without errors against the development MariaDB instance
  3. A test confirms the pool can acquire a connection and execute a simple query (e.g., `SELECT 1`) within the deadpool `interact()` closure
  4. The `db/` module compiles with `diesel-async` connection types replacing all `rocket_db_pools::Connection` usages

**Plans:** 3 plans

Plans:
- [ ] 02-01-PLAN.md — Add deadpool-diesel dependency, DbPool type alias, pool init in AppState, InteractError handling, migrate recipe
- [ ] 02-02-PLAN.md — Port all db/ modules from rocket_db_pools to sync MysqlConnection signatures
- [ ] 02-03-PLAN.md — Wire startup migration runner, add pool connectivity integration tests

---

### Phase 3: Authentication

**Goal**: Users can log in with email and password, receive a JWT token, and use that token to access protected endpoints — with configurable token expiration and a typed system role.

**Depends on**: Phase 2

**Requirements**: AUTH-01, AUTH-02, AUTH-03, AUTH-04

**Success Criteria** (what must be TRUE):
  1. A user with valid credentials receives a JWT token from `POST /auth/login`; invalid credentials receive 401
  2. The JWT token expiration duration is read from an environment variable or config file (not hardcoded); changing the value changes token lifetime
  3. The system role (`user` / `admin`) is stored and returned as a typed enum, not a raw string; deserialization of an unknown role value returns an error
  4. A user can log out by discarding the token client-side; the frontend login page redirects to the main view on success

**Plans:** 3 plans

Plans:
- [ ] 03-01-PLAN.md — Role enum, AuthConfig, Tokenizer in AppState, AuthUser/AdminUser extractors
- [ ] 03-02-PLAN.md — POST /auth/login and /auth/logout handlers, remove require_auth middleware
- [ ] 03-03-PLAN.md — Frontend authApi URL update, AuthContext logout, TanStack Router v1 migration

---

### Phase 4: User Management

**Goal**: Admins can create, view, update, and delete user accounts through both the API and the frontend admin panel.

**Depends on**: Phase 3

**Requirements**: USER-01, USER-02, USER-03, USER-04

**Success Criteria** (what must be TRUE):
  1. An admin can create a new user account via the admin UI; the account appears in the paginated user list
  2. The user list endpoint is paginated; an admin can page through all users and see username, name, email, and role
  3. An admin can update a user's name, email, and role; changes are reflected immediately in the list
  4. An admin can delete a user; the account disappears from the list and the deleted user can no longer log in
  5. A non-admin user receives 403 Forbidden on all user management endpoints

**Plans**: TBD

Plans:
- [ ] 04-01: Implement GET /users (paginated), POST /users, PUT /users/:id, DELETE /users/:id with admin-only middleware
- [ ] 04-02: Port existing db/users.rs query functions to deadpool-diesel; add password hashing on create
- [ ] 04-03: Build admin user management UI (list, create form, edit form, delete confirmation)

---

### Phase 5: Reference Data

**Goal**: Admins can manage clients, projects, and activities through the API and UI; all users can browse them to use when creating time entries.

**Depends on**: Phase 4

**Requirements**: CLNT-01, CLNT-02, CLNT-03, CLNT-04, PROJ-01, PROJ-02, PROJ-03, PROJ-04, ACTV-01, ACTV-02, ACTV-03, ACTV-04

**Success Criteria** (what must be TRUE):
  1. An admin can create, update, and delete clients; a user can view the paginated client list
  2. An admin can create projects linked to a specific client; a user can filter the project list by client and page through results
  3. An admin can create activities with an optional short token (e.g., "E-Mail" → "EM"); users can view the full activity list
  4. Admin-only operations (create, update, delete) return 403 for non-admin users on all three entity types
  5. All three list endpoints support pagination and return correct total counts

**Plans**: TBD

Plans:
- [ ] 05-01: Implement client CRUD endpoints (GET paginated, POST, PUT, DELETE) with admin guards; port db/clients.rs
- [ ] 05-02: Implement project CRUD endpoints with client filter on GET; port db/projects.rs
- [ ] 05-03: Implement activity CRUD endpoints with optional token field; port db/activities.rs
- [ ] 05-04: Build frontend management UI for clients, projects, and activities (shared list+form pattern)

---

### Phase 6: Time Entries

**Goal**: Users can create, view, edit, and manage their own time entries with independently editable performed and billed hours; admins can see and manage entries for all users.

**Depends on**: Phase 5

**Requirements**: TRCK-01, TRCK-02, TRCK-03, TRCK-04, TRCK-05, TRCK-06, TRCK-07, TRCK-08, TRCK-09

**Success Criteria** (what must be TRUE):
  1. A user can create a time entry with date, begin/end time, pause, performed hours, billed hours, description, client, project, and one or more activities; performed and billed hours are independently editable (not auto-linked)
  2. A user viewing the entry list sees only their own entries by default; an admin can view all users' entries
  3. A user can edit and delete their own entries; an admin can edit or delete any entry
  4. A user can filter their entry list by date range and see only matching entries
  5. A user can copy an existing entry to create a new one pre-filled with the same values, ready for the date to be changed

**Plans**: TBD

Plans:
- [ ] 06-01: Implement GET /tracking (paginated, user-scoped, date filter), POST /tracking; fix billed=performed hardcoding bug
- [ ] 06-02: Implement PUT /tracking/:id, DELETE /tracking/:id with user-own / admin-any permission logic
- [ ] 06-03: Port tracking db/ query functions; add user-scoping WHERE clause and date range filter
- [ ] 06-04: Build time entry list UI with user-scoped view, date range filter, admin toggle for all-user view
- [ ] 06-05: Build time entry create/edit form with activity multi-select, independent billed/performed fields
- [ ] 06-06: Implement copy-entry action in the UI (pre-fill form from existing entry)

---

### Phase 7: Sub-Projects

**Goal**: Admins can manage Teilprojekte (sub-projects nested under a project), and users can optionally assign a sub-project when creating or editing a time entry.

**Depends on**: Phase 6

**Requirements**: SUBP-01, SUBP-02, SUBP-03, SUBP-04, SUBP-05

**Success Criteria** (what must be TRUE):
  1. The database migration adds the `sub_project` table and a nullable `sub_project_id` foreign key on `tracking` without breaking existing time entry data
  2. An admin can create, update, and delete sub-projects linked to a parent project; the sub-project list is filterable by project
  3. A user sees available sub-projects when a project is selected on the time entry form and can optionally assign one
  4. Existing time entries without a sub-project continue to display and function correctly after the migration

**Plans**: TBD

Plans:
- [ ] 07-01: Write and run Diesel migration: create sub_project table, add nullable sub_project_id to tracking; regenerate schema.rs
- [ ] 07-02: Add SubProject Rust structs, update Tracking struct; implement sub-project CRUD endpoints with admin guard
- [ ] 07-03: Update time entry form to show sub-project selector (filtered by selected project); update create/edit handlers to accept sub_project_id

---

### Phase 8: Travel Routes

**Goal**: Users can create and manage Fahrstrecken (travel route entries) as a separate entity from time entries; admins can view all users' routes.

**Depends on**: Phase 7

**Requirements**: TRVL-01, TRVL-02, TRVL-03, TRVL-04, TRVL-05

**Success Criteria** (what must be TRUE):
  1. The database migration adds a `travel_route` table with from, to, distance, and date fields; existing data is unaffected
  2. A user can create a travel route entry with origin, destination, distance, and date; it appears in their paginated route list
  3. A user can edit and delete their own travel routes; an admin can view all users' routes and delete any
  4. Travel routes appear in a separate section of the UI, distinct from time entries

**Plans**: TBD

Plans:
- [ ] 08-01: Write and run Diesel migration: create travel_route table; regenerate schema.rs; add TravelRoute Rust structs
- [ ] 08-02: Implement GET /travel-routes (paginated, user-scoped), POST, PUT/:id, DELETE/:id with user-own / admin-any permissions
- [ ] 08-03: Build travel routes UI (list with pagination, create/edit form, admin all-users view)

---

### Phase 9: Frontend Polish

**Goal**: The frontend is fully modernized with TanStack Query for server state management, consistent Tailwind Plus component styling, and all API integration points are clean and maintainable.

**Depends on**: Phase 8

**Requirements**: INFR-07

**Success Criteria** (what must be TRUE):
  1. All data-fetching components use TanStack Query instead of manual useState/useEffect/axios patterns; components no longer exceed 300 lines
  2. All pages use consistent Tailwind Plus component patterns (forms, tables, buttons, modals) with no visual inconsistency between sections
  3. The application is usable on a tablet viewport (768px+) with no horizontal overflow or broken layouts
  4. All API errors (network failure, 401, 403, 4xx, 5xx) display a human-readable message to the user rather than a blank state or console error

**Plans**: TBD

Plans:
- [ ] 09-01: Add @tanstack/react-query 5; wrap app in QueryClientProvider; migrate TrackingPage data fetching to useQuery/useMutation
- [ ] 09-02: Migrate remaining pages (clients, projects, activities, users, travel routes) to React Query pattern
- [ ] 09-03: Audit and apply Tailwind Plus component patterns across all pages; fix responsive layout issues
- [ ] 09-04: Add error boundary and error display components; ensure all API errors surface a user-facing message

---

## Progress

**Execution Order:**
Phases execute in numeric order: 1 -> 2 -> 3 -> 4 -> 5 -> 6 -> 7 -> 8 -> 9

| Phase | Plans Complete | Status | Completed |
|-------|----------------|--------|-----------|
| 1. Framework Foundation | 0/3 | Planning complete | - |
| 2. Database Layer | 0/3 | Planning complete | - |
| 3. Authentication | 0/3 | Not started | - |
| 4. User Management | 0/3 | Not started | - |
| 5. Reference Data | 0/4 | Not started | - |
| 6. Time Entries | 0/6 | Not started | - |
| 7. Sub-Projects | 0/3 | Not started | - |
| 8. Travel Routes | 0/3 | Not started | - |
| 9. Frontend Polish | 0/4 | Not started | - |

---
*Roadmap created: 2026-02-13*
*Coverage: 46/46 v1 requirements mapped*
