# Codebase Structure

**Analysis Date:** 2026-02-13

## Directory Layout

```
zeiterfassung/
├── backend/                    # Rust backend application (Rocket + Diesel)
│   └── src/
│       ├── bin/
│       │   └── backend.rs      # Server entry point
│       ├── auth/               # JWT authentication & tokenization
│       │   ├── mod.rs
│       │   └── tokenizer.rs
│       ├── db/                 # Database models and operations (Diesel)
│       │   ├── mod.rs
│       │   ├── activity.rs     # Activity CRUD
│       │   ├── client.rs       # Client CRUD
│       │   ├── project.rs      # Project CRUD
│       │   ├── user.rs         # User CRUD + password hashing
│       │   ├── helper.rs       # Database utilities
│       │   └── tracking/       # Time tracking entity with sub-modules
│       │       ├── mod.rs
│       │       ├── tracking.rs
│       │       ├── middlelayer.rs
│       │       └── tracking_to_activity.rs
│       ├── routes/             # HTTP endpoint handlers
│       │   ├── mod.rs
│       │   ├── login.rs        # POST /login with JWT generation
│       │   ├── activity.rs     # CRUD for activities
│       │   ├── client.rs       # CRUD for clients
│       │   ├── project.rs      # CRUD for projects
│       │   ├── user.rs         # CRUD for users
│       │   └── tracking.rs     # CRUD for time entries
│       ├── test/               # Test utilities and fixtures
│       │   ├── mod.rs
│       │   ├── db.rs
│       │   ├── gen_fns.rs      # Test data generation
│       │   ├── methods.rs
│       │   └── token.rs        # Token generation for tests
│       ├── lib.rs              # Rocket app factory + module exports
│       ├── main.rs             # Server startup (binary)
│       ├── error.rs            # Error enum + HTTP status mapping
│       ├── guard.rs            # JWT extraction from requests
│       ├── schema.rs           # Diesel-generated schema (DO NOT EDIT)
│       ├── tracing.rs          # Logging setup
│       └── catchers.rs         # Global error handlers
├── frontend/                   # React/TypeScript frontend (Vite)
│   ├── src/
│   │   ├── api/                # API client modules
│   │   │   ├── index.ts        # Barrel export
│   │   │   ├── axiosClient.ts  # Axios instance + interceptors
│   │   │   ├── authApi.ts      # Login, token management
│   │   │   ├── userApi.ts      # User CRUD calls
│   │   │   ├── clientApi.ts    # Client CRUD calls
│   │   │   ├── projectApi.ts   # Project CRUD calls
│   │   │   ├── activityApi.ts  # Activity CRUD calls
│   │   │   └── trackingApi.ts  # Time entry CRUD calls
│   │   ├── components/
│   │   │   ├── layout/
│   │   │   │   └── Layout.tsx  # Main layout wrapper with sidebar/nav
│   │   │   └── ui/             # Reusable UI components
│   │   │       ├── Button.tsx
│   │   │       ├── Input.tsx
│   │   │       ├── Dialog.tsx
│   │   │       └── DataTable.tsx
│   │   ├── pages/              # Page components (one per route)
│   │   │   ├── LoginPage.tsx
│   │   │   ├── DashboardPage.tsx
│   │   │   ├── TrackingPage.tsx
│   │   │   ├── ActivitiesPage.tsx
│   │   │   ├── ClientsPage.tsx
│   │   │   ├── ProjectsPage.tsx
│   │   │   └── UsersPage.tsx
│   │   ├── contexts/           # React Context for global state
│   │   │   ├── AuthContext.tsx # User + auth state
│   │   │   └── ThemeContext.tsx# Dark/light mode toggle
│   │   ├── types/
│   │   │   └── index.ts        # TypeScript interfaces (User, Tracking, etc)
│   │   ├── utils/
│   │   │   ├── date-utils.ts   # Date/time formatting helpers
│   │   │   └── ui-utils.ts     # UI helper functions (cn())
│   │   ├── assets/             # Images, static files
│   │   ├── index.css           # Global styles (Tailwind directives)
│   │   ├── App.tsx             # App wrapper with providers
│   │   ├── router.tsx          # TanStack Router configuration
│   │   └── main.tsx            # React root initialization
│   ├── public/                 # Static files served as-is
│   ├── package.json            # Dependencies (React, axios, Tailwind, etc)
│   ├── vite.config.ts          # Vite build configuration
│   ├── tsconfig.json           # TypeScript configuration
│   ├── tsconfig.app.json       # TypeScript app-specific config
│   ├── tsconfig.node.json      # TypeScript node environment config
│   ├── tailwind.config.js      # Tailwind CSS configuration
│   ├── eslint.config.js        # ESLint rules
│   └── index.html              # HTML entry point
├── migrations/                 # Diesel SQL migrations
│   ├── YYYY-MM-DD-HHMMSS_create_user/
│   ├── YYYY-MM-DD-HHMMSS_create_client/
│   ├── YYYY-MM-DD-HHMMSS_create_project/
│   ├── YYYY-MM-DD-HHMMSS_create_activity/
│   ├── YYYY-MM-DD-HHMMSS_create_tracking/
│   └── YYYY-MM-DD-HHMMSS_create_tracking_to_activity/
├── docs/                       # Documentation
├── .github/workflows/          # CI/CD pipeline configurations
├── .planning/codebase/         # GSD analysis documents
├── Cargo.toml                  # Rust workspace config
├── Cargo.lock                  # Rust dependency lock
├── Rocket.toml                 # Rocket config (DB connection, secret key)
├── diesel.toml                 # Diesel CLI config
├── rustfmt.toml                # Rust formatter config (2-space hard tabs)
├── justfile                    # Task runner (build, test, lint)
└── CLAUDE.md                   # Instructions for Claude Code
```

## Directory Purposes

**backend/src:**
- Purpose: All Rust backend code
- Contains: Route handlers, database models, authentication, error handling
- Key files: `lib.rs` (Rocket factory), `main.rs` (entry point)

**backend/src/routes:**
- Purpose: HTTP endpoint definitions
- Contains: One file per entity type with GET/POST/PATCH/DELETE handlers
- Key files: All route files mount themselves via `mount()` function called in `lib.rs`

**backend/src/db:**
- Purpose: Database layer with Diesel ORM models
- Contains: CRUD operations, model structs, database utilities
- Key files: `mod.rs` (DB connection pool setup), individual entity files

**backend/src/auth:**
- Purpose: JWT generation and verification
- Contains: Tokenizer struct that signs/verifies JWT tokens
- Key files: `tokenizer.rs` (core implementation)

**frontend/src:**
- Purpose: All React/TypeScript frontend code
- Contains: Components, pages, API clients, state management
- Key files: `App.tsx` (root component), `main.tsx` (entry point), `router.tsx` (routing)

**frontend/src/api:**
- Purpose: HTTP client modules for backend communication
- Contains: Axios-based API calls organized by entity type
- Key files: `axiosClient.ts` (base HTTP client with interceptors), `index.ts` (barrel export)

**frontend/src/pages:**
- Purpose: Full-page components mapped to routes
- Contains: One component per route, manages page-level state
- Key files: LoginPage, DashboardPage, and entity pages

**frontend/src/components:**
- Purpose: Reusable UI components
- Contains: Layout wrapper and UI primitives
- Key files: `layout/Layout.tsx` (main template), `ui/*.tsx` (button, input, etc)

**frontend/src/contexts:**
- Purpose: Global state management via React Context
- Contains: AuthContext (user + login), ThemeContext (dark/light mode)
- Key files: Provider components with useContext hooks

**migrations:**
- Purpose: Diesel SQL migration files
- Contains: up.sql (create/alter) and down.sql (rollback) for each migration
- Generated by: `diesel migration generate <name>`

## Key File Locations

**Entry Points:**
- `backend/src/bin/backend.rs`: Server startup - calls `zeiterfassung_backend::rocket()` and launches
- `frontend/src/main.tsx`: React root - creates root and renders App
- `frontend/src/pages/LoginPage.tsx`: First page unauthenticated users see

**Configuration:**
- `Rocket.toml`: Database connection URL and secret key (MySQL connection string)
- `frontend/vite.config.ts`: Vite build configuration (minimal - uses React plugin)
- `rustfmt.toml`: Rust code formatting (2-space hard tabs)
- `diesel.toml`: Diesel CLI database URL configuration

**Core Logic:**
- `backend/src/lib.rs`: Rocket application factory - attaches all routes, fairings, database
- `backend/src/routes/`: All HTTP endpoints - one module per entity
- `frontend/src/router.tsx`: Client-side routing - defines all pages and auth guards
- `frontend/src/api/axiosClient.ts`: HTTP client - handles auth, errors, base URL

**Testing:**
- `backend/src/test/`: Test utilities for fake data generation, DB setup
- `backend/src/test/mod.rs`: Test module definitions

**Special Directories:**
- `target/`: Compiled Rust binaries and build artifacts (generated, not committed)
- `frontend/node_modules/`: npm dependencies (generated, not committed)
- `frontend/dist/`: Built frontend assets (generated, not committed)
- `/var/log/zeiterfassung/`: Server logs written by tracing appender (runtime, not in repo)

## Naming Conventions

**Files:**
- Backend Rust: `snake_case.rs` for modules
- Frontend TypeScript: `PascalCase.tsx` for React components, `camelCase.ts` for utilities/APIs
- Examples: `user.rs`, `UserProfile.tsx`, `axiosClient.ts`, `date-utils.ts`

**Directories:**
- Entity modules: plural names when grouping multiple files (`db/`, `routes/`, `api/`)
- Feature directories: descriptive names (`auth/`, `components/`, `contexts/`)
- Layers: standard names (`pages/`, `utils/`, `types/`)

**Functions/Methods:**
- Backend: snake_case functions (`create()`, `read()`, `update()`, `delete()`)
- Frontend: camelCase for functions/methods, PascalCase for React components
- API clients: verb-noun pattern (`getTrackings()`, `createUser()`, `updateProject()`)

**Types:**
- Backend: PascalCase structs (`User`, `Tracking`, `CreateActivity`)
- Frontend: PascalCase interfaces (`LoginRequest`, `PaginationResult<T>`)
- Both: Suffix DTO types with action (e.g., `CreateUser`, `UpdateTracking`)

**Constants:**
- Environment variables: SCREAMING_SNAKE_CASE (e.g., `VITE_API_URL`)
- Routes: lowercase with slashes (e.g., `/login`, `/tracking`, `/user/page/20/0`)

## Where to Add New Code

**New Feature (e.g., new entity type):**
- Primary code:
  - Backend: Create `backend/src/db/entity_name.rs` with model struct + CRUD methods
  - Backend: Create `backend/src/routes/entity_name.rs` with HTTP handlers + mount in `lib.rs`
  - Database: Create migration `migrations/YYYY-MM-DD-HHMMSS_create_entity_name/`
  - Frontend: Create `frontend/src/api/entityNameApi.ts` with API client
  - Frontend: Create `frontend/src/pages/EntityNamesPage.tsx` with page component
- Tests:
  - Backend: Add test module in `backend/src/db/entity_name.rs` using #[cfg(test)] + fake crate
  - Frontend: Would go in co-located `*.test.tsx` files (not currently set up)
- Types:
  - Frontend: Add interfaces to `frontend/src/types/index.ts` (Entity, CreateEntity, UpdateEntity)

**New Component/Module:**
- Implementation:
  - UI Component: `frontend/src/components/ui/ComponentName.tsx`
  - Layout Component: `frontend/src/components/layout/ComponentName.tsx`
  - Page Component: `frontend/src/pages/PageName.tsx`
- Utilities:
  - Helper functions: `frontend/src/utils/utility-name.ts`
  - API integrations: Already separated in `frontend/src/api/`

**Utilities:**
- Shared helpers: `frontend/src/utils/` (date formatting, UI utilities, etc)
- Database helpers: `backend/src/db/helper.rs`
- Error handling: `backend/src/error.rs` - add new Error variant

## Special Directories

**target/ (Rust build output):**
- Purpose: Compiled binaries, intermediate build artifacts
- Generated: Yes (via `cargo build`)
- Committed: No (.gitignore)

**frontend/node_modules/ (npm dependencies):**
- Purpose: Installed JavaScript packages
- Generated: Yes (via `npm install`)
- Committed: No (.gitignore)

**frontend/dist/ (built assets):**
- Purpose: Optimized frontend bundle ready for production
- Generated: Yes (via `npm run build`)
- Committed: No (.gitignore)

**/var/log/zeiterfassung/ (server logs):**
- Purpose: Daily rolling server logs with tracing output
- Generated: Yes (by tracing appender at runtime)
- Committed: No (not in repo)

**.planning/codebase/ (GSD analysis):**
- Purpose: Codebase analysis documents (ARCHITECTURE.md, STRUCTURE.md, etc)
- Generated: Yes (by GSD mapping commands)
- Committed: Yes (documents are version controlled)

**migrations/ (Diesel migrations):**
- Purpose: Database schema evolution scripts
- Generated: Via `diesel migration generate <name>`
- Committed: Yes (essential for reproducible database state)
