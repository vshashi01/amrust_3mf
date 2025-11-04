pub mod content_types;
pub mod error;
pub mod relationship;

mod utils;
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
mod feature_gate {
    #[path = "../threemf_package.rs"]
    pub mod threemf_package;

    #[path = "../query.rs"]
    pub mod query;
}

#[cfg(any(
    feature = "io-write",
    feature = "io-memory-optimized-read",
    feature = "io-speed-optimized-read"
))]
pub use feature_gate::{query, threemf_package::ThreemfPackage};

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
