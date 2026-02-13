# Requirements: Zeiterfassung

**Defined:** 2026-02-13
**Core Value:** Employees can accurately record and retrieve time entries with performed vs billed hour tracking for client billing

## v1 Requirements

Requirements for initial release. Each maps to roadmap phases.

### Authentication

- [ ] **AUTH-01**: User can log in with email and password and receive a JWT token
- [ ] **AUTH-02**: JWT token expiration is configurable via environment/config (not hardcoded)
- [ ] **AUTH-03**: User can log out (token invalidation on client side)
- [ ] **AUTH-04**: System role is a typed enum (user/admin) not a raw string

### Users

- [ ] **USER-01**: Admin can create new user accounts
- [ ] **USER-02**: Admin can view list of all users (paginated)
- [ ] **USER-03**: Admin can update user details
- [ ] **USER-04**: Admin can delete users

### Clients

- [ ] **CLNT-01**: Admin can create clients
- [ ] **CLNT-02**: User can view list of clients (paginated)
- [ ] **CLNT-03**: Admin can update client details
- [ ] **CLNT-04**: Admin can delete clients

### Projects

- [ ] **PROJ-01**: Admin can create projects linked to a client
- [ ] **PROJ-02**: User can view projects filtered by client (paginated)
- [ ] **PROJ-03**: Admin can update project details
- [ ] **PROJ-04**: Admin can delete projects

### Sub-Projects

- [ ] **SUBP-01**: Admin can create sub-projects linked to a project
- [ ] **SUBP-02**: User can view sub-projects filtered by project
- [ ] **SUBP-03**: Admin can update sub-project details
- [ ] **SUBP-04**: Admin can delete sub-projects
- [ ] **SUBP-05**: User can optionally select a sub-project on a time entry

### Activities

- [ ] **ACTV-01**: Admin can create activities with optional token (short code)
- [ ] **ACTV-02**: User can view list of activities
- [ ] **ACTV-03**: Admin can update activity details
- [ ] **ACTV-04**: Admin can delete activities

### Time Entries

- [ ] **TRCK-01**: User can create a time entry with date, begin, end, pause, performed hours, billed hours, description, client, project, and activities
- [ ] **TRCK-02**: Performed and billed hours are independently editable (not auto-linked)
- [ ] **TRCK-03**: User can edit their own time entries
- [ ] **TRCK-04**: Admin can edit any user's time entries
- [ ] **TRCK-05**: Admin can delete time entries
- [ ] **TRCK-06**: User sees only their own entries by default
- [ ] **TRCK-07**: Admin can view all users' entries
- [ ] **TRCK-08**: User can filter entries by date range
- [ ] **TRCK-09**: User can copy an existing entry to create a new one (Copy/Paste)

### Travel Routes

- [ ] **TRVL-01**: User can create a travel route entry (from, to, distance, date)
- [ ] **TRVL-02**: User can view their travel routes (paginated)
- [ ] **TRVL-03**: User can edit their travel routes
- [ ] **TRVL-04**: Admin can view all travel routes
- [ ] **TRVL-05**: Admin can delete travel routes

### Infrastructure

- [ ] **INFR-01**: Backend built with Axum (replacing Rocket)
- [ ] **INFR-02**: Diesel async with deadpool-diesel for MariaDB connection pooling
- [ ] **INFR-03**: Structured logging with tracing
- [ ] **INFR-04**: CORS configuration via environment variable (not hardcoded)
- [ ] **INFR-05**: Database migrations for schema evolution (sub-projects, travel routes)
- [ ] **INFR-06**: Common commands in justfile (build, run, test, lint, format, migrations)
- [ ] **INFR-07**: Frontend built with modern lightweight framework and Tailwind Plus styling

## v2 Requirements

Deferred to future release. Tracked but not in current roadmap.

### UX Enhancements

- **CALV-01**: Calendar week view with Day/Week/Month toggle
- **CALV-02**: Per-user daily total display in calendar
- **ADMN-01**: Admin-gated UI (hide management functions from regular users in frontend)
- **ADMN-02**: Employee filter in admin view

### Reporting

- **REPT-01**: Hierarchical summary view (Client > Month > Project drill-down with billed/performed rollups)
- **REPT-02**: PDF export for billing
- **REPT-03**: CSV data export
- **REPT-04**: Employee calendar overview (Mitarbeiterkalender)

## Out of Scope

Explicitly excluded. Documented to prevent scope creep.

| Feature | Reason |
|---------|--------|
| Real-time timer / start-stop tracking | Consultants log time after the fact; timer adds complexity for wrong workflow |
| OAuth/SSO login | Small team; email/password with JWT is sufficient |
| Mobile app | Web-first; ensure responsive design for occasional mobile use |
| Invoice generation | Invoicing has legal/tax requirements (German Vorsteuer); let accounting software handle it |
| Billable rates per user/project | Team uses explicit billed hours, not rate-based calculations |
| WebSockets / real-time updates | Small team; no concurrent editing scenarios |
| Automated email reports | Build reporting UI first; manual export is sufficient |
| Project budget tracking | Separate product feature scope; performed/billed gives natural ceiling |

## Traceability

Which phases cover which requirements. Updated during roadmap creation.

| Requirement | Phase | Status |
|-------------|-------|--------|
| AUTH-01 | — | Pending |
| AUTH-02 | — | Pending |
| AUTH-03 | — | Pending |
| AUTH-04 | — | Pending |
| USER-01 | — | Pending |
| USER-02 | — | Pending |
| USER-03 | — | Pending |
| USER-04 | — | Pending |
| CLNT-01 | — | Pending |
| CLNT-02 | — | Pending |
| CLNT-03 | — | Pending |
| CLNT-04 | — | Pending |
| PROJ-01 | — | Pending |
| PROJ-02 | — | Pending |
| PROJ-03 | — | Pending |
| PROJ-04 | — | Pending |
| SUBP-01 | — | Pending |
| SUBP-02 | — | Pending |
| SUBP-03 | — | Pending |
| SUBP-04 | — | Pending |
| SUBP-05 | — | Pending |
| ACTV-01 | — | Pending |
| ACTV-02 | — | Pending |
| ACTV-03 | — | Pending |
| ACTV-04 | — | Pending |
| TRCK-01 | — | Pending |
| TRCK-02 | — | Pending |
| TRCK-03 | — | Pending |
| TRCK-04 | — | Pending |
| TRCK-05 | — | Pending |
| TRCK-06 | — | Pending |
| TRCK-07 | — | Pending |
| TRCK-08 | — | Pending |
| TRCK-09 | — | Pending |
| TRVL-01 | — | Pending |
| TRVL-02 | — | Pending |
| TRVL-03 | — | Pending |
| TRVL-04 | — | Pending |
| TRVL-05 | — | Pending |
| INFR-01 | — | Pending |
| INFR-02 | — | Pending |
| INFR-03 | — | Pending |
| INFR-04 | — | Pending |
| INFR-05 | — | Pending |
| INFR-06 | — | Pending |
| INFR-07 | — | Pending |

**Coverage:**
- v1 requirements: 42 total
- Mapped to phases: 0
- Unmapped: 42

---
*Requirements defined: 2026-02-13*
*Last updated: 2026-02-13 after initial definition*
