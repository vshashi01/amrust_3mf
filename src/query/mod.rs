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
    path: &Option<String>,
    parent_model: &Option<String>,
) -> (Option<&'a Object>, Option<String>) {
    match path {
        Some(sub_model_path) => {
            if let Some(model) = package.sub_models.get(sub_model_path) {
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
                if let Some(model) = package.sub_models.get(model_path) {
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

pub fn get_mesh_objects<'a>(package: &'a ThreemfPackage) -> impl Iterator<Item = &'a Object> {
    std::iter::once(&package.root)
        .chain(package.sub_models.values())
        .flat_map(get_mesh_objects_from_model)
}

pub fn get_mesh_objects_from_model<'a>(model: &'a Model) -> impl Iterator<Item = &'a Object> {
    model.resources.object.iter().filter(|o| o.mesh.is_some())
}

pub fn get_composedpart_objects<'a>(
    package: &'a ThreemfPackage,
) -> impl Iterator<Item = &'a Object> {
    std::iter::once(&package.root)
        .chain(package.sub_models.values())
        .flat_map(get_composedpart_objects_from_model)
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
    std::iter::once(&package.root)
        .chain(package.sub_models.values())
        .flat_map(get_beam_lattice_objects_from_model)
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
