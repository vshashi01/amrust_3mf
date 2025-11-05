#[cfg(any(
    feature = "io-memory-optimized-read",
    feature = "io-speed-optimized-read",
    feature = "io-lazy-read"
))]
#[cfg(test)]
mod smoke_tests {
    use pretty_assertions::assert_eq;

    use std::{fs::File, path::PathBuf};

    #[cfg(feature = "io-memory-optimized-read")]
    #[test]
    fn read_threemf_package_memory_optimized() {
        use amrust_3mf::io::ThreemfPackage;
        use amrust_3mf::io::query::get_composedpart_objects;
        use amrust_3mf::io::query::get_mesh_objects;
        use amrust_3mf::io::query::get_object_ref_from_id;
        use amrust_3mf::io::query::get_objects;

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

    #[cfg(feature = "io-speed-optimized-read")]
    #[test]
    fn read_threemf_package_speed_optimized() {
        use amrust_3mf::io::ThreemfPackage;
        use amrust_3mf::io::query::get_composedpart_objects;
        use amrust_3mf::io::query::get_mesh_objects;
        use amrust_3mf::io::query::get_object_ref_from_id;
        use amrust_3mf::io::query::get_objects;

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

    #[cfg(feature = "io-lazy-read")]
    #[test]
    fn read_threemf_package_lazy_memory_optimized() {
        use amrust_3mf::io::{CachePolicy, ThreemfPackageLazyReader};

        let path = PathBuf::from("./tests/data/mesh-composedpart-separate-model-files.3mf");
        let reader = File::open(path).unwrap();

        let result = ThreemfPackageLazyReader::from_reader_with_memory_optimized_deserializer(
            reader,
            CachePolicy::NoCache,
        );

        assert!(result.is_ok());

        match result {
            Ok(package) => {
                assert_eq!(package.relationships().len(), 4);

                // Count total objects using with_model pattern
                let mut total_objects = 0;
                for model_path in package.model_paths() {
                    package
                        .with_model(model_path, |model| {
                            total_objects += model.resources.object.len();
                        })
                        .unwrap();
                }
                assert_eq!(total_objects, 4);

                // Count mesh objects and check UUID using with_model pattern
                let mut mesh_objects = 0;
                let mut can_find_object_by_uuid = false;
                for model_path in package.model_paths() {
                    package
                        .with_model(model_path, |model| {
                            for obj in &model.resources.object {
                                if obj.mesh.is_some() {
                                    mesh_objects += 1;
                                    if let Some(uuid) = &obj.uuid
                                        && uuid == "79f98073-4eaa-4737-b065-041b98fb50a6"
                                    {
                                        can_find_object_by_uuid = true;
                                    }
                                }
                            }
                        })
                        .unwrap();
                }
                assert_eq!(mesh_objects, 3);
                assert!(can_find_object_by_uuid);

                // Count composedpart objects using with_model pattern
                let mut composedpart_objects = 0;
                for model_path in package.model_paths() {
                    package
                        .with_model(model_path, |model| {
                            composedpart_objects += model
                                .resources
                                .object
                                .iter()
                                .filter(|o| o.components.is_some())
                                .count();
                        })
                        .unwrap();
                }
                assert_eq!(composedpart_objects, 1);

                // Check object by ID in specific model path
                let mut found_object = false;
                package
                    .with_model("/3D/Objects/Object.model", |model| {
                        if model.resources.object.iter().any(|o| o.id == 1) {
                            found_object = true;
                        }
                    })
                    .unwrap();
                assert!(found_object);

                // Check build item UUID in root model
                let root_model = package.root_model().unwrap();
                let can_find_build_item_by_uuid = root_model.build.item.iter().find(|i| {
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
