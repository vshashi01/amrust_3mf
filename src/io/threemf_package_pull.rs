use std::cell::RefCell;
use std::collections::HashMap;
use std::io::{Read, Seek};

use once_cell::unsync::OnceCell;
use zip::ZipArchive;

use image::{DynamicImage, load_from_memory};

use crate::core::model::Model;
use crate::io::{
    content_types::{ContentTypes, DefaultContentTypeEnum},
    error::Error,
    relationship::{RelationshipType, Relationships},
};

use std::ffi::OsStr;

/// Cache policy for lazy-loaded data
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CachePolicy {
    /// Cache everything after first access (best for typical usage where data is accessed multiple times)
    CacheAll,
    /// Never cache, always re-read from zip (best for memory-constrained environments, read-once patterns)
    #[default]
    NoCache,
}

#[non_exhaustive]
#[derive(Debug, Copy, Clone)]
enum ReadStrategy {
    #[cfg(feature = "memory-optimized-read")]
    MemoryOptimized,

    #[cfg(feature = "speed-optimized-read")]
    SpeedOptimized,
}

/// Represents a 3mf package with lazy/pull-based loading.
/// Unlike [`ThreemfPackage`](crate::io::ThreemfPackage), this struct only parses metadata upfront
/// (content types and relationships), and loads models, thumbnails, and other data on-demand.
///
/// This is ideal for memory-constrained environments or when you need to inspect package contents
/// without loading all data.
///
pub struct ThreemfPackagePull<R: Read + Seek> {
    /// The ZIP archive reader
    archive: RefCell<ZipArchive<R>>,
    read_strategy: ReadStrategy,
    cache_policy: CachePolicy,

    // EAGER: Small metadata (parsed immediately on construction)
    /// Content types from [Content_Types].xml
    content_types: ContentTypes,
    /// All relationships in the package
    relationships: HashMap<String, Relationships>,
    /// Path to the root model
    root_model_path: String,

    // LAZY: Large data (loaded on demand)
    /// Root model (loaded once on first access)
    root_model: OnceCell<Model>,
    /// Sub-models cache (if CachePolicy::CacheAll)
    sub_models: RefCell<HashMap<String, Model>>,
    /// Thumbnails cache (if CachePolicy::CacheAll)
    thumbnails: RefCell<HashMap<String, DynamicImage>>,
    /// Unknown parts cache (if CachePolicy::CacheAll)
    unknown_parts: RefCell<HashMap<String, Vec<u8>>>,
}

impl<R: Read + Seek> ThreemfPackagePull<R> {
    /// Create a new pull-based package reader.
    ///
    /// This eagerly parses only the content types and relationships (small XML files),
    /// but defers loading of models, thumbnails, and unknown parts until they are accessed.
    fn from_reader(
        reader: R,
        read_strategy: ReadStrategy,
        cache_policy: CachePolicy,
    ) -> Result<Self, Error> {
        let mut zip = ZipArchive::new(reader)?;

        // Eagerly parse content types (small XML)
        let content_types: ContentTypes = {
            let content_types_file = zip.by_name("[Content_Types].xml");
            match content_types_file {
                Ok(mut file) => {
                    let mut xml_string: String = Default::default();
                    let _ = file.read_to_string(&mut xml_string)?;

                    match read_strategy {
                        #[cfg(feature = "memory-optimized-read")]
                        ReadStrategy::MemoryOptimized => {
                            instant_xml::from_str::<ContentTypes>(&xml_string)?
                        }
                        #[cfg(feature = "speed-optimized-read")]
                        ReadStrategy::SpeedOptimized => {
                            serde_roxmltree::from_str::<ContentTypes>(&xml_string)?
                        }
                    }
                }
                Err(err) => {
                    return Err(Error::Zip(err));
                }
            }
        };

        let rels_ext = {
            let rels_content = content_types
                .defaults
                .iter()
                .find(|t| t.content_type == DefaultContentTypeEnum::Relationship);

            match rels_content {
                Some(rels) => &rels.extension,
                None => "rels",
            }
        };

        let root_rels_filename: &str = &format!("_rels/.{rels_ext}");

        // Eagerly parse all relationships (small XML files)
        let mut relationships = HashMap::<String, Relationships>::new();
        let root_rels: Relationships =
            Self::relationships_from_zip_by_name(&mut zip, root_rels_filename, read_strategy)?;

        // Find root model path
        let root_model_path = root_rels
            .relationships
            .iter()
            .find(|rels| rels.relationship_type == RelationshipType::Model)
            .map(|rel| rel.target.clone())
            .ok_or_else(|| Error::ReadError("Root model relationship not found".to_owned()))?;

        relationships.insert(root_rels_filename.to_owned(), root_rels);

        // Parse all other relationship files
        for value in 0..zip.len() {
            use std::path::Path;

            let file = zip.by_index(value)?;

            if file.is_file()
                && let Some(path) = file.enclosed_name()
                && Some(OsStr::new(rels_ext)) == path.extension()
                && path != Path::new(root_rels_filename)
            {
                match path.to_str() {
                    Some(path_str) => {
                        let rels = Self::relationships_from_zipfile(file, read_strategy)?;
                        relationships.insert(format!("/{path_str}"), rels);
                    }
                    None => {
                        return Err(Error::ReadError(
                            "Failed to read the relationship file path".to_owned(),
                        ));
                    }
                }
            }
        }

        Ok(Self {
            archive: RefCell::new(zip),
            read_strategy,
            cache_policy,
            content_types,
            relationships,
            root_model_path,
            root_model: OnceCell::new(),
            sub_models: RefCell::new(HashMap::new()),
            thumbnails: RefCell::new(HashMap::new()),
            unknown_parts: RefCell::new(HashMap::new()),
        })
    }

    /// Get content types (always available, no I/O)
    pub fn content_types(&self) -> &ContentTypes {
        &self.content_types
    }

    /// Get all relationships (always available, no I/O)
    pub fn relationships(&self) -> &HashMap<String, Relationships> {
        &self.relationships
    }

    /// Get the root model path (always available, no I/O)
    pub fn root_model_path(&self) -> &str {
        &self.root_model_path
    }

    /// Iterate over all model paths in the package (no I/O)
    pub fn model_paths(&self) -> impl Iterator<Item = &str> {
        self.relationships
            .values()
            .flat_map(|r| &r.relationships)
            .filter_map(|rel| {
                if matches!(rel.relationship_type, RelationshipType::Model) {
                    Some(rel.target.as_str())
                } else {
                    None
                }
            })
    }

    /// Iterate over all thumbnail paths in the package (no I/O)
    pub fn thumbnail_paths(&self) -> impl Iterator<Item = &str> {
        self.relationships
            .values()
            .flat_map(|r| &r.relationships)
            .filter_map(|rel| {
                if matches!(rel.relationship_type, RelationshipType::Thumbnail) {
                    Some(rel.target.as_str())
                } else {
                    None
                }
            })
    }

    /// Iterate over all unknown part paths in the package (no I/O)
    pub fn unknown_part_paths(&self) -> impl Iterator<Item = &str> {
        self.relationships
            .values()
            .flat_map(|r| &r.relationships)
            .filter_map(|rel| {
                if matches!(rel.relationship_type, RelationshipType::Unknown(_)) {
                    Some(rel.target.as_str())
                } else {
                    None
                }
            })
    }

    /// Get the root model (lazy loaded, cached based on policy)
    pub fn root_model(&self) -> Result<&Model, Error> {
        self.root_model
            .get_or_try_init(|| self.load_model_from_archive(&self.root_model_path))
    }

    /// Get a sub-model by path (lazy loaded, cached based on policy)
    ///
    /// Returns `None` if no model exists at the given path.
    pub fn get_sub_model(&self, path: &str) -> Result<Option<&Model>, Error> {
        // Don't load if it's the root model path
        if path == self.root_model_path {
            return Ok(None);
        }

        // Check if it's a valid model path
        let is_model = self
            .relationships
            .values()
            .flat_map(|r| &r.relationships)
            .any(|rel| {
                rel.target == path && matches!(rel.relationship_type, RelationshipType::Model)
            });

        if !is_model {
            return Ok(None);
        }

        match self.cache_policy {
            CachePolicy::NoCache => {
                // Always load fresh, don't cache
                // We can't return a reference to temporary data, so we must cache at least temporarily
                // Check if already in cache from a previous call
                if self.sub_models.borrow().contains_key(path) {
                    // SAFETY: We never remove from cache, only add
                    // RefCell guarantees no concurrent mutation
                    let cache = self.sub_models.borrow();
                    let model_ptr = cache.get(path).unwrap() as *const Model;
                    unsafe { Ok(Some(&*model_ptr)) }
                } else {
                    let model = self.load_model_from_archive(path)?;
                    self.sub_models.borrow_mut().insert(path.to_string(), model);
                    let cache = self.sub_models.borrow();
                    let model_ptr = cache.get(path).unwrap() as *const Model;
                    unsafe { Ok(Some(&*model_ptr)) }
                }
            }
            CachePolicy::CacheAll => {
                // Check cache first
                if self.sub_models.borrow().contains_key(path) {
                    // SAFETY: We never remove from cache, only add
                    // RefCell guarantees no concurrent mutation
                    let cache = self.sub_models.borrow();
                    let model_ptr = cache.get(path).unwrap() as *const Model;
                    unsafe { Ok(Some(&*model_ptr)) }
                } else {
                    // Load and cache
                    let model = self.load_model_from_archive(path)?;
                    self.sub_models.borrow_mut().insert(path.to_string(), model);
                    let cache = self.sub_models.borrow();
                    let model_ptr = cache.get(path).unwrap() as *const Model;
                    unsafe { Ok(Some(&*model_ptr)) }
                }
            }
        }
    }

    /// Get a thumbnail by path (lazy loaded, cached based on policy)
    ///
    /// Returns `None` if no thumbnail exists at the given path.
    pub fn get_thumbnail(&self, path: &str) -> Result<Option<&DynamicImage>, Error> {
        // Check if it's a valid thumbnail path
        let is_thumbnail = self
            .relationships
            .values()
            .flat_map(|r| &r.relationships)
            .any(|rel| {
                rel.target == path && matches!(rel.relationship_type, RelationshipType::Thumbnail)
            });

        if !is_thumbnail {
            return Ok(None);
        }

        // Check cache (works for both policies since we need to return a reference)
        if self.thumbnails.borrow().contains_key(path) {
            // SAFETY: We never remove from cache, only add
            // RefCell guarantees no concurrent mutation
            let cache = self.thumbnails.borrow();
            let img_ptr = cache.get(path).unwrap() as *const DynamicImage;
            unsafe { Ok(Some(&*img_ptr)) }
        } else {
            // Load and cache
            let image = self.load_thumbnail_from_archive(path)?;
            self.thumbnails.borrow_mut().insert(path.to_string(), image);
            let cache = self.thumbnails.borrow();
            let img_ptr = cache.get(path).unwrap() as *const DynamicImage;
            unsafe { Ok(Some(&*img_ptr)) }
        }
    }

    /// Get an unknown part by path (lazy loaded, cached based on policy)
    ///
    /// Returns `None` if no unknown part exists at the given path.
    pub fn get_unknown_part(&self, path: &str) -> Result<Option<&[u8]>, Error> {
        // Check if it's a valid unknown part path
        let is_unknown = self
            .relationships
            .values()
            .flat_map(|r| &r.relationships)
            .any(|rel| {
                rel.target == path && matches!(rel.relationship_type, RelationshipType::Unknown(_))
            });

        if !is_unknown {
            return Ok(None);
        }

        // Check cache (works for both policies since we need to return a reference)
        if self.unknown_parts.borrow().contains_key(path) {
            // SAFETY: We never remove from cache, only add
            // RefCell guarantees no concurrent mutation
            let cache = self.unknown_parts.borrow();
            let bytes_ptr = cache.get(path).unwrap() as *const Vec<u8>;
            unsafe { Ok(Some(&(&*bytes_ptr)[..])) }
        } else {
            // Load and cache
            let bytes = self.load_unknown_part_from_archive(path)?;
            self.unknown_parts
                .borrow_mut()
                .insert(path.to_string(), bytes);
            let cache = self.unknown_parts.borrow();
            let bytes_ptr = cache.get(path).unwrap() as *const Vec<u8>;
            unsafe { Ok(Some(&(&*bytes_ptr)[..])) }
        }
    }

    // Internal helper methods

    fn relationships_from_zip_by_name(
        zip: &mut ZipArchive<R>,
        zip_filename: &str,
        read_strategy: ReadStrategy,
    ) -> Result<Relationships, Error> {
        let rels_file = zip.by_name(zip_filename);
        match rels_file {
            Ok(file) => Self::relationships_from_zipfile(file, read_strategy),
            Err(err) => Err(Error::Zip(err)),
        }
    }

    fn relationships_from_zipfile<S: Read>(
        mut file: zip::read::ZipFile<'_, S>,
        read_strategy: ReadStrategy,
    ) -> Result<Relationships, Error> {
        let mut xml_string: String = Default::default();
        let _ = file.read_to_string(&mut xml_string)?;
        let rels = match read_strategy {
            #[cfg(feature = "memory-optimized-read")]
            ReadStrategy::MemoryOptimized => instant_xml::from_str::<Relationships>(&xml_string)?,
            #[cfg(feature = "speed-optimized-read")]
            ReadStrategy::SpeedOptimized => {
                serde_roxmltree::from_str::<Relationships>(&xml_string)?
            }
        };

        Ok(rels)
    }

    fn load_model_from_archive(&self, path: &str) -> Result<Model, Error> {
        let mut archive = self.archive.borrow_mut();
        let mut file = archive.by_name(try_strip_leading_slash(path))?;
        let mut xml_string = String::new();
        file.read_to_string(&mut xml_string)?;

        match self.read_strategy {
            #[cfg(feature = "memory-optimized-read")]
            ReadStrategy::MemoryOptimized => {
                instant_xml::from_str::<Model>(&xml_string).map_err(Error::from)
            }
            #[cfg(feature = "speed-optimized-read")]
            ReadStrategy::SpeedOptimized => {
                serde_roxmltree::from_str::<Model>(&xml_string).map_err(Error::from)
            }
        }
    }

    fn load_thumbnail_from_archive(&self, path: &str) -> Result<DynamicImage, Error> {
        let mut archive = self.archive.borrow_mut();
        let mut file = archive.by_name(try_strip_leading_slash(path))?;
        let mut bytes: Vec<u8> = vec![];
        file.read_to_end(&mut bytes)?;

        let image = load_from_memory(&bytes)?;
        Ok(image)
    }

    fn load_unknown_part_from_archive(&self, path: &str) -> Result<Vec<u8>, Error> {
        let mut archive = self.archive.borrow_mut();
        let mut file = archive.by_name(try_strip_leading_slash(path))?;
        let mut bytes: Vec<u8> = vec![];
        file.read_to_end(&mut bytes)?;
        Ok(bytes)
    }
}

// Convenience constructors for specific deserialization strategies

#[cfg(feature = "memory-optimized-read")]
impl<R: Read + Seek> ThreemfPackagePull<R> {
    /// Create a pull-based package with memory-optimized deserialization
    ///
    /// # Arguments
    /// * `reader` - A readable and seekable source (e.g., `File`)
    /// * `cache_policy` - Whether to cache loaded data (`CachePolicy::NoCache` is default)
    pub fn from_reader_with_memory_optimized_deserializer(
        reader: R,
        cache_policy: CachePolicy,
    ) -> Result<Self, Error> {
        Self::from_reader(reader, ReadStrategy::MemoryOptimized, cache_policy)
    }
}

#[cfg(feature = "speed-optimized-read")]
impl<R: Read + Seek> ThreemfPackagePull<R> {
    /// Create a pull-based package with speed-optimized deserialization
    ///
    /// # Arguments
    /// * `reader` - A readable and seekable source (e.g., `File`)
    /// * `cache_policy` - Whether to cache loaded data (`CachePolicy::NoCache` is default)
    pub fn from_reader_with_speed_optimized_deserializer(
        reader: R,
        cache_policy: CachePolicy,
    ) -> Result<Self, Error> {
        Self::from_reader(reader, ReadStrategy::SpeedOptimized, cache_policy)
    }
}

fn try_strip_leading_slash(target: &str) -> &str {
    match target.strip_prefix('/') {
        Some(stripped) => stripped,
        None => target,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;
    use std::fs::File;
    use std::path::PathBuf;

    #[cfg(feature = "memory-optimized-read")]
    #[test]
    fn test_pull_based_root_model_lazy_load() {
        let path = PathBuf::from("./tests/data/mesh-composedpart.3mf");
        let reader = File::open(path).unwrap();

        let package = ThreemfPackagePull::from_reader_with_memory_optimized_deserializer(
            reader,
            CachePolicy::NoCache,
        )
        .unwrap();

        // Metadata available immediately
        assert_eq!(package.relationships().len(), 1);
        assert!(package.root_model_path().contains("3dmodel.model"));

        // Model paths available without loading models
        let paths: Vec<_> = package.model_paths().collect();
        assert!(!paths.is_empty());

        // Root model loaded lazily
        let root = package.root_model().unwrap();
        assert_eq!(root.build.item.len(), 2);
    }

    #[cfg(feature = "memory-optimized-read")]
    #[test]
    fn test_pull_based_with_sub_models() {
        let path = PathBuf::from("./tests/data/third-party/P_XPX_0702_02.3mf");
        let reader = File::open(path).unwrap();

        let package = ThreemfPackagePull::from_reader_with_memory_optimized_deserializer(
            reader,
            CachePolicy::CacheAll,
        )
        .unwrap();

        // Check metadata
        assert_eq!(package.content_types().defaults.len(), 3);
        assert_eq!(package.relationships().len(), 2);

        // Model paths available
        let model_paths: Vec<_> = package.model_paths().collect();
        assert!(model_paths.len() >= 2); // root + at least one sub-model

        // Load root model
        let root = package.root_model().unwrap();
        assert!(!root.resources.object.is_empty());

        // Load sub-model
        let sub_model_path = "/3D/midway.model";
        let sub_model = package.get_sub_model(sub_model_path).unwrap();
        assert!(sub_model.is_some());
    }

    #[cfg(feature = "memory-optimized-read")]
    #[test]
    fn test_pull_based_thumbnails() {
        let path = PathBuf::from("./tests/data/third-party/P_XPX_0702_02.3mf");
        let reader = File::open(path).unwrap();

        let package = ThreemfPackagePull::from_reader_with_memory_optimized_deserializer(
            reader,
            CachePolicy::NoCache,
        )
        .unwrap();

        // Thumbnail paths available
        let thumbnail_paths: Vec<_> = package.thumbnail_paths().collect();
        assert!(!thumbnail_paths.is_empty());

        // Load thumbnail lazily
        let thumbnail_path = thumbnail_paths[0];
        let thumbnail = package.get_thumbnail(thumbnail_path).unwrap();
        assert!(thumbnail.is_some());

        let img = thumbnail.unwrap();
        assert!(img.width() > 0);
        assert!(img.height() > 0);
    }

    #[cfg(feature = "speed-optimized-read")]
    #[test]
    fn test_pull_based_speed_optimized() {
        let path = PathBuf::from("./tests/data/mesh-composedpart.3mf");
        let reader = File::open(path).unwrap();

        let package = ThreemfPackagePull::from_reader_with_speed_optimized_deserializer(
            reader,
            CachePolicy::CacheAll,
        )
        .unwrap();

        // Metadata available immediately
        assert!(!package.relationships().is_empty());

        // Root model loaded lazily
        let root = package.root_model().unwrap();
        assert_eq!(root.build.item.len(), 2);
    }
}
