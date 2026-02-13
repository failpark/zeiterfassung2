# Technology Stack

**Analysis Date:** 2026-02-13

## Languages

**Primary:**
- Rust 2021 edition - Backend service and core business logic
- TypeScript 5.7 - React frontend application
- SQL - Database migrations and schema

**Secondary:**
- JavaScript (ES2020+) - Node.js tooling and build scripts
- TOML - Configuration files (Cargo.toml, diesel.toml)

## Runtime

**Backend:**
- Rust (compiled binary) - `backend/src/bin/backend.rs` provides async runtime via rocket
- Rocket 0.5.0 web framework with async/await support

**Frontend:**
- Node.js (development and build time)
- Browser runtime (modern ES2020 compatible browsers)

**Package Manager:**
- Rust: Cargo with workspace configuration
  - Lockfile: `Cargo.lock` present
- Node.js: npm
  - Lockfile: `frontend/package-lock.json` present

## Frameworks

**Backend Web:**
- Rocket 0.5.0 - REST API framework with Tokio async runtime
- Rocket CORS 0.6.0 - Cross-origin resource sharing support
- Rocket DB Pools 0.1.0 - Database connection pooling with Diesel MySQL

**Frontend UI:**
- React 19.0.0 - Core UI framework
- TanStack React Router 1.115.3 - Client-side routing
- Vite 6.2.0 - Build tool and dev server
- Tailwind CSS 4.1.3 - Utility-first CSS framework

**Frontend Forms:**
- React Hook Form 7.55.0 - Form state management
- Zod 3.24.2 - Schema validation
- @hookform/resolvers 5.0.1 - Form validation resolver

**Frontend UI Components:**
- HeadlessUI React 2.2.1 - Unstyled accessible components
- Heroicons React 2.2.0 - Icon library
- Lucide React 0.487.0 - Additional icon library
- React DatePicker 8.3.0 - Date input component

**Build/Dev:**
- TypeScript 5.7.2 - Type checking for frontend
- Vite 6.2.0 - Frontend bundler and dev server
- @vitejs/plugin-react 4.3.4 - React Fast Refresh support
- ESLint 9.21.0 - JavaScript linting
- typescript-eslint 8.24.1 - TypeScript linting
- Tailwind CSS CLI 4.1.3 - CSS processing

## Key Dependencies

**Backend - Critical:**
- Diesel 2.1.4 - ORM for MySQL database interactions
- Rocket 0.5.0 - REST API framework with middleware support
- JWT-Simple 0.11.9 - JWT token generation and validation
- Argon2 0.5.2 - Password hashing algorithm
- Chrono 0.4.31 - DateTime handling and serialization

**Backend - Infrastructure:**
- Rocket DB Pools 0.1.0 - Connection pooling layer
- Diesel Migrations 2.1.0 - Schema migration management
- Tracing 0.1.40 - Structured logging framework
- Tracing Appender 0.2.3 - Async file logging
- Tracing Subscriber 0.3.18 - Log subscriber with JSON output
- Serde 1.0.193 - JSON serialization/deserialization
- Thiserror 1.0.50 - Error type derivation
- Typeshare 1.0.1 - Type sharing between Rust and TypeScript

**Backend - Development:**
- Test-case 3.3.1 - Parameterized testing
- Pretty Assertions 1.4.0 - Assertion output formatting
- Fake 2.9.1 - Test data generation
- Rand 0.8.5 - Random number generation
- Tracing-test 0.2.4 - Tracing framework integration tests

**Frontend - Critical:**
- Axios 1.8.4 - HTTP client with request/response interceptors
- JWT Decode 4.0.0 - JWT token decoding (client-side)
- Date-fns 4.1.0 - Date utility functions

**Frontend - Utilities:**
- CLSX 2.1.1 - Conditional class name utilities
- Tailwind Merge 3.2.0 - Tailwind CSS class composition
- Autoprefixer 10.4.21 - CSS vendor prefix generator
- PostCSS 8.5.3 - CSS transformation

## Configuration

**Backend Environment:**
- Database: MySQL configured via `Rocket.toml` - `databases.zeiterfassung.url`
- Secret key: Defined in `Rocket.toml` - `secret_key`
- CORS: Hardcoded to allow `http://localhost:5173` in `backend/src/lib.rs` line 28
- Sample config: `.env.sample` shows `DATABASE_URL=mysql://user:pass@ip:port/database`

**Frontend Environment:**
- Development: `.env.development` sets `VITE_API_URL=http://localhost:8000`
- Production: `.env.production` sets `VITE_API_URL=/api`
- Build target: ES2020, DOM APIs enabled

**Code Style:**
- Rust: Formatted via rustfmt with `rustfmt.toml`:
  - Hard tabs with 2 spaces
  - Vertical import layout
  - Grouped imports (Std/External/Crate)
- TypeScript/JavaScript: ESLint 9.21.0 with typescript-eslint for React and modern JS

**Build Configuration:**
- Frontend: `vite.config.ts` with React plugin enabled
- Backend: Cargo workspace with distributed build support via cargo-dist
- TypeScript: Strict mode enabled, target ES2020, with isolated modules

## Platform Requirements

**Development:**
- Rust 1.70+ (for 2021 edition)
- Node.js 16+ (for npm and Vite)
- MySQL 5.7+ (local or remote instance)
- Diesel CLI (for migrations)
- `just` command runner (justfile present at root)

**Production:**
- MySQL database server
- Linux x86_64 target (cargo-dist configured for `x86_64-unknown-linux-gnu`)
- Static file serving (Rocket handles API, frontend built to static assets)

**Log Output:**
- Stdout: Pretty-printed logs from `zeiterfassung_backend` crate
- File: JSON-formatted logs to `/var/log/zeiterfassung/server.log` (daily rolling)

---

*Stack analysis: 2026-02-13*
