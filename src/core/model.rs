use instant_xml::ToXml;

#[cfg(feature = "memory-optimized-read")]
use instant_xml::FromXml;

#[cfg(feature = "speed-optimized-read")]
use serde::Deserialize;

use crate::{
    core::{build::Build, metadata::Metadata, resources::Resources},
    threemf_namespaces::{CORE_NS, CORE_TRIANGLESET_NS, PROD_NS},
};

#[cfg_attr(feature = "speed-optimized-read", derive(Deserialize))]
#[cfg_attr(feature = "speed-optimized-read", serde(rename = "model"))]
#[cfg_attr(feature = "memory-optimized-read", derive(FromXml))]
#[derive(ToXml, Debug, PartialEq)]
#[xml(ns(CORE_NS, p = PROD_NS, t = CORE_TRIANGLESET_NS), rename = "model")]
pub struct Model {
    // #[xml(attribute)]
    // pub xmlns:Option<String>,
    #[cfg_attr(feature = "speed-optimized-read", serde(default))]
    #[xml(attribute)]
    pub unit: Option<Unit>,

    #[xml(attribute)]
    pub requiredextensions: Option<String>,

    #[xml(attribute)]
    pub recommendedextensions: Option<String>,

    #[cfg_attr(feature = "speed-optimized-read", serde(default))]
    pub metadata: Vec<Metadata>,

    pub resources: Resources,

    pub build: Build,
}

/// Model measurement unit, default is millimeter
#[cfg_attr(feature = "speed-optimized-read", derive(Deserialize))]
#[cfg_attr(feature = "speed-optimized-read", serde(from = "String"))]
#[cfg_attr(feature = "memory-optimized-read", derive(FromXml))]
#[derive(ToXml, Default, Debug, PartialEq, Eq)]
#[xml(scalar, rename_all = "lowercase")]
pub enum Unit {
    Micron,
    #[default]
    Millimeter,
    Centimeter,
    Inch,
    Foot,
    Meter,
}

impl From<String> for Unit {
    fn from(value: String) -> Self {
        match value.to_ascii_lowercase().as_str() {
            "micron" => Unit::Micron,
            "millimeter" => Unit::Millimeter,
            "centimeter" => Unit::Centimeter,
            "inch" => Unit::Inch,
            "foot" => Unit::Foot,
            "meter" => Unit::Meter,
            _ => Unit::Millimeter,
        }
    }
}

#[cfg(test)]
pub mod write_tests {
    use instant_xml::{ToXml, to_string};
    use pretty_assertions::assert_eq;

    use crate::{
        core::{
            build::{Build, Item},
            metadata::Metadata,
            object::{Object, ObjectType},
            resources::Resources,
        },
        threemf_namespaces::{
            CORE_NS, CORE_TRIANGLESET_NS, CORE_TRIANGLESET_PREFIX, PROD_NS, PROD_PREFIX,
        },
    };

    use super::{Model, Unit};

    #[test]
    pub fn toxml_simple_model_test() {
        let xml_string = format!(
            r#"<model xmlns="{}" xmlns:{}="{}" xmlns:{}="{}" unit="millimeter"><metadata name="Trial Metadata" /><resources><object id="346" type="model" name="test part"></object></resources><build><item objectid="346" /></build></model>"#,
            CORE_NS, PROD_PREFIX, PROD_NS, CORE_TRIANGLESET_PREFIX, CORE_TRIANGLESET_NS
        );
        let model = Model {
            // xmlns: None,
            unit: Some(Unit::Millimeter),
            requiredextensions: None,
            recommendedextensions: None,
            metadata: vec![Metadata {
                name: "Trial Metadata".to_owned(),
                preserve: None,
                value: None,
            }],
            resources: Resources {
                basematerials: vec![],
                object: vec![Object {
                    id: 346,
                    objecttype: Some(ObjectType::Model),
                    thumbnail: None,
                    partnumber: None,
                    name: Some("test part".to_owned()),
                    pid: None,
                    pindex: None,
                    uuid: None,
                    mesh: None,
                    components: None,
                }],
            },
            build: Build {
                uuid: None,
                item: vec![Item {
                    objectid: 346,
                    transform: None,
                    partnumber: None,
                    path: None,
                    uuid: None,
                }],
            },
        };
        let model_string = to_string(&model).unwrap();

        assert_eq!(model_string, xml_string);
    }

    #[derive(Debug, ToXml, PartialEq, Eq)]
    struct UnitsType {
        unit: Vec<Unit>,
    }

    #[test]
    pub fn toxml_units_test() {
        let xml_string = "<UnitsType><unit>micron</unit><unit>millimeter</unit><unit>centimeter</unit><unit>inch</unit><unit>foot</unit><unit>meter</unit></UnitsType>";
        let unitsvector = UnitsType {
            unit: vec![
                Unit::Micron,
                Unit::Millimeter,
                Unit::Centimeter,
                Unit::Inch,
                Unit::Foot,
                Unit::Meter,
            ],
        };
        let unitsvector_string = to_string(&unitsvector).unwrap();

        assert_eq!(unitsvector_string, xml_string);
    }
}

#[cfg(feature = "memory-optimized-read")]
#[cfg(test)]
pub mod memory_optimized_read_tests {
    use instant_xml::FromXml;
    use instant_xml::from_str;
    use pretty_assertions::assert_eq;

    use crate::{
        core::{
            build::{Build, Item},
            component::{Component, Components},
            metadata::Metadata,
            object::{Object, ObjectType},
            resources::Resources,
        },
        threemf_namespaces::{CORE_NS, PROD_NS},
    };

    use super::{Model, Unit};

    #[test]
    pub fn fromxml_simple_model_test() {
        let xml_string = format!(
            r#"<model xmlns="{}"><metadata name="Trial Metadata" /><resources><object id="346" type="model" name="test part"></object></resources><build><item objectid="346" /></build></model>"#,
            CORE_NS
        );

        let model = from_str::<Model>(&xml_string).unwrap();

        assert_eq!(
            model,
            Model {
                // xmlns: None,
                unit: None, //ToDo: Set the default value when unit is not supplied.
                requiredextensions: None,
                recommendedextensions: None,
                metadata: vec![Metadata {
                    name: "Trial Metadata".to_owned(),
                    preserve: None,
                    value: None,
                }],
                resources: Resources {
                    basematerials: vec![],
                    object: vec![Object {
                        id: 346,
                        objecttype: Some(ObjectType::Model),
                        thumbnail: None,
                        partnumber: None,
                        name: Some("test part".to_owned()),
                        pid: None,
                        pindex: None,
                        uuid: None,
                        mesh: None,
                        components: None,
                    }],
                },
                build: Build {
                    uuid: None,
                    item: vec![Item {
                        objectid: 346,
                        transform: None,
                        partnumber: None,
                        path: None,
                        uuid: None,
                    }],
                },
            }
        );
    }

    #[test]
    pub fn fromxml_production_model_test() {
        const CUSTOM_PROD_PREFIX: &str = "custom";
        let xml_string = format!(
            r#"<model xmlns="{}" xmlns:{}="{}" xml:lang="en-us" unit="millimeter"><metadata name="Trial Metadata" /><resources><object id="346" type="model" name="test part" {}:UUID="someObjectUUID"><components><component objectid="1" {}:path="//somePath//Component" {}:UUID="someComponentUUID" /></components></object></resources><build {}:UUID="someBuildUUID"><item objectid="346" {}:UUID="someItemUUID"/></build></model>"#,
            CORE_NS,
            CUSTOM_PROD_PREFIX,
            PROD_NS,
            CUSTOM_PROD_PREFIX,
            CUSTOM_PROD_PREFIX,
            CUSTOM_PROD_PREFIX,
            CUSTOM_PROD_PREFIX,
            CUSTOM_PROD_PREFIX,
        );
        let model = from_str::<Model>(&xml_string).unwrap();

        assert_eq!(
            model,
            Model {
                // xmlns: None,
                unit: Some(Unit::Millimeter),
                requiredextensions: None,
                recommendedextensions: None,
                metadata: vec![Metadata {
                    name: "Trial Metadata".to_owned(),
                    preserve: None,
                    value: None,
                }],
                resources: Resources {
                    basematerials: vec![],
                    object: vec![Object {
                        id: 346,
                        objecttype: Some(ObjectType::Model),
                        thumbnail: None,
                        partnumber: None,
                        name: Some("test part".to_owned()),
                        pid: None,
                        pindex: None,
                        uuid: Some("someObjectUUID".to_owned()),
                        mesh: None,
                        components: Some(Components {
                            component: vec![Component {
                                objectid: 1,
                                transform: None,
                                path: Some("//somePath//Component".to_owned()),
                                uuid: Some("someComponentUUID".to_owned()),
                            }]
                        }),
                    }],
                },
                build: Build {
                    uuid: Some("someBuildUUID".to_owned()),
                    item: vec![Item {
                        objectid: 346,
                        transform: None,
                        partnumber: None,
                        path: None,
                        uuid: Some("someItemUUID".to_owned()),
                    }],
                },
            }
        );
    }

    #[derive(FromXml, Debug, PartialEq, Eq)]
    struct UnitsType {
        unit: Vec<Unit>,
        #[xml(rename = "attr", attribute)]
        attribute: Option<Unit>,
    }

    #[test]
    pub fn fromxml_units_test() {
        let xml_string = r#"<UnitsType attr="inch"><unit>micron</unit><unit>millimeter</unit><unit>centimeter</unit><unit>inch</unit><unit>foot</unit><unit>meter</unit></UnitsType>"#;
        let unitsvector = from_str::<UnitsType>(xml_string).unwrap();

        assert_eq!(
            unitsvector,
            UnitsType {
                attribute: Some(Unit::Inch),
                unit: vec![
                    Unit::Micron,
                    Unit::Millimeter,
                    Unit::Centimeter,
                    Unit::Inch,
                    Unit::Foot,
                    Unit::Meter,
                ],
            }
        );
    }
}

#[cfg(feature = "speed-optimized-read")]
#[cfg(test)]
pub mod speed_optimized_read_tests {
    use pretty_assertions::assert_eq;
    use serde::Deserialize;
    use serde_roxmltree::from_str;

    use crate::{
        core::{
            build::{Build, Item},
            component::{Component, Components},
            metadata::Metadata,
            object::{Object, ObjectType},
            resources::Resources,
        },
        threemf_namespaces::{CORE_NS, PROD_NS},
    };

    use super::{Model, Unit};

    #[test]
    pub fn fromxml_simple_model_test() {
        let xml_string = format!(
            r#"<model xmlns="{}"><metadata name="Trial Metadata" /><resources><object id="346" type="model" name="test part"></object></resources><build><item objectid="346" /></build></model>"#,
            CORE_NS
        );

        let model = from_str::<Model>(&xml_string).unwrap();

        assert_eq!(
            model,
            Model {
                // xmlns: None,
                unit: None, //ToDo: Set the default value when unit is not supplied.
                requiredextensions: None,
                recommendedextensions: None,
                metadata: vec![Metadata {
                    name: "Trial Metadata".to_owned(),
                    preserve: None,
                    value: Some("".to_string()), //ToDo: Import output for empty value
                }],
                resources: Resources {
                    basematerials: vec![],
                    object: vec![Object {
                        id: 346,
                        objecttype: Some(ObjectType::Model),
                        thumbnail: None,
                        partnumber: None,
                        name: Some("test part".to_owned()),
                        pid: None,
                        pindex: None,
                        uuid: None,
                        mesh: None,
                        components: None,
                    }],
                },
                build: Build {
                    uuid: None,
                    item: vec![Item {
                        objectid: 346,
                        transform: None,
                        partnumber: None,
                        path: None,
                        uuid: None,
                    }],
                },
            }
        );
    }

    #[test]
    pub fn fromxml_production_model_test() {
        const CUSTOM_PROD_PREFIX: &str = "custom";
        let xml_string = format!(
            r#"<model xmlns="{}" xmlns:{}="{}" xml:lang="en-us" unit="millimeter"><metadata name="Trial Metadata" /><resources><object id="346" type="model" name="test part" {}:UUID="someObjectUUID"><components><component objectid="1" {}:path="//somePath//Component" {}:UUID="someComponentUUID" /></components></object></resources><build {}:UUID="someBuildUUID"><item objectid="346" {}:UUID="someItemUUID"/></build></model>"#,
            CORE_NS,
            CUSTOM_PROD_PREFIX,
            PROD_NS,
            CUSTOM_PROD_PREFIX,
            CUSTOM_PROD_PREFIX,
            CUSTOM_PROD_PREFIX,
            CUSTOM_PROD_PREFIX,
            CUSTOM_PROD_PREFIX,
        );
        let model = from_str::<Model>(&xml_string).unwrap();

        assert_eq!(
            model,
            Model {
                // xmlns: None,
                unit: Some(Unit::Millimeter),
                requiredextensions: None,
                recommendedextensions: None,
                metadata: vec![Metadata {
                    name: "Trial Metadata".to_owned(),
                    preserve: None,
                    value: Some("".to_string()), //ToDo: Improve output for empty value
                }],
                resources: Resources {
                    basematerials: vec![],
                    object: vec![Object {
                        id: 346,
                        objecttype: Some(ObjectType::Model),
                        thumbnail: None,
                        partnumber: None,
                        name: Some("test part".to_owned()),
                        pid: None,
                        pindex: None,
                        uuid: Some("someObjectUUID".to_owned()),
                        mesh: None,
                        components: Some(Components {
                            component: vec![Component {
                                objectid: 1,
                                transform: None,
                                path: Some("//somePath//Component".to_owned()),
                                uuid: Some("someComponentUUID".to_owned()),
                            }]
                        }),
                    }],
                },
                build: Build {
                    uuid: Some("someBuildUUID".to_owned()),
                    item: vec![Item {
                        objectid: 346,
                        transform: None,
                        partnumber: None,
                        path: None,
                        uuid: Some("someItemUUID".to_owned()),
                    }],
                },
            }
        );
    }

    #[derive(Deserialize, Debug, PartialEq, Eq)]
    struct UnitsType {
        // #[serde(rename = "unit")]
        unit: Vec<Unit>,
        #[serde(rename = "attr")]
        attribute: Option<Unit>,
    }

    #[test]
    pub fn fromxml_units_test() {
        let xml_string = r#"<UnitsType attr="Inch"><unit>micron</unit><unit>millimeter</unit><unit>centimeter</unit><unit>inch</unit><unit>foot</unit><unit>meter</unit></UnitsType>"#;
        let unitsvector = from_str::<UnitsType>(xml_string).unwrap();

        assert_eq!(
            unitsvector,
            UnitsType {
                attribute: Some(Unit::Inch),
                unit: vec![
                    Unit::Micron,
                    Unit::Millimeter,
                    Unit::Centimeter,
                    Unit::Inch,
                    Unit::Foot,
                    Unit::Meter,
                ],
            }
        );
    }
}
