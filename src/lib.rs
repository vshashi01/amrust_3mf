//! # 3MF (3D Manufacturing Format) support for Rust
//!
//! This library provides support for [3MF] files to programs written in the
//! Rust programming language. 3MF is a file format commonly used for 3D
//! printing. It is typically exported from a CAD program, and imported to a
//! slicer.
//!
//!
//! [3MF]: https://en.wikipedia.org/wiki/3D_Manufacturing_Format
//! This library was originally taken from the Threemf crate, however my goals deviated from the goals
//! of the original package and its maintainers as such I decided to take this into my own packages.
//! Thanks for the great work of the original maintainers.
//!
//! ## Further Reading
//!
//! See [3MF specification] and [Open Packaging Conventions].
//!
//! [3MF specification]: https://3mf.io/specification/
//! [Open Packaging Conventions]: https://standards.iso.org/ittf/PubliclyAvailableStandards/c061796_ISO_IEC_29500-2_2012.zip

pub mod core;
pub mod threemf_namespaces;

#[cfg(any(feature = "io", feature = "unpack-only"))]
pub mod io;
