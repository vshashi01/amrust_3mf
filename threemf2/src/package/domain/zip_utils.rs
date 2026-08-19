//! ZIP archive utilities for reading 3MF package files.

use compact_str::format_compact;
use zip::ZipArchive;

use crate::{
    model::{PathResource, StrResource},
    package::{
        Error,
        domain::{
            content_types::{ContentTypes, DefaultContentTypeEnum},
            relationship::Relationships,
        },
    },
};

use crate::model::domain::model::Model;

use std::ffi::OsStr;
use std::io::{Read, Seek};
use std::path::Path;

/// Enum for different XML deserialization strategies
#[cfg(feature = "package-memory-optimized-read")]
#[derive(Clone, Copy)]
pub(crate) enum XmlDeserializer {
    #[cfg(feature = "package-memory-optimized-read")]
    MemoryOptimized,
}

impl XmlDeserializer {
    pub(crate) fn deserialize_content_types(
        &self,
        xml_string: &str,
    ) -> Result<ContentTypes, Error> {
        match self {
            #[cfg(feature = "package-memory-optimized-read")]
            XmlDeserializer::MemoryOptimized => {
                instant_xml::from_str::<ContentTypes>(xml_string).map_err(Error::from)
            }
        }
    }

    pub(crate) fn deserialize_relationships(
        &self,
        xml_string: &str,
    ) -> Result<Relationships, Error> {
        match self {
            #[cfg(feature = "package-memory-optimized-read")]
            XmlDeserializer::MemoryOptimized => {
                instant_xml::from_str::<Relationships>(xml_string).map_err(Error::from)
            }
        }
    }

    pub(crate) fn deserialize_model(&self, xml_string: &str) -> Result<Model, Error> {
        let model = match self {
            #[cfg(feature = "package-memory-optimized-read")]
            XmlDeserializer::MemoryOptimized => instant_xml::from_str::<Model>(xml_string)?,
        };

        Ok(model)
    }
}

pub(crate) fn read_zipfile_to_string<R: Read>(
    file: &mut zip::read::ZipFile<'_, R>,
) -> Result<String, Error> {
    let size_hint = file.size() as usize;
    let mut xml_string = String::with_capacity(size_hint);
    file.read_to_string(&mut xml_string)?;
    Ok(xml_string)
}

pub(crate) fn setup_archive_and_content_types<R: Read + Seek>(
    reader: R,
    deserializer: XmlDeserializer,
) -> Result<(ZipArchive<R>, ContentTypes, String, PathResource), Error> {
    let mut zip = ZipArchive::new(reader)?;

    let (content_types, content_types_string) = parse_content_types(&mut zip, deserializer)?;
    let rels_ext = determine_relationships_extension(&content_types);

    //let root_rels_filename = "_rels/.{extension}".replace("{extension}", &rels_ext);
    let root_rels_filename = format_compact!("_rels/.{}", rels_ext);

    match PathResource::new(&root_rels_filename, true) {
        Ok(path) => Ok((zip, content_types, content_types_string, path)),
        Err(err) => Err(Error::PathResourceError(err)),
    }

    //Ok((zip, content_types, content_types_string, root_rels_filename))
}

fn parse_content_types<R: Read + Seek>(
    zip: &mut ZipArchive<R>,
    deserializer: XmlDeserializer,
) -> Result<(ContentTypes, String), Error> {
    let content_types_file = zip.by_name("[Content_Types].xml");
    match content_types_file {
        Ok(mut file) => {
            let xml_string = read_zipfile_to_string(&mut file)?;
            let content_types = deserializer.deserialize_content_types(&xml_string)?;
            Ok((content_types, xml_string))
        }
        Err(err) => Err(Error::Zip(err)),
    }
}

fn determine_relationships_extension(content_types: &ContentTypes) -> StrResource {
    content_types
        .defaults
        .iter()
        .find(|t| t.content_type == DefaultContentTypeEnum::Relationship)
        .map(|rels| rels.extension.clone())
        .unwrap_or_else(|| "rels".into())
}

/// Find all relationship files in the archive (excluding the root relationships file)
pub(crate) fn discover_relationship_files<R: Read + Seek>(
    zip: &mut ZipArchive<R>,
    rels_ext: &str,
    root_rels_filename: &str,
) -> Result<Vec<PathResource>, Error> {
    let mut rel_files = Vec::new();

    for i in 0..zip.len() {
        let file = zip.by_index(i)?;

        if file.is_file()
            && let Some(path) = file.enclosed_name()
            && path.extension() == Some(OsStr::new(rels_ext))
            && path != Path::new(root_rels_filename)
        {
            let zip_name = path
                .components()
                .map(|c| c.as_os_str().to_string_lossy())
                .collect::<Vec<_>>()
                .join("/");
            let final_path = format_compact!("/{zip_name}");

            match PathResource::new(&final_path, true) {
                Ok(path) => rel_files.push(path),
                Err(err) => return Err(Error::PathResourceError(err)),
            }
        }
    }

    Ok(rel_files)
}

pub(crate) fn relationships_from_zipfile<R: Read>(
    mut file: zip::read::ZipFile<'_, R>,
    deserializer: &XmlDeserializer,
) -> Result<Relationships, Error> {
    let xml_string = read_zipfile_to_string(&mut file)?;
    deserializer.deserialize_relationships(&xml_string)
}

pub(crate) fn relationships_from_zip_by_name<R: Read + Seek>(
    zip: &mut ZipArchive<R>,
    zip_filename: &PathResource,
    deserializer: &XmlDeserializer,
) -> Result<Relationships, Error> {
    let rels_file = zip.by_name(zip_filename.as_str_without_leading_slash());
    match rels_file {
        Ok(file) => relationships_from_zipfile(file, deserializer),
        Err(err) => Err(Error::Zip(err)),
    }
}
