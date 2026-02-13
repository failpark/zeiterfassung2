# Testing Patterns

**Analysis Date:** 2026-02-13

## Test Framework

**Runner:**
- Rust: `cargo test` (default), `cargo nextest run` (faster parallel execution via `just nextest`)
- TypeScript/React: None detected - no testing framework currently configured

**Assertion Library:**
- Rust: Standard library `assert_eq!`, `assert!` macros, plus `pretty_assertions` crate for better diff output

**Test-specific dependencies (Rust, from `backend/Cargo.toml`):**
- `fake` v2.9.1 - Generate fake test data with derive macros
- `pretty_assertions` v1.4.0 - Better assertion failure output
- `rand` v0.8.5 - Random number generation for tests
- `test-case` v3.3.1 - Parameterized testing
- `paste` v1.0.14 - Macro composition
- `itertools` v0.12.1 - Iterator utilities
- `tracing-test` v0.2.4 - Tracing integration for tests

**Run Commands:**
```bash
cargo test                    # Run all tests
cargo nextest run             # Run all tests with nextest (faster)
just test                     # Alias for cargo test
just t                        # Alias for nextest
just ct                       # Compile tests without running
cargo test <test_name>        # Run single test
just test <test_name>         # Run single test via justfile
```

## Test File Organization

**Location:**
- Rust: Co-located with source code using `#[cfg(test)]` conditional modules
- Test utilities: Centralized in `backend/src/test/` directory (not nested in #[cfg(test)])
- TypeScript/React: No tests present

**Naming:**
- Rust test modules: Named as `test` (e.g., `#[cfg(test)] mod test;`)
- Test utility modules: `test/token.rs`, `test/db.rs`, `test/methods.rs`, `test/gen_fns.rs`

**Structure:**
```
backend/src/
├── test/                      # Test utilities (public modules)
│   ├── mod.rs                 # Re-exports for test utilities
│   ├── db.rs                  # Database test helpers
│   ├── token.rs               # Authentication token helpers
│   ├── methods.rs             # HTTP request method wrappers
│   └── gen_fns.rs             # Test data generation functions
└── [source files]
    └── (embedded tests in each module via #[cfg(test)])
```

## Test Structure

**Suite Organization:**
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::test::*;  // Import test utilities

    #[tokio::test]
    async fn test_something() {
        // Setup
        let client = Client::tracked(rocket());
        let token = get_token_admin(&client);

        // Action
        let response = client.get("/endpoint")
            .add_auth_header(token)
            .dispatch();

        // Assert
        assert_eq!(response.status(), Status::Ok);
    }
}
```

**Patterns:**
- Setup: Create Rocket test client, acquire tokens via helper functions
- Teardown: Cleanup done in helper functions (e.g., `cleanup_admin_user`)
- Assertion: Use standard Rust assertions with clear expected vs actual

**Example from `backend/src/test/token.rs`:**
```rust
pub fn get_admin_token(client: &Client, password: Option<String>) -> [String; 2] {
    let [admin_email, admin_password] = create_admin(client, password).unwrap();
    println!("Admin email: {admin_email}");
    let token = client
        .post("/login")
        .body(to_string(&Login::new(admin_email.as_str(), admin_password.as_str()))
            .expect("Could not serialize Login"))
        .dispatch()
        .into_json::<Token>()
        .unwrap();
    [token.token, admin_email]
}
```

## Mocking

**Framework:**
- Rust: `fake` crate with derive macros for generating test data
- No external mocking framework; use Rocket's test client for HTTP mocking

**Patterns:**
```rust
// Generate fake data
use fake::{faker::internet::en::*, Dummy};

#[cfg_attr(test, derive(Dummy))]
pub struct CreateUser {
    #[cfg_attr(test, dummy(faker = "SafeEmail()"))]
    pub email: String,
}

// Generate instances
let user = fake::faker::Dummy::dummy(&fake::Fake);
```

**Test data generation example from `backend/src/test/gen_fns.rs`:**
- Macro-based generation of faker functions
- Pattern: `macro_rules! build_faker_fn!` generates `generate_user()`, `generate_project()`, etc.

**What to Mock:**
- Database: Use real test database (integration testing approach)
- HTTP requests: Use Rocket's `local::blocking::Client` for real endpoint testing
- Tokens: Generate real tokens via `Tokenizer` for realistic auth testing

**What NOT to Mock:**
- Database layers: Prefer real database for integration tests
- External services: None detected in codebase

## Fixtures and Factories

**Test Data:**
- Rust uses the `fake` crate with derive macros for data generation
- Pattern: `#[cfg_attr(test, derive(Dummy))]` on struct definitions
- Field-level fakers: `#[cfg_attr(test, dummy(faker = "SafeEmail()"))]`

**Location:**
- Fake derive annotations: Directly on struct definitions (e.g., `backend/src/db/user.rs`)
- Generator functions: `backend/src/test/gen_fns.rs` contains macro-generated factory functions
- Database helpers: `backend/src/test/db.rs` contains `create_user()`, `create_admin()`

**Example from `backend/src/db/user.rs`:**
```rust
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, Insertable)]
#[cfg_attr(test, derive(Dummy))]
#[diesel(table_name=user)]
pub struct CreateUser {
    #[cfg_attr(test, dummy(faker = "SafeEmail()"))]
    pub email: String,
    #[cfg_attr(test, dummy(faker = "Username()"))]
    pub username: String,
}
```

## Coverage

**Requirements:** Not enforced (no coverage configuration detected)

**View Coverage:** Not available without additional tooling

**Coverage notes:**
- Tracing integration via `tracing-test` enables logging inspection in tests
- No code coverage measurement currently configured

## Test Types

**Unit Tests:**
- Scope: Individual functions and modules
- Approach: Use `#[cfg(test)]` modules within source files
- Example: Token signing/verification in `Tokenizer` struct
- Pattern: Direct function calls with synthetic inputs

**Integration Tests:**
- Scope: HTTP endpoints, database operations, authentication flows
- Approach: Use Rocket's test client (`local::blocking::Client`) for endpoint testing
- Example: Test endpoints with real tokens and database state
- Database: Real test database accessed via connection pool (see `backend/src/test/db.rs`)

**E2E Tests:**
- Framework: Not configured
- Notes: Full end-to-end testing would require running frontend + backend together

## Common Patterns

**Async Testing (Rust):**
```rust
#[tokio::test]
async fn test_async_operation() {
    // Async tests use #[tokio::test] attribute
    let result = some_async_function().await;
    assert!(result.is_ok());
}
```

**Error Testing:**
```rust
#[test]
fn test_error_case() {
    let result = some_fallible_operation();
    assert!(result.is_err());
    match result {
        Err(Error::NotFound) => { /* expected */ },
        _ => panic!("wrong error type"),
    }
}
```

**HTTP Testing Pattern:**
```rust
#[test]
fn test_endpoint() {
    let client = Client::tracked(rocket());
    let token = get_token_admin(&client);

    let response = client
        .post("/tracking")
        .add_auth_header(token)
        .json(&CreateTracking { /* ... */ })
        .dispatch();

    assert_eq!(response.status(), Status::Ok);
    let tracking = response.into_json::<Tracking>().unwrap();
    assert_eq!(tracking.user_id, expected_user_id);
}
```

**Database Testing Pattern:**
```rust
#[test]
fn test_db_operation() {
    let client = Client::tracked(rocket());
    let user = generate_user();
    create_user(&client, user).expect("setup failed");

    // Test operation

    cleanup_admin_user(&client, user.email);
}
```

**Parameterized Testing (via `test-case` crate):**
```rust
use test_case::test_case;

#[test_case("input1", "expected1")]
#[test_case("input2", "expected2")]
fn test_with_parameters(input: &str, expected: &str) {
    assert_eq!(process(input), expected);
}
```

## Test Client Setup

**Rocket Test Client initialization:**
```rust
use rocket::local::blocking::Client;

let client = Client::tracked(rocket());
```

**Request building with helpers:**
- `backend/src/test/methods.rs` provides wrappers:
  - `get()` - GET request with auth
  - `post()` - POST request with auth
  - `put()` - PUT request with auth
  - `patch()` - PATCH request with auth
  - `delete()` - DELETE request with auth
- All methods accept client, URI, and token

**Auth header injection pattern:**
```rust
pub trait AuthHeader<'a> {
    fn add_auth_header(self, token: &'_ str) -> Self;
}

impl AuthHeader<'_> for LocalRequest<'_> {
    fn add_auth_header(mut self, token: &str) -> Self {
        self.add_header(Header::new("Authorization", format!("Bearer {}", token)));
        self
    }
}

// Usage:
client.get("/endpoint")
    .add_auth_header(token)
    .dispatch()
```

## TypeScript/React Testing Notes

**Current state:** No test framework configured for frontend

**Recommendation for future setup:**
- Framework: Vitest or Jest
- Component testing: React Testing Library
- Config location: `frontend/vitest.config.ts` or `frontend/jest.config.js`
- Test files: Co-locate with components as `*.test.tsx` or `*.spec.tsx`

---

*Testing analysis: 2026-02-13*
