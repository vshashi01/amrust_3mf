use crate::io::{
    content_types::DefaultContentTypeEnum,
    error::Error,
    relationship::{RelationshipType, Relationships},
    zip_utils::{self, RelationshipProcessor, XmlDeserializer},
};

use std::collections::HashMap;
use std::io::{Read, Seek};

struct ThreemfUnpackedProcessor {
    root: String,
    sub_models: HashMap<String, String>,
    thumbnails: HashMap<String, Vec<u8>>,
    unknown_parts: HashMap<String, Vec<u8>>,
    relationships: HashMap<String, String>,
    content_types: String,
}

impl ThreemfUnpackedProcessor {
    fn new(content_types: String, relationships: HashMap<String, String>) -> Self {
        Self {
            root: String::new(),
            sub_models: HashMap::new(),
            thumbnails: HashMap::new(),
            unknown_parts: HashMap::new(),
            relationships,
            content_types,
        }
    }

    fn into_threemf_unpacked(self) -> ThreemfUnpacked {
        ThreemfUnpacked {
            root: self.root,
            sub_models: self.sub_models,
            thumbnails: self.thumbnails,
            unknown_parts: self.unknown_parts,
            relationships: self.relationships,
            content_types: self.content_types,
        }
    }
}

impl RelationshipProcessor for ThreemfUnpackedProcessor {
    fn process_model(
        &mut self,
        target: &str,
        xml_reader: &mut impl Read,
        _deserializer: &zip_utils::XmlDeserializer,
        is_root: bool,
    ) -> Result<(), Error> {
        let mut xml_content = String::new();
        xml_reader.read_to_string(&mut xml_content)?;
        if is_root {
            self.root = xml_content;
        } else {
            self.sub_models.insert(target.to_string(), xml_content);
        }
        Ok(())
    }

    fn process_thumbnail(&mut self, target: &str, image_bytes: &[u8]) -> Result<(), Error> {
        self.thumbnails
            .insert(target.to_string(), image_bytes.to_vec());
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

/// Represents a 3mf package, the nested folder structure of the parts
/// in the 3mf package will be flattened into respective dictionaries with
/// the key being the path of the part in the archive package.
#[derive(Debug, PartialEq)]
pub struct ThreemfUnpacked {
    /// The root model of the 3mf package.
    /// Expected to always exist and be a valid model with a [Build](crate::core::build::Build) object.
    pub root: String,

    /// The sub models contained in the file. Usually this is to represent the [Object](crate::core::object::Object)
    /// that are to be referenced in the [root](ThreemfPackage::root) model part.
    /// The key is the path of the model in the archive package.
    pub sub_models: HashMap<String, String>,

    /// The thumbnails contained in the file.
    /// The key is the path of the thumbnail in the archive package.
    /// The thumbnail paths defined in the [Model](crate::core::model::Model) object should match the keys in this dictionary.
    pub thumbnails: HashMap<String, Vec<u8>>,

    /// Bytes of additional data found through Unknown relationship
    /// The key is the path of the thumbnail in the archive package.
    pub unknown_parts: HashMap<String, Vec<u8>>,

    /// The relationships between the different parts in the 3mf package.
    /// The key is the path of the relationship file in the archive package.
    /// Always expected to have at least 1 relationship file,
    /// the root relationship file placed within "_rels" folder at the root of the package
    pub relationships: HashMap<String, String>,

    /// A summary of all Default Content Types that exists in the current 3mf package.
    /// The reader/writer will still read and write data not currently known to library as
    /// unknown data.
    /// The extensions defined in the [ContentTypes.xml]
    ///  file should match the extensions of the parts in the package.
    pub content_types: String,
}

impl ThreemfUnpacked {
    /// Reads a 3mf package from a type [Read] + [io::Seek].
    /// Expected to deal with nested parts of the 3mf package and flatten them into the respective dictionaries.
    /// Only If [process_sub_models] is set to true, it will process the sub models and thumbnails associated with the sub models in the package.
    /// Will return an error if the package is not a valid 3mf package or if the package contains unsupported content types.
    pub fn from_reader<R: Read + Seek>(reader: R, process_sub_models: bool) -> Result<Self, Error> {
        let deserializer = XmlDeserializer::Raw;
        let (mut zip, _content_types, content_types_string, root_rels_filename) =
            zip_utils::setup_archive_and_content_types(reader, deserializer)?;

        let rels_ext = {
            let rels_content = _content_types
                .defaults
                .iter()
                .find(|t| t.content_type == DefaultContentTypeEnum::Relationship);

            match rels_content {
                Some(rels) => &rels.extension,
                None => "rels",
            }
        };

        let mut relationships = HashMap::<String, Relationships>::new();
        let mut rels_strings_map = HashMap::<String, String>::new();

        let root_rels: (Relationships, String) =
            zip_utils::relationships_from_zip_by_name_with_raw(&mut zip, &root_rels_filename)?;

        let root_model_rel = root_rels
            .0
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

        relationships.insert(root_rels_filename.clone(), root_rels.0.clone());
        rels_strings_map.insert(root_rels_filename.clone(), root_rels.1);

        if process_sub_models {
            let rel_files =
                zip_utils::discover_relationship_files(&mut zip, rels_ext, &root_rels_filename)?;
            for rel_file_path in rel_files {
                let rels = zip_utils::relationships_from_zip_by_name_with_raw(
                    &mut zip,
                    &rel_file_path[1..],
                )?;
                relationships.insert(rel_file_path.clone(), rels.0.clone());
                rels_strings_map.insert(rel_file_path, rels.1);
            }
        }

        let mut processor = ThreemfUnpackedProcessor::new(content_types_string, rels_strings_map);

        // Process all relationships
        zip_utils::process_relationships(
            &mut zip,
            &relationships,
            &mut processor,
            &deserializer,
            &root_model_path,
        )?;

        // Move root model from sub_models to root if it was processed as a sub-model
        if let Some(root_content) = processor.sub_models.remove(&root_model_path) {
            processor.root = root_content;
        }

        Ok(processor.into_threemf_unpacked())
    }
}

impl RelationshipProcessor for ThreemfUnpacked {
    fn process_model(
        &mut self,
        target: &str,
        xml_reader: &mut impl Read,
        _deserializer: &XmlDeserializer,
        is_root: bool,
    ) -> Result<(), Error> {
        let mut xml_content = String::new();
        xml_reader.read_to_string(&mut xml_content)?;
        if is_root {
            self.root = xml_content;
        } else {
            self.sub_models.insert(target.to_string(), xml_content);
        }
        Ok(())
    }

    fn process_thumbnail(&mut self, target: &str, image_bytes: &[u8]) -> Result<(), Error> {
        self.thumbnails
            .insert(target.to_string(), image_bytes.to_vec());
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

#[cfg(test)]
pub mod smoke_tests {
    use pretty_assertions::assert_eq;

    use super::ThreemfUnpacked;

    use std::io::Cursor;

    #[test]
    pub fn from_reader_root_model_test() {
        let bytes = include_bytes!("../../tests/data/third-party/P_XPX_0702_02.3mf");
        let reader = Cursor::new(bytes);

        let result = ThreemfUnpacked::from_reader(reader, true);

        match result {
            Ok(threemf) => {
                assert_eq!(threemf.content_types.len(), 407);
                assert_eq!(threemf.root.len(), 988);
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
}
