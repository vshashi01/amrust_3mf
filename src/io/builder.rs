use thiserror::Error;

use crate::{
    core::{
        build::{Build, Item},
        component::{Component, Components},
        mesh::{Mesh, Triangle, Triangles, Vertex, Vertices},
        metadata::Metadata,
        model::Model,
        object::Object,
        resources::Resources,
        transform::Transform,
    },
    io::XmlNamespace,
    threemf_namespaces::{
        self, BEAM_LATTICE_BALLS_NS, BEAM_LATTICE_BALLS_PREFIX, BEAM_LATTICE_NS,
        BEAM_LATTICE_PREFIX, PROD_NS, PROD_PREFIX,
    },
};

use std::{
    collections::HashSet,
    ops::{Deref, DerefMut},
};

pub use crate::core::model::Unit;
pub use crate::core::object::ObjectType;

#[derive(Debug, Error, Clone)]
pub enum ModelError {
    #[error("Build is not set for the Model. Root Model and adding Build Items requires a Build!")]
    BuildItemNotSet,

    #[error("Build is not allowed in non-root Model")]
    BuildOnlyAllowedInRootModel,

    #[error("Something wrong when adding Build")]
    BuildError(#[from] BuildError),

    #[error("Something wrong when adding Items")]
    ItemError(#[from] ItemError),
}

#[derive(Debug, Error, Clone, Copy, PartialEq)]
pub enum ProductionExtensionError {
    #[error("Object Uuid is not set with Production extension enabled!")]
    ObjectUuidNotSet,

    #[error("Component Uuid is not set with Production extension enabled!")]
    ComponentUuidNotSet,

    #[error("Item Uuid is not set with Production extension enabled!")]
    ItemUuidNotSet,

    #[error("Build Uuid is not set with Production extension enabled!")]
    BuildUuidNotSet,

    #[error("Component Path is set without Production extension enabled!")]
    PathUsedOnComponent,

    #[error("Item Path is set without Production extension enabled!")]
    PathUsedOnItem,
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

    is_beam_lattice_ext_required: bool,
    is_beam_lattice_balls_ext_required: bool,
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
            is_root,
            next_object_id: 1.into(),
            is_production_ext_required: false,
            is_beam_lattice_ext_required: false,
            is_beam_lattice_balls_ext_required: false,
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

    pub fn make_production_extension_required(
        &mut self,
    ) -> Result<&mut Self, ProductionExtensionError> {
        // at the time the production extension is set as required,
        // we should anyway check if existing items fulfil the contract
        //
        for o in &self.resources.objects {
            if o.uuid.is_none() {
                return Err(ProductionExtensionError::ObjectUuidNotSet);
            } else if let Some(components) = &o.components
                && components.component.iter().any(|c| c.uuid.is_none())
            {
                return Err(ProductionExtensionError::ComponentUuidNotSet);
            }

            if let Some(build) = &self.build {
                if build.uuid.is_some() {
                    if build.items.iter().any(|i| i.uuid.is_none()) {
                        return Err(ProductionExtensionError::ItemUuidNotSet);
                    }
                } else {
                    return Err(ProductionExtensionError::BuildUuidNotSet);
                }
            }
        }
        self.is_production_ext_required = true;
        Ok(self)
    }

    pub fn make_beam_lattice_extension_required(
        &mut self,
        enable_beam_lattice_balls: bool,
    ) -> &mut Self {
        self.is_beam_lattice_ext_required = true;
        self.is_beam_lattice_balls_ext_required = enable_beam_lattice_balls;

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
    pub fn add_mesh_object<F>(&mut self, f: F) -> Result<ObjectId, MeshObjectError>
    where
        F: FnOnce(&mut MeshObjectBuilder) -> Result<(), MeshObjectError>,
    {
        let id = self.next_object_id;

        let mut obj_builder = MeshObjectBuilder::new(id, self.is_production_ext_required);
        f(&mut obj_builder)?;

        self.add_mesh_object_from_builder(obj_builder)
    }

    pub fn add_mesh_object_from_builder(
        &mut self,
        builder: MeshObjectBuilder,
    ) -> Result<ObjectId, MeshObjectError> {
        let id = builder.object_id;
        let object = builder.build()?;

        if let Some(mesh) = &object.mesh {
            self.set_recommended_namespaces_for_mesh(mesh);
        }

        self.resources.objects.push(object);
        self.next_object_id = ObjectId(id.0 + 1);

        Ok(id)
    }

    /// Add an object using a builder function, returns the assigned ObjectId
    pub fn add_components_object<F>(&mut self, f: F) -> Result<ObjectId, ComponentsObjectError>
    where
        F: FnOnce(&mut ComponentsObjectBuilder) -> Result<(), ComponentsObjectError>,
    {
        let id = self.next_object_id;

        let all_object_ids = self
            .resources
            .objects
            .iter()
            .map(|o| ObjectId(o.id))
            .collect::<Vec<_>>();

        let mut obj_builder =
            ComponentsObjectBuilder::new(id, &all_object_ids, self.is_production_ext_required);
        f(&mut obj_builder)?;

        self.add_composed_part_object_from_builder(obj_builder)
    }

    pub fn add_composed_part_object_from_builder(
        &mut self,
        builder: ComponentsObjectBuilder,
    ) -> Result<ObjectId, ComponentsObjectError> {
        let id = builder.object_id;
        let object = builder.build()?;

        self.resources.objects.push(object);
        self.next_object_id = ObjectId(id.0 + 1);

        Ok(id)
    }

    pub fn add_build(&mut self, uuid: Option<String>) -> Result<&mut Self, ModelError> {
        if !self.is_root {
            return Err(ModelError::BuildOnlyAllowedInRootModel);
        }
        let mut build_builder = BuildBuilder::new();
        if let Some(uuid) = uuid {
            build_builder.uuid(uuid);
        }

        //check if the Build can be created at this time
        build_builder.can_build(self.is_production_ext_required)?;
        self.build = Some(build_builder);

        Ok(self)
    }

    /// Add a build item referencing an object by ID
    pub fn add_build_item(&mut self, object_id: ObjectId) -> Result<&mut Self, ModelError> {
        self.add_build_item_advanced(object_id, |f| {})
    }

    /// Add a build item referencing an object by ID
    pub fn add_build_item_advanced<F>(
        &mut self,
        object_id: ObjectId,
        f: F,
    ) -> Result<&mut Self, ModelError>
    where
        F: FnOnce(&mut ItemBuilder),
    {
        match &mut self.build {
            Some(build) => {
                build.add_build_item(object_id, self.is_production_ext_required, f)?;

                Ok(self)
            }
            None => Err(ModelError::BuildItemNotSet),
        }
    }

    /// Build the final Model
    pub fn build(self) -> Result<Model, ModelError> {
        let required_extensions = self.process_required_extensions();

        let requiredextensions = get_extensions_definition(&required_extensions);
        let recommendedextensions = get_extensions_definition(&self.recommendedextensions);

        if self.is_root && self.build.is_none() {
            return Err(ModelError::BuildItemNotSet);
        }

        if !self.is_root && self.build.is_some() {
            return Err(ModelError::BuildOnlyAllowedInRootModel);
        }

        let build = if let Some(builder) = self.build {
            builder.build(self.is_production_ext_required)?
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

    fn set_recommended_namespaces_for_mesh(&mut self, mesh: &Mesh) {
        use threemf_namespaces::{CORE_TRIANGLESET_NS, CORE_TRIANGLESET_PREFIX};
        if mesh.trianglesets.is_some()
            && self
                .recommendedextensions
                .iter()
                .all(|ns| ns.uri == CORE_TRIANGLESET_NS)
        {
            self.recommendedextensions.push(XmlNamespace {
                prefix: Some(CORE_TRIANGLESET_PREFIX.to_owned()),
                uri: CORE_TRIANGLESET_NS.to_owned(),
            });
        }
    }

    fn process_required_extensions(&self) -> Vec<XmlNamespace> {
        let mut required_extensions = self.requiredextensions.clone();
        if self.is_production_ext_required {
            let is_prod_ext_set = required_extensions.iter().find(|ns| ns.uri == PROD_NS);
            if is_prod_ext_set.is_none() {
                required_extensions.push(XmlNamespace {
                    prefix: Some(PROD_PREFIX.to_owned()),
                    uri: PROD_NS.to_owned(),
                });
            }
        }

        if self.is_beam_lattice_ext_required {
            let is_bl_ext_set = required_extensions
                .iter()
                .find(|ns| ns.uri == BEAM_LATTICE_NS);
            if is_bl_ext_set.is_none() {
                required_extensions.push(XmlNamespace {
                    prefix: Some(BEAM_LATTICE_PREFIX.to_owned()),
                    uri: BEAM_LATTICE_NS.to_owned(),
                });
            }

            if self.is_beam_lattice_balls_ext_required {
                let is_bl_balls_ext_set = required_extensions
                    .iter()
                    .find(|ns| ns.uri == BEAM_LATTICE_BALLS_NS);
                if is_bl_balls_ext_set.is_none() {
                    required_extensions.push(XmlNamespace {
                        prefix: Some(BEAM_LATTICE_BALLS_PREFIX.to_owned()),
                        uri: BEAM_LATTICE_BALLS_NS.to_owned(),
                    });
                }
            }
        }

        required_extensions
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
        let mut unique_namespaces: HashSet<XmlNamespace> = HashSet::new();

        for ns in extensions {
            unique_namespaces.insert(ns.clone());
        }

        for ns in unique_namespaces {
            if let Some(prefix) = &ns.prefix {
                extension_string.push_str(prefix);
                extension_string.push(' ');
            }
        }

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
            basematerials: Vec::new(),
        }
    }
}

#[derive(Debug, Error, Clone, Copy)]
pub enum BuildError {
    #[error("Production extension is enabled but Uuid for the Build is not set")]
    BuildUuidNotSet,
}

/// Builder for Build section
pub struct BuildBuilder {
    items: Vec<Item>,
    uuid: Option<String>,
}

impl BuildBuilder {
    fn new() -> Self {
        Self {
            items: Vec::new(),
            uuid: None,
        }
    }

    fn uuid(&mut self, uuid: String) -> &mut Self {
        self.uuid = Some(uuid);

        self
    }

    fn add_build_item<F>(
        &mut self,
        objectid: ObjectId,
        is_production_ext_enabled: bool,
        f: F,
    ) -> Result<&mut Self, ItemError>
    where
        F: FnOnce(&mut ItemBuilder),
    {
        let mut builder = ItemBuilder::new(objectid);
        f(&mut builder);

        let item = builder.build(is_production_ext_enabled)?;
        self.items.push(item);
        Ok(self)
    }

    fn can_build(&self, is_production_ext_enabled: bool) -> Result<(), BuildError> {
        if is_production_ext_enabled && self.uuid.is_none() {
            return Err(BuildError::BuildUuidNotSet);
        }

        Ok(())
    }

    fn build(self, is_production_ext_required: bool) -> Result<Build, BuildError> {
        self.can_build(is_production_ext_required)?;

        Ok(Build {
            uuid: None,
            item: self.items,
        })
    }
}

#[derive(Debug, Error, Clone, Copy, PartialEq)]
pub enum ItemError {
    #[error("Item path is set without the Production extension enabled!")]
    ItemPathSetWithoutProductionExtension,

    #[error("Production extension is enabled but Uuid is not set!")]
    ItemUuidNotSet,
}

/// Builder to setup a Build item
pub struct ItemBuilder {
    objectid: ObjectId,
    transform: Option<Transform>,
    partnumber: Option<String>,
    path: Option<String>,
    uuid: Option<String>,
}

impl ItemBuilder {
    fn new(objectid: ObjectId) -> Self {
        Self {
            objectid,
            transform: None,
            partnumber: None,
            path: None,
            uuid: None,
        }
    }

    pub fn transform(&mut self, transform: Transform) -> &mut Self {
        self.transform = Some(transform);
        self
    }

    pub fn partnumber(&mut self, partnumber: &str) -> &mut Self {
        self.partnumber = Some(partnumber.to_owned());
        self
    }

    pub fn uuid(&mut self, uuid: &str) -> &mut Self {
        self.uuid = Some(uuid.to_owned());
        self
    }

    pub fn path(&mut self, path: &str) -> &mut Self {
        self.path = Some(path.to_owned());
        self
    }

    fn build(self, is_production_ext_enabled: bool) -> Result<Item, ItemError> {
        if !is_production_ext_enabled && self.path.is_some() {
            return Err(ItemError::ItemPathSetWithoutProductionExtension);
        } else if is_production_ext_enabled && self.uuid.is_none() {
            return Err(ItemError::ItemUuidNotSet);
        }
        Ok(Item {
            objectid: self.objectid.0,
            transform: self.transform,
            partnumber: self.partnumber,
            path: self.path,
            uuid: self.uuid,
        })
    }
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
pub struct ObjectBuilder<T> {
    entity: T,
    object_id: ObjectId,
    objecttype: Option<ObjectType>,
    thumbnail: Option<String>,
    partnumber: Option<String>,
    name: Option<String>,
    pid: Option<usize>,
    pindex: Option<usize>,
    uuid: Option<String>,

    // sets if the production ext is required.
    // if yes will ensure UUID is set before building the object
    is_production_ext_required: bool,
}

impl<T> ObjectBuilder<T> {
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
}

impl<T> Deref for ObjectBuilder<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.entity
    }
}

impl<T> DerefMut for ObjectBuilder<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.entity
    }
}

#[derive(Debug, Error, Clone, PartialEq)]
pub enum MeshObjectError {
    #[error("Production extension is enabled but Uuid is not set!")]
    ObjectUuidNotSet,
}

pub type MeshObjectBuilder = ObjectBuilder<MeshBuilder>;

impl MeshObjectBuilder {
    fn new(object_id: ObjectId, is_production_ext_required: bool) -> Self {
        Self {
            entity: MeshBuilder::new(),
            object_id,
            objecttype: Some(ObjectType::Model),
            thumbnail: None,
            partnumber: None,
            name: None,
            pid: None,
            pindex: None,
            uuid: None,
            is_production_ext_required,
        }
    }

    fn build(self) -> Result<Object, MeshObjectError> {
        let mesh = self.entity.build_mesh().unwrap();

        if self.is_production_ext_required && self.uuid.is_none() {
            return Err(MeshObjectError::ObjectUuidNotSet);
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
            mesh: Some(mesh),
            components: None,
        })
    }
}

/// Builder for Mesh
pub struct MeshBuilder {
    vertices: Vec<Vertex>,
    triangles: Vec<Triangle>,
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

    fn build_mesh(self) -> Result<Mesh, MeshObjectError> {
        Ok(Mesh {
            vertices: Vertices {
                vertex: self.vertices,
            },
            triangles: Triangles {
                triangle: self.triangles,
            },
            trianglesets: None,
            beamlattice: None,
        })
    }
}

#[derive(Debug, Error, Clone)]
pub enum ComponentsObjectError {
    #[error("production extension is enabled but uuid is not set")]
    ComponentUuidNotSet,

    #[error("Path is set for a Component without enabling the Production extension")]
    PathSetWithoutProductionExtension,

    #[error("One or more Component References unknown objects")]
    ObjectReferenceNotFoundForComponent,

    #[error("Production extension is enabled but Uuid is not set")]
    ObjectUuidNotSet,
}

pub type ComponentsObjectBuilder = ObjectBuilder<ComponentsBuilder>;

impl ComponentsObjectBuilder {
    fn new(
        object_id: ObjectId,
        all_existing_object_ids: &[ObjectId],
        is_production_ext_required: bool,
    ) -> Self {
        Self {
            entity: ComponentsBuilder::new(all_existing_object_ids),
            object_id,
            objecttype: Some(ObjectType::Model),
            thumbnail: None,
            partnumber: None,
            name: None,
            pid: None,
            pindex: None,
            uuid: None,
            is_production_ext_required,
        }
    }

    fn build(self) -> Result<Object, ComponentsObjectError> {
        let components = self
            .entity
            .build_components(self.is_production_ext_required)?;

        if self.is_production_ext_required && self.uuid.is_none() {
            return Err(ComponentsObjectError::ObjectUuidNotSet);
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
            mesh: None,
            components: Some(components),
        })
    }
}

pub struct ComponentsBuilder {
    components: Vec<Component>,

    all_existing_object_ids: Vec<ObjectId>,
}

impl ComponentsBuilder {
    fn new(all_existing_object_ids: &[ObjectId]) -> Self {
        ComponentsBuilder {
            components: vec![],
            all_existing_object_ids: all_existing_object_ids.to_vec(),
        }
    }

    pub fn add_component(&mut self, object_id: ObjectId) -> &mut Self {
        self.add_component_advanced(object_id, |_| {});
        self
    }

    pub fn add_component_advanced<F>(&mut self, object_id: ObjectId, f: F) -> &mut Self
    where
        F: FnOnce(&mut ComponentBuilder),
    {
        let mut builder = ComponentBuilder::new(object_id);
        f(&mut builder);

        let component = builder.build();
        self.components.push(component);

        self
    }

    fn build_components(
        self,
        is_production_ext_required: bool,
    ) -> Result<Components, ComponentsObjectError> {
        if is_production_ext_required {
            let all_uuid_set = self.components.iter().all(|c| c.uuid.is_some());
            if !all_uuid_set {
                return Err(ComponentsObjectError::ComponentUuidNotSet);
            }
        } else {
            let all_path_is_not_set = self.components.iter().all(|c| c.path.is_none());
            if !all_path_is_not_set {
                return Err(ComponentsObjectError::PathSetWithoutProductionExtension);
            }
        }

        let all_object_exists = self
            .components
            .iter()
            .all(|c| self.all_existing_object_ids.contains(&ObjectId(c.objectid)));

        if !all_object_exists {
            return Err(ComponentsObjectError::ObjectReferenceNotFoundForComponent);
        }

        Ok(Components {
            component: self.components,
        })
    }
}

pub struct ComponentBuilder {
    objectid: usize,
    transform: Option<Transform>,
    path: Option<String>,
    uuid: Option<String>,
}

impl ComponentBuilder {
    pub fn new(object_id: ObjectId) -> Self {
        Self {
            objectid: object_id.0,
            transform: None,
            path: None,
            uuid: None,
        }
    }

    pub fn transform(&mut self, transform: Transform) -> &mut Self {
        self.transform = Some(transform);
        self
    }

    pub fn uuid(&mut self, uuid: &str) -> &mut Self {
        self.uuid = Some(uuid.to_owned());
        self
    }

    pub fn path(&mut self, path: &str) -> &mut Self {
        self.path = Some(path.to_owned());
        self
    }

    fn build(self) -> Component {
        Component {
            objectid: self.objectid,
            transform: self.transform,
            path: self.path,
            uuid: self.uuid,
        }
    }
}

#[cfg(test)]
mod smoke_tests {
    use crate::core::object::ObjectType;

    use super::*;

    #[test]
    fn test_model_builder_basic() {
        let mut builder = ModelBuilder::new(Unit::Millimeter, true);
        builder.unit(Unit::Millimeter);
        builder.add_metadata("Application", Some("Test App"));
        builder.add_build(None).unwrap();

        let cube_id = builder
            .add_mesh_object(|obj| {
                obj.name("Cube");
                obj.object_type(ObjectType::Model);
                //obj.mesh(|mesh| {
                obj.add_vertex(&[0.0, 0.0, 0.0])
                    .add_vertex(&[10.0, 0.0, 0.0])
                    .add_vertex(&[10.0, 10.0, 0.0])
                    .add_vertex(&[0.0, 10.0, 0.0])
                    .add_triangle(&[0, 1, 2])
                    .add_triangle(&[0, 2, 3]);
                //});

                Ok(())
            })
            .unwrap();

        builder.add_build_item(cube_id).unwrap();

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
        let mut builder = ModelBuilder::new(Unit::Millimeter, false);

        let id1 = builder
            .add_mesh_object(|obj| {
                obj.name("Obj1");
                Ok(())
            })
            .unwrap();
        let id2 = builder
            .add_mesh_object(|obj| {
                obj.name("Obj2");
                Ok(())
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
            .add_mesh_object(|obj| {
                obj.name("First");
                Ok(())
            })
            .unwrap();

        // Second pass
        builder.add_metadata("Pass", Some("Second"));
        let obj2_id = builder
            .add_mesh_object(|obj| {
                obj.name("Second");
                Ok(())
            })
            .unwrap();

        // Third pass
        builder.add_build(None).unwrap();
        builder.add_build_item(obj1_id).unwrap();
        builder.add_build_item(obj2_id).unwrap();

        let model = builder.build().unwrap();
        assert_eq!(model.unit, Some(Unit::Centimeter));
        assert_eq!(model.metadata.len(), 1);
        assert_eq!(model.resources.object.len(), 2);
        assert_eq!(model.build.item.len(), 2);
    }

    #[test]
    fn test_error_cases_for_build_in_model() {
        // Test BuildItemNotSet: root model without build
        let mut builder = ModelBuilder::new(Unit::Millimeter, true);
        builder.add_mesh_object(|obj| Ok(())).unwrap();
        assert!(matches!(builder.build(), Err(ModelError::BuildItemNotSet)));

        // Test BuildOnlyAllowedInRootModel: non-root model with build
        let mut builder = ModelBuilder::new(Unit::Millimeter, false);
        assert!(matches!(
            builder.add_build(None),
            Err(ModelError::BuildOnlyAllowedInRootModel)
        ));
    }

    #[test]
    fn test_production_ext_add_prod_ns_to_required_extensions() {
        let mut builder = ModelBuilder::new(Unit::Millimeter, true);
        builder.make_production_extension_required().unwrap(); //should not return error
        builder.add_build(Some("build-uuid".to_string())).unwrap();
        let obj_id = builder
            .add_mesh_object(|obj| {
                obj.uuid("obj-uuid");
                obj.name("test");

                Ok(())
            })
            .unwrap();
        builder
            .add_build_item_advanced(obj_id, |i| {
                i.uuid("item-uuid");
            })
            .unwrap();
        let model = builder.build().unwrap();
        assert_eq!(model.requiredextensions, Some("p ".to_string()));
    }

    #[test]
    fn test_production_ext_requires_object_uuid() {
        let mut builder = ModelBuilder::new(Unit::Millimeter, true);
        builder.make_production_extension_required().unwrap(); // should not return error;
        builder.add_build(Some("build-uuid".to_string())).unwrap();
        let obj_id = builder.add_mesh_object(|obj| {
            obj.name("test");
            // no uuid
            Ok(())
        });
        assert!(matches!(obj_id, Err(MeshObjectError::ObjectUuidNotSet)));
    }

    #[test]
    fn test_production_ext_requires_build_uuid() {
        let mut builder = ModelBuilder::new(Unit::Millimeter, true);
        builder.make_production_extension_required().unwrap(); //should not return error
        let result = builder.add_build(None);
        assert!(matches!(
            result,
            Err(ModelError::BuildError(BuildError::BuildUuidNotSet))
        ));
    }

    #[test]
    fn test_production_ext_requires_build_item_uuid() {
        let mut builder = ModelBuilder::new(Unit::Millimeter, true);
        builder.make_production_extension_required().unwrap(); //should not return error
        builder.add_build(Some("build-uuid".to_string())).unwrap();
        let obj_id = builder
            .add_mesh_object(|obj| {
                obj.name("test");
                obj.uuid("some-uuid");
                Ok(())
            })
            .unwrap();

        let err = builder.add_build_item(obj_id);
        assert!(matches!(
            err,
            Err(ModelError::ItemError(ItemError::ItemUuidNotSet))
        ));
    }

    #[test]
    fn test_production_ext_required_for_build_item_path() {
        let mut builder = ModelBuilder::new(Unit::Millimeter, true);
        builder.add_build(None).unwrap();
        let obj_id = builder.add_mesh_object(|obj| Ok(())).unwrap();
        let result = builder.add_build_item_advanced(obj_id, |i| {
            i.path("some-path");
        });
        assert!(matches!(
            result,
            Err(ModelError::ItemError(
                ItemError::ItemPathSetWithoutProductionExtension
            ))
        ));
    }

    #[test]
    fn test_production_ext_allows_path_on_component_and_build_item() {
        // Test path allowed when production ext enabled
        let mut builder = ModelBuilder::new(Unit::Millimeter, true);
        builder.make_production_extension_required().unwrap(); // should not return error

        let mesh_obj_id = builder
            .add_mesh_object(|obj| {
                obj.uuid("object-uuid");
                Ok(())
            })
            .unwrap();

        if let Err(err) = builder.add_components_object(|obj| {
            obj.uuid("obj-uuid");
            //obj.composed_part(|cp| {
            obj.add_component_advanced(mesh_obj_id, |c| {
                c.uuid("comp-uuid").path("comp-path");
            });
            //});

            Ok(())
        }) {
            panic!("{err:?}");
        }

        builder.add_build(Some("build-uuid".to_owned())).unwrap();
        if let Err(err) = builder.add_build_item_advanced(mesh_obj_id, |i| {
            i.uuid("some-uuid").path("some-path");
        }) {
            panic!("{err:?}");
        }
    }

    #[test]
    fn test_extension_tests() {
        let mut builder = ModelBuilder::new(Unit::Millimeter, true);
        builder.add_required_extension(crate::io::XmlNamespace {
            prefix: Some("test".to_string()),
            uri: "http://example.com/test".to_string(),
        });
        builder.add_recommended_extension(crate::io::XmlNamespace {
            prefix: Some("rec".to_string()),
            uri: "http://example.com/rec".to_string(),
        });
        builder.add_build(None).unwrap();
        let model = builder.build().unwrap();
        assert_eq!(model.requiredextensions, Some("test ".to_string()));
        assert_eq!(model.recommendedextensions, Some("rec ".to_string()));
    }

    #[test]
    fn test_build_item_advanced_tests() {
        let mut builder = ModelBuilder::new(Unit::Millimeter, true);
        builder.make_production_extension_required();
        let obj_id = builder
            .add_mesh_object(|obj| {
                obj.name("test").uuid("obj-uuid");
                Ok(())
            })
            .unwrap();
        builder.add_build(Some("build-uuid".to_owned())).unwrap();

        let transform = crate::core::transform::Transform([
            1.0, 0.0, 0.0, 1.0, 0.0, 0.0, 1.0, 0.0, 0.0, 1.0, 0.0, 0.0,
        ]);
        builder
            .add_build_item_advanced(obj_id, |i| {
                i.transform(transform.clone())
                    .partnumber("part")
                    .path("path")
                    .uuid("uuid");
            })
            .unwrap();

        let model = builder.build().unwrap();
        let item = &model.build.item[0];
        assert_eq!(item.objectid, 1);
        assert_eq!(item.transform, Some(transform));
        assert_eq!(item.partnumber, Some("part".to_string()));
        assert_eq!(item.path, Some("path".to_string()));
        assert_eq!(item.uuid, Some("uuid".to_string()));
    }

    #[test]
    fn test_object_builder_tests() {
        let mut builder = ModelBuilder::new(Unit::Millimeter, true);
        let obj_id = builder
            .add_mesh_object(|obj| {
                obj.object_type(crate::core::object::ObjectType::Support);
                obj.name("support obj");
                obj.part_number("part123");
                obj.uuid("obj-uuid");
                Ok(())
            })
            .unwrap();
        builder.add_build(None).unwrap();
        builder.add_build_item(obj_id).unwrap();
        let model = builder.build().unwrap();
        let obj = &model.resources.object[0];
        assert_eq!(
            obj.objecttype,
            Some(crate::core::object::ObjectType::Support)
        );
        assert_eq!(obj.name, Some("support obj".to_string()));
        assert_eq!(obj.partnumber, Some("part123".to_string()));
        assert_eq!(obj.uuid, Some("obj-uuid".to_string()));
    }

    #[test]
    fn test_add_mesh_object() {
        let mut builder = ModelBuilder::new(Unit::Millimeter, true);
        let obj_id = builder
            .add_mesh_object(|obj| {
                obj.add_vertices(&[[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]]);
                obj.add_vertices_flat(&[0.0, 0.0, 1.0, 1.0, 1.0, 0.0]);
                obj.add_triangles(&[[0, 1, 2]]);
                obj.add_triangles_flat(&[0, 2, 3, 1, 3, 4]);

                Ok(())
            })
            .unwrap();
        builder.add_build(None).unwrap();
        builder.add_build_item(obj_id).unwrap();
        let model = builder.build().unwrap();
        let mesh = model.resources.object[0].mesh.as_ref().unwrap();
        assert_eq!(mesh.vertices.vertex.len(), 5);
        assert_eq!(mesh.triangles.triangle.len(), 3);
    }

    #[test]
    fn test_add_composed_part_object() {
        let mut builder = ModelBuilder::new(Unit::Millimeter, true);
        let obj1_id = builder
            .add_mesh_object(|obj| {
                obj.name("obj1");

                Ok(())
            })
            .unwrap();
        let obj2_id = builder
            .add_components_object(|obj| {
                obj.add_component(obj1_id);
                Ok(())
            })
            .unwrap();
        builder.add_build(None).unwrap();
        builder.add_build_item(obj2_id).unwrap();
        let model = builder.build().unwrap();
        let obj = &model.resources.object[1];
        assert!(obj.components.is_some());
        let comp = &obj.components.as_ref().unwrap().component[0];
        assert_eq!(comp.objectid, obj1_id.into());
    }

    #[test]
    fn test_add_composed_part_object_errors_without_production_extension() {
        let mut builder = ModelBuilder::new(Unit::Millimeter, true);
        let obj1_id = builder
            .add_mesh_object(|obj| {
                obj.name("obj1");

                Ok(())
            })
            .unwrap();
        let obj2_id = builder
            .add_components_object(|obj| {
                obj.add_component(obj1_id);
                Ok(())
            })
            .unwrap();

        // Test non-existent object reference
        let result = builder.add_components_object(|obj| {
            obj.add_component(ObjectId(999));
            Ok(())
        });
        assert!(matches!(
            result,
            Err(ComponentsObjectError::ObjectReferenceNotFoundForComponent)
        ));

        // Path set without Production extension
        let result = builder.add_components_object(|obj| {
            obj.add_component_advanced(obj1_id, |c| {
                c.path("some-path");
            });

            Ok(())
        });

        assert!(matches!(
            result,
            Err(ComponentsObjectError::PathSetWithoutProductionExtension)
        ));
    }

    #[test]
    fn test_add_composed_part_object_errors_with_production_extension() {
        let mut builder = ModelBuilder::new(Unit::Millimeter, true);
        builder.make_production_extension_required().unwrap(); // dont expect to return error;
        let obj1_id = builder
            .add_mesh_object(|obj| {
                obj.name("obj1");
                obj.uuid("some-mesh-uuid");

                Ok(())
            })
            .unwrap();

        //Test missing Object Uuid
        let result = builder.add_components_object(|obj| {
            obj.add_component_advanced(obj1_id, |c| {
                c.uuid("some-component-uuid").path("some-component-path");
            });
            Ok(())
        });

        assert!(matches!(
            result,
            Err(ComponentsObjectError::ObjectUuidNotSet)
        ));

        //Test missing Component Uuid
        let result = builder.add_components_object(|obj| {
            obj.uuid("some-obj-uuid");
            obj.add_component_advanced(obj1_id, |c| {
                c.path("some-component-path");
            });
            Ok(())
        });

        assert!(matches!(
            result,
            Err(ComponentsObjectError::ComponentUuidNotSet)
        ));
    }

    #[test]
    fn test_root_nonroot_tests() {
        // Non-root cannot have build
        let mut builder = ModelBuilder::new(Unit::Millimeter, false);
        assert!(matches!(
            builder.add_build(None),
            Err(ModelError::BuildOnlyAllowedInRootModel)
        ));

        // Root requires build
        let mut builder = ModelBuilder::new(Unit::Millimeter, true);
        builder.add_mesh_object(|obj| Ok(())).unwrap();
        assert!(matches!(builder.build(), Err(ModelError::BuildItemNotSet)));

        // make_root(false) allows no build
        let mut builder = ModelBuilder::new(Unit::Millimeter, true);
        builder.make_root(false);
        builder.add_mesh_object(|obj| Ok(())).unwrap();
        let model = builder.build().unwrap();
        assert!(model.build.item.is_empty());
    }

    #[test]
    fn test_metadata_tests() {
        let mut builder = ModelBuilder::new(Unit::Millimeter, true);
        builder.add_metadata("key1", Some("value1"));
        builder.add_metadata("key2", None);
        builder.add_metadata("key3", Some("value3"));
        builder.add_build(None).unwrap();
        let model = builder.build().unwrap();
        assert_eq!(model.metadata.len(), 3);
        assert_eq!(model.metadata[0].name, "key1");
        assert_eq!(model.metadata[0].value, Some("value1".to_string()));
        assert_eq!(model.metadata[1].name, "key2");
        assert_eq!(model.metadata[1].value, None);
        assert_eq!(model.metadata[2].name, "key3");
        assert_eq!(model.metadata[2].value, Some("value3".to_string()));
    }

    #[test]
    fn test_object_id_tests() {
        let id: ObjectId = 42.into();
        assert_eq!(id.0, 42);
        let usize_id: usize = id.into();
        assert_eq!(usize_id, 42);
    }
}
