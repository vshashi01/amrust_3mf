pub mod content_types;
pub mod error;
pub mod relationship;

mod utils;
pub use utils::parse_xmlns_attributes;

/// Represents an XML namespace declaration with its prefix and URI
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct XmlNamespace {
    /// The namespace prefix (None for default namespace)
    pub prefix: Option<String>,
    /// The namespace URI
    pub uri: String,
}

/// Stores namespace information for a specific model file
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModelNamespaces {
    /// Path to the model file
    pub path: String,
    /// List of namespaces declared in that model
    pub namespaces: Vec<XmlNamespace>,
}
#[cfg(any(
    feature = "io-memory-optimized-read",
    feature = "io-speed-optimized-read"
))]
mod zip_utils;

#[cfg(any(
    feature = "io-write",
    feature = "io-memory-optimized-read",
    feature = "io-speed-optimized-read"
))]
mod threemf_package;
#[cfg(any(
    feature = "io-write",
    feature = "io-memory-optimized-read",
    feature = "io-speed-optimized-read"
))]
pub use threemf_package::ThreemfPackage;

#[cfg(any(
    feature = "io-write",
    feature = "io-memory-optimized-read",
    feature = "io-speed-optimized-read"
))]
pub mod query;

#[cfg(all(
    feature = "io-lazy-read",
    any(
        feature = "io-memory-optimized-read",
        feature = "io-speed-optimized-read"
    )
))]
mod threemf_package_lazy_reader;
#[cfg(all(
    feature = "io-lazy-read",
    any(
        feature = "io-memory-optimized-read",
        feature = "io-speed-optimized-read"
    )
))]
pub use threemf_package_lazy_reader::{CachePolicy, ThreemfPackageLazyReader};
