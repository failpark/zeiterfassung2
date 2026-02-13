# Codebase Concerns

**Analysis Date:** 2026-02-13

## Tech Debt

**Hardcoded CORS Configuration:**
- Issue: CORS origin is hardcoded to `http://localhost:5173` in production code
- Files: `backend/src/lib.rs:28`
- Impact: Cannot deploy to different environments without code changes; security risk if frontend URL varies
- Fix approach: Move to environment configuration using Rocket Figment, add dynamic origin validation

**Incomplete Token Refresh Mechanism:**
- Issue: Auth system lacks automatic token refresh; tokens expire after server restart by design but no refresh endpoint
- Files: `backend/src/auth/tokenizer.rs:28-30`, `frontend/src/contexts/AuthContext.tsx:47-62`
- Impact: Users must re-login after token expiration; poor user experience
- Fix approach: Implement refresh token flow with separate long-lived refresh tokens; add `/auth/refresh` endpoint

**Console Logging in Frontend:**
- Issue: Multiple unguarded `console.error()` and `console.log()` statements in production code
- Files: `frontend/src/pages/TrackingPage.tsx:118`, `frontend/src/pages/UsersPage.tsx` (3+ locations), `frontend/src/contexts/AuthContext.tsx:34,57`, `frontend/src/pages/LoginPage.tsx`
- Impact: Sensitive information may leak to browser console; inconsistent error handling patterns
- Fix approach: Replace with structured logging library; implement proper error boundaries

## Security Considerations

**JWT Tokens Stored in localStorage:**
- Risk: XSS vulnerability could expose JWT tokens; localStorage is vulnerable to XSS attacks
- Files: `frontend/src/api/authApi.ts:15,20,25`, `frontend/src/contexts/AuthContext.tsx:32`
- Current mitigation: None
- Recommendations: Migrate to httpOnly secure cookies; implement CSRF tokens; add Content Security Policy headers

**Bearer Token Parsing Vulnerability:**
- Risk: Token parsing uses basic string split without robust header validation
- Files: `backend/src/guard.rs:32-39`
- Current mitigation: Token is still verified by JWT signature
- Recommendations: Use dedicated HTTP header parsing; validate header format before attempting extraction

**Unused Password Field Exposure:**
- Issue: `CreateUser` struct exposes password in serialization; password not being hashed before storage in some flows
- Files: `backend/src/db/user.rs:67-86`
- Impact: Password history or leakage in logs/responses
- Fix approach: Separate password input from user creation payload; hash immediately on input

**Missing Input Validation in Frontend:**
- Risk: Zod schemas only validate form shape, not business logic constraints
- Files: `frontend/src/pages/TrackingPage.tsx:35-44`
- Impact: Invalid time ranges (end before begin) can be submitted to backend
- Fix approach: Add cross-field validation in Zod schemas; validate date/time logic in frontend

## Known Issues

**Duplicate Tracking Allowed:**
- Issue: Comment indicates duplicate tracking is intentionally allowed "for now"
- Files: `backend/src/routes/tracking.rs:~170` (TODO comment)
- Workaround: Check backend response for conflicts
- Impact: Data integrity risk; unclear business rules

**Type Safety Issues with `any` Types:**
- Issue: Multiple locations use TypeScript `any` type, bypassing type checking
- Files: `frontend/src/utils/ui-utils.ts`, `frontend/src/api/axiosClient.ts:48`, `frontend/src/pages/LoginPage.tsx`, `frontend/src/pages/UsersPage.tsx`, `frontend/src/pages/DashboardPage.tsx`
- Impact: Loss of type safety in error handling and component props
- Priority: High - increases maintenance burden and runtime errors

## Fragile Areas

**TrackingPage Component:**
- Files: `frontend/src/pages/TrackingPage.tsx`
- Size: 766 lines
- Why fragile: Large monolithic component with 20+ state variables, complex form handling with two independent forms, multiple useEffect hooks with interdependent state
- Safe modification: Break into smaller components (FormSection, ProjectFilter, DataDisplay); extract form logic to custom hook; use state management library
- Test coverage: No tests exist; critical path has no coverage
- Risk: High - changes to filtering logic, pagination, or form submission can cause cascading failures

**Tracking Routes (Backend):**
- Files: `backend/src/routes/tracking.rs:331 lines`
- Why fragile: Route handler includes extensive test code inline; no separation of concerns between route logic and test fixtures
- Safe modification: Extract test fixtures to separate test module; move test setup to common utilities
- Test coverage: Tests present but tightly coupled to implementation

**Tracking Middle Layer:**
- Files: `backend/src/db/tracking/middlelayer.rs:367 lines`
- Why fragile: Complex business logic translating between frontend and database models; uses `unwrap()` on activities arrays
- Safe modification: Add proper error handling for missing activities; extract validation logic; add unit tests
- Risk: Line 310 and 330 use `.unwrap()` on activities which could panic

## Error Handling Issues

**Excessive Panics and Unwraps:**
- Issue: Multiple `expect()` and `unwrap()` calls in startup and database code
- Files: `backend/src/db/mod.rs:59,63,65,68`, `backend/src/lib.rs:36`, `backend/src/catchers.rs:21`
- Impact: Unrecoverable errors crash the server; no graceful degradation
- Fix approach: Use `Result<T>` types throughout; implement proper error logging and recovery paths

**Unwraps in Test Code (Production Code Concern):**
- Issue: Test code in route modules contains multiple unwraps; if tests are compiled in, production crashes if assertions fail
- Files: `backend/src/routes/activity.rs:138-258`, `backend/src/routes/tracking.rs:150+`
- Impact: Test logic could affect production if `#[cfg(test)]` boundary is breached
- Mitigation: Already protected by `#[cfg(test)]` but violates best practices
- Fix approach: Refactor test infrastructure to separate test helpers; use `?` operator with `Result` types

**Generic Error Messages:**
- Issue: Multiple error variants have minimal context
- Files: `backend/src/error.rs:46-51` (NotFound, ZeiterfassungInsert, Unknown, Internal)
- Impact: Difficult to debug production issues; unclear what went wrong
- Fix approach: Add context to errors using anyhow's context chains; log full error details server-side

## Performance Bottlenecks

**No Database Query Optimization:**
- Issue: Frontend loads "last page" with size 100 for all reference data on mount
- Files: `frontend/src/pages/TrackingPage.tsx:107-109`
- Impact: Large payloads on every page load; inefficient for large datasets
- Improvement: Implement pagination/virtualization; lazy-load reference data; add caching

**Unoptimized Re-renders:**
- Issue: No `useCallback`, `useMemo` used; TrackingPage re-fetches data on every dependency change
- Files: `frontend/src/pages/TrackingPage.tsx:126-130`
- Impact: Potential N+1 query patterns; unnecessary network requests
- Fix: Memoize fetch functions; deduplicate requests; use React Query or SWR

**Synchronous Database Connection Blocking:**
- Issue: Migrations run synchronously on startup in blocking thread
- Files: `backend/src/db/mod.rs:61-68`
- Impact: Server startup blocked until migrations complete; no timeout or cancellation mechanism
- Fix: Move to async migrations or implement timeout handling

## Missing Critical Features

**No Automatic Token Refresh:**
- Problem: Users must log in again after 5 days or server restart
- Blocks: Seamless user experience; long-running sessions
- Solution required: Refresh token flow with separate endpoints

**No Rate Limiting:**
- Problem: API has no protection against brute force or DoS attacks
- Blocks: Production deployment; security audits
- Solution: Implement per-IP rate limiting; add request throttling

**No Request/Response Logging (Production):**
- Problem: Tracing configured but structured logging not fully integrated into all routes
- Blocks: Production debugging; audit trails
- Solution: Comprehensive request/response logging middleware

**No Data Validation on Client Input:**
- Problem: Backend accepts arbitrary form values without business logic validation
- Blocks: Data quality guarantees
- Solution: Add validation layer in routes; return specific error messages to frontend

## Test Coverage Gaps

**Frontend Has No Tests:**
- What's not tested: All React components, hooks, API client, form validation
- Files: `frontend/src/**/*.tsx`, `frontend/src/**/*.ts`
- Risk: Component changes can break silently; no regression detection
- Priority: High - frontend is user-facing code

**Backend Test Code Mixed with Routes:**
- What's not tested: Error cases; edge conditions in database layer
- Files: `backend/src/routes/*.rs` (tests inline with routes)
- Coverage: Integration tests present but unit test coverage minimal
- Priority: Medium - core logic has tests but not isolated

**No E2E Tests:**
- What's not tested: Full workflows (login → create tracking → view data)
- Risk: Integration issues between frontend and backend undetected
- Priority: Medium - critical user paths untested

## Scaling Limits

**Single Database Connection Pool:**
- Current capacity: Rocket's default pool (likely 5-10 connections)
- Limit: Performance degrades with concurrent users; no connection pool tuning visible
- Scaling path: Configure `r2d2` pool size in Rocket config; implement connection timeouts

**Frontend Data Loading:**
- Current capacity: Loads 100 items per "last page" request
- Limit: UI becomes sluggish with >100 items in any category
- Scaling path: Implement proper pagination; add search/filter before loading; use virtualization

## Dependencies at Risk

**Rocket 0.5.0 (Outdated):**
- Risk: Version 0.5.0 released ~2021; current stable is 0.5.x but newer versions available; security patches may not be applied
- Impact: Known vulnerabilities in dependencies; limited maintenance
- Migration plan: Evaluate upgrading to latest Rocket 0.5.x patch; plan future migration to Rocket 0.6.0+

**jwt-simple 0.11.9:**
- Risk: Community-maintained library; should verify active maintenance
- Impact: JWT implementation security depends on library updates
- Alternative: Consider moving to `jsonwebtoken` crate (more widely maintained)

**React Router TanStack 1.115.3:**
- Risk: Major version mismatch potential; verify compatibility with React 19
- Impact: Routing may break with React updates
- Mitigation: Lock version and test thoroughly before updating

## Code Quality Issues

**No Prettier Configuration:**
- Issue: Frontend has no `.prettierrc` or Prettier config; relies only on ESLint
- Impact: Inconsistent code formatting; wasted review time on style issues
- Fix: Add `.prettierrc` with opinionated defaults

**Limited ESLint Rules:**
- Issue: ESLint config is minimal; only React hooks and refresh rules configured
- Files: `frontend/eslint.config.js`
- Impact: No rules for naming, import ordering, complexity checking
- Fix: Add comprehensive ESLint config; enable `@typescript-eslint` strict rules

**Inconsistent Error Handling Pattern:**
- Issue: Some pages use try/catch, others use error states, inconsistent error message display
- Files: Multiple page components
- Impact: Users see different error UI/messaging; maintenance burden
- Fix: Create centralized error handling hook; standardize error display patterns

---

*Concerns audit: 2026-02-13*
