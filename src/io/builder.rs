use thiserror::Error;

use crate::{
    core::{
        build::{Build, Item},
        component::{Component, Components},
        mesh::{Mesh, Triangle, Triangles, Vertex, Vertices},
        metadata::Metadata,
        model::{Model, Unit},
        object::{Object, ObjectType},
        resources::Resources,
        transform::Transform,
    },
    io::XmlNamespace,
    threemf_namespaces::{self, PROD_NS, PROD_PREFIX},
};

#[derive(Debug, Error, Clone)]
pub enum BuilderError {
    #[error("Build is not set for the Model. Root Model and adding Build Items requires a Build!")]
    BuildItemNotSet,

    #[error("Build is not allowed in non-root Model")]
    BuildOnlyAllowedInRootModel,

    #[error(
        "UUID is not set for the Object. UUID is required when production extension is enabled!"
    )]
    UuidNotSet,

    #[error("Production extension is required for setting Path!")]
    ProductionExtensionRequiredForPath,

    #[error("ObjectBuilder already has a Mesh set")]
    ObjectBuilderAlreadyContainsMesh,

    #[error("ObjectBuilder already has a Composed Part set")]
    ObjectBuilderAlreadyContainsComposedPart,

    #[error("An unknown object specified in Component")]
    ObjectBuilderReferencesANonExistentComponent,
}

/// Builder for constructing 3MF Model structs with a fluent API
pub struct ModelBuilder {
    unit: Option<Unit>,
    requiredextensions: Vec<XmlNamespace>,
    recommendedextensions: Vec<XmlNamespace>,
    metadata: Vec<Metadata>,
    resources: ResourcesBuilder,
    build: Option<BuildBuilder>, //in submodels, Build item is not required

    // tracks if the model is intended as a root model
    // if true, Build is required
    // else adding Build is not allowed
    is_root: bool,

    // tracks next object id
    next_object_id: ObjectId,

    // tracks if the model requires production ext
    // ensures UUID is set at the minimum
    is_production_ext_required: bool,
}

impl ModelBuilder {
    /// Create a new ModelBuilder with default values
    pub fn new(unit: Unit, is_root: bool) -> Self {
        Self {
            unit: Some(unit),
            requiredextensions: vec![],
            recommendedextensions: vec![],
            metadata: Vec::new(),
            resources: ResourcesBuilder::new(),
            build: None,
            next_object_id: 1.into(),
            is_production_ext_required: false,
            is_root,
        }
    }

    /// Set the unit for the model
    pub fn unit(&mut self, unit: Unit) -> &mut Self {
        self.unit = Some(unit);
        self
    }

    pub fn make_root(&mut self, is_root: bool) -> &mut Self {
        self.is_root = is_root;
        self
    }

    pub fn make_production_extension_required(&mut self) -> &mut Self {
        use threemf_namespaces::{PROD_NS, PROD_PREFIX};

        self.is_production_ext_required = true;
        let is_prod_ext_set = self.requiredextensions.iter().find(|ns| ns.uri == PROD_NS);
        if is_prod_ext_set.is_none() {
            self.requiredextensions.push(XmlNamespace {
                prefix: Some(PROD_PREFIX.to_owned()),
                uri: PROD_NS.to_owned(),
            });
        }

        self
    }

    /// Add a required extension
    pub fn add_required_extension(&mut self, extension: XmlNamespace) -> &mut Self {
        self.requiredextensions.push(extension);
        self
    }

    /// Set recommended extensions
    pub fn add_recommended_extension(&mut self, extension: XmlNamespace) -> &mut Self {
        self.recommendedextensions.push(extension);
        self
    }

    /// Add metadata to the model
    pub fn add_metadata(&mut self, name: &str, value: Option<&str>) -> &mut Self {
        self.metadata.push(Metadata {
            name: name.to_owned(),
            preserve: None,
            value: value.map(|v| v.to_owned()),
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

        let all_object_ids = self
            .resources
            .objects
            .iter()
            .map(|o| ObjectId(o.id))
            .collect::<Vec<_>>();

        let mut obj_builder =
            ObjectBuilder::new(id, &all_object_ids, self.is_production_ext_required);
        f(&mut obj_builder);
        let object = obj_builder.build();

        match object {
            Ok(o) => self.resources.objects.push(o),
            Err(err) => return Err(err),
        }

        Ok(id)
    }

    pub fn add_build(&mut self, uuid: Option<String>) -> Result<&mut Self, BuilderError> {
        if !self.is_root {
            return Err(BuilderError::BuildOnlyAllowedInRootModel);
        }

        if self.is_production_ext_required && uuid.is_none() {
            return Err(BuilderError::UuidNotSet);
        }

        let mut build_builder = BuildBuilder::new(self.is_production_ext_required);
        if let Some(uuid) = uuid {
            build_builder.uuid(uuid);
        }
        self.build = Some(build_builder);

        Ok(self)
    }

    /// Add a build item referencing an object by ID
    pub fn add_build_item(&mut self, object_id: ObjectId) -> Result<&mut Self, BuilderError> {
        if self.is_production_ext_required {
            return Err(BuilderError::UuidNotSet);
        }

        match &mut self.build {
            Some(build) => {
                build.items.push(BuildItem {
                    objectid: object_id,
                    transform: None,
                    partnumber: None,
                    path: None,
                    uuid: None,
                });

                Ok(self)
            }
            None => Err(BuilderError::BuildItemNotSet),
        }
    }

    pub fn add_build_item_advanced(
        &mut self,
        object_id: ObjectId,
        transform: Option<Transform>,
        partnumber: Option<String>,
        path: Option<String>,
        uuid: Option<String>,
    ) -> Result<&mut Self, BuilderError> {
        if self.is_production_ext_required && uuid.is_none() {
            return Err(BuilderError::UuidNotSet);
        }
        match &mut self.build {
            Some(build) => {
                build.items.push(BuildItem {
                    objectid: object_id,
                    transform,
                    partnumber,
                    path,
                    uuid,
                });

                Ok(self)
            }
            None => Err(BuilderError::BuildItemNotSet),
        }
    }

    /// Build the final Model
    pub fn build(self) -> Result<Model, BuilderError> {
        let requiredextensions = get_extensions_definition(&self.requiredextensions);
        let recommendedextensions = get_extensions_definition(&self.recommendedextensions);

        if self.is_root && self.build.is_none() {
            return Err(BuilderError::BuildItemNotSet);
        }

        if !self.is_root && self.build.is_some() {
            return Err(BuilderError::BuildOnlyAllowedInRootModel);
        }

        let build = if let Some(builder) = self.build {
            builder.build()?
        } else {
            Build {
                uuid: None,
                item: vec![],
            }
        };

        Ok(Model {
            unit: self.unit,
            requiredextensions,
            recommendedextensions,
            metadata: self.metadata,
            resources: self.resources.build(),
            build,
        })
    }
}

impl Default for ModelBuilder {
    fn default() -> Self {
        Self::new(Unit::Millimeter, true)
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
    uuid: Option<String>,

    is_production_ext_required: bool,
}

impl BuildBuilder {
    fn new(is_production_ext_required: bool) -> Self {
        Self {
            items: Vec::new(),
            uuid: None,
            is_production_ext_required,
        }
    }

    fn uuid(&mut self, uuid: String) -> &mut Self {
        self.uuid = Some(uuid);

        self
    }

    fn build(self) -> Result<Build, BuilderError> {
        if self.is_production_ext_required && self.uuid.is_none() {
            return Err(BuilderError::UuidNotSet);
        }

        Ok(Build {
            uuid: None,
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
        })
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
    object_id: ObjectId,
    objecttype: Option<ObjectType>,
    thumbnail: Option<String>,
    partnumber: Option<String>,
    name: Option<String>,
    pid: Option<usize>,
    pindex: Option<usize>,
    uuid: Option<String>,
    mesh: Option<Mesh>,
    components: Option<Components>,

    all_existing_object_ids: Vec<ObjectId>,

    // sets if the production ext is required.
    // if yes will ensure UUID is set before building the object
    is_production_ext_required: bool,
}

impl ObjectBuilder {
    fn new(
        object_id: ObjectId,
        all_existing_object_ids: &[ObjectId],
        is_production_ext_required: bool,
    ) -> Self {
        Self {
            object_id,
            objecttype: Some(ObjectType::Model),
            thumbnail: None,
            partnumber: None,
            name: None,
            pid: None,
            pindex: None,
            uuid: None,
            mesh: None,
            components: None,
            all_existing_object_ids: all_existing_object_ids.to_vec(),
            is_production_ext_required,
        }
    }

    /// Set the object type
    pub fn object_type(&mut self, object_type: ObjectType) -> &mut Self {
        self.objecttype = Some(object_type);
        self
    }

    /// Set the object name
    pub fn name(&mut self, name: &str) -> &mut Self {
        self.name = Some(name.to_owned());
        self
    }

    /// Set the part number
    pub fn part_number(&mut self, part_number: &str) -> &mut Self {
        self.partnumber = Some(part_number.to_owned());
        self
    }

    pub fn uuid(&mut self, uuid: &str) -> &mut Self {
        self.uuid = Some(uuid.to_owned());
        self
    }

    /// Add a mesh using a builder function
    pub fn mesh<F>(&mut self, f: F) -> Result<&mut Self, BuilderError>
    where
        F: FnOnce(&mut MeshBuilder),
    {
        if self.components.is_some() {
            return Err(BuilderError::ObjectBuilderAlreadyContainsComposedPart);
        }
        let mut mesh_builder = MeshBuilder::new();
        f(&mut mesh_builder);
        self.mesh = Some(mesh_builder.build());
        Ok(self)
    }

    pub fn composed_part<F>(&mut self, f: F) -> Result<&mut Self, BuilderError>
    where
        F: FnOnce(&mut ComposedPartBuilder) -> Result<&mut ComposedPartBuilder, BuilderError>,
    {
        if self.mesh.is_some() {
            return Err(BuilderError::ObjectBuilderAlreadyContainsMesh);
        }
        let mut cp_builder = ComposedPartBuilder::new(self.is_production_ext_required);
        f(&mut cp_builder)?;

        let all_object_exists = cp_builder
            .components
            .iter()
            .all(|c| self.all_existing_object_ids.contains(&ObjectId(c.objectid)));

        if !all_object_exists {
            return Err(BuilderError::ObjectBuilderReferencesANonExistentComponent);
        }

        self.components = Some(cp_builder.build());

        Ok(self)
    }

    fn build(self) -> Result<Object, BuilderError> {
        if self.is_production_ext_required && self.uuid.is_none() {
            return Err(BuilderError::UuidNotSet);
        }

        Ok(Object {
            id: self.object_id.0,
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

pub struct ComposedPartBuilder {
    components: Vec<Component>,

    is_production_ext_required: bool,
}

impl ComposedPartBuilder {
    fn new(is_production_ext_required: bool) -> Self {
        ComposedPartBuilder {
            components: vec![],
            is_production_ext_required,
        }
    }

    pub fn add_component(
        &mut self,
        object_id: ObjectId,
        transform: Option<Transform>,
    ) -> Result<&mut Self, BuilderError> {
        if self.is_production_ext_required {
            return Err(BuilderError::UuidNotSet);
        }

        self.components.push(Component {
            objectid: object_id.into(),
            transform,
            path: None,
            uuid: None,
        });

        Ok(self)
    }

    pub fn add_component_advanced(
        &mut self,
        object_id: ObjectId,
        uuid: String,
        transform: Option<Transform>,
        path: Option<String>,
    ) -> Result<&mut Self, BuilderError> {
        if path.is_some() && !self.is_production_ext_required {
            return Err(BuilderError::ProductionExtensionRequiredForPath);
        }

        self.components.push(Component {
            objectid: object_id.into(),
            transform,
            path,
            uuid: Some(uuid),
        });

        Ok(self)
    }

    pub fn build(self) -> Components {
        Components {
            component: self.components,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::object::ObjectType;

    #[test]
    fn test_model_builder_basic() {
        let mut builder = ModelBuilder::new(Unit::Millimeter, true);
        builder.unit(Unit::Millimeter);
        builder.add_metadata("Application", Some("Test App"));

        let cube_id = builder
            .add_object(|obj| {
                if obj
                    .name("Cube")
                    .object_type(ObjectType::Model)
                    .mesh(|mesh| {
                        mesh.add_vertex(&[0.0, 0.0, 0.0])
                            .add_vertex(&[10.0, 0.0, 0.0])
                            .add_vertex(&[10.0, 10.0, 0.0])
                            .add_vertex(&[0.0, 10.0, 0.0])
                            .add_triangle(&[0, 1, 2])
                            .add_triangle(&[0, 2, 3]);
                    })
                    .is_ok()
                {}
            })
            .unwrap();

        builder.add_build_item(cube_id);

        let model = builder.build().unwrap();

        assert_eq!(model.unit, Some(Unit::Millimeter));
        assert_eq!(model.metadata.len(), 1);
        assert_eq!(model.metadata[0].name, "Application");
        assert_eq!(model.resources.object.len(), 1);
        assert_eq!(model.resources.object[0].name, Some("Cube".to_owned()));
        assert_eq!(model.build.item.len(), 1);
        assert_eq!(model.build.item[0].objectid, 1);
    }

    #[test]
    fn test_object_id_assignment() {
        let mut builder = ModelBuilder::new(Unit::Millimeter, true);

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

        let model = builder.build().unwrap();
        assert_eq!(model.resources.object.len(), 2);
        assert_eq!(model.resources.object[0].id, 1);
        assert_eq!(model.resources.object[1].id, 2);
    }

    #[test]
    fn test_multiple_passes() {
        let mut builder = ModelBuilder::new(Unit::Millimeter, true);

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

        let model = builder.build().unwrap();
        assert_eq!(model.unit, Some(Unit::Centimeter));
        assert_eq!(model.metadata.len(), 1);
        assert_eq!(model.resources.object.len(), 2);
        assert_eq!(model.build.item.len(), 2);
    }
}
