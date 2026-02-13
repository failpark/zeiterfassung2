# Architecture

**Analysis Date:** 2026-02-13

## Pattern Overview

**Overall:** Monolithic client-server architecture with layered backend and component-driven frontend. The backend uses Rocket web framework with Diesel ORM for database abstraction, while the frontend is a React SPA with TanStack Router for client-side routing.

**Key Characteristics:**
- Separation of concerns through distinct API routes and database modules
- JWT-based authentication with Bearer token validation
- REST API with pagination support for all list endpoints
- Async/await patterns throughout (Rocket async handlers, React hooks)
- Type-safe database queries via Diesel with auto-generated schema

## Layers

**Frontend Layer (React):**
- Purpose: User interface and client-side state management
- Location: `/Users/phedias/code/zeiterfassung/frontend/src/`
- Contains: React components, pages, routing, contexts, API clients
- Depends on: API services, backend HTTP endpoints
- Used by: Browser/client applications

**API/Route Layer (Rocket):**
- Purpose: Expose REST endpoints for CRUD operations on all entities
- Location: `/Users/phedias/code/zeiterfassung/backend/src/routes/`
- Contains: HTTP handlers for login, activity, user, client, project, tracking
- Depends on: Database layer, authentication/guards, error handling
- Used by: Frontend, external consumers of REST API

**Business Logic/Database Layer:**
- Purpose: Entity operations and database interactions via Diesel ORM
- Location: `/Users/phedias/code/zeiterfassung/backend/src/db/`
- Contains: Create, Read, Update, Delete operations for User, Activity, Client, Project, Tracking entities
- Depends on: Database connection pool, Diesel query builder, schema definitions
- Used by: Route handlers

**Authentication/Security Layer:**
- Purpose: Token generation, verification, and user extraction from requests
- Location: `/Users/phedias/code/zeiterfassung/backend/src/auth/`, `/Users/phedias/code/zeiterfassung/backend/src/guard.rs`
- Contains: JWT tokenizer, Bearer token extraction from headers
- Depends on: JWT library (jwt_simple), Rocket request guards
- Used by: Route handlers, guard middleware

**Error Handling Layer:**
- Purpose: Unified error types and HTTP status mapping
- Location: `/Users/phedias/code/zeiterfassung/backend/src/error.rs`
- Contains: Error enum with variants for each error type, conversion to HTTP status codes
- Depends on: Rocket responder trait, thiserror crate
- Used by: All route handlers and database operations

**Infrastructure Layer:**
- Purpose: Logging, observability, CORS configuration
- Location: `/Users/phedias/code/zeiterfassung/backend/src/tracing.rs`, `/Users/phedias/code/zeiterfassung/backend/src/catchers.rs`, `/Users/phedias/code/zeiterfassung/backend/src/lib.rs`
- Contains: Tracing subscriber setup, error catching, CORS fairings
- Depends on: Tracing appender, Rocket fairings
- Used by: Rocket startup sequence

## Data Flow

**Authentication Flow:**
1. Frontend sends login credentials (email, password) to `/login` endpoint
2. Backend validates credentials against hashed password in `user` table
3. Backend generates JWT token with 5-day expiration (hardcoded in `lib.rs`)
4. Frontend stores token in localStorage and includes in all subsequent requests via Bearer header
5. Backend extracts and verifies token in guard implementation before processing requests

**Entity CRUD Flow:**
1. Frontend calls typed API client method (e.g., `trackingApi.createTracking()`)
2. API client uses axios instance configured with auth interceptors to make HTTP request
3. Rocket route handler extracts User from JWT guard, receives connection pool
4. Route handler calls database layer method (e.g., `Tracking::create()`)
5. Diesel ORM translates to SQL, executes against MySQL database
6. Result marshalled to JSON response, sent back to frontend
7. Frontend receives paginated result and updates component state

**State Management:**
- **Backend:** Stateless HTTP handlers, database is single source of truth
- **Frontend:** React Context API (AuthContext, ThemeContext) for global state, component-level useState for local UI state, localStorage for token persistence

## Key Abstractions

**Database Models:**
- Purpose: Represent tables as Rust structs with type-safe operations
- Examples: `User` (`backend/src/db/user.rs`), `Tracking` (`backend/src/db/tracking/tracking.rs`), `Activity` (`backend/src/db/activity.rs`)
- Pattern: Queryable/Selectable derivations for SELECT, Insertable for INSERT, create/read/update/delete async methods

**Pagination:**
- Purpose: Enable efficient querying of large datasets
- Examples: All list endpoints use `/page/<page_size>/<page>` pattern
- Pattern: Returns `PaginationResult<T>` struct with items, total_items, page, page_size, num_pages

**API Response Types:**
- Purpose: Ensure frontend and backend have matching type contracts
- Examples: `LoginRequest`, `CreateTracking`, `UpdateTracking` in `frontend/src/types/index.ts`
- Pattern: Request DTOs sent by frontend, response DTOs returned by backend, kept in sync

**Axios Client:**
- Purpose: Centralized HTTP client with auth interceptors
- Location: `frontend/src/api/axiosClient.ts`
- Pattern: Request interceptor adds Bearer token from localStorage, response interceptor redirects to login on 401

**Route Handlers:**
- Purpose: Extract user, parse request body, call DB layer, return response
- Examples: `backend/src/routes/activity.rs`, `backend/src/routes/tracking.rs`
- Pattern: Role-based access control (admin check via `user.sys_role`), automatic error serialization via Responder impl

## Entry Points

**Backend Server:**
- Location: `backend/src/bin/backend.rs`
- Triggers: `cargo run` or `just run`
- Responsibilities: Initialize tracing, build and launch Rocket server with all fairings, routes, and database pool

**Frontend Application:**
- Location: `frontend/src/main.tsx`
- Triggers: `npm run dev` (Vite dev server)
- Responsibilities: Create React root, render App component with providers

**Rocket Application Factory:**
- Location: `backend/src/lib.rs` (`rocket()` function)
- Triggers: Called from backend.rs main function
- Responsibilities: Attach CORS config, database pool, run migrations, attach all route handlers, register catchers, set up tokenizer state

**Frontend App Component:**
- Location: `frontend/src/App.tsx`
- Triggers: Rendered from main.tsx
- Responsibilities: Wrap application with AuthProvider and ThemeProvider, provide TanStack router

**Frontend Router Configuration:**
- Location: `frontend/src/router.tsx`
- Triggers: Called from App component
- Responsibilities: Define all routes, protect authenticated routes with beforeLoad redirects, render LoginPage for unauthenticated users

## Error Handling

**Strategy:** Comprehensive error enum with thiserror, automatic HTTP status mapping, JSON serialization for all responses

**Patterns:**
- **Database Errors:** Diesel errors converted via `#[from]` to `Error::Database`, mapped to 500 Internal Server Error
- **Authentication Errors:** Missing/invalid tokens return `Error::UnauthenticatedUser` (401), wrong credentials return `Error::WrongCredentials` (401)
- **Authorization Errors:** Role checks (admin-only endpoints) return `Error::ForbiddenAccess` (403)
- **Validation Errors:** Invalid input returns `Error::BadRequest(message)` (400)
- **Not Found:** Missing records return `Error::NotFound` (404)
- **Custom Responder:** Error type implements Rocket Responder trait to automatically serialize and send JSON with status code

**Frontend Error Handling:**
- Axios interceptor catches 401 responses and clears localStorage token, redirects to login
- Wrapped API calls in try-catch blocks in components, error messages extracted from `error.response?.data?.message`

## Cross-Cutting Concerns

**Logging:** Tracing subscriber configured in `backend/src/tracing.rs` with dual output: pretty-printed JSON to stdout for app logs, rolling daily files to `/var/log/zeiterfassung/server.log`. Filtered to only capture logs from crates starting with "zeiterfassung" target.

**Validation:**
- **Backend:** Diesel constraints (NOT NULL, VARCHAR length limits in schema), manual role checks in route handlers
- **Frontend:** Zod schemas defined in API clients can be added, currently relying on TypeScript types

**Authentication:** JWT tokens with 5-day expiration hardcoded in `backend/src/lib.rs` line 46-49. Token verified on every request via `guard.rs` FromRequest implementation. Claims include User struct with id, username, firstname, lastname, email, sys_role.

**CORS:** Hardcoded to allow `http://localhost:5173` (Vite dev server) in `backend/src/lib.rs` line 28. Configurable via rocket-cors fairing but currently development-only.

**Database Pooling:** Rocket database fairing manages MySQL connection pool, migrations run automatically on startup via `db::run_migrations` adhoc fairing.
