# Agent Guidelines for slothtime-rs

Refer to README.md for what this app does and is for.

## Build/Test Commands
- **Build**: `cargo build` (debug) or `cargo build --release` (optimized)
- **Run**: `cargo run` or `./target/debug/slothtime-rs`
- **Test**: `cargo test` (no tests exist yet - add unit tests for new functions)
- **Check**: `cargo check` (fast compilation check)
- **Single test**: `cargo test test_name` (once tests are added)

## Code Style Guidelines

### Formatting & Linting
- **Format**: `cargo fmt` (uses rustfmt)
- **Lint**: `cargo clippy` (fix all warnings)
- **Edition**: Rust 2021
- **Line width**: Default rustfmt (100 chars)

### Naming Conventions
- **Functions/Methods**: `snake_case`
- **Types/Structs/Enums**: `PascalCase`
- **Constants**: `SCREAMING_SNAKE_CASE`
- **Modules**: `snake_case`
- **Fields**: `snake_case`

### Imports & Dependencies
- Group imports: std, external crates, then local modules
- Use `use` statements at top of file
- Prefer explicit imports over glob (`*`) imports
- Use `crate::` for internal module references

### Error Handling
- Use `anyhow::Result<T>` for application errors
- Use `?` operator for error propagation
- Use `unwrap_or_else` for graceful fallbacks
- Main function: `Result<(), Box<dyn std::error::Error>>`

### Types & Derives
- Common derives: `Debug, Clone, Serialize, Deserialize`
- Use `Option<T>` for optional values
- Use `Vec<T>` for collections
- Prefer owned types (`String`, `Vec`) over borrowed (`&str`, `&[T]`)

### Patterns
- Use `match` statements for exhaustive pattern matching
- Use `if let` for single pattern matches
- Use struct update syntax: `Struct { field: value, ..default }`
- Use `impl` blocks for methods
- Use `Default` trait for default values

### Testing
- Add `#[test]` functions in same file or `tests/` module
- Use `#[cfg(test)]` for test-only code
- Test public APIs, not private implementation details
