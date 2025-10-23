use instant_xml::ToXml;

#[cfg(feature = "memory-optimized-read")]
use instant_xml::FromXml;

#[cfg(feature = "speed-optimized-read")]
use serde::Deserialize;

use crate::{
    core::{Mesh, component::Components},
    threemf_namespaces::{CORE_NS, PROD_NS},
};

#[cfg_attr(feature = "speed-optimized-read", derive(Deserialize))]
#[cfg_attr(feature = "memory-optimized-read", derive(FromXml))]
#[derive(ToXml, PartialEq, Debug)]
#[xml(ns(CORE_NS, p=PROD_NS), rename="object")]
pub struct Object {
    #[xml(attribute)]
    pub id: usize,

    #[xml(rename = "type", attribute)]
    #[cfg_attr(feature = "speed-optimized-read", serde(rename = "type"))]
    // #[serde(rename = "type")]
    pub objecttype: Option<ObjectType>,

    #[xml(attribute)]
    // #[cfg_attr(feature = "speed-optimized-read", serde(default))]
    pub thumbnail: Option<String>,

    #[xml(attribute)]
    pub partnumber: Option<String>,

    #[xml(attribute)]
    pub name: Option<String>,

    #[xml(attribute)]
    pub pid: Option<usize>,

    #[xml(attribute)]
    pub pindex: Option<usize>,

    #[xml(attribute, ns(PROD_NS), rename = "UUID")]
    #[cfg_attr(feature = "speed-optimized-read", serde(rename = "UUID"))]
    pub uuid: Option<String>,

    pub mesh: Option<Mesh>,

    pub components: Option<Components>,
}

#[cfg_attr(feature = "speed-optimized-read", derive(Deserialize))]
#[cfg_attr(feature = "speed-optimized-read", serde(from = "String"))]
#[cfg_attr(feature = "memory-optimized-read", derive(FromXml))]
#[derive(Debug, ToXml, Default, PartialEq, Eq)]
#[xml(scalar, rename_all = "lowercase")]
pub enum ObjectType {
    #[default]
    Model,
    Support,
    SolidSupport,
    Surface,
    Other,
}

impl From<String> for ObjectType {
    fn from(value: String) -> Self {
        match value.to_ascii_lowercase().as_str() {
            "model" => Self::Model,
            "support" => Self::Support,
            "solidsupport" => Self::SolidSupport,
            "surface" => Self::Surface,
            "other" => Self::Other,
            _ => Self::Model,
        }
    }
}

#[cfg(test)]
pub mod write_test {
    use instant_xml::{ToXml, to_string};
    use pretty_assertions::assert_eq;

    use crate::{
        core::{
            Mesh, Triangles, Vertices,
            component::{Component, Components},
        },
        threemf_namespaces::{
            CORE_NS, CORE_TRIANGLESET_NS, CORE_TRIANGLESET_PREFIX, PROD_NS, PROD_PREFIX,
        },
    };

    use super::{Object, ObjectType};

    use std::vec;

    #[test]
    pub fn toxml_simple_object_test() {
        let xml_string = format!(
            r#"<object xmlns="{}" xmlns:{}="{}" id="4"></object>"#,
            CORE_NS, PROD_PREFIX, PROD_NS
        );
        let object = Object {
            id: 4,
            objecttype: None,
            thumbnail: None,
            partnumber: None,
            name: None,
            pid: None,
            pindex: None,
            uuid: None,
            mesh: None,
            components: None,
        };
        let object_string = to_string(&object).unwrap();

        assert_eq!(object_string, xml_string);
    }

    #[test]
    pub fn toxml_production_object_test() {
        let xml_string = format!(
            r#"<object xmlns="{}" xmlns:{}="{}" id="4" {}:UUID="someUUID"></object>"#,
            CORE_NS, PROD_PREFIX, PROD_NS, PROD_PREFIX
        );
        let object = Object {
            id: 4,
            objecttype: None,
            thumbnail: None,
            partnumber: None,
            name: None,
            pid: None,
            pindex: None,
            uuid: Some("someUUID".to_owned()),
            mesh: None,
            components: None,
        };
        let object_string = to_string(&object).unwrap();

        assert_eq!(object_string, xml_string);
    }

    #[test]
    pub fn toxml_intermediate_object_test() {
        let xml_string = format!(
            r#"<object xmlns="{}" xmlns:{}="{}" id="4" type="model" thumbnail="\thumbnail\part_thumbnail.png" partnumber="part_1" name="Object Part"></object>"#,
            CORE_NS, PROD_PREFIX, PROD_NS
        );
        let object = Object {
            id: 4,
            objecttype: Some(ObjectType::Model),
            thumbnail: Some("\\thumbnail\\part_thumbnail.png".to_string()),
            partnumber: Some("part_1".to_string()),
            name: Some("Object Part".to_string()),
            pid: None,
            pindex: None,
            uuid: None,
            mesh: None,
            components: None,
        };
        let object_string = to_string(&object).unwrap();
        println!("{}", object_string);

        assert_eq!(object_string, xml_string);
    }

    #[test]
    pub fn toxml_advanced_mesh_object_test() {
        let xml_string = format!(
            r##"<object xmlns="{}" xmlns:{}="{}" id="4" type="model" thumbnail="\thumbnail\part_thumbnail.png" partnumber="part_1" name="Object Part"><mesh xmlns:{}="{}"><vertices></vertices><triangles></triangles></mesh></object>"##,
            CORE_NS, PROD_PREFIX, PROD_NS, CORE_TRIANGLESET_PREFIX, CORE_TRIANGLESET_NS
        );
        let object = Object {
            id: 4,
            objecttype: Some(ObjectType::Model),
            thumbnail: Some("\\thumbnail\\part_thumbnail.png".to_string()),
            partnumber: Some("part_1".to_string()),
            name: Some("Object Part".to_string()),
            pid: None,
            pindex: None,
            uuid: None,
            mesh: Some(Mesh {
                vertices: Vertices { vertex: vec![] },
                triangles: Triangles { triangle: vec![] },
                trianglesets: None,
            }),
            components: None,
        };
        let object_string = to_string(&object).unwrap();

        assert_eq!(object_string, xml_string);
    }

    #[test]
    pub fn toxml_advanced_component_object_test() {
        let xml_string = format!(
            r##"<object xmlns="{}" xmlns:{}="{}" id="4" type="model" thumbnail="\thumbnail\part_thumbnail.png" partnumber="part_1" name="Object Part"><components><component objectid="23" /></components></object>"##,
            CORE_NS, PROD_PREFIX, PROD_NS
        );
        let object = Object {
            id: 4,
            objecttype: Some(ObjectType::Model),
            thumbnail: Some("\\thumbnail\\part_thumbnail.png".to_string()),
            partnumber: Some("part_1".to_string()),
            name: Some("Object Part".to_string()),
            pid: None,
            pindex: None,
            uuid: None,
            mesh: None,
            components: Some(Components {
                component: vec![Component {
                    objectid: 23,
                    transform: None,
                    path: None,
                    uuid: None,
                }],
            }),
        };
        let object_string = to_string(&object).unwrap();

        assert_eq!(object_string, xml_string);
    }

    #[derive(Debug, ToXml)]
    pub struct ObjectTypes {
        #[xml(rename = "children")]
        objecttype: Vec<ObjectType>,

        #[xml(rename = "attr", attribute)]
        attribute: Option<ObjectType>,
    }

    #[test]
    pub fn toxml_objecttype_test() {
        let xml_string = format!(
            r#"<ObjectTypes attr="model"><{s}>model</{s}><{s}>support</{s}><{s}>solidsupport</{s}><{s}>support</{s}><{s}>other</{s}></ObjectTypes>"#,
            s = "children"
        );
        let objecttypes = ObjectTypes {
            attribute: Some(ObjectType::Model),
            objecttype: vec![
                ObjectType::Model,
                ObjectType::Support,
                ObjectType::SolidSupport,
                ObjectType::Support,
                ObjectType::Other,
            ],
        };
        let objecttype_string = to_string(&objecttypes).unwrap();

        assert_eq!(objecttype_string, xml_string);
    }
}

#[cfg(feature = "memory-optimized-read")]
#[cfg(test)]
pub mod memory_read_test {
    use instant_xml::{FromXml, from_str};
    use pretty_assertions::assert_eq;

    use crate::{
        core::{
            Mesh, Triangles, Vertices,
            component::{Component, Components},
        },
        threemf_namespaces::{
            CORE_NS, CORE_TRIANGLESET_NS, CORE_TRIANGLESET_PREFIX, PROD_NS, PROD_PREFIX,
        },
    };

    use super::{Object, ObjectType};

    use std::vec;

    #[test]
    pub fn fromxml_simple_object_test() {
        let xml_string = format!(r#"<object xmlns="{}" id="4"></object>"#, CORE_NS);
        let object = from_str::<Object>(&xml_string).unwrap();

        assert_eq!(
            object,
            Object {
                id: 4,
                objecttype: None,
                thumbnail: None,
                partnumber: None,
                name: None,
                pid: None,
                pindex: None,
                uuid: None,
                mesh: None,
                components: None,
            }
        );
    }

    #[test]
    pub fn fromxml_production_object_test() {
        const CUSTOM_PROD_PREFIX: &str = "custom";
        let xml_string = format!(
            r#"<object xmlns="{}" xmlns:{}="{}" id="4" {}:UUID="someUUID"></object>"#,
            CORE_NS, CUSTOM_PROD_PREFIX, PROD_NS, CUSTOM_PROD_PREFIX,
        );
        let object = from_str::<Object>(&xml_string).unwrap();

        assert_eq!(
            object,
            Object {
                id: 4,
                objecttype: None,
                thumbnail: None,
                partnumber: None,
                name: None,
                pid: None,
                pindex: None,
                uuid: Some("someUUID".to_owned()),
                mesh: None,
                components: None,
            }
        );
    }

    #[test]
    pub fn fromxml_intermediate_object_test() {
        let xml_string = format!(
            r#"<object xmlns="{}" id="4" type="model" thumbnail="\thumbnail\part_thumbnail.png" partnumber="part_1" name="Object Part" pid="123" pindex="123"></object>"#,
            CORE_NS
        );
        let object = from_str::<Object>(&xml_string).unwrap();

        assert_eq!(
            object,
            Object {
                id: 4,
                objecttype: Some(ObjectType::Model),
                thumbnail: Some("\\thumbnail\\part_thumbnail.png".to_string()),
                partnumber: Some("part_1".to_string()),
                name: Some("Object Part".to_string()),
                pid: Some(123),
                pindex: Some(123),
                uuid: None,
                mesh: None,
                components: None,
            }
        );
    }

    #[test]
    pub fn fromxml_intermediate_object_test_x() {
        let xml_string = format!(
            r#"<object xmlns="{}" id="4" type="model" thumbnail="\thumbnail\part_thumbnail.png" partnumber="part_1" name="Object Part" pid="123" pindex="123"></object>"#,
            CORE_NS
        );
        let object = from_str::<Object>(&xml_string).unwrap();

        assert_eq!(
            object,
            Object {
                id: 4,
                objecttype: Some(ObjectType::Model),
                thumbnail: Some("\\thumbnail\\part_thumbnail.png".to_string()),
                partnumber: Some("part_1".to_string()),
                name: Some("Object Part".to_string()),
                pid: Some(123),
                pindex: Some(123),
                uuid: None,
                mesh: None,
                components: None,
            }
        );
    }

    #[test]
    pub fn fromxml_advanced_mesh_object_test() {
        let xml_string = format!(
            r##"<object xmlns="{}" xmlns:{}="{}" id="4" type="model" thumbnail="\thumbnail\part_thumbnail.png" partnumber="part_1" name="Object Part"><mesh xmlns:{}="{}"><vertices></vertices><triangles></triangles></mesh></object>"##,
            CORE_NS, PROD_PREFIX, PROD_NS, CORE_TRIANGLESET_PREFIX, CORE_TRIANGLESET_NS
        );
        let object = from_str::<Object>(&xml_string).unwrap();

        assert_eq!(
            object,
            Object {
                id: 4,
                objecttype: Some(ObjectType::Model),
                thumbnail: Some("\\thumbnail\\part_thumbnail.png".to_string()),
                partnumber: Some("part_1".to_string()),
                name: Some("Object Part".to_string()),
                pid: None,
                pindex: None,
                uuid: None,
                mesh: Some(Mesh {
                    vertices: Vertices { vertex: vec![] },
                    triangles: Triangles { triangle: vec![] },
                    trianglesets: None,
                }),
                components: None,
            }
        );
    }

    #[test]
    pub fn fromxml_advanced_component_object_test() {
        let xml_string = format!(
            r##"<object xmlns="{}" xmlns:{}="{}" id="4" type="model" thumbnail="\thumbnail\part_thumbnail.png" partnumber="part_1" name="Object Part"><components><component objectid="23" /></components></object>"##,
            CORE_NS, PROD_PREFIX, PROD_NS
        );
        let object = from_str::<Object>(&xml_string).unwrap();

        assert_eq!(
            object,
            Object {
                id: 4,
                objecttype: Some(ObjectType::Model),
                thumbnail: Some("\\thumbnail\\part_thumbnail.png".to_string()),
                partnumber: Some("part_1".to_string()),
                name: Some("Object Part".to_string()),
                pid: None,
                pindex: None,
                uuid: None,
                mesh: None,
                components: Some(Components {
                    component: vec![Component {
                        objectid: 23,
                        transform: None,
                        path: None,
                        uuid: None,
                    }],
                }),
            }
        );
    }

    #[derive(Debug, FromXml, PartialEq)]
    pub struct ObjectTypes {
        #[xml(rename = "children")]
        childs: Vec<ObjectType>,

        #[xml(rename = "attr", attribute)]
        attribute: Option<ObjectType>,
    }

    #[test]
    pub fn fromxml_objecttype_test() {
        let xml_string = format!(
            r#"<ObjectTypes attr="model"><{s}>model</{s}><{s}>support</{s}><{s}>solidsupport</{s}><{s}>support</{s}><{s}>other</{s}></ObjectTypes>"#,
            s = "children"
        );
        let objecttypes = from_str::<ObjectTypes>(&xml_string).unwrap();

        assert_eq!(
            objecttypes,
            ObjectTypes {
                attribute: Some(ObjectType::Model),
                childs: vec![
                    ObjectType::Model,
                    ObjectType::Support,
                    ObjectType::SolidSupport,
                    ObjectType::Support,
                    ObjectType::Other,
                ],
            }
        );
    }
}

#[cfg(feature = "speed-optimized-read")]
#[cfg(test)]
pub mod speed_read_test {
    use pretty_assertions::assert_eq;
    use serde::Deserialize;
    use serde_roxmltree::from_str;

    use crate::{
        core::{
            Mesh, Triangles, Vertices,
            component::{Component, Components},
        },
        threemf_namespaces::{
            CORE_NS, CORE_TRIANGLESET_NS, CORE_TRIANGLESET_PREFIX, PROD_NS, PROD_PREFIX,
        },
    };

    use super::{Object, ObjectType};

    use std::vec;

    #[test]
    pub fn fromxml_simple_object_test() {
        let xml_string = format!(r#"<object xmlns="{}" id="4"></object>"#, CORE_NS);
        let object = from_str::<Object>(&xml_string).unwrap();

        assert_eq!(
            object,
            Object {
                id: 4,
                objecttype: None,
                thumbnail: None,
                partnumber: None,
                name: None,
                pid: None,
                pindex: None,
                uuid: None,
                mesh: None,
                components: None,
            }
        );
    }

    #[test]
    pub fn fromxml_production_object_test() {
        const CUSTOM_PROD_PREFIX: &str = "custom";
        let xml_string = format!(
            r#"<object xmlns="{}" xmlns:{}="{}" id="4" {}:UUID="someUUID"></object>"#,
            CORE_NS, CUSTOM_PROD_PREFIX, PROD_NS, CUSTOM_PROD_PREFIX,
        );
        let object = from_str::<Object>(&xml_string).unwrap();

        assert_eq!(
            object,
            Object {
                id: 4,
                objecttype: None,
                thumbnail: None,
                partnumber: None,
                name: None,
                pid: None,
                pindex: None,
                uuid: Some("someUUID".to_owned()),
                mesh: None,
                components: None,
            }
        );
    }

    #[test]
    pub fn fromxml_intermediate_object_test() {
        let xml_string = format!(
            r#"<object xmlns="{}" id="4" type="model" thumbnail="\thumbnail\part_thumbnail.png" partnumber="part_1" name="Object Part" pid="123" pindex="123"></object>"#,
            CORE_NS
        );
        let object = from_str::<Object>(&xml_string).unwrap();

        assert_eq!(
            object,
            Object {
                id: 4,
                objecttype: Some(ObjectType::Model),
                thumbnail: Some("\\thumbnail\\part_thumbnail.png".to_string()),
                partnumber: Some("part_1".to_string()),
                name: Some("Object Part".to_string()),
                pid: Some(123),
                pindex: Some(123),
                uuid: None,
                mesh: None,
                components: None,
            }
        );
    }

    #[test]
    pub fn fromxml_intermediate_object_test_x() {
        let xml_string = format!(
            r#"<object xmlns="{}" id="4" type="model" thumbnail="\thumbnail\part_thumbnail.png" partnumber="part_1" name="Object Part" pid="123" pindex="123"></object>"#,
            CORE_NS
        );
        let object = from_str::<Object>(&xml_string).unwrap();

        assert_eq!(
            object,
            Object {
                id: 4,
                objecttype: Some(ObjectType::Model),
                thumbnail: Some("\\thumbnail\\part_thumbnail.png".to_string()),
                partnumber: Some("part_1".to_string()),
                name: Some("Object Part".to_string()),
                pid: Some(123),
                pindex: Some(123),
                uuid: None,
                mesh: None,
                components: None,
            }
        );
    }

    #[test]
    pub fn fromxml_advanced_mesh_object_test() {
        let xml_string = format!(
            r##"<object xmlns="{}" xmlns:{}="{}" id="4" type="model" thumbnail="\thumbnail\part_thumbnail.png" partnumber="part_1" name="Object Part"><mesh xmlns:{}="{}"><vertices></vertices><triangles></triangles></mesh></object>"##,
            CORE_NS, PROD_PREFIX, PROD_NS, CORE_TRIANGLESET_PREFIX, CORE_TRIANGLESET_NS
        );
        let object = from_str::<Object>(&xml_string).unwrap();

        assert_eq!(
            object,
            Object {
                id: 4,
                objecttype: Some(ObjectType::Model),
                thumbnail: Some("\\thumbnail\\part_thumbnail.png".to_string()),
                partnumber: Some("part_1".to_string()),
                name: Some("Object Part".to_string()),
                pid: None,
                pindex: None,
                uuid: None,
                mesh: Some(Mesh {
                    vertices: Vertices { vertex: vec![] },
                    triangles: Triangles { triangle: vec![] },
                    trianglesets: None,
                }),
                components: None,
            }
        );
    }

    #[test]
    pub fn fromxml_advanced_component_object_test() {
        let xml_string = format!(
            r##"<object xmlns="{}" xmlns:{}="{}" id="4" type="model" thumbnail="\thumbnail\part_thumbnail.png" partnumber="part_1" name="Object Part"><components><component objectid="23" /></components></object>"##,
            CORE_NS, PROD_PREFIX, PROD_NS
        );
        let object = from_str::<Object>(&xml_string).unwrap();

        assert_eq!(
            object,
            Object {
                id: 4,
                objecttype: Some(ObjectType::Model),
                thumbnail: Some("\\thumbnail\\part_thumbnail.png".to_string()),
                partnumber: Some("part_1".to_string()),
                name: Some("Object Part".to_string()),
                pid: None,
                pindex: None,
                uuid: None,
                mesh: None,
                components: Some(Components {
                    component: vec![Component {
                        objectid: 23,
                        transform: None,
                        path: None,
                        uuid: None,
                    }],
                }),
            }
        );
    }

    #[derive(Debug, Deserialize, PartialEq)]
    pub struct ObjectTypes {
        #[serde(rename = "children")]
        childs: Vec<ObjectType>,

        #[serde(rename = "attr")]
        attribute: Option<ObjectType>,
    }

    #[test]
    pub fn fromxml_objecttype_test() {
        let xml_string = format!(
            r#"<ObjectTypes attr="model"><{s}>model</{s}><{s}>support</{s}><{s}>solidsupport</{s}><{s}>support</{s}><{s}>other</{s}><{s}>somethingelse</{s}></ObjectTypes>"#,
            s = "children"
        );
        let objecttypes = from_str::<ObjectTypes>(&xml_string).unwrap();

        assert_eq!(
            objecttypes,
            ObjectTypes {
                attribute: Some(ObjectType::Model),
                childs: vec![
                    ObjectType::Model,
                    ObjectType::Support,
                    ObjectType::SolidSupport,
                    ObjectType::Support,
                    ObjectType::Other,
                    ObjectType::Model,
                ],
            }
        );
    }
}
