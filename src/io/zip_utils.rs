use zip::ZipArchive;

use crate::io::{
    content_types::{ContentTypes, DefaultContentTypeEnum},
    error::Error,
    relationship::{RelationshipType, Relationships},
    utils,
};

// #[cfg(any(
//     feature = "io-memory-optimized-read",
//     feature = "io-speed-optimized-read"
// ))]
use crate::core::model::Model;

use std::collections::HashMap;
use std::ffi::OsStr;
use std::io::{Read, Seek};
use std::path::Path;

/// Enum for different XML deserialization strategies
#[cfg(any(
    feature = "io-memory-optimized-read",
    feature = "io-speed-optimized-read"
))]
#[derive(Clone, Copy)]
pub(crate) enum XmlDeserializer {
    #[cfg(feature = "io-memory-optimized-read")]
    MemoryOptimized,
    #[cfg(feature = "io-speed-optimized-read")]
    SpeedOptimized,
    // #[cfg(feature = "io-unpack")]
    // Raw,
}

impl XmlDeserializer {
    pub(crate) fn deserialize_content_types<R: Read>(
        &self,
        mut reader: R,
    ) -> Result<ContentTypes, Error> {
        match self {
            #[cfg(feature = "io-memory-optimized-read")]
            XmlDeserializer::MemoryOptimized => {
                let mut xml_string = String::new();
                reader.read_to_string(&mut xml_string)?;
                instant_xml::from_str::<ContentTypes>(&xml_string).map_err(Error::from)
            }
            #[cfg(feature = "io-speed-optimized-read")]
            XmlDeserializer::SpeedOptimized => {
                let mut xml_string = String::new();
                reader.read_to_string(&mut xml_string)?;
                serde_roxmltree::from_str::<ContentTypes>(&xml_string).map_err(Error::from)
            } // #[cfg(feature = "io-unpack")]
              // XmlDeserializer::Raw => {
              //     let mut xml_string = String::new();
              //     reader.read_to_string(&mut xml_string)?;
              //     serde_roxmltree::from_str::<ContentTypes>(&xml_string).map_err(Error::from)
              // }
        }
    }

    pub(crate) fn deserialize_relationships<R: Read>(
        &self,
        mut reader: R,
    ) -> Result<Relationships, Error> {
        match self {
            #[cfg(feature = "io-memory-optimized-read")]
            XmlDeserializer::MemoryOptimized => {
                let mut xml_string = String::new();
                reader.read_to_string(&mut xml_string)?;
                instant_xml::from_str::<Relationships>(&xml_string).map_err(Error::from)
            }
            #[cfg(feature = "io-speed-optimized-read")]
            XmlDeserializer::SpeedOptimized => {
                let mut xml_string = String::new();
                reader.read_to_string(&mut xml_string)?;
                serde_roxmltree::from_str::<Relationships>(&xml_string).map_err(Error::from)
            } // #[cfg(feature = "io-unpack")]
              // XmlDeserializer::Raw => {
              //     let mut xml_string = String::new();
              //     reader.read_to_string(&mut xml_string)?;
              //     serde_roxmltree::from_str::<Relationships>(&xml_string).map_err(Error::from)
              // }
        }
    }

    pub(crate) fn deserialize_model<R: Read>(&self, mut reader: R) -> Result<Model, Error> {
        match self {
            #[cfg(feature = "io-memory-optimized-read")]
            XmlDeserializer::MemoryOptimized => {
                let mut xml_string = String::new();
                reader.read_to_string(&mut xml_string)?;
                instant_xml::from_str::<Model>(&xml_string).map_err(Error::from)
            }
            #[cfg(feature = "io-speed-optimized-read")]
            XmlDeserializer::SpeedOptimized => {
                let mut xml_string = String::new();
                reader.read_to_string(&mut xml_string)?;
                serde_roxmltree::from_str::<Model>(&xml_string).map_err(Error::from)
            } // #[cfg(feature = "io-unpack")]
              // XmlDeserializer::Raw => unreachable!("Raw deserializer should not deserialize models"),
        }
    }
}

pub(crate) trait RelationshipProcessor {
    fn process_model(
        &mut self,
        target: &str,
        xml_reader: &mut impl Read,
        deserializer: &XmlDeserializer,
        is_root: bool,
    ) -> Result<(), Error>;

    fn process_thumbnail(&mut self, target: &str, image_bytes: &[u8]) -> Result<(), Error>;

    fn process_unknown(
        &mut self,
        target: &str,
        content_type: &str,
        data: &[u8],
    ) -> Result<(), Error>;
}

pub(crate) fn setup_archive_and_content_types<R: Read + Seek>(
    reader: R,
    deserializer: XmlDeserializer,
) -> Result<(ZipArchive<R>, ContentTypes, String, String), Error> {
    let mut zip = ZipArchive::new(reader)?;

    let (content_types, content_types_string) = parse_content_types(&mut zip, deserializer)?;
    let rels_ext = determine_relationships_extension(&content_types);

    let root_rels_filename = "_rels/.{extension}".replace("{extension}", &rels_ext);

    Ok((zip, content_types, content_types_string, root_rels_filename))
}

fn parse_content_types<R: Read + Seek>(
    zip: &mut ZipArchive<R>,
    deserializer: XmlDeserializer,
) -> Result<(ContentTypes, String), Error> {
    let content_types_file = zip.by_name("[Content_Types].xml");
    match content_types_file {
        Ok(mut file) => {
            let mut xml_string = String::new();
            file.read_to_string(&mut xml_string)?;
            let content_types = deserializer.deserialize_content_types(xml_string.as_bytes())?;
            Ok((content_types, xml_string))
        }
        Err(err) => Err(Error::Zip(err)),
    }
}

fn determine_relationships_extension(content_types: &ContentTypes) -> String {
    content_types
        .defaults
        .iter()
        .find(|t| t.content_type == DefaultContentTypeEnum::Relationship)
        .map(|rels| rels.extension.clone())
        .unwrap_or_else(|| "rels".to_string())
}

/// Find all relationship files in the archive (excluding the root relationships file)
pub(crate) fn discover_relationship_files<R: Read + Seek>(
    zip: &mut ZipArchive<R>,
    rels_ext: &str,
    root_rels_filename: &str,
) -> Result<Vec<String>, Error> {
    let mut rel_files = Vec::new();

    for i in 0..zip.len() {
        let file = zip.by_index(i)?;

        if file.is_file()
            && let Some(path) = file.enclosed_name()
            && path.extension() == Some(OsStr::new(rels_ext))
            && path != Path::new(root_rels_filename)
        {
            if let Some(path_str) = path.to_str() {
                rel_files.push(format!("/{path_str}"));
            } else {
                return Err(Error::ReadError(
                    "Failed to read relationship file path".to_owned(),
                ));
            }
        }
    }

    Ok(rel_files)
}

pub(crate) fn relationships_from_zipfile<R: Read>(
    file: zip::read::ZipFile<'_, R>,
    deserializer: &XmlDeserializer,
) -> Result<Relationships, Error> {
    deserializer.deserialize_relationships(file)
}

pub(crate) fn relationships_from_zip_by_name<R: Read + Seek>(
    zip: &mut ZipArchive<R>,
    zip_filename: &str,
    deserializer: &XmlDeserializer,
) -> Result<Relationships, Error> {
    let rels_file = zip.by_name(zip_filename);
    match rels_file {
        Ok(file) => relationships_from_zipfile(file, deserializer),
        Err(err) => Err(Error::Zip(err)),
    }
}

// #[cfg(feature = "io-unpack")]
// pub(crate) fn relationships_from_zipfile_with_raw<R: Read>(
//     mut file: zip::read::ZipFile<'_, R>,
// ) -> Result<(Relationships, String), Error> {
//     let mut xml_string = String::new();
//     file.read_to_string(&mut xml_string)?;
//     let rels = serde_roxmltree::from_str::<Relationships>(&xml_string)?;
//     Ok((rels, xml_string))
// }

// #[cfg(feature = "io-unpack")]
// pub(crate) fn relationships_from_zip_by_name_with_raw<R: Read + Seek>(
//     zip: &mut ZipArchive<R>,
//     zip_filename: &str,
// ) -> Result<(Relationships, String), Error> {
//     let rels_file = zip.by_name(zip_filename);
//     match rels_file {
//         Ok(file) => relationships_from_zipfile_with_raw(file),
//         Err(err) => Err(Error::Zip(err)),
//     }
// }

pub(crate) fn process_relationships<R: Read + Seek, P: RelationshipProcessor>(
    zip: &mut ZipArchive<R>,
    relationships: &HashMap<String, Relationships>,
    processor: &mut P,
    deserializer: &XmlDeserializer,
    root_model_path: &str,
) -> Result<(), Error> {
    for rels in relationships.values() {
        for rel in &rels.relationships {
            let name = utils::try_strip_leading_slash(&rel.target);
            let zip_file = zip.by_name(name);

            match zip_file {
                Ok(mut file) => {
                    if file.is_dir() {
                        return Err(Error::ReadError(format!(
                            r#"Found a folder "{:?}" instead of a file"#,
                            file.enclosed_name()
                        )));
                    }

                    match rel.relationship_type {
                        RelationshipType::Thumbnail => {
                            let mut bytes = Vec::new();
                            file.read_to_end(&mut bytes)?;
                            processor.process_thumbnail(&rel.target, &bytes)?;
                        }
                        RelationshipType::Model => {
                            let is_root = rel.target == root_model_path;
                            processor.process_model(
                                &rel.target,
                                &mut file,
                                deserializer,
                                is_root,
                            )?;
                        }
                        RelationshipType::Unknown(ref content_type) => {
                            let mut bytes = Vec::new();
                            file.read_to_end(&mut bytes)?;
                            processor.process_unknown(&rel.target, content_type, &bytes)?;
                        }
                    }
                }
                Err(err) => return Err(Error::Zip(err)),
            }
        }
    }
    Ok(())
}
