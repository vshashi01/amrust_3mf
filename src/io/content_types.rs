use instant_xml::{Error, ToXml};

#[cfg(feature = "memory-optimized-read")]
use instant_xml::{FromXml, Kind};

#[cfg(feature = "speed-optimized-read")]
use serde::Deserialize;

/// Content types for the Open Packaging Conventions (OPC).
/// Contains a collection of [DefaultContentTypes].
/// [DefaultContentTypes] contains the [DefaultContetnTypeEnum] specifying the content type.
/// [DefaultContentTypes] contains the file extension that is used for the specified content type.
#[cfg_attr(feature = "speed-optimized-read", derive(Deserialize))]
#[cfg_attr(feature = "speed-optimized-read", serde(rename = "Types"))]
#[cfg_attr(feature = "memory-optimized-read", derive(FromXml))]
// #[cfg_attr(
//     feature = "memory-optimized-read",
//     xml(ns(CORE_TRIANGLESET_NS), rename = "trianglesets")
// )]
#[derive(ToXml, Debug, PartialEq, Eq)]
#[xml(ns(CONTENT_TYPES_NS), rename = "Types")]
pub struct ContentTypes {
    #[cfg_attr(feature = "speed-optimized-read", serde(rename = "Default"))]
    pub defaults: Vec<DefaultContentTypes>,
}

/// Predefined content types supported by [threemf::io] currently.
/// If a content type is not found, it will fail the 3mf file parsing.
#[cfg_attr(feature = "speed-optimized-read", derive(Deserialize))]
#[cfg_attr(feature = "speed-optimized-read", serde(from = "String"))]
#[derive(Debug, PartialEq, Eq)]
pub enum DefaultContentTypeEnum {
    /// Represents a relationship content.
    Relationship,

    /// Represents a 3D model content.
    Model,

    /// Represents a PNG image content.
    ImagePng,

    /// Represents a JPEG image content.
    ImageJPEG,

    // Represents a Content Type that is not currently known to this library
    // content namespace is stored in the tuple.
    Unknown(String),
}

const RELATIONSHIP_NS: &str = "application/vnd.openxmlformats-package.relationships+xml";
const MODEL_NS: &str = "application/vnd.ms-package.3dmanufacturing-3dmodel+xml";
const PNG_NS: &str = "image/png";
const JPEG_NS: &str = "image/jpeg";

impl ToXml for DefaultContentTypeEnum {
    fn serialize<W: std::fmt::Write + ?Sized>(
        &self,
        _: Option<instant_xml::Id<'_>>,
        serializer: &mut instant_xml::Serializer<W>,
    ) -> Result<(), Error> {
        let ns_str = match self {
            Self::Relationship => RELATIONSHIP_NS,
            Self::Model => MODEL_NS,
            Self::ImagePng => PNG_NS,
            Self::ImageJPEG => JPEG_NS,
            Self::Unknown(ns) => ns,
        };

        serializer.write_str(ns_str)?;
        Ok(())
    }
}

#[cfg(feature = "memory-optimized-read")]
impl<'xml> FromXml<'xml> for DefaultContentTypeEnum {
    fn matches(id: instant_xml::Id<'_>, field: Option<instant_xml::Id<'_>>) -> bool {
        match field {
            Some(field) => id == field,
            None => false,
        }
    }

    fn deserialize<'cx>(
        into: &mut Self::Accumulator,
        field: &'static str,
        deserializer: &mut instant_xml::Deserializer<'cx, 'xml>,
    ) -> Result<(), Error> {
        if into.is_some() {
            return Err(Error::DuplicateValue(field));
        }

        let value = match deserializer.take_str()? {
            Some(value) => value,
            None => return Err(Error::MissingValue("No ContentType string found")),
        };

        let content_type = DefaultContentTypeEnum::from(value.into_owned());
        *into = Some(content_type);
        Ok(())
    }

    type Accumulator = Option<Self>;

    const KIND: Kind = Kind::Scalar;
}

impl From<String> for DefaultContentTypeEnum {
    fn from(value: String) -> Self {
        match value.as_ref() {
            RELATIONSHIP_NS => Self::Relationship,
            MODEL_NS => Self::Model,
            PNG_NS => Self::ImagePng,
            JPEG_NS => Self::ImageJPEG,
            value => Self::Unknown(value.to_owned()),
        }
    }
}

/// Internal structure for serde of [ContentTypes].
#[cfg_attr(feature = "speed-optimized-read", derive(Deserialize))]
#[cfg_attr(feature = "speed-optimized-read", serde(rename = "Default"))]
#[cfg_attr(feature = "memory-optimized-read", derive(FromXml))]
#[derive(ToXml, Debug, PartialEq, Eq)]
#[xml(ns(CONTENT_TYPES_NS), rename = "Default")]
pub struct DefaultContentTypes {
    #[xml(attribute, rename = "Extension")]
    #[cfg_attr(feature = "speed-optimized-read", serde(rename = "Extension"))]
    pub extension: String,

    #[xml(attribute, rename = "ContentType")]
    #[cfg_attr(feature = "speed-optimized-read", serde(rename = "ContentType"))]
    pub content_type: DefaultContentTypeEnum,
}

const CONTENT_TYPES_NS: &str = "http://schemas.openxmlformats.org/package/2006/content-types";

#[cfg(test)]
pub mod tests {
    use instant_xml::to_string;
    use pretty_assertions::assert_eq;

    use super::{
        CONTENT_TYPES_NS, ContentTypes, DefaultContentTypeEnum, DefaultContentTypes, JPEG_NS,
        MODEL_NS, PNG_NS, RELATIONSHIP_NS,
    };

    #[test]
    pub fn toxml_content_types_test() {
        let xml_string = format!(
            r#"<Types xmlns="{}"><Default Extension="rels" ContentType="{}" /><Default Extension="model" ContentType="{}" /><Default Extension="png" ContentType="{}" /><Default Extension="jpg" ContentType="{}" /><Default Extension="unknown" ContentType="//some//unknown//content" /></Types>"#,
            CONTENT_TYPES_NS, RELATIONSHIP_NS, MODEL_NS, PNG_NS, JPEG_NS
        );
        let content = ContentTypes {
            defaults: vec![
                DefaultContentTypes {
                    extension: "rels".to_owned(),
                    content_type: DefaultContentTypeEnum::Relationship,
                },
                DefaultContentTypes {
                    extension: "model".to_owned(),
                    content_type: DefaultContentTypeEnum::Model,
                },
                DefaultContentTypes {
                    extension: "png".to_owned(),
                    content_type: DefaultContentTypeEnum::ImagePng,
                },
                DefaultContentTypes {
                    extension: "jpg".to_owned(),
                    content_type: DefaultContentTypeEnum::ImageJPEG,
                },
                DefaultContentTypes {
                    extension: "unknown".to_owned(),
                    content_type: DefaultContentTypeEnum::Unknown(
                        "//some//unknown//content".to_owned(),
                    ),
                },
            ],
        };
        let content_string = to_string(&content).unwrap();

        assert_eq!(content_string, xml_string);
    }
}

#[cfg(feature = "memory-optimized-read")]
#[cfg(test)]
pub mod memory_read_tests {
    use instant_xml::from_str;
    use pretty_assertions::assert_eq;

    use super::{
        CONTENT_TYPES_NS, ContentTypes, DefaultContentTypeEnum, DefaultContentTypes, JPEG_NS,
        MODEL_NS, PNG_NS, RELATIONSHIP_NS,
    };

    #[test]
    pub fn fromxml_content_types_test() {
        let xml_string = format!(
            r#"<Types xmlns="{}"><Default Extension="rels" ContentType="{}" /><Default Extension="model" ContentType="{}" /><Default Extension="png" ContentType="{}" /><Default Extension="jpg" ContentType="{}" /></Types>"#,
            CONTENT_TYPES_NS, RELATIONSHIP_NS, MODEL_NS, PNG_NS, JPEG_NS
        );

        let content = from_str::<ContentTypes>(&xml_string).unwrap();

        assert_eq!(
            content,
            ContentTypes {
                defaults: vec![
                    DefaultContentTypes {
                        extension: "rels".to_owned(),
                        content_type: DefaultContentTypeEnum::Relationship,
                    },
                    DefaultContentTypes {
                        extension: "model".to_owned(),
                        content_type: DefaultContentTypeEnum::Model,
                    },
                    DefaultContentTypes {
                        extension: "png".to_owned(),
                        content_type: DefaultContentTypeEnum::ImagePng,
                    },
                    DefaultContentTypes {
                        extension: "jpg".to_owned(),
                        content_type: DefaultContentTypeEnum::ImageJPEG,
                    },
                ],
            }
        );
    }

    #[test]
    pub fn fromxml_unknown_content_types_test() {
        let xml_string = format!(
            r#"<Types xmlns="{}"><Default Extension="rels" ContentType="{}"/><Default Extension="model" ContentType="{}"/><Default Extension="unknown" ContentType="some/unknown/content"/></Types>"#,
            CONTENT_TYPES_NS, RELATIONSHIP_NS, MODEL_NS,
        );
        let content = from_str::<ContentTypes>(&xml_string).unwrap();

        assert_eq!(
            content,
            ContentTypes {
                defaults: vec![
                    DefaultContentTypes {
                        extension: "rels".to_owned(),
                        content_type: DefaultContentTypeEnum::Relationship,
                    },
                    DefaultContentTypes {
                        extension: "model".to_owned(),
                        content_type: DefaultContentTypeEnum::Model,
                    },
                    DefaultContentTypes {
                        extension: "unknown".to_owned(),
                        content_type: DefaultContentTypeEnum::Unknown(
                            "some/unknown/content".to_owned()
                        ),
                    }
                ]
            }
        );
    }
}

#[cfg(feature = "speed-optimized-read")]
#[cfg(test)]
pub mod speed_read_tests {
    use pretty_assertions::assert_eq;
    use serde_roxmltree::from_str;

    use super::{
        CONTENT_TYPES_NS, ContentTypes, DefaultContentTypeEnum, DefaultContentTypes, JPEG_NS,
        MODEL_NS, PNG_NS, RELATIONSHIP_NS,
    };

    #[test]
    pub fn fromxml_content_types_test() {
        let xml_string = format!(
            r#"<Types xmlns="{}"><Default Extension="rels" ContentType="{}" /><Default Extension="model" ContentType="{}" /><Default Extension="png" ContentType="{}" /><Default Extension="jpg" ContentType="{}" /></Types>"#,
            CONTENT_TYPES_NS, RELATIONSHIP_NS, MODEL_NS, PNG_NS, JPEG_NS
        );

        let content = from_str::<ContentTypes>(&xml_string).unwrap();

        assert_eq!(
            content,
            ContentTypes {
                defaults: vec![
                    DefaultContentTypes {
                        extension: "rels".to_owned(),
                        content_type: DefaultContentTypeEnum::Relationship,
                    },
                    DefaultContentTypes {
                        extension: "model".to_owned(),
                        content_type: DefaultContentTypeEnum::Model,
                    },
                    DefaultContentTypes {
                        extension: "png".to_owned(),
                        content_type: DefaultContentTypeEnum::ImagePng,
                    },
                    DefaultContentTypes {
                        extension: "jpg".to_owned(),
                        content_type: DefaultContentTypeEnum::ImageJPEG,
                    },
                ],
            }
        );
    }

    #[test]
    pub fn fromxml_unknown_content_types_test() {
        let xml_string = format!(
            r#"<Types xmlns="{}"><Default Extension="rels" ContentType="{}"/><Default Extension="model" ContentType="{}"/><Default Extension="unknown" ContentType="some/unknown/content"/></Types>"#,
            CONTENT_TYPES_NS, RELATIONSHIP_NS, MODEL_NS,
        );
        let content = from_str::<ContentTypes>(&xml_string).unwrap();

        assert_eq!(
            content,
            ContentTypes {
                defaults: vec![
                    DefaultContentTypes {
                        extension: "rels".to_owned(),
                        content_type: DefaultContentTypeEnum::Relationship,
                    },
                    DefaultContentTypes {
                        extension: "model".to_owned(),
                        content_type: DefaultContentTypeEnum::Model,
                    },
                    DefaultContentTypes {
                        extension: "unknown".to_owned(),
                        content_type: DefaultContentTypeEnum::Unknown(
                            "some/unknown/content".to_owned()
                        ),
                    }
                ]
            }
        );
    }
}
