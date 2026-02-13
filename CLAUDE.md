# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Build/Test Commands

### Rust Backend
- Build: `cargo build` or `just run`
- Lint: `just check` or `cargo clippy`
- Format: `just fmt -w` (write changes) or `just fmt` (check only)
- Test all: `just test` or `just t` (using nextest)
- Test single: `just test <test_name>` or `cargo test <test_name>`
- Compile tests without running: `just ct`

### React Frontend
- Dev server: `cd frontend && npm run dev`
- Build: `cd frontend && npm run build`
- Lint: `cd frontend && npm run lint`

## Code Style Guidelines

### Rust
- Use hard tabs with 2 spaces
- Imports: Vertical layout, grouped by Std/External/Crate
- Error handling: Use thiserror with meaningful error variants and messages
- Naming: Follow Rust standard (snake_case for functions/variables, CamelCase for types)

### TypeScript/React
- Use TypeScript interfaces for component props
- React: Use functional components with hooks, named exports
- Tailwind: Use utility classes with composition via cn() utility
- Component structure: Props at top, followed by hooks, handlers, renders