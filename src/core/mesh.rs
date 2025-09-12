use std::collections::HashMap;

use instant_xml::*;
use regex::Regex;

use crate::core::triangle_set::TriangleSets;
use crate::threemf_namespaces::{CORE_NS, CORE_TRIANGLESET_NS};

/// A triangle mesh
///
/// This is a very basic types that lacks any amenities for constructing it or
/// for iterating over its data.
///
/// This is by design. Providing a generally usable and feature-rich triangle
/// mesh type is out of scope for this library. It is expected that users of
/// this library will use their own mesh type anyway, and the simplicity of
/// `TriangleMesh` provides an easy target for conversion from such a type.
#[derive(FromXml, ToXml, PartialEq, Clone, Debug)]
#[xml(ns(CORE_NS, t = CORE_TRIANGLESET_NS), rename = "mesh")]
pub struct Mesh {
    /// The vertices of the mesh
    ///
    /// This defines the vertices that are part of the mesh, but not the mesh's
    /// structure. See the `triangles` field.
    pub vertices: Vertices,

    /// The triangles that make up the mesh
    ///
    /// Each triangle consists of indices that refer back to the `vertices`
    /// field.
    pub triangles: Triangles,

    #[xml(ns(CORE_TRIANGLESET_NS))]
    pub trianglesets: Option<TriangleSets>,
}

/// A list of vertices, as a struct mainly to comply with easier serde xml
#[derive(PartialEq, Clone, Debug)]
pub struct Vertices {
    pub vertex: Vec<Vertex>,
}

impl<'xml> FromXml<'xml> for Vertices {
    fn matches(id: Id<'_>, _field: Option<Id<'_>>) -> bool {
        id.name == "vertices" && id.ns == CORE_NS
    }

    fn deserialize<'cx>(
        into: &mut Self::Accumulator,
        field: &'static str,
        deserializer: &mut Deserializer<'cx, 'xml>,
    ) -> Result<(), Error> {
        if into.is_some() {
            return Err(Error::DuplicateValue(field));
        }

        let mut vertices: Vec<Vertex> = Vec::with_capacity(30000);

        while let Some(node) = deserializer.next() {
            if let Ok(n) = node
                && let de::Node::Open(element) = n
            {
                //println!("This is element value {:?}", element);
                let mut vertex_value: Option<Vertex> = None;
                let mut nested = deserializer.nested(element);
                if Vertex::deserialize(&mut vertex_value, "vertex", &mut nested).is_ok()
                    && let Some(vertex) = vertex_value
                {
                    vertices.push(vertex);
                };
            }
        }

        vertices.shrink_to_fit();
        *into = Some(Vertices { vertex: vertices });

        Ok(())
    }

    type Accumulator = Option<Self>;

    const KIND: Kind = Kind::Scalar;
}

impl ToXml for Vertices {
    fn serialize<W: std::fmt::Write + ?Sized>(
        &self,
        field: Option<Id<'_>>,
        serializer: &mut Serializer<W>,
    ) -> Result<(), Error> {
        serializer.write_start("vertices", CORE_NS)?;
        serializer.end_start()?;

        self.vertex.iter().for_each(|v| {
            let _ = serializer.write_str(&format!(
                "<vertex x=\"{}\" y=\"{}\" z=\"{}\" />",
                v.x, v.y, v.z
            ));
        });
        serializer.write_close(None, "vertices")?;

        Ok(())
    }
}

/// A vertex in a triangle mesh
#[derive(FromXml, ToXml, PartialEq, Clone, Debug)]
#[xml(ns(CORE_NS), rename = "vertex")]
pub struct Vertex {
    #[xml(attribute)]
    pub x: f64,

    #[xml(attribute)]
    pub y: f64,

    #[xml(attribute)]
    pub z: f64,
}

// impl<'xml> FromXml<'xml> for Vertex {
//     fn matches(id: Id<'_>, field: Option<Id<'_>>) -> bool {
//         match field {
//             Some(field) => field == id,
//             None => id.ns == CORE_NS && id.name == "vertex",
//         }
//         // true
//     }

//     fn deserialize<'cx>(
//         into: &mut Self::Accumulator,
//         field: &'static str,
//         deserializer: &mut Deserializer<'cx, 'xml>,
//     ) -> Result<(), Error> {
//         if into.is_some() {
//             return Err(Error::DuplicateValue("Vertex already exists"));
//         }

//         let value = match deserializer.take_str()? {
//             Some(value) => value,
//             None => return Err(Error::MissingValue("Missing vertex attributes")),
//         };

//         println!("The value is {value:?}");

//         Ok(())
//     }

//     type Accumulator = Option<Self>;

//     const KIND: Kind = Kind::Scalar;
// }

/// A list of triangles, as a struct mainly to comply with easier serde xml
#[derive(FromXml, ToXml, PartialEq, Clone, Debug)]
#[xml(ns(CORE_NS), rename = "triangles")]
pub struct Triangles {
    pub triangle: Vec<Triangle>,
}

// impl<'xml> FromXml<'xml> for Triangles {
//     fn matches(id: Id<'_>, _field: Option<Id<'_>>) -> bool {
//         id.name == "triangles" && id.ns == CORE_NS
//     }

//     fn deserialize<'cx>(
//         into: &mut Self::Accumulator,
//         field: &'static str,
//         deserializer: &mut Deserializer<'cx, 'xml>,
//     ) -> Result<(), Error> {
//         if into.is_some() {
//             return Err(Error::DuplicateValue(field));
//         }

//         let mut triangles: Vec<Triangle> = Vec::with_capacity(10000);
//         while let Some(node) = deserializer.next() {
//             if let Ok(n) = node
//                 && let de::Node::Open(element) = n
//             {
//                 //println!("This is element value {:?}", element);
//                 let mut triangle_value: Option<Triangle> = None;
//                 let mut nested = deserializer.nested(element);
//                 if Triangle::deserialize(&mut triangle_value, field, &mut nested).is_ok()
//                     && let Some(vertex) = triangle_value
//                 {
//                     triangles.push(vertex);
//                 }
//             }
//         }

//         triangles.shrink_to_fit();
//         *into = Some(Triangles {
//             triangle: triangles,
//         });

//         Ok(())
//     }

//     type Accumulator = Option<Self>;

//     const KIND: Kind = Kind::Element;
// }

// impl ToXml for Triangles {
//     fn serialize<W: std::fmt::Write + ?Sized>(
//         &self,
//         field: Option<Id<'_>>,
//         serializer: &mut Serializer<W>,
//     ) -> Result<(), Error> {
//         serializer.write_start("triangles", CORE_NS)?;
//         serializer.end_start()?;

//         self.triangle.iter().for_each(|v| {
//             //let _ = v.serialize(Some(Id { ns: CORE_NS, name: "triangle" }), serializer);
//             //let _ = v.serialize(None, serializer);
//             let _ = serializer.write_str(&format!(
//                 "<triangle v1=\"{}\" v2=\"{}\" v3=\"{}\" />",
//                 v.v1, v.v2, v.v3
//             ));
//         });
//         serializer.write_close(None, "triangles")?;

//         Ok(())
//     }
// }

/// A triangle in a triangle mesh
///
/// The triangle consists of indices that refer to the vertices of the mesh. See
/// [`TriangleMesh`].
#[derive(FromXml, ToXml, PartialEq, Clone, Debug)]
#[xml(ns(CORE_NS), rename = "triangle")]
pub struct Triangle {
    #[xml(attribute)]
    pub v1: usize,

    #[xml(attribute)]
    pub v2: usize,

    #[xml(attribute)]
    pub v3: usize,

    #[xml(attribute)]
    pub p1: Option<usize>,

    #[xml(attribute)]
    pub p2: Option<usize>,

    #[xml(attribute)]
    pub p3: Option<usize>,

    #[xml(attribute)]
    pub pid: Option<usize>,
}

#[cfg(test)]
pub mod tests {
    use instant_xml::{from_str, to_string};
    use pretty_assertions::assert_eq;

    use crate::threemf_namespaces::{CORE_NS, CORE_TRIANGLESET_NS, CORE_TRIANGLESET_PREFIX};

    use super::{Mesh, Triangle, Triangles, Vertex, Vertices};

    #[test]
    pub fn toxml_vertex_test() {
        let xml_string = format!(r#"<vertex xmlns="{}" x="100.5" y="100" z="0" />"#, CORE_NS);
        let vertex = Vertex {
            x: 100.5,
            y: 100.0,
            z: 0.0,
        };
        let vertex_string = to_string(&vertex).unwrap();

        assert_eq!(vertex_string, xml_string);
    }

    #[test]
    pub fn fromxml_vertex_test() {
        let xml_string = format!(r#"<vertex xmlns="{}" x="100.5" y="100" z="0" />"#, CORE_NS);
        let vertex = from_str::<Vertex>(&xml_string).unwrap();

        assert_eq!(
            vertex,
            Vertex {
                x: 100.5,
                y: 100.0,
                z: 0.0,
            }
        );
    }

    #[test]
    pub fn toxml_vertices_test() {
        let xml_string = format!(
            r#"<vertices xmlns="{}"><vertex x="100" y="110.5" z="0" /><vertex x="0.156" y="55.6896" z="-10" /></vertices>"#,
            CORE_NS
        );
        let vertices = Vertices {
            vertex: vec![
                Vertex {
                    x: 100.,
                    y: 110.5,
                    z: 0.0,
                },
                Vertex {
                    x: 0.156,
                    y: 55.6896,
                    z: -10.0,
                },
            ],
        };
        let vertices_string = to_string(&vertices).unwrap();

        assert_eq!(vertices_string, xml_string)
    }

    #[test]
    pub fn fromxml_vertices_test() {
        let xml_string = format!(
            r#"<vertices xmlns="{}"><vertex x="100" y="110.5" z="0" /><vertex x="0.156" y="55.6896" z="-10" /></vertices>"#,
            CORE_NS
        );
        let vertices = from_str::<Vertices>(&xml_string).unwrap();

        assert_eq!(
            vertices,
            Vertices {
                vertex: vec![
                    Vertex {
                        x: 100.,
                        y: 110.5,
                        z: 0.0,
                    },
                    Vertex {
                        x: 0.156,
                        y: 55.6896,
                        z: -10.0,
                    },
                ],
            }
        )
    }

    #[test]
    pub fn toxml_required_fields_triangle_test() {
        let xml_string = format!(r#"<triangle xmlns="{}" v1="1" v2="2" v3="3" />"#, CORE_NS);
        let triangle = Triangle {
            v1: 1,
            v2: 2,
            v3: 3,
            p1: None,
            p2: None,
            p3: None,
            pid: None,
        };
        let triangle_string = to_string(&triangle).unwrap();

        assert_eq!(triangle_string, xml_string);
    }

    #[test]
    pub fn fromxml_required_fields_triangle_test() {
        let xml_string = format!(r#"<triangle xmlns="{}" v1="1" v2="2" v3="3" />"#, CORE_NS);
        let triangle = from_str::<Triangle>(&xml_string).unwrap();

        assert_eq!(
            triangle,
            Triangle {
                v1: 1,
                v2: 2,
                v3: 3,
                p1: None,
                p2: None,
                p3: None,
                pid: None,
            }
        );
    }

    #[test]
    pub fn toxml_triangles_test() {
        let xml_string = format!(
            r#"<triangles xmlns="{}"><triangle v1="1" v2="2" v3="3" /><triangle v1="2" v2="3" v3="4" /></triangles>"#,
            CORE_NS
        );
        let triangles = Triangles {
            triangle: vec![
                Triangle {
                    v1: 1,
                    v2: 2,
                    v3: 3,
                    p1: None,
                    p2: None,
                    p3: None,
                    pid: None,
                },
                Triangle {
                    v1: 2,
                    v2: 3,
                    v3: 4,
                    p1: None,
                    p2: None,
                    p3: None,
                    pid: None,
                },
            ],
        };
        let triangles_string = to_string(&triangles).unwrap();

        assert_eq!(triangles_string, xml_string);
    }

    #[test]
    pub fn fromxml_triangles_test() {
        let xml_string = format!(
            r#"<triangles xmlns="{}"><triangle v1="1" v2="2" v3="3" /><triangle v1="2" v2="3" v3="4" /></triangles>"#,
            CORE_NS
        );
        let triangles = from_str::<Triangles>(&xml_string).unwrap();

        assert_eq!(
            triangles,
            Triangles {
                triangle: vec![
                    Triangle {
                        v1: 1,
                        v2: 2,
                        v3: 3,
                        p1: None,
                        p2: None,
                        p3: None,
                        pid: None,
                    },
                    Triangle {
                        v1: 2,
                        v2: 3,
                        v3: 4,
                        p1: None,
                        p2: None,
                        p3: None,
                        pid: None,
                    },
                ],
            }
        );
    }

    #[test]
    pub fn toxml_mesh_test() {
        let xml_string = format!(
            r##"<mesh xmlns="{}" xmlns:{}="{}"><vertices><vertex x="-1" y="-1" z="0" /><vertex x="1" y="-1" z="0" /><vertex x="1" y="1" z="0" /><vertex x="-1" y="1" z="0" /></vertices><triangles><triangle v1="0" v2="1" v3="2" /><triangle v1="0" v2="2" v3="3" /></triangles></mesh>"##,
            CORE_NS, CORE_TRIANGLESET_PREFIX, CORE_TRIANGLESET_NS
        );
        let mesh = Mesh {
            vertices: Vertices {
                vertex: vec![
                    Vertex {
                        x: -1.0,
                        y: -1.0,
                        z: 0.0,
                    },
                    Vertex {
                        x: 1.0,
                        y: -1.0,
                        z: 0.0,
                    },
                    Vertex {
                        x: 1.0,
                        y: 1.0,
                        z: 0.0,
                    },
                    Vertex {
                        x: -1.0,
                        y: 1.0,
                        z: 0.0,
                    },
                ],
            },
            triangles: Triangles {
                triangle: vec![
                    Triangle {
                        v1: 0,
                        v2: 1,
                        v3: 2,
                        p1: None,
                        p2: None,
                        p3: None,
                        pid: None,
                    },
                    Triangle {
                        v1: 0,
                        v2: 2,
                        v3: 3,
                        p1: None,
                        p2: None,
                        p3: None,
                        pid: None,
                    },
                ],
            },
            trianglesets: None,
        };
        let mesh_string = to_string(&mesh).unwrap();

        assert_eq!(mesh_string, xml_string);
    }

    #[test]
    pub fn fromxml_mesh_test() {
        let xml_string = format!(
            r##"<mesh xmlns="{}"><vertices><vertex x="-1" y="-1" z="0" /><vertex x="1" y="-1" z="0" /><vertex x="1" y="1" z="0" /><vertex x="-1" y="1" z="0" /></vertices><triangles><triangle v1="0" v2="1" v3="2" /><triangle v1="0" v2="2" v3="3" /></triangles></mesh>"##,
            CORE_NS
        );
        let mesh = from_str::<Mesh>(&xml_string).unwrap();

        assert_eq!(
            mesh,
            Mesh {
                vertices: Vertices {
                    vertex: vec![
                        Vertex {
                            x: -1.0,
                            y: -1.0,
                            z: 0.0
                        },
                        Vertex {
                            x: 1.0,
                            y: -1.0,
                            z: 0.0
                        },
                        Vertex {
                            x: 1.0,
                            y: 1.0,
                            z: 0.0
                        },
                        Vertex {
                            x: -1.0,
                            y: 1.0,
                            z: 0.0
                        }
                    ]
                },
                triangles: Triangles {
                    triangle: vec![
                        Triangle {
                            v1: 0,
                            v2: 1,
                            v3: 2,
                            p1: None,
                            p2: None,
                            p3: None,
                            pid: None,
                        },
                        Triangle {
                            v1: 0,
                            v2: 2,
                            v3: 3,
                            p1: None,
                            p2: None,
                            p3: None,
                            pid: None,
                        }
                    ]
                },
                trianglesets: None,
            }
        )
    }
}
