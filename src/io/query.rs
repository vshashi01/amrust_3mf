#![allow(clippy::needless_lifetimes)]

use crate::{
    core::{model::Model, object::Object},
    io::ThreemfPackage,
};

/// Gets a reference to the object and the path to the container Parent Model from some
/// object id and the parent model path.
/// If path is not specified, then the parent model is the default place to look for the object
/// If the parent model path is not specified then the root model is always the core search model
pub fn get_object_ref_from_id<'a>(
    object_id: usize,
    package: &'a ThreemfPackage,
    path: Option<String>,
    parent_model: Option<String>,
) -> (Option<&'a Object>, Option<String>) {
    match path {
        Some(sub_model_path) => {
            if let Some(model) = package.sub_models.get(&sub_model_path) {
                (
                    get_object_ref_from_model(object_id, model),
                    Some(sub_model_path.clone()),
                )
            } else {
                (None, None)
            }
        }
        None => match parent_model {
            Some(model_path) => {
                if let Some(model) = package.sub_models.get(&model_path) {
                    (
                        get_object_ref_from_model(object_id, model),
                        Some(model_path.clone()),
                    )
                } else {
                    (None, None)
                }
            }
            None => (get_object_ref_from_model(object_id, &package.root), None),
        },
    }
}

pub fn get_object_ref_from_model(object_id: usize, model: &Model) -> Option<&Object> {
    model.resources.object.iter().find(|o| o.id == object_id)
}

pub fn get_objects<'a>(package: &'a ThreemfPackage) -> impl Iterator<Item = &'a Object> {
    iter_objects_from(package, get_objects_from_model)
}

pub fn get_objects_from_model<'a>(model: &'a Model) -> impl Iterator<Item = &'a Object> {
    model.resources.object.iter()
}

pub fn get_mesh_objects<'a>(package: &'a ThreemfPackage) -> impl Iterator<Item = &'a Object> {
    iter_objects_from(package, get_mesh_objects_from_model)
}

pub fn get_mesh_objects_from_model<'a>(model: &'a Model) -> impl Iterator<Item = &'a Object> {
    model.resources.object.iter().filter(|o| o.mesh.is_some())
}

pub fn get_composedpart_objects<'a>(
    package: &'a ThreemfPackage,
) -> impl Iterator<Item = &'a Object> {
    iter_objects_from(package, get_composedpart_objects_from_model)
}

pub fn get_composedpart_objects_from_model<'a>(
    model: &'a Model,
) -> impl Iterator<Item = &'a Object> {
    model
        .resources
        .object
        .iter()
        .filter(|o| o.components.is_some())
}

pub fn get_beam_lattice_objects<'a>(
    package: &'a ThreemfPackage,
) -> impl Iterator<Item = &'a Object> {
    iter_objects_from(package, get_beam_lattice_objects_from_model)
}

pub fn get_beam_lattice_objects_from_model<'a>(
    model: &'a Model,
) -> impl Iterator<Item = &'a Object> {
    model.resources.object.iter().filter(|o| {
        if let Some(mesh) = &o.mesh {
            mesh.beamlattice.is_some()
        } else {
            false
        }
    })
}

pub fn iter_models<'a>(package: &'a ThreemfPackage) -> impl Iterator<Item = &'a Model> {
    std::iter::once(&package.root).chain(package.sub_models.values())
}

fn iter_objects_from<'a, I, F>(
    package: &'a ThreemfPackage,
    f: F,
) -> impl Iterator<Item = &'a Object>
where
    F: Fn(&'a Model) -> I + Copy,
    I: IntoIterator<Item = &'a Object>,
{
    iter_models(package).flat_map(f)
}

#[cfg(feature = "memory-optimized-read")]
#[cfg(test)]
mod tests {
    use instant_xml::from_str;
    use std::path::PathBuf;

    use super::*;
    use crate::core::model::Model;

    #[test]
    fn test_get_object_ref_from_package() {
        let path =
            PathBuf::from("tests/data/mesh-composedpart-beamlattice-separate-model-files.3mf")
                .canonicalize()
                .unwrap();
        let file = std::fs::File::open(path).unwrap();
        let package =
            ThreemfPackage::from_reader_with_memory_optimized_deserializer(file, true).unwrap();

        let (object, _) = get_object_ref_from_id(
            1,
            &package,
            Some("/3D/Objects/Object.model".to_string()),
            None,
        );

        match object {
            Some(obj) => {
                assert!(obj.mesh.is_some());
                assert_eq!(obj.id, 1);
            }
            None => panic!("Object ref not found"),
        }
    }

    #[test]
    fn test_get_object_ref_from_model() {
        let path = PathBuf::from("tests/data/mesh-composedpart-beamlattice.model")
            .canonicalize()
            .unwrap();
        let text = std::fs::read_to_string(path).unwrap();
        let model = from_str::<Model>(&text).unwrap();

        let object = get_object_ref_from_model(1, &model);

        match object {
            Some(obj) => {
                assert!(obj.mesh.is_some());
                assert_eq!(obj.id, 1);
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
    fn test_get_objects_from_model() {
        let path = PathBuf::from("tests/data/mesh-composedpart-beamlattice.model")
            .canonicalize()
            .unwrap();
        let text = std::fs::read_to_string(path).unwrap();
        let model = from_str::<Model>(&text).unwrap();

        let objects = get_objects_from_model(&model).collect::<Vec<_>>();
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
    fn test_get_mesh_objects_from_model() {
        let path = PathBuf::from("tests/data/mesh-composedpart-beamlattice.model")
            .canonicalize()
            .unwrap();
        let text = std::fs::read_to_string(path).unwrap();
        let model = from_str::<Model>(&text).unwrap();

        let objects = get_mesh_objects_from_model(&model).collect::<Vec<_>>();
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
    fn test_get_composedpart_objects_from_model() {
        let path = PathBuf::from("tests/data/mesh-composedpart-beamlattice.model")
            .canonicalize()
            .unwrap();
        let text = std::fs::read_to_string(path).unwrap();
        let model = from_str::<Model>(&text).unwrap();

        let objects = get_composedpart_objects_from_model(&model).collect::<Vec<_>>();
        assert_eq!(objects.len(), 1)
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
    fn test_get_beamlattice_objects_from_model() {
        let path = PathBuf::from("tests/data/mesh-composedpart-beamlattice.model")
            .canonicalize()
            .unwrap();
        let text = std::fs::read_to_string(path).unwrap();
        let model = from_str::<Model>(&text).unwrap();

        let objects = get_beam_lattice_objects_from_model(&model).collect::<Vec<_>>();
        assert_eq!(objects.len(), 2)
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
