# Coding Conventions

**Analysis Date:** 2026-02-13

## Naming Patterns

**Files:**
- Rust backend: `snake_case` for files (e.g., `tokenizer.rs`, `user.rs`, `gen_fns.rs`)
- TypeScript/React: `camelCase` for files with single export (e.g., `authApi.ts`, `ui-utils.ts`), `PascalCase` for component files (e.g., `Button.tsx`, `AuthContext.tsx`)

**Functions:**
- Rust: `snake_case` for all functions (e.g., `get_admin_token`, `create_user`, `hash_password`, `db_url`)
- TypeScript: `camelCase` for functions and async functions (e.g., `getTracking`, `createTracking`, `formatDate`)
- React: `camelCase` for utility/hook functions (e.g., `useAuth`, `calculateDuration`), `PascalCase` for components (e.g., `AuthProvider`, `Button`)

**Variables:**
- Rust: `snake_case` for all variables and constants (e.g., `admin_email`, `admin_password`, `admin_role`)
- TypeScript: `camelCase` for variables and function parameters (e.g., `isLoading`, `isAuthenticated`, `pageSize`)
- Booleans: Prefix with `is`, `has`, or `can` (e.g., `isLoading`, `isAuthenticated`)

**Types/Interfaces:**
- TypeScript: `PascalCase` for all interfaces (e.g., `User`, `CreateUser`, `AuthContextType`, `ButtonProps`)
- Rust: `PascalCase` for structs and enums (e.g., `Tokenizer`, `User`, `Error`, `CreateUser`)

## Code Style

**Formatting:**
- Rust: Configured via `rustfmt.toml`
  - Hard tabs with 2 spaces: `hard_tabs = true`, `tab_spaces = 2`
  - Vertical imports layout: `imports_layout = 'Vertical'`
  - Grouped imports: `group_imports = 'StdExternalCrate'`
  - Crate-level granularity: `imports_granularity = 'Crate'`
  - Format strings enabled: `format_strings = true`
- TypeScript/React: Integrated ESLint with TypeScript support
  - Target: ECMAScript 2020
  - Browser globals enabled

**Linting:**
- Rust: `cargo clippy` via `just check` command
- TypeScript: ESLint with config at `frontend/eslint.config.js`
  - Extends: `@eslint/js` recommended and `typescript-eslint` recommended
  - Plugins: `react-hooks` (recommended rules), `react-refresh`
  - Key rule: `react-refresh/only-export-components` warns on non-component exports with exception for constants

## Import Organization

**Order (Rust):**
1. Standard library imports: `use std::...`
2. External crates: `use rocket::...`, `use diesel::...`
3. Crate-local imports: `use crate::...`

**Pattern example from `backend/src/db/user.rs`:**
```rust
use argon2::password_hash::{
	PasswordHash,
	PasswordVerifier,
};
#[cfg(test)]
use fake::{
	faker::internet::en::*,
	faker::name::en::*,
	Dummy,
};
use rocket_db_pools::{
	diesel::{
		insert_into,
		prelude::*,
	},
	Connection,
};
use serde::{
	Deserialize,
	Serialize,
};
use tracing::{
	debug,
	error,
	trace,
};

use super::{
	last_insert_id,
	PaginationResult,
};
use crate::{
	schema::*,
	Error,
	Result,
	DB,
};
```

**Order (TypeScript):**
1. External packages: `import { createContext } from 'react'`
2. Internal API imports: `import { authApi } from '../api'`
3. Internal type imports: `import { LoginRequest, User } from '../types'`

**Pattern example from `frontend/src/contexts/AuthContext.tsx`:**
```typescript
import { createContext, useState, useContext, ReactNode, useEffect } from 'react';
import jwt_decode from 'jwt-decode';
import { authApi } from '../api';
import { LoginRequest, User } from '../types';
```

**Path Aliases:**
- No explicit aliases configured; use relative imports throughout

## Error Handling

**Rust Patterns:**
- Use `thiserror` crate with `#[derive(Error)]` for custom error enums (see `backend/src/error.rs`)
- Define type alias: `pub type Result<T> = std::result::Result<T, Error>;`
- Use `#[from]` attribute for automatic `From` trait implementation for wrapped errors
- Implement custom error mapping via `impl From<SomeError> for Error`
- Log errors with `error!` macro before returning
- Map errors with context: `err.context("operation description")`

**Example from `backend/src/error.rs`:**
```rust
#[derive(Error, Debug)]
pub enum Error {
	#[error("Database Error: {0}")]
	Database(#[from] diesel::result::Error),
	#[error("Password Error: {0}")]
	Argon2PasswordHash(argon2::password_hash::Error),
	#[error("Not found")]
	NotFound,
	#[error("{0}")]
	BadRequest(String),
}

pub type Result<T> = std::result::Result<T, Error>;
```

**TypeScript Patterns:**
- Use `try/catch` blocks with `throw` for error propagation
- Log errors with `console.error()` for debugging
- Handle promise rejections in `.catch()` blocks or `try/catch` in async functions

**Example from `frontend/src/contexts/AuthContext.tsx`:**
```typescript
try {
  const response = await authApi.login(credentials);
  authApi.setToken(response.token);
  const decodedToken = jwt_decode<{ custom: User }>(response.token);
  setUser(decodedToken.custom);
} catch (error) {
  console.error('Login failed', error);
  throw error;
}
```

## Logging

**Framework:**
- Rust: `tracing` crate with `tracing-subscriber` for structured logging
  - Log levels: `trace`, `debug`, `error`
  - Example: `trace!("Generating Token for User: {:?}", user.id)`
- TypeScript: `console` object for simple logging
  - Used for errors: `console.error('Login failed', error)`
  - Only for debugging/error reporting

**Patterns:**
- Rust: Log at start/end of operations with context
  - Use `trace!` for detailed flow information during development
  - Use `error!` when handling errors to log the failure
  - Include variable details in debug output: `{:?}` for Debug format

## Comments

**When to Comment:**
- Rust: Use `///` for public item documentation (e.g., struct fields, function descriptions)
- Inline comments: Use `//` for explaining complex logic or non-obvious decisions
- Example from `backend/src/db/user.rs`:
  ```rust
  /// Field representing column `id`
  pub id: i32,
  ```
- Block comments: Use `/* */` for multi-line explanations (rarely used)

**JSDoc/TSDoc:**
- Not enforced in TypeScript
- Inline comments used for type annotations and intent clarification
- Example from `frontend/src/types/index.ts`:
  ```typescript
  // User types
  export interface User {
    id: number;
    // ...
  }
  ```

**Notable convention:** Meaningful code comments are minimal; the codebase prioritizes self-documenting code with clear naming.

## Function Design

**Size:** Small, focused functions (5-20 lines typical)
- Example: `formatDate` at `frontend/src/utils/date-utils.ts` is 5 lines
- Example: `generateToken` in `backend/src/auth/tokenizer.rs` is 12 lines

**Parameters:**
- Rust: Prefer owned types for function arguments, use references for borrowed data
  - Example: `fn hash_password(password: &[u8]) -> Result<String>`
  - Keep parameter count low (2-4 typical)
- TypeScript: Use object destructuring for multiple parameters
  - Example: `createTracking: async (tracking: CreateTracking): Promise<Tracking>`

**Return Values:**
- Rust: Use custom `Result<T>` type for all fallible operations
  - Return concrete types, not Option (use Result instead)
  - Async functions return futures that resolve to Result
- TypeScript: Use `Promise<T>` for async operations, return concrete types
  - Example: `async (id: number): Promise<Tracking>`
  - Use type generics for generic APIs: `apiRequest<T>(...)`

## Module Design

**Exports:**
- Rust: Use `pub mod` for modules, `pub use` for re-exporting commonly used types
  - Example from `backend/src/lib.rs`:
    ```rust
    pub use db::{
      user::User,
      DB,
    };
    pub use error::{
      Error,
      Result,
    };
    ```
- TypeScript: Default exports for single component/function, named exports for utility modules
  - Example: `export default App;` for components
  - Example: `export const trackingApi = { ... }` for API modules

**Barrel Files:**
- TypeScript: Use minimal barrel exports
  - Example: `frontend/src/api/index.ts` would re-export all API modules (not observed, but follows pattern)
  - Prefer importing directly from modules to minimize circular dependencies

**Test Module Pattern (Rust):**
- Conditional module: `#[cfg(test)] mod test;`
- Test submodules organized as: `test/mod.rs`, `test/token.rs`, `test/db.rs`, etc.
- Test utilities exported from `test/mod.rs` with `pub use`

---

*Convention analysis: 2026-02-13*
