#![allow(clippy::needless_lifetimes)]

use std::ops::Deref;

use crate::{
    core::{
        build::Item,
        component::Components,
        mesh::Mesh,
        model::Model,
        object::{Object, ObjectType},
        transform::Transform,
    },
    io::ThreemfPackage,
};

/// A reference to an object within a 3MF model, including its path if from a sub-model.
pub struct ObjectRef<'a> {
    /// The object itself.
    pub object: &'a Object,
    /// The path to the model containing this object, if None then it is the root model.
    pub path: Option<&'a str>,
}

/// Retrieves an object by ID from a given model. Returns None if not found.
pub fn get_object_from_model<'a>(object_id: usize, model: &'a Model) -> Option<ObjectRef<'a>> {
    model
        .resources
        .object
        .iter()
        .find(|o| o.id == object_id)
        .map(|lala| ObjectRef {
            object: lala,
            path: None,
        })
}

/// Returns an iterator over all objects in the package, including sub-models.
pub fn get_objects<'a>(package: &'a ThreemfPackage) -> impl Iterator<Item = ObjectRef<'a>> {
    iter_objects_from(package, get_objects_from_model_ref)
}

/// Returns an iterator over all objects in the model.
pub fn get_objects_from_model<'a>(model: &'a Model) -> impl Iterator<Item = ObjectRef<'a>> {
    get_objects_from_model_ref(ModelRef { model, path: None })
}

/// Returns an iterator over all objects in the model reference.
pub fn get_objects_from_model_ref<'a>(
    model_ref: ModelRef<'a>,
) -> impl Iterator<Item = ObjectRef<'a>> {
    model_ref
        .model
        .resources
        .object
        .iter()
        .map(move |o| ObjectRef {
            object: o,
            path: model_ref.path,
        })
}

/// A generic reference to an object entity with common metadata fields.
pub struct GenericObjectRef<'a, T> {
    /// The entity itself (e.g., Mesh, Components).
    entity: &'a T,
    pub id: usize,
    pub object_type: ObjectType,
    pub thumbnail: Option<String>,
    pub part_number: Option<String>,
    pub name: Option<String>,
    pub pid: Option<usize>,
    pub pindex: Option<usize>,
    pub uuid: Option<String>,
    /// Path to the originating model.
    pub origin_model_path: Option<&'a str>,
}

/// A reference to a mesh object with all the object data.
pub struct MeshObjectRef<'a>(GenericObjectRef<'a, Mesh>);

impl<'a> MeshObjectRef<'a> {
    fn new(o: ObjectRef<'a>) -> Self {
        MeshObjectRef(GenericObjectRef {
            entity: o.object.mesh.as_ref().unwrap(),
            id: o.object.id,
            object_type: o.object.objecttype.unwrap_or(ObjectType::Model),
            thumbnail: o.object.thumbnail.clone(),
            part_number: o.object.partnumber.clone(),
            name: o.object.name.clone(),
            pid: o.object.pid,
            pindex: o.object.pindex,
            uuid: o.object.uuid.clone(),
            origin_model_path: o.path,
        })
    }

    /// Returns the mesh data.
    pub fn mesh(&self) -> &'a Mesh {
        self.entity
    }
}

impl<'a> Deref for MeshObjectRef<'a> {
    type Target = GenericObjectRef<'a, Mesh>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// Returns an iterator over mesh objects in the package.
pub fn get_mesh_objects<'a>(
    package: &'a ThreemfPackage,
) -> impl Iterator<Item = MeshObjectRef<'a>> {
    iter_objects_from(package, get_mesh_objects_from_model_ref).map(MeshObjectRef::new)
}

/// Returns an iterator over mesh objects in the model.
pub fn get_mesh_objects_from_model<'a>(
    model: &'a Model,
) -> impl Iterator<Item = MeshObjectRef<'a>> {
    get_mesh_objects_from_model_ref(ModelRef { model, path: None }).map(MeshObjectRef::new)
}

/// Returns an iterator over mesh objects in the model reference.
pub fn get_mesh_objects_from_model_ref<'a>(
    model_ref: ModelRef<'a>,
) -> impl Iterator<Item = ObjectRef<'a>> {
    model_ref
        .model
        .resources
        .object
        .iter()
        .filter(|o| o.mesh.is_some())
        .map(move |o| ObjectRef {
            object: o,
            path: model_ref.path,
        })
}

/// A reference to a composed part object with metadata.
pub struct ComposedPartObjectRef<'a>(GenericObjectRef<'a, Components>);

impl<'a> ComposedPartObjectRef<'a> {
    fn new(o: ObjectRef<'a>) -> Self {
        ComposedPartObjectRef(GenericObjectRef {
            entity: o.object.components.as_ref().unwrap(),
            id: o.object.id,
            object_type: o.object.objecttype.unwrap_or(ObjectType::Model),
            thumbnail: o.object.thumbnail.clone(),
            part_number: o.object.partnumber.clone(),
            name: o.object.name.clone(),
            pid: o.object.pid,
            pindex: o.object.pindex,
            uuid: o.object.uuid.clone(),
            origin_model_path: o.path,
        })
    }

    /// Returns an iterator over the components.
    pub fn components(&self) -> impl Iterator<Item = ComponentRef> {
        self.entity.component.iter().map(|c| {
            let comp_path = match &c.path {
                Some(path) => Some(path.clone()),
                None => self
                    .origin_model_path
                    .map(|parent_path| parent_path.to_owned()),
            };

            ComponentRef {
                objectid: c.objectid,
                transform: c.transform.clone(),
                path_to_look_for: comp_path,
                uuid: c.uuid.clone(),
            }
        })
    }
}

impl<'a> Deref for ComposedPartObjectRef<'a> {
    type Target = GenericObjectRef<'a, Components>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// A reference to a component within a composed part.
pub struct ComponentRef {
    /// ID of the referenced object.
    pub objectid: usize,
    /// Path to look for the object,
    /// if specified else it will be the parent Model where the object is originating from.
    pub path_to_look_for: Option<String>,
    /// Transform applied to the component.
    pub transform: Option<Transform>,
    /// UUID of the component.
    pub uuid: Option<String>,
}

/// A reference to a build item within a 3MF model, including its path if from a sub-model.
pub struct ItemRef<'a> {
    /// The item itself.
    pub item: &'a Item,
    /// The path to the model containing this item, if None then it is the root model.
    pub origin_model_path: Option<&'a str>,
}

impl<'a> ItemRef<'a> {
    /// Returns the objectid that this item references.
    pub fn objectid(&self) -> usize {
        self.item.objectid
    }

    /// Returns the transform applied to this item.
    pub fn transform(&self) -> Option<&Transform> {
        self.item.transform.as_ref()
    }

    /// Returns the part number of this item.
    pub fn partnumber(&self) -> Option<&str> {
        self.item.partnumber.as_deref()
    }

    /// Returns the path attribute (production extension) for cross-model references.
    pub fn path(&self) -> Option<&str> {
        self.item.path.as_deref()
    }

    /// Returns the UUID of this item (production extension).
    pub fn uuid(&self) -> Option<&str> {
        self.item.uuid.as_deref()
    }
}

/// Returns an iterator over composed part objects in the package.
pub fn get_composedpart_objects<'a>(
    package: &'a ThreemfPackage,
) -> impl Iterator<Item = ComposedPartObjectRef<'a>> {
    iter_objects_from(package, get_composedpart_objects_from_model_ref)
        .map(ComposedPartObjectRef::new)
}

/// Returns an iterator over composed part objects in the model.
pub fn get_composedpart_objects_from_model<'a>(
    model: &'a Model,
) -> impl Iterator<Item = ComposedPartObjectRef<'a>> {
    get_composedpart_objects_from_model_ref(ModelRef { model, path: None })
        .map(ComposedPartObjectRef::new)
}

/// Returns an iterator over composed part objects in the model reference.
pub fn get_composedpart_objects_from_model_ref<'a>(
    model_ref: ModelRef<'a>,
) -> impl Iterator<Item = ObjectRef<'a>> {
    model_ref
        .model
        .resources
        .object
        .iter()
        .filter(|o| o.components.is_some())
        .map(move |o| ObjectRef {
            object: o,
            path: model_ref.path,
        })
}

/// Returns an iterator over all build items in the package, including sub-models.
pub fn get_items<'a>(package: &'a ThreemfPackage) -> impl Iterator<Item = ItemRef<'a>> {
    iter_models(package).flat_map(get_items_from_model_ref)
}

/// Returns an iterator over all build items in the model.
pub fn get_items_from_model<'a>(model: &'a Model) -> impl Iterator<Item = ItemRef<'a>> {
    get_items_from_model_ref(ModelRef { model, path: None })
}

/// Returns an iterator over all build items in the model reference.
pub fn get_items_from_model_ref<'a>(model_ref: ModelRef<'a>) -> impl Iterator<Item = ItemRef<'a>> {
    model_ref.model.build.item.iter().map(move |item| ItemRef {
        item,
        origin_model_path: model_ref.path,
    })
}

/// Returns an iterator over build items that reference a specific object ID.
/// Note: Multiple items can reference the same object ID.
pub fn get_items_by_objectid<'a>(
    package: &'a ThreemfPackage,
    objectid: usize,
) -> impl Iterator<Item = ItemRef<'a>> {
    get_items(package).filter(move |item_ref| item_ref.item.objectid == objectid)
}

/// Finds a build item by its UUID (production extension).
/// Returns None if not found. UUIDs should be unique across the package.
pub fn get_item_by_uuid<'a>(package: &'a ThreemfPackage, uuid: &str) -> Option<ItemRef<'a>> {
    get_items(package).find(|item_ref| {
        if let Some(item_uuid) = &item_ref.item.uuid {
            item_uuid == uuid
        } else {
            false
        }
    })
}

/// A reference to a model within a package, including its path.
pub struct ModelRef<'a> {
    /// The model itself.
    pub model: &'a Model,
    /// The path to the model, if it's a sub-model.
    pub path: Option<&'a str>,
}

/// Returns an iterator over all models in the package, including the root and sub-models.
pub fn iter_models<'a>(package: &'a ThreemfPackage) -> impl Iterator<Item = ModelRef<'a>> {
    std::iter::once(ModelRef {
        model: &package.root,
        path: None,
    })
    .chain(package.sub_models.iter().map(|(path, model)| ModelRef {
        model,
        path: Some(path),
    }))
}

fn iter_objects_from<'a, I, F>(
    package: &'a ThreemfPackage,
    f: F,
) -> impl Iterator<Item = ObjectRef<'a>>
where
    F: Fn(ModelRef<'a>) -> I + Copy,
    I: IntoIterator<Item = ObjectRef<'a>>,
{
    iter_models(package).flat_map(f)
}

#[cfg(feature = "io-memory-optimized-read")]
#[cfg(test)]
mod smoke_tests {
    use super::*;

    use std::path::PathBuf;

    #[test]
    fn test_get_object_ref_from_package() {
        let path =
            PathBuf::from("tests/data/mesh-composedpart-beamlattice-separate-model-files.3mf")
                .canonicalize()
                .unwrap();
        let file = std::fs::File::open(path).unwrap();
        let package =
            ThreemfPackage::from_reader_with_memory_optimized_deserializer(file, true).unwrap();

        let object_ref = get_objects(&package)
            .filter(|r| matches!(r.path, Some("/3D/Objects/Object.model")))
            .find(|r| r.object.id == 1);

        match object_ref {
            Some(obj_ref) => {
                assert!(obj_ref.object.mesh.is_some());
                assert_eq!(obj_ref.object.id, 1);
            }
            None => panic!("Object ref not found"),
        }
    }

    #[test]
    fn test_get_objects_from_package() {
        let path =
            PathBuf::from("tests/data/mesh-composedpart-beamlattice-separate-model-files.3mf")
                .canonicalize()
                .unwrap();
        let file = std::fs::File::open(path).unwrap();
        let package =
            ThreemfPackage::from_reader_with_memory_optimized_deserializer(file, true).unwrap();

        let objects = get_objects(&package).collect::<Vec<_>>();
        assert_eq!(objects.len(), 6);
    }

    #[test]
    fn test_get_mesh_objects_from_package() {
        let path =
            PathBuf::from("tests/data/mesh-composedpart-beamlattice-separate-model-files.3mf")
                .canonicalize()
                .unwrap();
        let file = std::fs::File::open(path).unwrap();
        let package =
            ThreemfPackage::from_reader_with_memory_optimized_deserializer(file, true).unwrap();

        let objects = get_mesh_objects(&package).collect::<Vec<_>>();
        assert_eq!(objects.len(), 5);
    }

    #[test]
    fn test_get_composedpart_objects_from_package() {
        let path =
            PathBuf::from("tests/data/mesh-composedpart-beamlattice-separate-model-files.3mf")
                .canonicalize()
                .unwrap();
        let file = std::fs::File::open(path).unwrap();
        let package =
            ThreemfPackage::from_reader_with_memory_optimized_deserializer(file, true).unwrap();

        let objects = get_composedpart_objects(&package).collect::<Vec<_>>();
        assert_eq!(objects.len(), 1);
    }

    #[test]
    fn test_get_beamlattice_objects_from_package() {
        let path =
            PathBuf::from("tests/data/mesh-composedpart-beamlattice-separate-model-files.3mf")
                .canonicalize()
                .unwrap();
        let file = std::fs::File::open(path).unwrap();
        let package =
            ThreemfPackage::from_reader_with_memory_optimized_deserializer(file, true).unwrap();

        let objects = get_mesh_objects(&package)
            .filter(|mesh_ref| mesh_ref.mesh().beamlattice.is_some())
            .count();
        assert_eq!(objects, 2);
    }

    #[test]
    fn test_iter_models_from_package() {
        let path =
            PathBuf::from("tests/data/mesh-composedpart-beamlattice-separate-model-files.3mf")
                .canonicalize()
                .unwrap();
        let file = std::fs::File::open(path).unwrap();
        let package =
            ThreemfPackage::from_reader_with_memory_optimized_deserializer(file, true).unwrap();

        let models = iter_models(&package).collect::<Vec<_>>();
        assert_eq!(models.len(), 5);
        assert!(models[0].path.is_none());
        for model_ref in &models[1..] {
            assert!(model_ref.path.is_some());
        }
    }

    #[test]
    fn test_integration_component_resolution() {
        let path =
            PathBuf::from("tests/data/mesh-composedpart-beamlattice-separate-model-files.3mf")
                .canonicalize()
                .unwrap();
        let file = std::fs::File::open(path).unwrap();
        let package =
            ThreemfPackage::from_reader_with_memory_optimized_deserializer(file, true).unwrap();

        let composed_objects = get_composedpart_objects(&package).collect::<Vec<_>>();
        assert_eq!(composed_objects.len(), 1);
        let components = composed_objects[0].components().collect::<Vec<_>>();
        assert!(!components.is_empty());
        // Check that components have valid objectids
        for comp in components {
            assert!(comp.objectid > 0);
            // Optionally check if path is set for sub-model references
        }
    }

    #[test]
    fn test_get_items_from_package() {
        let path =
            PathBuf::from("tests/data/mesh-composedpart-beamlattice-separate-model-files.3mf")
                .canonicalize()
                .unwrap();
        let file = std::fs::File::open(path).unwrap();
        let package =
            ThreemfPackage::from_reader_with_memory_optimized_deserializer(file, true).unwrap();

        let items = get_items(&package).collect::<Vec<_>>();
        // Root model has 1 item, check if sub-models have items too
        assert!(!items.is_empty());
    }

    #[test]
    fn test_get_items_by_objectid() {
        let path =
            PathBuf::from("tests/data/mesh-composedpart-beamlattice-separate-model-files.3mf")
                .canonicalize()
                .unwrap();
        let file = std::fs::File::open(path).unwrap();
        let package =
            ThreemfPackage::from_reader_with_memory_optimized_deserializer(file, true).unwrap();

        // Get the first item's objectid
        let items = get_items(&package).collect::<Vec<_>>();
        assert!(!items.is_empty());
        let first_objectid = items[0].objectid();

        // Search for items with that objectid
        let items_with_id = get_items_by_objectid(&package, first_objectid).collect::<Vec<_>>();
        assert!(!items_with_id.is_empty());
        for item in items_with_id {
            assert_eq!(item.objectid(), first_objectid);
        }
    }

    #[test]
    fn test_item_ref_origin_model_path() {
        let path =
            PathBuf::from("tests/data/mesh-composedpart-beamlattice-separate-model-files.3mf")
                .canonicalize()
                .unwrap();
        let file = std::fs::File::open(path).unwrap();
        let package =
            ThreemfPackage::from_reader_with_memory_optimized_deserializer(file, true).unwrap();

        let items = get_items(&package).collect::<Vec<_>>();
        // At least one item should have origin_model_path = None (from root)
        let root_items = items
            .iter()
            .filter(|i| i.origin_model_path.is_none())
            .count();
        assert!(root_items > 0);
    }
}

#[cfg(feature = "memory-optimized-read")]
#[cfg(test)]
mod tests {
    use instant_xml::from_str;
    use std::path::PathBuf;

    use super::*;
    use crate::core::model::Model;

    #[test]
    fn test_get_object_ref_from_model() {
        let path = PathBuf::from("tests/data/lfs/mesh-composedpart-beamlattice.model")
            .canonicalize()
            .unwrap();
        let text = std::fs::read_to_string(path).unwrap();
        let model = from_str::<Model>(&text).unwrap();

        let object_ref = get_object_from_model(1, &model);

        match object_ref {
            Some(obj_ref) => {
                assert!(obj_ref.object.mesh.is_some());
                assert_eq!(obj_ref.object.id, 1);
            }
            None => panic!("Object ref not found"),
        }
    }

    #[test]
    fn test_get_objects_from_model() {
        let path = PathBuf::from("tests/data/lfs/mesh-composedpart-beamlattice.model")
            .canonicalize()
            .unwrap();
        let text = std::fs::read_to_string(path).unwrap();
        let model = from_str::<Model>(&text).unwrap();

        let objects = get_objects_from_model(&model).collect::<Vec<_>>();
        assert_eq!(objects.len(), 6);
    }

    #[test]
    fn test_get_mesh_objects_from_model() {
        let path = PathBuf::from("tests/data/lfs/mesh-composedpart-beamlattice.model")
            .canonicalize()
            .unwrap();
        let text = std::fs::read_to_string(path).unwrap();
        let model = from_str::<Model>(&text).unwrap();

        let objects = get_mesh_objects_from_model(&model).collect::<Vec<_>>();
        assert_eq!(objects.len(), 5);
    }

    #[test]
    fn test_get_composedpart_objects_from_model() {
        let path = PathBuf::from("tests/data/lfs/mesh-composedpart-beamlattice.model")
            .canonicalize()
            .unwrap();
        let text = std::fs::read_to_string(path).unwrap();
        let model = from_str::<Model>(&text).unwrap();

        let objects = get_composedpart_objects_from_model(&model).collect::<Vec<_>>();
        assert_eq!(objects.len(), 1)
    }

    #[test]
    fn test_get_beamlattice_objects_from_model() {
        let path = PathBuf::from("tests/data/lfs/mesh-composedpart-beamlattice.model")
            .canonicalize()
            .unwrap();
        let text = std::fs::read_to_string(path).unwrap();
        let model = from_str::<Model>(&text).unwrap();

        let objects = get_mesh_objects_from_model(&model)
            .filter(|mesh_ref| mesh_ref.mesh().beamlattice.is_some())
            .count();
        assert_eq!(objects, 2)
    }

    #[test]
    fn test_get_object_from_model_non_existent_id() {
        let path = PathBuf::from("tests/data/lfs/mesh-composedpart-beamlattice.model")
            .canonicalize()
            .unwrap();
        let text = std::fs::read_to_string(path).unwrap();
        let model = from_str::<Model>(&text).unwrap();

        let object_ref = get_object_from_model(999, &model);
        assert!(object_ref.is_none());
    }

    #[test]
    fn test_get_objects_from_model_ref() {
        let path = PathBuf::from("tests/data/lfs/mesh-composedpart-beamlattice.model")
            .canonicalize()
            .unwrap();
        let text = std::fs::read_to_string(path).unwrap();
        let model = from_str::<Model>(&text).unwrap();
        let model_ref = ModelRef {
            model: &model,
            path: Some("test_path"),
        };

        let objects = get_objects_from_model_ref(model_ref).collect::<Vec<_>>();
        assert_eq!(objects.len(), 6);
        for obj in objects {
            assert_eq!(obj.path, Some("test_path"));
        }
    }

    #[test]
    fn test_get_mesh_objects_from_model_ref() {
        let path = PathBuf::from("tests/data/lfs/mesh-composedpart-beamlattice.model")
            .canonicalize()
            .unwrap();
        let text = std::fs::read_to_string(path).unwrap();
        let model = from_str::<Model>(&text).unwrap();
        let model_ref = ModelRef {
            model: &model,
            path: None,
        };

        let objects = get_mesh_objects_from_model_ref(model_ref).collect::<Vec<_>>();
        assert_eq!(objects.len(), 5);
        for obj in objects {
            assert!(obj.object.mesh.is_some());
        }
    }

    #[test]
    fn test_mesh_object_ref_impl() {
        let path = PathBuf::from("tests/data/lfs/mesh-composedpart-beamlattice.model")
            .canonicalize()
            .unwrap();
        let text = std::fs::read_to_string(path).unwrap();
        let model = from_str::<Model>(&text).unwrap();

        let mesh_objects = get_mesh_objects_from_model(&model).collect::<Vec<_>>();
        assert!(!mesh_objects.is_empty());
        let mesh_ref = &mesh_objects[0];
        assert_eq!(mesh_ref.id, 1);
        assert!(!mesh_ref.mesh().vertices.vertex.is_empty());
    }

    #[test]
    fn test_composedpart_object_ref_impl() {
        let path = PathBuf::from("tests/data/lfs/mesh-composedpart-beamlattice.model")
            .canonicalize()
            .unwrap();
        let text = std::fs::read_to_string(path).unwrap();
        let model = from_str::<Model>(&text).unwrap();

        let composedpart_objects = get_composedpart_objects_from_model(&model).collect::<Vec<_>>();
        assert!(!composedpart_objects.is_empty());
        let composed_part = &composedpart_objects[0];
        assert_eq!(composed_part.id, 4);
        assert_eq!(composed_part.components().count(), 2);

        for comp in composed_part.components() {
            assert!(comp.objectid > 0);
        }
    }

    #[test]
    fn test_beam_lattice_object_ref_impl() {
        let path = PathBuf::from("tests/data/lfs/mesh-composedpart-beamlattice.model")
            .canonicalize()
            .unwrap();
        let text = std::fs::read_to_string(path).unwrap();
        let model = from_str::<Model>(&text).unwrap();

        let beam_objects = get_mesh_objects_from_model(&model)
            .filter(|mesh_ref| mesh_ref.mesh().beamlattice.is_some())
            .collect::<Vec<_>>();
        assert!(!beam_objects.is_empty());
        let mesh_ref = &beam_objects[0];
        assert_eq!(mesh_ref.id, 5);
        assert!(
            !mesh_ref
                .mesh()
                .beamlattice
                .as_ref()
                .unwrap()
                .beams
                .beam
                .is_empty()
        );
    }

    #[test]
    fn test_model_ref_fields() {
        let path = PathBuf::from("tests/data/lfs/mesh-composedpart-beamlattice.model")
            .canonicalize()
            .unwrap();
        let text = std::fs::read_to_string(path).unwrap();
        let model = from_str::<Model>(&text).unwrap();

        let model_ref = ModelRef {
            model: &model,
            path: Some("sub/model.model"),
        };
        assert_eq!(model_ref.path, Some("sub/model.model"));
        assert_eq!(model_ref.model as *const _, &model as *const _);
    }

    #[test]
    fn test_get_items_from_model() {
        let path = PathBuf::from("tests/data/lfs/mesh-composedpart-beamlattice.model")
            .canonicalize()
            .unwrap();
        let text = std::fs::read_to_string(path).unwrap();
        let model = from_str::<Model>(&text).unwrap();

        let items = get_items_from_model(&model).collect::<Vec<_>>();
        assert_eq!(items.len(), 4);
        assert_eq!(items[0].objectid(), 1);
        assert!(items[0].origin_model_path.is_none());
    }

    #[test]
    fn test_get_items_from_model_ref() {
        let path = PathBuf::from("tests/data/lfs/mesh-composedpart-beamlattice.model")
            .canonicalize()
            .unwrap();
        let text = std::fs::read_to_string(path).unwrap();
        let model = from_str::<Model>(&text).unwrap();
        let model_ref = ModelRef {
            model: &model,
            path: Some("sub/model.model"),
        };

        let items = get_items_from_model_ref(model_ref).collect::<Vec<_>>();
        assert_eq!(items.len(), 4);
        assert_eq!(items[0].origin_model_path, Some("sub/model.model"));
    }

    #[test]
    fn test_item_ref_methods() {
        let path = PathBuf::from("tests/data/lfs/mesh-composedpart-beamlattice.model")
            .canonicalize()
            .unwrap();
        let text = std::fs::read_to_string(path).unwrap();
        let model = from_str::<Model>(&text).unwrap();

        let items = get_items_from_model(&model).collect::<Vec<_>>();
        assert_eq!(items.len(), 4);
        let item_ref = &items[0];
        assert_eq!(item_ref.objectid(), 1);
        assert!(item_ref.transform().is_some());
        assert_eq!(item_ref.partnumber(), Some("Pyramid"));
        assert!(item_ref.path().is_none());
        assert_eq!(
            item_ref.uuid(),
            Some("4e44739e-3ba0-4639-b8ad-1eb80b1cb5a5")
        );
    }
}
