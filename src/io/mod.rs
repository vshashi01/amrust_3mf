pub mod content_types;
pub mod error;
pub mod relationship;
mod threemf_package;
mod threemf_unpacked;

pub use threemf_package::ThreemfPackage;
pub use threemf_unpacked::ThreemfUnpacked;

#[non_exhaustive]
#[derive(Debug, Copy, Clone)]
pub enum ReadStrategy {
    #[cfg(feature = "memory-optimized-read")]
    MemoryOptimized,

    #[cfg(feature = "speed-optimized-read")]
    SpeedOptimized,
}
