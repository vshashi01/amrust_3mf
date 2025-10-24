# amrust_3mf

A small Rust library for reading and writing 3MF (3D Manufacturing Format) packages.  
This crate exposes a compact core model representation and IO helpers for reading/writing 3MF packages.

## Supported 3MF Extensions and Maximum Supported Versions

| 3MF Specifications | Type      | Optional | Current supported version |
| ------------------ | --------- | :------: | ------------------------: |
| 3MF Core Spec      | Core      |    No    |                     1.3.0 |
| Production         | Extension |    No    |                     1.2.0 |

## Overview

amrust_3mf provides:

- A set of core data structures representing 3MF models: [`Model`](src/core/model.rs), [`Object`](src/core/object.rs), [`Mesh`](src/core/mesh.rs), triangles and more with serialization and deserialization primitives.
- Readers and writers for 3MF packages: [`io::ThreemfPackage`](src/io/threemf_package.rs) with multiple 3MF Model support.
- Support for reading and writing of known parts (e.g. thumbnails) and unknown parts (e.g. Custom XML data) in 3MF Package by host application.

Key types and files:

- Core model types in [src/core/](src/core/) — `model`, `object`, `resources`, `mesh`, `transform`, etc.
- [`io::ThreemfPackage`](src/io/threemf_package.rs) — main read/write entry.
- [`io::content_types::ContentTypes`](src/io/content_types.rs) and [`io::relationship::RelationshipType`](src/io/relationship.rs) — for OPC package content and relationship handling

## Cargo features

This crate uses optional Cargo features to include different (de)serialization backends and IO capabilities. Enable only what you need.

- `io` — Enables packaging IO API (`ThreemfPackage`). Must be used in conjunction with additional features such as `write`, `memory-optimized-read`, `speed-optimized-read`, or `unpack-only`
- `write` — Enable writing 3MF packages (uses `instant_xml` for serialization).
  - `ToXML`implementations across all 3MF types
  - If `io` is enabled then [`ThreemfPackage::write`](src/io/threemf_package.rs) is enabled to write an in memory 3MF Package to a writer.
- `memory-optimized-read` — Enable read using `instant_xml` deserializer which is optimized for low-memory parsing.
  - `FromXml` implementations across all 3MF Types
  - If `io` is enabled then [`ThreemfPackage::from_reader_with_memory_optimized_deserializer`](src/io/threemf_package.rs) is enabled to create an in memory 3MF Package from a reader
- `speed-optimized-read` — Enable read using `serde_roxmltree` for faster deserialization speed.
  - `serde::Deserialize` implementations across all 3MF Types
  - If `io` is enabled then[`ThreemfPackage::from_reader_with_speed_optimized_deserializer`](src/io/threemf_package.rs) is enabled to create an in memory 3MF Package from a reader
- `unpack-only` — Builds struct [`io::ThreemfUnpacked`](src/io/threemf_unpacked.rs) that only creates the package structure without deserializing the actual 3MF Models (useful if you only need to extract files/metadata from 3MF).

## Quick example

A minimal example that constructs a simple package in-memory, writes it, and reads it back using the memory-optimized reader.

Note: The example below can be compiled just with the default features of the library

```rust
use std::io::Cursor;

use amrust_3mf::io::ThreemfPackage;
use amrust_3mf::core::{
    model::Model,
    resources::Resources,
    object::{Object, ObjectType},
    build::Build,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Build a tiny model
    let model = Model {
        unit: Some(model::Unit::Millimeter),
        metadata: vec![],
        resources: Resources {
            object: vec![Object {
                id: 1,
                objecttype: Some(ObjectType::Model),
                name: Some("Some object".to_owned()),
                mesh: None,
                components: None,
                thumbnail: None,
                partnumber: None,
                pid: None,
                pindex: None,
                uuid: Some("uuid".to_owned()),
            }],
            basematerials: vec![],
        },
        build: Build { uuid: None, item: vec![] },
        ..Default::default()
    };

    let package = ThreemfPackage {
        root: model,
        sub_models: std::collections::HashMap::new(),
        thumbnails: std::collections::HashMap::new(),
        unknown_parts: std::collections::HashMap::new(),
        relationships: std::collections::HashMap::new(),
        content_types: amrust_3mf::io::content_types::ContentTypes {
            defaults: vec![
                amrust_3mf::io::content_types::DefaultContentTypes {
                    extension: "rels".into(),
                    content_type: amrust_3mf::io::content_types::DefaultContentTypeEnum::Relationship,
                },
                amrust_3mf::io::content_types::DefaultContentTypes {
                    extension: "model".into(),
                    content_type: amrust_3mf::io::content_types::DefaultContentTypeEnum::Model,
                },
            ],
        },
    };

    // Write package to memory
    let mut buf = Cursor::new(Vec::<u8>::new());
    // requires the "write" feature
    package.write(&mut buf)?;

    // Read it back (choose "memory-optimized-read or "speed-optimized-read" backend; this example uses "memory-optimized-read")
    let mut read_buf = Cursor::new(buf.into_inner());
    let read_pkg = ThreemfPackage::from_reader_with_memory_optimized_deserializer(&mut read_buf, false)?;
    println!("Read package root model objects: {}", read_pkg.root.resources.object.len());
    Ok(())
}
```

## Licensese

This project and its source code are released under [MIT](/LICENSE-MIT) or [Apache 2.0](/LICENSE-APACHE) licenses.

## Contributing

Contributions are welcome.

- Open an issue to discuss major changes or report bugs.
- Fork the repo and create a feature branch.
- Add tests that exercise new behavior (tests may be feature-gated).
- Run tests locally with all possible features, preferably use `cargo all-features test`:
- Submit a pull request with a clear description and link to any related issue.

By contributing you agree to license your contributions under MIT or Apache 2.0 licenses.
