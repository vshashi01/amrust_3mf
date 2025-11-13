#![allow(clippy::needless_lifetimes)]

use crate::{
    core::{
        beamlattice::BeamLattice,
        component::Components,
        mesh::Mesh,
        model::Model,
        object::{Object, ObjectType},
    },
    io::ThreemfPackage,
};

pub struct ObjectRef<'a> {
    pub object: &'a Object,
    pub path: Option<&'a str>,
}

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

pub fn get_objects<'a>(package: &'a ThreemfPackage) -> impl Iterator<Item = ObjectRef<'a>> {
    iter_objects_from(package, get_objects_from_model_ref)
}

pub fn get_objects_from_model<'a>(model: &'a Model) -> impl Iterator<Item = ObjectRef<'a>> {
    get_objects_from_model_ref(ModelRef { model, path: None })
}

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

pub struct GenericObjectRef<'a, T> {
    pub entity: &'a T,
    pub id: usize,
    pub object_type: ObjectType,
    pub thumbnail: Option<String>,
    pub part_number: Option<String>,
    pub name: Option<String>,
    pub pid: Option<usize>,
    pub pindex: Option<usize>,
    pub uuid: Option<String>,
    pub origin_model_path: Option<&'a str>,
}

pub type MeshObjectRef<'a> = GenericObjectRef<'a, Mesh>;

impl<'a> MeshObjectRef<'a> {
    fn new(o: ObjectRef<'a>) -> Self {
        MeshObjectRef {
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
        }
    }
}

pub fn get_mesh_objects<'a>(
    package: &'a ThreemfPackage,
) -> impl Iterator<Item = MeshObjectRef<'a>> {
    iter_objects_from(package, get_mesh_objects_from_model_ref).map(MeshObjectRef::new)
}

pub fn get_mesh_objects_from_model<'a>(
    model: &'a Model,
) -> impl Iterator<Item = MeshObjectRef<'a>> {
    get_mesh_objects_from_model_ref(ModelRef { model, path: None }).map(MeshObjectRef::new)
}

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

pub type ComposedPartObjectRef<'a> = GenericObjectRef<'a, Components>;

impl<'a> ComposedPartObjectRef<'a> {
    fn new(o: ObjectRef<'a>) -> Self {
        ComposedPartObjectRef {
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
        }
    }
}

pub fn get_composedpart_objects<'a>(
    package: &'a ThreemfPackage,
) -> impl Iterator<Item = ComposedPartObjectRef<'a>> {
    iter_objects_from(package, get_composedpart_objects_from_model_ref)
        .map(ComposedPartObjectRef::new)
}

pub fn get_composedpart_objects_from_model<'a>(
    model: &'a Model,
) -> impl Iterator<Item = ComposedPartObjectRef<'a>> {
    get_composedpart_objects_from_model_ref(ModelRef { model, path: None })
        .map(ComposedPartObjectRef::new)
}

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

pub type BeamLatticeObjectRef<'a> = GenericObjectRef<'a, BeamLattice>;

impl<'a> BeamLatticeObjectRef<'a> {
    fn new(o: ObjectRef<'a>) -> Self {
        BeamLatticeObjectRef {
            entity: o
                .object
                .mesh
                .as_ref()
                .unwrap()
                .beamlattice
                .as_ref()
                .unwrap(),
            id: o.object.id,
            object_type: o.object.objecttype.unwrap_or(ObjectType::Model),
            thumbnail: o.object.thumbnail.clone(),
            part_number: o.object.partnumber.clone(),
            name: o.object.name.clone(),
            pid: o.object.pid,
            pindex: o.object.pindex,
            uuid: o.object.uuid.clone(),
            origin_model_path: o.path,
        }
    }
}

pub fn get_beam_lattice_objects<'a>(
    package: &'a ThreemfPackage,
) -> impl Iterator<Item = BeamLatticeObjectRef<'a>> {
    iter_objects_from(package, get_beam_lattice_objects_from_model_ref)
        .map(BeamLatticeObjectRef::new)
}

pub fn get_beam_lattice_objects_from_model<'a>(
    model: &'a Model,
) -> impl Iterator<Item = BeamLatticeObjectRef<'a>> {
    get_beam_lattice_objects_from_model_ref(ModelRef { model, path: None })
        .map(BeamLatticeObjectRef::new)
}

pub fn get_beam_lattice_objects_from_model_ref<'a>(
    model_ref: ModelRef<'a>,
) -> impl Iterator<Item = ObjectRef<'a>> {
    model_ref
        .model
        .resources
        .object
        .iter()
        .filter(|o| {
            if let Some(mesh) = &o.mesh {
                mesh.beamlattice.is_some()
            } else {
                false
            }
        })
        .map(move |o| ObjectRef {
            object: o,
            path: model_ref.path,
        })
}

pub struct ModelRef<'a> {
    pub model: &'a Model,
    pub path: Option<&'a str>,
}

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

        let objects = get_beam_lattice_objects(&package).collect::<Vec<_>>();
        assert_eq!(objects.len(), 2);
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

        let objects = get_beam_lattice_objects_from_model(&model).collect::<Vec<_>>();
        assert_eq!(objects.len(), 2)
    }
}
