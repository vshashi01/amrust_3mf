#[cfg(feature = "write")]
use instant_xml::ToXml;

#[cfg(feature = "memory-optimized-read")]
use instant_xml::FromXml;

#[cfg(feature = "speed-optimized-read")]
use serde::Deserialize;

use crate::core::transform::Transform;
use crate::threemf_namespaces::BOOLEAN_NS;

#[cfg_attr(feature = "speed-optimized-read", derive(Deserialize))]
#[cfg_attr(feature = "speed-optimized-read", serde(rename = "booleanshape"))]
#[cfg_attr(feature = "memory-optimized-read", derive(FromXml))]
#[cfg_attr(feature = "write", derive(ToXml))]
#[derive(Debug, PartialEq, Clone)]
#[cfg_attr(
    any(feature = "write", feature = "memory-optimized-read"),
    xml(ns(BOOLEAN_NS), rename = "booleanshape")
)]
pub struct BooleanShape {
    #[cfg_attr(
        any(feature = "write", feature = "memory-optimized-read"),
        xml(attribute)
    )]
    pub objectid: usize,

    #[cfg_attr(
        any(feature = "write", feature = "memory-optimized-read"),
        xml(attribute)
    )]
    pub operation: Option<BooleanOperation>,

    #[cfg_attr(
        any(feature = "write", feature = "memory-optimized-read"),
        xml(attribute)
    )]
    pub transform: Option<Transform>,

    #[cfg_attr(
        any(feature = "write", feature = "memory-optimized-read"),
        xml(attribute)
    )]
    pub path: Option<String>,

    pub boolean: Vec<Boolean>,
}

#[cfg_attr(feature = "speed-optimized-read", derive(Deserialize))]
#[cfg_attr(feature = "speed-optimized-read", serde(from = "String"))]
#[cfg_attr(feature = "memory-optimized-read", derive(FromXml))]
#[cfg_attr(feature = "write", derive(ToXml))]
#[derive(Default, Debug, PartialEq, Eq, Clone)]
#[cfg_attr(
    any(feature = "write", feature = "memory-optimized-read"),
    xml(scalar, ns(BOOLEAN_NS), rename_all = "lowercase")
)]
pub enum BooleanOperation {
    #[default]
    Union,
    Difference,
    Intersection,
}

impl From<String> for BooleanOperation {
    fn from(value: String) -> Self {
        match value.to_ascii_lowercase().as_str() {
            "union" => Self::Union,
            "difference" => Self::Difference,
            "intersection" => Self::Intersection,
            _ => Self::Union,
        }
    }
}

#[cfg_attr(feature = "speed-optimized-read", derive(Deserialize))]
#[cfg_attr(feature = "speed-optimized-read", serde(rename = "boolean"))]
#[cfg_attr(feature = "memory-optimized-read", derive(FromXml))]
#[cfg_attr(feature = "write", derive(ToXml))]
#[derive(Debug, PartialEq, Clone)]
#[cfg_attr(
    any(feature = "write", feature = "memory-optimized-read"),
    xml(ns(BOOLEAN_NS), rename = "boolean")
)]
pub struct Boolean {
    #[cfg_attr(
        any(feature = "write", feature = "memory-optimized-read"),
        xml(attribute)
    )]
    pub objectid: usize,

    #[cfg_attr(
        any(feature = "write", feature = "memory-optimized-read"),
        xml(attribute)
    )]
    pub transform: Option<Transform>,

    #[cfg_attr(
        any(feature = "write", feature = "memory-optimized-read"),
        xml(attribute)
    )]
    pub path: Option<String>,
}

#[cfg(test)]
mod write_tests {
    use instant_xml::to_string;
    use pretty_assertions::assert_eq;

    use super::*;

    use crate::threemf_namespaces::BOOLEAN_NS;

    #[test]
    fn toxml_boolean_shape_full_test() {
        let xml_string = format!(
            r#"<booleanshape xmlns="{}" objectid="3" operation="difference" transform="1.000000 0.000000 0.000000 0.000000 0.000000 1.000000 0.000000 0.000000 0.000000 0.000000 1.000000 0.000000" path="someObjectPath"><boolean objectid="2" /><boolean objectid="1" path="someBooleanPath" /></booleanshape>"#,
            BOOLEAN_NS
        );

        let boolean = BooleanShape {
            objectid: 3,
            operation: Some(BooleanOperation::Difference),
            transform: Some(Transform([
                1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0,
            ])),
            path: Some("someObjectPath".to_owned()),
            boolean: vec![
                Boolean {
                    objectid: 2,
                    transform: None,
                    path: None,
                },
                Boolean {
                    objectid: 1,
                    transform: None,
                    path: Some("someBooleanPath".to_owned()),
                },
            ],
        };

        let boolean_string = to_string(&boolean).unwrap();
        assert_eq!(boolean_string, xml_string)
    }

    #[test]
    fn toxml_boolean_shape_test() {
        let xml_string = format!(
            r#"<booleanshape xmlns="{}" objectid="3"><boolean objectid="2" /><boolean objectid="1" /></booleanshape>"#,
            BOOLEAN_NS
        );

        let boolean = BooleanShape {
            objectid: 3,
            operation: None,
            transform: None,
            path: None,
            boolean: vec![
                Boolean {
                    objectid: 2,
                    transform: None,
                    path: None,
                },
                Boolean {
                    objectid: 1,
                    transform: None,
                    path: None,
                },
            ],
        };

        let boolean_string = to_string(&boolean).unwrap();
        assert_eq!(boolean_string, xml_string)
    }

    #[test]
    fn toxml_boolean_full_test() {
        let xml_string = format!(
            r#"<boolean xmlns="{}" objectid="1" transform="1.000000 0.000000 0.000000 0.000000 0.000000 1.000000 0.000000 0.000000 0.000000 0.000000 1.000000 0.000000" path="someBooleanPath" />"#,
            BOOLEAN_NS
        );

        let boolean = Boolean {
            objectid: 1,
            transform: Some(Transform([
                1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0,
            ])),
            path: Some("someBooleanPath".to_owned()),
        };

        let boolean_string = to_string(&boolean).unwrap();
        assert_eq!(boolean_string, xml_string)
    }

    #[test]
    fn toxml_boolean_test() {
        let xml_string = format!(r#"<boolean xmlns="{}" objectid="1" />"#, BOOLEAN_NS);

        let boolean = Boolean {
            objectid: 1,
            transform: None,
            path: None,
        };

        let boolean_string = to_string(&boolean).unwrap();
        assert_eq!(boolean_string, xml_string)
    }

    #[derive(Debug, ToXml, PartialEq, Eq)]
    #[xml(ns(bo = BOOLEAN_NS))]
    struct EnumBooleanOperation {
        operation: Vec<BooleanOperation>,
    }

    #[test]
    pub fn toxml_enums_test() {
        let xml_string = format!(
            r#"<EnumBooleanOperation xmlns:bo="{BOOLEAN_NS}"><bo:operation>union</bo:operation><bo:operation>intersection</bo:operation><bo:operation>difference</bo:operation></EnumBooleanOperation>"#
        );
        let enum_test = EnumBooleanOperation {
            operation: vec![
                BooleanOperation::Union,
                BooleanOperation::Intersection,
                BooleanOperation::Difference,
            ],
        };
        let enum_test_string = to_string(&enum_test).unwrap();

        assert_eq!(enum_test_string, xml_string);
    }
}

#[cfg(feature = "memory-optimized-read")]
#[cfg(test)]
mod memory_optimized_read_tests {
    use instant_xml::from_str;
    use pretty_assertions::assert_eq;

    use super::*;

    use crate::threemf_namespaces::BOOLEAN_NS;

    #[test]
    fn fromxml_boolean_full_test() {
        let xml_string = format!(
            r#"<boolean xmlns="{}" objectid="1" transform="1 0 0 0 0 1 0 0 0 0 1 0" path="someBooleanPath" />"#,
            BOOLEAN_NS
        );

        let boolean = from_str::<Boolean>(&xml_string).unwrap();
        assert_eq!(
            boolean,
            Boolean {
                objectid: 1,
                transform: Some(Transform([
                    1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0,
                ])),
                path: Some("someBooleanPath".to_owned()),
            }
        )
    }

    #[test]
    fn fromxml_boolean_test() {
        let xml_string = format!(r#"<boolean xmlns="{}" objectid="1" />"#, BOOLEAN_NS);

        let boolean = from_str::<Boolean>(&xml_string).unwrap();
        assert_eq!(
            boolean,
            Boolean {
                objectid: 1,
                transform: None,
                path: None,
            }
        );
    }

    #[derive(Debug, FromXml, PartialEq, Eq)]
    #[xml(ns(bo = BOOLEAN_NS))]
    struct EnumBooleanOperation {
        operation: Vec<BooleanOperation>,
    }

    #[test]
    pub fn fromxml_enums_test() {
        let xml_string = format!(
            r#"<EnumBooleanOperation xmlns:bo="{BOOLEAN_NS}"><bo:operation>union</bo:operation><bo:operation>intersection</bo:operation><bo:operation>difference</bo:operation></EnumBooleanOperation>"#
        );
        let enum_result = from_str::<EnumBooleanOperation>(&xml_string).unwrap();

        assert_eq!(
            enum_result,
            EnumBooleanOperation {
                operation: vec![
                    BooleanOperation::Union,
                    BooleanOperation::Intersection,
                    BooleanOperation::Difference,
                ],
            }
        );
    }
}
