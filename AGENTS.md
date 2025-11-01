# AGENTS.md - Development Guide for Coding Agents

## Build & Test Commands
- **Build**: `cargo build --all-features`
- **Test all**: `cargo test --all-features`
- **Test single**: `cargo test --all-features test_name` (e.g., `cargo test --all-features read_threemf_package_memory_optimized`)
- **Lint/Format check**: `cargo fmt --all -- --check && cargo clippy --all-targets --all-features -- -D warnings`
- **Format code**: `cargo fmt --all`
- **MSRV**: 1.89.0 (Rust 2024 edition)

## Code Style & Conventions
- **Imports**: Group external crates, internal modules, and then std; use feature-gated imports (`#[cfg(feature = "...")]`)
- **Features**: This crate uses extensive feature flags (`io`, `write`, `memory-optimized-read`, `speed-optimized-read`, `unpack-only`); gate code with `#[cfg(feature = "...")]`
- **Derive macros**: Feature-gate serialization derives (`#[cfg_attr(feature = "write", derive(ToXml))]`, `#[cfg_attr(feature = "memory-optimized-read", derive(FromXml))]`, etc.)
- **Error handling**: Use `thiserror` for error types; see `src/io/error.rs` for the Error enum pattern
- **Naming**: Snake_case for files/functions, PascalCase for types/enums, SCREAMING_SNAKE_CASE for constants
- **Tests**: Place integration tests in `tests/` with feature gates; use `pretty_assertions` for test assertions
- **Documentation**: Add doc comments (`///`) for public APIs; reference file paths as `[Type](src/path/file.rs)`
- **Clippy**: Code must pass `clippy --all-targets --all-features -- -D warnings` (warnings are errors in CI)

## Project Structure
- `src/core/`: Core 3MF data structures (model, object, mesh, resources, transform, beamlattice)
- `src/io/`: Package IO operations (ThreemfPackage, content_types, relationships, query helpers)
- `tests/`: Integration tests (core_io, production_io, beamlattice_io, third_party_read)
- `examples/`: Feature-specific examples (write.rs, unpack.rs, memory/speed-optimized reads)
