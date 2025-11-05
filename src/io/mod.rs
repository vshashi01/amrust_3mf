pub mod content_types;
pub mod error;
pub mod relationship;

mod utils;
pub use utils::parse_xmlns_attributes;
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
