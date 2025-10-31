pub mod content_types;
pub mod error;
pub mod relationship;

#[cfg(feature = "io")]
mod threemf_package;
#[cfg(feature = "io")]
pub use threemf_package::ThreemfPackage;

#[cfg(feature = "io")]
pub mod query;

#[cfg(feature = "unpack-only")]
mod threemf_unpacked;
#[cfg(feature = "unpack-only")]
pub use threemf_unpacked::ThreemfUnpacked;
