use thiserror::Error;

use crate::{
    core::{
        build::{Build, Item},
        component::Components,
        mesh::{Mesh, Triangle, Triangles, Vertex, Vertices},
        metadata::Metadata,
        model::{Model, Unit},
        object::{Object, ObjectType},
        resources::Resources,
        transform::Transform,
    },
    io::XmlNamespace,
};

#[derive(Debug, Error)]
pub enum BuilderError {
    #[error("Object Id is not set for the Object Builder")]
    ObjectIdNotSet,
}

/// Builder for constructing 3MF Model structs with a fluent API
pub struct ModelBuilder {
    unit: Option<Unit>,
    requiredextensions: Vec<XmlNamespace>,
    recommendedextensions: Vec<XmlNamespace>,
    metadata: Vec<Metadata>,
    resources: ResourcesBuilder,
    build: BuildBuilder,
    next_object_id: ObjectId,
}

impl ModelBuilder {
    /// Create a new ModelBuilder with default values
    pub fn new() -> Self {
        Self {
            unit: Some(Unit::Millimeter),
            requiredextensions: vec![],
            recommendedextensions: vec![],
            metadata: Vec::new(),
            resources: ResourcesBuilder::new(),
            build: BuildBuilder::new(),
            next_object_id: 1.into(),
        }
    }

    /// Set the unit for the model
    pub fn unit(&mut self, unit: Unit) -> &mut Self {
        self.unit = Some(unit);
        self
    }

    /// Add a required extension
    pub fn add_required_extension(&mut self, extension: &XmlNamespace) -> &mut Self {
        self.requiredextensions.push(extension.clone());
        self
    }

    /// Set recommended extensions
    pub fn add_recommended_extension(&mut self, extension: &XmlNamespace) -> &mut Self {
        self.recommendedextensions.push(extension.clone());
        self
    }

    /// Add metadata to the model
    pub fn add_metadata(&mut self, name: &str, value: Option<&str>) -> &mut Self {
        self.metadata.push(Metadata {
            name: name.to_string(),
            preserve: None,
            value: value.map(|v| v.to_string()),
        });
        self
    }

    /// Add an object using a builder function, returns the assigned ObjectId
    pub fn add_object<F>(&mut self, f: F) -> Result<ObjectId, BuilderError>
    where
        F: FnOnce(&mut ObjectBuilder),
    {
        let id = self.next_object_id;
        self.next_object_id = ObjectId(id.0 + 1);

        let mut obj_builder = ObjectBuilder::new(id);
        f(&mut obj_builder);
        let object = obj_builder.build();

        match object {
            Ok(o) => self.resources.objects.push(o),
            Err(err) => return Err(err),
        }

        Ok(id)
    }

    /// Add a build item referencing an object by ID
    pub fn add_build_item(&mut self, object_id: ObjectId) -> &mut Self {
        self.build.items.push(BuildItem {
            objectid: object_id,
            transform: None,
            partnumber: None,
            path: None,
            uuid: None,
        });
        self
    }

    pub fn add_build_item_advanced(
        &mut self,
        object_id: ObjectId,
        transform: Option<Transform>,
        partnumber: Option<String>,
    ) {
        self.build.items.push(BuildItem {
            objectid: object_id,
            transform,
            partnumber,
            path: None,
            uuid: None,
        });
    }

    /// Build the final Model
    pub fn build(self) -> Model {
        let requiredextensions = get_extensions_definition(&self.requiredextensions);
        let recommendedextensions = get_extensions_definition(&self.recommendedextensions);
        Model {
            unit: self.unit,
            requiredextensions,
            recommendedextensions,
            metadata: self.metadata,
            resources: self.resources.build(),
            build: self.build.build(),
        }
    }
}

impl Default for ModelBuilder {
    fn default() -> Self {
        Self::new()
    }
}

fn get_extensions_definition(extensions: &[XmlNamespace]) -> Option<String> {
    if extensions.is_empty() {
        None
    } else {
        let mut extension_string = String::new();
        extensions.iter().for_each(|ns| {
            if let Some(prefix) = &ns.prefix {
                extension_string.push_str(prefix);
                extension_string.push(' ');
            }
        });

        Some(extension_string)
    }
}

/// Builder for Resources
pub struct ResourcesBuilder {
    objects: Vec<Object>,
}

impl ResourcesBuilder {
    fn new() -> Self {
        Self {
            objects: Vec::new(),
        }
    }

    fn build(self) -> Resources {
        Resources {
            object: self.objects,
            basematerials: Vec::new(), // TODO: Add base materials support
        }
    }
}

/// Builder for Build section
pub struct BuildBuilder {
    items: Vec<BuildItem>,
}

impl BuildBuilder {
    fn new() -> Self {
        Self { items: Vec::new() }
    }

    fn build(self) -> Build {
        Build {
            uuid: None, // TODO: Add UUID support
            item: self
                .items
                .into_iter()
                .map(|bi| Item {
                    objectid: bi.objectid.0,
                    transform: bi.transform,
                    partnumber: bi.partnumber,
                    path: bi.path,
                    uuid: bi.uuid,
                })
                .collect(),
        }
    }
}

/// Internal representation of build items during building
struct BuildItem {
    objectid: ObjectId,
    transform: Option<Transform>,
    partnumber: Option<String>,
    path: Option<String>,
    uuid: Option<String>,
}

/// Type-safe wrapper for object IDs to prevent mix-ups
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ObjectId(usize);

impl From<usize> for ObjectId {
    fn from(id: usize) -> Self {
        ObjectId(id)
    }
}

impl From<ObjectId> for usize {
    fn from(id: ObjectId) -> usize {
        id.0
    }
}

/// Builder for Object
pub struct ObjectBuilder {
    object_id: Option<ObjectId>,
    objecttype: Option<ObjectType>,
    thumbnail: Option<String>,
    partnumber: Option<String>,
    name: Option<String>,
    pid: Option<usize>,
    pindex: Option<usize>,
    uuid: Option<String>,
    mesh: Option<Mesh>,
    components: Option<Components>,
}

impl ObjectBuilder {
    fn new(object_id: ObjectId) -> Self {
        Self {
            object_id: Some(object_id),
            objecttype: Some(ObjectType::Model),
            thumbnail: None,
            partnumber: None,
            name: None,
            pid: None,
            pindex: None,
            uuid: None,
            mesh: None,
            components: None,
        }
    }

    pub fn object_id(&mut self, object_id: ObjectId) -> &mut Self {
        let _ = self.object_id.insert(object_id);
        self
    }

    /// Set the object type
    pub fn object_type(&mut self, object_type: ObjectType) -> &mut Self {
        self.objecttype = Some(object_type);
        self
    }

    /// Set the object name
    pub fn name(&mut self, name: &str) -> &mut Self {
        self.name = Some(name.to_string());
        self
    }

    /// Set the part number
    pub fn part_number(&mut self, part_number: &str) -> &mut Self {
        self.partnumber = Some(part_number.to_string());
        self
    }

    /// Add a mesh using a builder function
    pub fn mesh<F>(&mut self, f: F) -> &mut Self
    where
        F: FnOnce(&mut MeshBuilder),
    {
        let mut mesh_builder = MeshBuilder::new();
        f(&mut mesh_builder);
        self.mesh = Some(mesh_builder.build());
        self
    }

    fn build(self) -> Result<Object, BuilderError> {
        if let Some(object_id) = self.object_id {
            Ok(Object {
                id: object_id.0,
                objecttype: self.objecttype,
                thumbnail: self.thumbnail,
                partnumber: self.partnumber,
                name: self.name,
                pid: self.pid,
                pindex: self.pindex,
                uuid: self.uuid,
                mesh: self.mesh,
                components: self.components,
            })
        } else {
            Err(BuilderError::ObjectIdNotSet)
        }
    }
}

/// Builder for Mesh
pub struct MeshBuilder {
    vertices: Vec<Vertex>,
    triangles: Vec<crate::core::mesh::Triangle>,
}

impl MeshBuilder {
    fn new() -> Self {
        Self {
            vertices: Vec::new(),
            triangles: Vec::new(),
        }
    }

    /// Add a vertex
    pub fn add_vertex(&mut self, coords: &[f64; 3]) -> &mut Self {
        self.vertices.push(Vertex {
            x: coords[0],
            y: coords[1],
            z: coords[2],
        });
        self
    }

    /// Add a collection of vertices in [[f64;3]] slice
    pub fn add_vertices(&mut self, vertices: &[[f64; 3]]) -> &mut Self {
        for vertex in vertices {
            self.add_vertex(vertex);
        }

        self
    }

    /// Add a collection of vertices in a flattened 1D slice
    pub fn add_vertices_flat(&mut self, vertices: &[f64]) -> &mut Self {
        for vertex in vertices.chunks_exact(3) {
            self.vertices.push(Vertex {
                x: vertex[0],
                y: vertex[1],
                z: vertex[2],
            });
        }

        self
    }

    /// Add a triangle from [usize;3] slice
    pub fn add_triangle(&mut self, indices: &[usize; 3]) -> &mut Self {
        self.triangles.push(Triangle {
            v1: indices[0],
            v2: indices[1],
            v3: indices[2],
            p1: None,
            p2: None,
            p3: None,
            pid: None,
        });
        self
    }

    /// Add a collection of triangles from a [[usize;3]] slice
    pub fn add_triangles(&mut self, triangles: &[[usize; 3]]) -> &mut Self {
        for triangle in triangles {
            self.add_triangle(triangle);
        }

        self
    }

    /// Add a collection of triangles from a flattened [[usize]] slice where every subsequent 3 indices
    /// are considered triangle vertex references.
    pub fn add_triangles_flat(&mut self, triangles: &[usize]) -> &mut Self {
        for triangle in triangles.chunks_exact(3) {
            self.triangles.push(Triangle {
                v1: triangle[0],
                v2: triangle[1],
                v3: triangle[2],
                p1: None,
                p2: None,
                p3: None,
                pid: None,
            });
        }

        self
    }

    fn build(self) -> Mesh {
        Mesh {
            vertices: Vertices {
                vertex: self.vertices,
            },
            triangles: Triangles {
                triangle: self.triangles,
            },
            trianglesets: None,
            beamlattice: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::object::ObjectType;

    #[test]
    fn test_model_builder_basic() {
        let mut builder = ModelBuilder::new();
        builder.unit(Unit::Millimeter);
        builder.add_metadata("Application", Some("Test App"));

        let cube_id = builder
            .add_object(|obj| {
                obj.name("Cube")
                    .object_type(ObjectType::Model)
                    .mesh(|mesh| {
                        mesh.add_vertex(&[0.0, 0.0, 0.0])
                            .add_vertex(&[10.0, 0.0, 0.0])
                            .add_vertex(&[10.0, 10.0, 0.0])
                            .add_vertex(&[0.0, 10.0, 0.0])
                            .add_triangle(&[0, 1, 2])
                            .add_triangle(&[0, 2, 3]);
                    });
            })
            .unwrap();

        builder.add_build_item(cube_id);

        let model = builder.build();

        assert_eq!(model.unit, Some(Unit::Millimeter));
        assert_eq!(model.metadata.len(), 1);
        assert_eq!(model.metadata[0].name, "Application");
        assert_eq!(model.resources.object.len(), 1);
        assert_eq!(model.resources.object[0].name, Some("Cube".to_string()));
        assert_eq!(model.build.item.len(), 1);
        assert_eq!(model.build.item[0].objectid, 1);
    }

    #[test]
    fn test_object_id_assignment() {
        let mut builder = ModelBuilder::new();

        let id1 = builder
            .add_object(|obj| {
                obj.name("Obj1");
            })
            .unwrap();
        let id2 = builder
            .add_object(|obj| {
                obj.name("Obj2");
            })
            .unwrap();

        assert_eq!(id1.0, 1);
        assert_eq!(id2.0, 2);

        let model = builder.build();
        assert_eq!(model.resources.object.len(), 2);
        assert_eq!(model.resources.object[0].id, 1);
        assert_eq!(model.resources.object[1].id, 2);
    }

    #[test]
    fn test_multiple_passes() {
        let mut builder = ModelBuilder::new();

        // First pass
        builder.unit(Unit::Centimeter);
        let obj1_id = builder
            .add_object(|obj| {
                obj.name("First");
            })
            .unwrap();

        // Second pass
        builder.add_metadata("Pass", Some("Second"));
        let obj2_id = builder
            .add_object(|obj| {
                obj.name("Second");
            })
            .unwrap();

        // Third pass
        builder.add_build_item(obj1_id);
        builder.add_build_item(obj2_id);

        let model = builder.build();
        assert_eq!(model.unit, Some(Unit::Centimeter));
        assert_eq!(model.metadata.len(), 1);
        assert_eq!(model.resources.object.len(), 2);
        assert_eq!(model.build.item.len(), 2);
    }
}
