# amrust_3mf

Library for reading and writing 3MF (3D Manufacturing Format) packages with both eager and lazy loading support.

This crate provides a compact core model representation and I/O helpers for reading/writing 3MF packages with multiple loading strategies optimized for different use cases.

## Supported 3MF Extensions and Maximum Supported Versions

| 3MF Specifications | Type      | Optional | Current supported version |
| ------------------ | --------- | :------: | ------------------------: |
| 3MF Core Spec      | Core      |    No    |                     1.3.0 |
| Production         | Extension |    No    |                     1.1.2 |
| Beam Lattice       | Extension |    No    |                     1.2.0 |

**Note: This library is still in active development, expect frequent API changes!!**

## Overview

amrust_3mf provides:

- **Core Data Structures**: Complete 3MF model representation ([`Model`](src/core/model.rs), [`Object`](src/core/object.rs), [`Mesh`](src/core/mesh.rs), etc.)
- **Multiple Loading Strategies**:
  - [`ThreemfPackage`](src/io/threemf_package.rs) - Eager loading for complete data access
  - [`ThreemfPackageLazyReader`](src/io/threemf_package_lazy_reader.rs) - Lazy loading for memory-constrained environments
- **Flexible I/O**: Support for reading/writing 3MF packages with different performance characteristics
- **Extension Support**: All 3MF extensions (Production, Beam Lattice, etc.) are always available
- **Custom Parts**: Support for known parts (thumbnails) and unknown parts (custom XML data)

## Quick Start

```rust
use amrust_3mf::io::ThreemfPackage;
use std::fs::File;

// Read a 3MF file
let file = File::open("model.3mf")?;
let package = ThreemfPackage::from_reader_with_memory_optimized_deserializer(file, true)?;

// Access the root model
let root_model = &package.root;
println!("Build items: {}", root_model.build.item.len());
```

## Performance Options

Choose the right loading strategy for your use case:

- **Memory-Optimized**: Lower memory usage, good for large files
- **Speed-Optimized**: Faster parsing, higher memory usage
- **Lazy Loading**: Defers loading until accessed, best for inspection-only use cases

Key types and files:

- Core model types in [src/core/](src/core/) — `model`, `object`, `resources`, `mesh`, `transform`, etc.
- [`io::ThreemfPackage`](src/io/threemf_package.rs) — eager loading entry point
- [`io::ThreemfPackageLazyReader`](src/io/threemf_package_lazy_reader.rs) — lazy loading entry point
- [`io::content_types::ContentTypes`](src/io/content_types.rs) and [`io::relationship::RelationshipType`](src/io/relationship.rs) — for OPC package content and relationship handling

## Cargo Features

This crate uses optional Cargo features to control functionality. Enable only what you need.

### Core Serialization Features
- `write` — Enable writing 3MF data (adds `ToXml` derive to all 3MF types using `instant_xml`)
- `memory-optimized-read` — Enable memory-efficient reading (adds `FromXml` derive to all 3MF types using `instant_xml`)
- `speed-optimized-read` — Enable fast reading (adds `serde::Deserialize` derive to all 3MF types using `serde_roxmltree`)

### Package I/O Features
- `io-write` — Package writing with ZIP creation (requires `write`)
- `io-memory-optimized-read` — Package reading with memory optimization (requires `memory-optimized-read`)
- `io-speed-optimized-read` — Package reading with speed optimization (requires `speed-optimized-read`)
- `io-lazy-read` — Lazy loading functionality (requires `io-memory-optimized-read`)

### Default Features
`io-write`, `io-memory-optimized-read`, `io-lazy-read`, `write`, `memory-optimized-read`

### Feature Combinations
```toml
# Basic reading
amrust_3mf = "0.1"

# Full I/O with lazy loading (default)
amrust_3mf = { version = "0.1", features = ["io-lazy-read"] }

# Memory-constrained environments
amrust_3mf = { version = "0.1", features = ["io-lazy-read"], default-features = false }

# High-performance reading
amrust_3mf = { version = "0.1", features = ["io-speed-optimized-read"] }
```

## Examples

The [examples/](examples/) directory contains runnable examples for different use cases:

- **`write.rs`** - Create and write 3MF packages
- **`unpack.rs`** - Lazy loading with `ThreemfPackageLazyReader`
- **`memory-optimized-read.rs`** - Memory-efficient reading
- **`speed-optimized-read.rs`** - High-performance reading
- **`string_extraction.rs`** - Access raw XML content
- **`beamlattice-write.rs`** - Working with beam lattice extensions

Run examples with:
```bash
cargo run --example write --features io-write
cargo run --example unpack --features io-lazy-read
```

## API Overview

### Core Data Structures
- `Model` - Root 3MF model with resources and build configuration
- `Object` - 3D objects (meshes or component assemblies)
- `Mesh` - Triangle mesh geometry with vertices and triangles
- `Component` - Object references with transforms

### Package I/O
- `ThreemfPackage` - Eager loading - loads all data upfront
- `ThreemfPackageLazyReader` - Lazy loading - loads metadata first, data on-demand

### Usage Patterns

#### Eager Loading (Complete Data Access)
```rust
use amrust_3mf::io::ThreemfPackage;
use std::fs::File;

let file = File::open("model.3mf")?;
let package = ThreemfPackage::from_reader_with_memory_optimized_deserializer(file, true)?;

// Access all data immediately
for object in &package.root.resources.object {
    println!("Object: {}", object.name.as_deref().unwrap_or("Unnamed"));
}
```

#### Lazy Loading (Memory Efficient)
```rust
use amrust_3mf::io::{ThreemfPackageLazyReader, CachePolicy};

let file = File::open("model.3mf")?;
let package = ThreemfPackageLazyReader::from_reader_with_memory_optimized_deserializer(
    file,
    CachePolicy::NoCache
)?;

// Load models on-demand
package.with_model("3D/model.model", |model| {
    println!("Objects in model: {}", model.resources.object.len());
})?;
```

## Building & Testing

### Requirements
- Rust 1.89.0 or later (2024 edition)
- Cargo package manager

### Build Commands
```bash
# Build with all features
cargo build --all-features

# Check formatting and linting
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
```

### Testing
```bash
# Install cargo-all-features and run tests for multiple feature combinations at once
cargo all-features test

# Run benchmarks
cargo bench --features "io-memory-optimized-read,io-speed-optimized-read"
```

## License

This project and its source code are released under [MIT](/LICENSE-MIT) or [Apache 2.0](/LICENSE-APACHE) licenses.

## Contributing

Contributions are welcome.

- Open an issue to discuss major changes or report bugs.
- Fork the repo and create a feature branch.
- Add tests that exercise new behavior (tests may be feature-gated).
- Run tests locally with all possible features, preferably use `cargo all-features test`:
- Add or update an example
- Add or update the documentation.
- Submit a pull request with a clear description and link to any related issue.

### AI-Assisted Contributions

We welcome contributions created with the assistance of AI tools. However, all contributors must:

- **Clearly disclose AI assistance** in your pull request description and commit messages
- **Provide due diligence** by:
  - Testing the code thoroughly (run all tests with `cargo-all-features`, see above)
  - Reviewing the generated code for correctness and adherence to project conventions
  - Understanding what the code does and why it works
  - Ensuring the contribution follows the project's style and patterns
- **Take responsibility** for the final code quality and functionality

AI tools can be excellent for productivity, but human oversight and understanding remain essential for maintaining code quality.

By contributing you agree to license your contributions under MIT or Apache 2.0 licenses.
