pub mod constants;
pub mod content_types;
pub mod error;
pub mod relationship;
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

// #[cfg(feature = "io-unpack")]
// mod threemf_unpacked;
// #[cfg(feature = "io-unpack")]
// pub use threemf_unpacked::ThreemfUnpacked;

#[cfg(feature = "io-pull-based-read")]
mod threemf_package_pull;
#[cfg(feature = "io-pull-based-read")]
pub use threemf_package_pull::{CachePolicy, ThreemfPackagePull};
