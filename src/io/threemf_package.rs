use image::DynamicImage;

#[cfg(feature = "io-write")]
use instant_xml::ToXml;
use zip::ZipWriter;
use zip::write::SimpleFileOptions;

use crate::io::content_types::DefaultContentTypes;
use crate::io::relationship::Relationship;
use crate::{
    core::model::Model,
    io::{
        content_types::{ContentTypes, DefaultContentTypeEnum},
        error::Error,
        relationship::{RelationshipType, Relationships},
        zip_utils::{self, try_strip_leading_slash},
    },
};

use crate::io::zip_utils::XmlDeserializer;

use std::collections::HashMap;
use std::io::{self, Cursor, Read, Seek, Write};

/// Represents a 3mf package, the nested folder structure of the parts
/// in the 3mf package will be flattened into respective dictionaries with
/// the key being the path of the part in the archive package.
#[derive(Debug, PartialEq)]
pub struct ThreemfPackage {
    /// The root model of the 3mf package.
    /// Expected to always exist and be a valid model with a [Build](crate::core::build::Build) object.
    pub root: Model,

    /// The sub models contained in the file. Usually this is to represent the [Object](crate::core::object::Object)
    /// that are to be referenced in the [root](ThreemfPackage::root) model part.
    /// The key is the path of the model in the archive package.
    pub sub_models: HashMap<String, Model>,

    /// The thumbnails contained in the file.
    /// The key is the path of the thumbnail in the archive package.
    /// The thumbnail paths defined in the [Model](crate::core::model::Model) object should match the keys in this dictionary.
    pub thumbnails: HashMap<String, DynamicImage>,

    /// Bytes of additional data found through Unknown relationship
    /// The key is the path of the thumbnail in the archive package.
    pub unknown_parts: HashMap<String, Vec<u8>>,

    /// The relationships between the different parts in the 3mf package.
    /// The key is the path of the relationship file in the archive package.
    /// Always expected to have at least 1 relationship file,
    /// the root relationship file placed within "_rels" folder at the root of the package
    pub relationships: HashMap<String, Relationships>,

    /// A summary of all Default Content Types that exists in the current 3mf package.
    /// The reader/writer will still read and write data not currently known to library as
    /// unknown data.
    /// The extensions defined in the [ContentTypes.xml]
    ///  file should match the extensions of the parts in the package.
    pub content_types: ContentTypes,
}

#[cfg(feature = "io-write")]
impl ThreemfPackage {
    /// Writes the 3mf package to a [writer].
    /// Expects a well formed [ThreemfPackage] object to write the package.
    /// A well formed packaged requires atleast 1 root model and 1 relationship file along with the content types.
    pub fn write<W: io::Write + io::Seek>(&self, threemf_archive: W) -> Result<(), Error> {
        let mut zip = ZipWriter::new(threemf_archive);

        Self::archive_write_xml_with_header(&mut zip, "[Content_Types].xml", &self.content_types)?;

        for (path, relationships) in &self.relationships {
            Self::archive_write_xml_with_header(&mut zip, path, &relationships)?;

            for relationship in &relationships.relationships {
                let filename = try_strip_leading_slash(&relationship.target);
                match relationship.relationship_type {
                    RelationshipType::Model => {
                        let model = if *path == *"_rels/.rels" {
                            &self.root
                        } else if let Some(model) = self.sub_models.get(&relationship.target) {
                            model
                        } else {
                            return Err(Error::WriteError(format!(
                                "No model found for relationship target {}",
                                relationship.target
                            )));
                        };
                        Self::archive_write_xml_with_header(&mut zip, filename, model)?;
                    }
                    RelationshipType::Thumbnail => {
                        if let Some(image) = self.thumbnails.get(&relationship.target) {
                            let mut buf = Cursor::new(Vec::<u8>::new());
                            image.write_to(&mut buf, image::ImageFormat::Png)?;

                            zip.start_file(filename, SimpleFileOptions::default())?;
                            zip.write_all(&buf.into_inner())?;
                        } else {
                            return Err(Error::WriteError(format!(
                                "No thumbnail image found for relationship target {}",
                                &relationship.target
                            )));
                        }
                    }
                    RelationshipType::Unknown(_) => {
                        if let Some(bytes) = self.unknown_parts.get(&relationship.target) {
                            zip.start_file(filename, SimpleFileOptions::default())?;
                            zip.write_all(bytes)?;
                        } else {
                            return Err(Error::WriteError(format!(
                                "No data found for relationship target {}",
                                &relationship.target
                            )));
                        }
                    }
                }
            }
        }
        zip.finish()?;
        Ok(())
    }

    fn archive_write_xml_with_header<W: Write + Seek, T: ToXml + ?Sized>(
        archive: &mut ZipWriter<W>,
        filename: &str,
        content: &T,
    ) -> Result<(), Error> {
        use instant_xml::to_string;

        const XML_HEADER: &str = r#"<?xml version="1.0" encoding="UTF-8" standalone="no"?>"#;

        let mut content_string = to_string(&content)?;
        content_string.insert_str(0, XML_HEADER);

        archive.start_file(filename, SimpleFileOptions::default())?;
        archive.write_all(content_string.as_bytes())?;
        Ok(())
    }
}

#[cfg(any(
    feature = "io-memory-optimized-read",
    feature = "io-speed-optimized-read"
))]
impl ThreemfPackage {
    #[cfg(feature = "io-memory-optimized-read")]
    pub fn from_reader_with_memory_optimized_deserializer<R: Read + io::Seek>(
        reader: R,
        process_sub_models: bool,
    ) -> Result<Self, Error> {
        Self::from_reader(reader, process_sub_models, XmlDeserializer::MemoryOptimized)
    }

    #[cfg(feature = "io-speed-optimized-read")]
    pub fn from_reader_with_speed_optimized_deserializer<R: Read + io::Seek>(
        reader: R,
        process_sub_models: bool,
    ) -> Result<Self, Error> {
        Self::from_reader(reader, process_sub_models, XmlDeserializer::SpeedOptimized)
    }

    /// Reads a 3mf package from a type [Read] + [io::Seek].
    /// Expected to deal with nested parts of the 3mf package and flatten them into the respective dictionaries.
    /// Only If [process_sub_models] is set to true, it will process the sub models and thumbnails associated with the sub models in the package.
    /// Will return an error if the package is not a valid 3mf package or if the package contains unsupported content types.
    fn from_reader<R: Read + io::Seek>(
        reader: R,
        process_sub_models: bool,
        deserializer: XmlDeserializer,
    ) -> Result<Self, Error> {
        use crate::io::threemf_package::processor::ThreemfPackageProcessor;

        let (mut zip, content_types, _, root_rels_filename) =
            zip_utils::setup_archive_and_content_types(reader, deserializer)?;

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

        let mut relationships = HashMap::<String, Relationships>::new();

        let root_rels: Relationships = zip_utils::relationships_from_zip_by_name(
            &mut zip,
            &root_rels_filename,
            &deserializer,
        )?;

        let root_model_rel = root_rels
            .relationships
            .iter()
            .find(|rels| rels.relationship_type == RelationshipType::Model);

        let root_model_path = match root_model_rel {
            Some(rel) => rel.target.clone(),
            None => {
                return Err(Error::ReadError(
                    "Root model relationship not found".to_owned(),
                ));
            }
        };

        relationships.insert(root_rels_filename.clone(), root_rels.clone());

        if process_sub_models {
            let rel_files =
                zip_utils::discover_relationship_files(&mut zip, rels_ext, &root_rels_filename)?;
            for rel_file_path in rel_files {
                let rels = zip_utils::relationships_from_zip_by_name(
                    &mut zip,
                    &rel_file_path[1..],
                    &deserializer,
                )?;
                relationships.insert(rel_file_path, rels);
            }
        }

        let mut processor = ThreemfPackageProcessor::new(content_types);

        // Process all relationships
        zip_utils::process_relationships(
            &mut zip,
            &relationships,
            &mut processor,
            &deserializer,
            &root_model_path,
        )?;

        processor.set_relationships(relationships);

        Ok(processor.into_threemf_package())
    }
}

#[cfg(any(
    feature = "io-memory-optimized-read",
    feature = "io-speed-optimized-read"
))]
mod processor {
    use image::{DynamicImage, load_from_memory};

    use crate::{
        core::model::Model,
        io::{
            ThreemfPackage,
            content_types::ContentTypes,
            error::Error,
            relationship::Relationships,
            zip_utils::{RelationshipProcessor, XmlDeserializer},
        },
    };

    use std::{collections::HashMap, io::Read};
    /// Temporary processor for building ThreemfPackage
    pub(crate) struct ThreemfPackageProcessor {
        root: Option<Model>,
        sub_models: HashMap<String, Model>,
        thumbnails: HashMap<String, DynamicImage>,
        unknown_parts: HashMap<String, Vec<u8>>,
        relationships: HashMap<String, Relationships>,
        content_types: ContentTypes,
    }

    impl ThreemfPackageProcessor {
        pub(crate) fn new(content_types: ContentTypes) -> Self {
            Self {
                root: None,
                sub_models: HashMap::new(),
                thumbnails: HashMap::new(),
                unknown_parts: HashMap::new(),
                relationships: HashMap::new(),
                content_types,
            }
        }

        pub(crate) fn set_relationships(&mut self, relationships: HashMap<String, Relationships>) {
            self.relationships = relationships;
        }

        pub(crate) fn into_threemf_package(self) -> ThreemfPackage {
            ThreemfPackage {
                root: self.root.expect("Root model should be set"),
                sub_models: self.sub_models,
                thumbnails: self.thumbnails,
                unknown_parts: self.unknown_parts,
                relationships: self.relationships,
                content_types: self.content_types,
            }
        }
    }

    impl RelationshipProcessor for ThreemfPackageProcessor {
        fn process_model(
            &mut self,
            target: &str,
            xml_reader: &mut impl Read,
            deserializer: &XmlDeserializer,
            is_root: bool,
        ) -> Result<(), Error> {
            let model = deserializer.deserialize_model(xml_reader)?;
            if is_root {
                self.root = Some(model);
            } else {
                self.sub_models.insert(target.to_string(), model);
            }
            Ok(())
        }

        fn process_thumbnail(&mut self, target: &str, image_bytes: &[u8]) -> Result<(), Error> {
            let image = load_from_memory(image_bytes)?;
            self.thumbnails.insert(target.to_string(), image);
            Ok(())
        }

        fn process_unknown(
            &mut self,
            target: &str,
            _content_type: &str,
            data: &[u8],
        ) -> Result<(), Error> {
            self.unknown_parts.insert(target.to_string(), data.to_vec());
            Ok(())
        }
    }
}

impl From<Model> for ThreemfPackage {
    fn from(value: Model) -> Self {
        let mut rels = HashMap::new();
        rels.insert(
            "_rels/.rels".to_owned(),
            Relationships {
                relationships: vec![Relationship {
                    id: "rel0".to_owned(),
                    target: "3D/3dmodel.model".to_owned(),
                    relationship_type: RelationshipType::Model,
                }],
            },
        );
        Self {
            root: value,
            sub_models: HashMap::new(),
            thumbnails: HashMap::new(),
            unknown_parts: HashMap::new(),
            relationships: rels,
            content_types: ContentTypes {
                defaults: vec![
                    DefaultContentTypes {
                        extension: "model".to_owned(),
                        content_type: DefaultContentTypeEnum::Model,
                    },
                    DefaultContentTypes {
                        extension: "rels".to_owned(),
                        content_type: DefaultContentTypeEnum::Relationship,
                    },
                ],
            },
        }
    }
}

#[cfg(test)]
pub mod smoke_tests {
    use image::load_from_memory;
    use pretty_assertions::assert_eq;

    use crate::{
        core::{
            build::Build,
            model::{self, Model},
            object::{Object, ObjectType},
            resources::Resources,
        },
        io::{content_types::*, relationship::*},
    };

    use super::ThreemfPackage;

    use std::fs::File;
    use std::path::PathBuf;
    use std::{collections::HashMap, io::Cursor};

    #[cfg(feature = "io-memory-optimized-read")]
    #[test]
    pub fn from_reader_root_model_with_memory_optimized_read_test() {
        use crate::io::zip_utils::XmlDeserializer;

        let path = PathBuf::from("./tests/data/third-party/P_XPX_0702_02.3mf");
        let reader = File::open(path).unwrap();

        let result = ThreemfPackage::from_reader(reader, true, XmlDeserializer::MemoryOptimized);
        // println!("{:?}", result);

        match result {
            Ok(threemf) => {
                assert_eq!(threemf.content_types.defaults.len(), 3);
                assert_eq!(threemf.sub_models.len(), 1);
                assert_eq!(threemf.thumbnails.len(), 1);
                assert_eq!(threemf.relationships.len(), 2);

                assert!(threemf.sub_models.contains_key("/3D/midway.model"));

                assert!(threemf.relationships.contains_key("_rels/.rels"));
                assert!(
                    threemf
                        .relationships
                        .contains_key("/3D/_rels/3dmodel.model.rels")
                );
                assert!(
                    threemf
                        .thumbnails
                        .contains_key("/Thumbnails/P_XPX_0702_02.png")
                )
            }
            Err(err) => panic!("{:?}", err),
        }
    }

    #[cfg(feature = "io-speed-optimized-read")]
    #[test]
    pub fn from_reader_root_model_with_speed_optimized_read_test() {
        use crate::io::zip_utils::XmlDeserializer;

        let path = PathBuf::from("./tests/data/third-party/P_XPX_0702_02.3mf");
        let reader = File::open(path).unwrap();

        let result = ThreemfPackage::from_reader(reader, true, XmlDeserializer::SpeedOptimized);
        // println!("{:?}", result);

        match result {
            Ok(threemf) => {
                assert_eq!(threemf.content_types.defaults.len(), 3);
                assert_eq!(threemf.sub_models.len(), 1);
                assert_eq!(threemf.thumbnails.len(), 1);
                assert_eq!(threemf.relationships.len(), 2);

                assert!(threemf.sub_models.contains_key("/3D/midway.model"));

                assert!(threemf.relationships.contains_key("_rels/.rels"));
                assert!(
                    threemf
                        .relationships
                        .contains_key("/3D/_rels/3dmodel.model.rels")
                );
                assert!(
                    threemf
                        .thumbnails
                        .contains_key("/Thumbnails/P_XPX_0702_02.png")
                )
            }
            Err(err) => panic!("{:?}", err),
        }
    }

    #[cfg(feature = "io-write")]
    #[test]
    pub fn write_root_model_test() {
        let bytes = {
            let bytes = Vec::<u8>::new();
            let mut writer = Cursor::new(bytes);
            let threemf = ThreemfPackage {
                root: Model {
                    // xmlns: None,
                    unit: Some(model::Unit::Centimeter),
                    requiredextensions: None,
                    recommendedextensions: None,
                    metadata: vec![],
                    resources: Resources {
                        object: vec![Object {
                            id: 1,
                            objecttype: Some(ObjectType::Model),
                            thumbnail: None,
                            partnumber: None,
                            name: Some("Some object".to_owned()),
                            pid: None,
                            pindex: None,
                            uuid: Some("uuid".to_owned()),
                            mesh: None,
                            components: None,
                        }],
                        basematerials: vec![],
                    },
                    build: Build {
                        uuid: None,
                        item: vec![],
                    },
                },
                sub_models: HashMap::new(),
                thumbnails: HashMap::new(),
                unknown_parts: HashMap::new(),
                relationships: HashMap::from([(
                    "_rels/.rels".to_owned(),
                    Relationships {
                        relationships: vec![Relationship {
                            id: "rel0".to_owned(),
                            target: "3D/3Dmodel.model".to_owned(),
                            relationship_type: RelationshipType::Model,
                        }],
                    },
                )]),
                content_types: ContentTypes {
                    defaults: vec![
                        DefaultContentTypes {
                            extension: "rels".to_owned(),
                            content_type: DefaultContentTypeEnum::Relationship,
                        },
                        DefaultContentTypes {
                            extension: "model".to_owned(),
                            content_type: DefaultContentTypeEnum::Model,
                        },
                    ],
                },
            };
            threemf.write(&mut writer).unwrap();
            writer
        };

        assert_eq!(bytes.into_inner().len(), 976);
    }

    #[cfg(all(feature = "io-memory-optimized-read", feature = "io-write"))]
    #[test]
    pub fn io_unknown_content_test() {
        let test_file_bytes = include_bytes!("../../tests/data/test.txt");
        let mut writer = Cursor::new(Vec::<u8>::new());
        let unknown_target = "/Metadata/test.txt";

        let package = ThreemfPackage {
            root: Model {
                // xmlns: None,
                unit: Some(model::Unit::Millimeter),
                requiredextensions: None,
                recommendedextensions: None,
                metadata: vec![],
                resources: Resources {
                    object: vec![],
                    basematerials: vec![],
                },
                build: Build {
                    uuid: None,
                    item: vec![],
                },
            },
            sub_models: HashMap::new(),
            thumbnails: HashMap::new(),
            unknown_parts: HashMap::from([(unknown_target.to_owned(), test_file_bytes.into())]),
            relationships: HashMap::from([(
                "_rels/.rels".to_owned(),
                Relationships {
                    relationships: vec![
                        Relationship {
                            id: "rel0".to_owned(),
                            target: "3D/3Dmodel.model".to_owned(),
                            relationship_type: RelationshipType::Model,
                        },
                        Relationship {
                            id: "rel1".to_owned(),
                            target: unknown_target.to_owned(),
                            relationship_type: RelationshipType::Unknown(
                                "Metadata/text".to_owned(),
                            ),
                        },
                    ],
                },
            )]),
            content_types: ContentTypes {
                defaults: vec![
                    DefaultContentTypes {
                        content_type: DefaultContentTypeEnum::Relationship,
                        extension: "rels".to_owned(),
                    },
                    DefaultContentTypes {
                        content_type: DefaultContentTypeEnum::Unknown("Metadata/text".to_owned()),
                        extension: "txt".to_owned(),
                    },
                    DefaultContentTypes {
                        extension: "model".to_owned(),
                        content_type: DefaultContentTypeEnum::Model,
                    },
                ],
            },
        };

        let write_result = package.write(&mut writer);
        assert!(write_result.is_ok());

        let read_result =
            ThreemfPackage::from_reader_with_memory_optimized_deserializer(writer, false);

        match read_result {
            Ok(package) => {
                assert!(package.unknown_parts.contains_key(unknown_target));
                let read_unknown_bytes = package.unknown_parts.get(unknown_target).unwrap();
                assert_eq!(read_unknown_bytes, test_file_bytes);
            }
            Err(_) => panic!("io unknown content test failed"),
        }
    }

    #[cfg(all(feature = "io-memory-optimized-read", feature = "io-write"))]
    #[test]
    pub fn io_thumbnail_content_test() {
        let test_file_bytes = include_bytes!("../../tests/data/test_thumbnail.png");
        let write_image = load_from_memory(test_file_bytes).unwrap();

        let mut writer = Cursor::new(Vec::<u8>::new());
        let thumbnail_target = "/Thumbnails/test_thumbnail.png";

        let package = ThreemfPackage {
            root: Model {
                // xmlns: None,
                unit: Some(model::Unit::Millimeter),
                requiredextensions: None,
                recommendedextensions: None,
                metadata: vec![],
                resources: Resources {
                    object: vec![],
                    basematerials: vec![],
                },
                build: Build {
                    uuid: None,
                    item: vec![],
                },
            },
            sub_models: HashMap::new(),
            thumbnails: HashMap::from([(thumbnail_target.to_owned(), write_image)]),
            unknown_parts: HashMap::new(),
            relationships: HashMap::from([(
                "_rels/.rels".to_owned(),
                Relationships {
                    relationships: vec![
                        Relationship {
                            id: "rel0".to_owned(),
                            target: "3D/3Dmodel.model".to_owned(),
                            relationship_type: RelationshipType::Model,
                        },
                        Relationship {
                            id: "rel0x".to_owned(),
                            target: thumbnail_target.to_owned(),
                            relationship_type: RelationshipType::Thumbnail,
                        },
                    ],
                },
            )]),
            content_types: ContentTypes {
                defaults: vec![
                    DefaultContentTypes {
                        content_type: DefaultContentTypeEnum::Relationship,
                        extension: "rels".to_owned(),
                    },
                    DefaultContentTypes {
                        content_type: DefaultContentTypeEnum::ImagePng,
                        extension: "png".to_owned(),
                    },
                    DefaultContentTypes {
                        extension: "model".to_owned(),
                        content_type: DefaultContentTypeEnum::Model,
                    },
                ],
            },
        };

        let write_result = package.write(&mut writer);
        assert!(write_result.is_ok());

        let read_result =
            ThreemfPackage::from_reader_with_memory_optimized_deserializer(writer, false);

        match read_result {
            Ok(package) => {
                assert!(package.thumbnails.contains_key(thumbnail_target));
                let read_image = package.thumbnails.get(thumbnail_target).unwrap();
                assert_eq!(read_image.height(), 300);
                assert_eq!(read_image.width(), 300);
            }
            Err(_) => panic!("io thumbnail test failed"),
        }
    }
}
