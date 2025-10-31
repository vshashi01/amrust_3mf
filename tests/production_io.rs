#[cfg(feature = "io")]
#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use amrust_3mf::io::ThreemfPackage;
    use amrust_3mf::io::query::{
        get_composedpart_objects, get_mesh_objects, get_object_ref_from_id, get_objects,
    };

    use std::{fs::File, path::PathBuf};

    #[cfg(feature = "memory-optimized-read")]
    #[test]
    fn read_threemf_package_memory_optimized() {
        let path = PathBuf::from("./tests/data/mesh-composedpart-separate-model-files.3mf");
        let reader = File::open(path).unwrap();

        let result = ThreemfPackage::from_reader_with_memory_optimized_deserializer(reader, true);

        assert!(result.is_ok());

        match result {
            Ok(package) => {
                assert_eq!(package.relationships.len(), 4);

                let objects = get_objects(&package).collect::<Vec<_>>();
                assert_eq!(objects.len(), 4);

                let mesh_objects = get_mesh_objects(&package).collect::<Vec<_>>();
                let can_find_object_by_uuid = mesh_objects.iter().find(|o| {
                    if let Some(uuid) = &o.uuid {
                        uuid == "79f98073-4eaa-4737-b065-041b98fb50a6"
                    } else {
                        false
                    }
                });
                assert_eq!(mesh_objects.len(), 3);
                assert!(can_find_object_by_uuid.is_some());

                let composedpart_objects = get_composedpart_objects(&package).collect::<Vec<_>>();
                assert_eq!(composedpart_objects.len(), 1);

                let object_by_id = get_object_ref_from_id(
                    1,
                    &package,
                    Some("/3D/Objects/Object.model".to_string()),
                    None,
                );
                assert!(object_by_id.0.is_some());

                let can_find_build_item_by_uuid = package.root.build.item.iter().find(|i| {
                    if let Some(uuid) = &i.uuid {
                        uuid == "637f47fa-39e6-4363-b3a9-100329fc5d9c"
                    } else {
                        false
                    }
                });
                assert!(can_find_build_item_by_uuid.is_some());
            }
            Err(err) => {
                panic!("read failed {:?}", err);
            }
        }
    }

    #[cfg(feature = "speed-optimized-read")]
    #[test]
    fn read_threemf_package_speed_optimized() {
        let path = PathBuf::from("./tests/data/mesh-composedpart-separate-model-files.3mf");
        let reader = File::open(path).unwrap();

        let result = ThreemfPackage::from_reader_with_speed_optimized_deserializer(reader, true);

        assert!(result.is_ok());

        match result {
            Ok(package) => {
                assert_eq!(package.relationships.len(), 4);

                let objects = get_objects(&package).collect::<Vec<_>>();
                assert_eq!(objects.len(), 4);

                let mesh_objects = get_mesh_objects(&package).collect::<Vec<_>>();
                let can_find_object_by_uuid = mesh_objects.iter().find(|o| {
                    if let Some(uuid) = &o.uuid {
                        uuid == "79f98073-4eaa-4737-b065-041b98fb50a6"
                    } else {
                        false
                    }
                });
                assert_eq!(mesh_objects.len(), 3);
                assert!(can_find_object_by_uuid.is_some());

                let composedpart_objects = get_composedpart_objects(&package).collect::<Vec<_>>();
                assert_eq!(composedpart_objects.len(), 1);

                let object_by_id = get_object_ref_from_id(
                    1,
                    &package,
                    Some("/3D/Objects/Object.model".to_string()),
                    None,
                );
                assert!(object_by_id.0.is_some());

                let can_find_build_item_by_uuid = package.root.build.item.iter().find(|i| {
                    if let Some(uuid) = &i.uuid {
                        uuid == "637f47fa-39e6-4363-b3a9-100329fc5d9c"
                    } else {
                        false
                    }
                });
                assert!(can_find_build_item_by_uuid.is_some());
            }
            Err(err) => {
                panic!("read failed {:?}", err);
            }
        }
    }
}
