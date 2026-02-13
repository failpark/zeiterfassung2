# Zeiterfassung

## What This Is

A professional time tracking application for a small consulting team. Employees log time entries against clients, projects, and sub-projects with activity types, tracking both performed and billed hours. The app replaces a legacy system and is being rewritten with a modern Axum (Rust) backend and a lightweight frontend with Tailwind CSS styling.

## Core Value

Employees can accurately record and retrieve time entries with performed vs billed hour tracking — this is the foundation for client billing.

## Requirements

### Validated

<!-- Inferred from existing codebase -->

- ✓ User authentication with JWT tokens — existing
- ✓ CRUD for clients — existing
- ✓ CRUD for projects (belong to clients) — existing
- ✓ CRUD for activities (work types with optional tokens) — existing
- ✓ CRUD for tracking entries (date, begin/end, pause, performed/billed hours, description) — existing
- ✓ Many-to-many relationship between tracking entries and activities — existing
- ✓ User management with system roles (user/admin) — existing
- ✓ Paginated list endpoints — existing
- ✓ Password hashing with Argon2 — existing
- ✓ Database migrations with Diesel — existing

### Active

- [ ] Rewrite backend from Rocket to Axum
- [ ] Rewrite frontend with modern lightweight framework and Tailwind Plus styling
- [ ] JWT authentication with configurable token expiration
- [ ] CRUD for clients, projects, sub-projects, activities, users, tracking entries
- [ ] Sub-project support (Teilprojekt — projects within projects)
- [ ] Travel routes tracking (Fahrstrecken)
- [ ] Role-based access control (admin vs regular user)
- [ ] Clean, responsive UI matching docs/ styling references
- [ ] Evolve MariaDB schema where needed (preserve existing data compatibility)
- [ ] Common commands in justfile

### Out of Scope

- Reporting and drilldown summaries (client/month/project rollups) — deferred to later phase
- PDF export for billing — deferred to later phase
- OAuth/SSO login — email/password sufficient for small team
- Mobile app — web-first
- Real-time features (WebSockets) — not needed for time tracking

## Context

This is a brownfield rewrite of an existing time tracking application. The current codebase uses Rocket 0.5 (Rust) with Diesel ORM and a React/TypeScript frontend with TanStack Router.

**Existing database schema (MariaDB):**
- `user` — employees with username, name, email, password hash, system role
- `client` — companies being billed (Auftraggeber)
- `project` — belongs to a client
- `activity` — types of work (e.g., "Abstimmung per E-Mail", "Fahrzeit") with optional short tokens
- `tracking` — time entries with date, begin/end times, pause, performed/billed hours, description
- `tracking_to_activity` — many-to-many join table

**Legacy system context:** The original application (visible in docs/ screenshots) was a more feature-rich desktop-style web app with calendar views, employee overviews, and hierarchical reporting. The current Rocket implementation is a partial rewrite of that system.

**Styling references:** The docs/ directory contains screenshots from the legacy application showing the expected data layout and UX patterns — calendar week view, time entry forms, detail lists with hour rollups.

**Users:** Small consulting team. Currently no active users, but the plan is team adoption after the rewrite.

## Constraints

- **Backend framework**: Axum (Rust) — chosen for active development, Tokio ecosystem alignment, and better async Diesel integration
- **ORM**: Diesel with async support — already in use, schema compatibility needed
- **Database**: MariaDB — existing production database with data to preserve
- **Frontend styling**: Tailwind CSS with Tailwind Plus (formerly Tailwind UI) components
- **Frontend framework**: Lightweight, modern best practice — user is a backend dev, wants something easy to understand
- **Build tooling**: justfile for common commands
- **Schema evolution**: Must evolve from current schema, not break existing data

## Key Decisions

| Decision | Rationale | Outcome |
|----------|-----------|---------|
| Axum over Rocket | More active development, tighter Tokio/Diesel async integration | — Pending |
| JWT with configurable expiration | Stateless auth works well for SPA, expiration was hardcoded before | — Pending |
| Keep Diesel ORM | Already in use, async support now available, schema compatibility | — Pending |
| Evolve schema (not fresh start) | Existing data in MariaDB must be preserved | — Pending |
| Keep sub-projects and travel routes | Features from legacy system still needed by team | — Pending |
| Defer reporting/PDF to later phase | Get core CRUD working first, add reporting later | — Pending |

---
*Last updated: 2026-02-13 after initialization*
