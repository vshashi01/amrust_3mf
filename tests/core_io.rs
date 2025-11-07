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

        let path = PathBuf::from("./tests/data/mesh-composedpart.3mf");
        let reader = File::open(path).unwrap();

        let result = ThreemfPackage::from_reader_with_memory_optimized_deserializer(reader, false);

        assert!(result.is_ok());

        match result {
            Ok(package) => {
                assert_eq!(package.relationships.len(), 1);

                let objects = get_objects(&package).collect::<Vec<_>>();
                assert_eq!(objects.len(), 4);

                let mesh_objects = get_mesh_objects(&package).collect::<Vec<_>>();
                assert_eq!(mesh_objects.len(), 3);

                let composedpart_objects = get_composedpart_objects(&package).collect::<Vec<_>>();
                assert_eq!(composedpart_objects.len(), 1);

                let object_by_id = get_object_ref_from_id(1, &package, None, None);
                assert!(object_by_id.0.is_some());

                assert_eq!(2, package.root.build.item.len());

                let ns = package.get_namespaces_on_model(None).unwrap();
                assert_eq!(ns.len(), 3);
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

        let path = PathBuf::from("./tests/data/mesh-composedpart.3mf");
        let reader = File::open(path).unwrap();

        let result = ThreemfPackage::from_reader_with_speed_optimized_deserializer(reader, false);

        assert!(result.is_ok());

        match result {
            Ok(package) => {
                assert_eq!(package.relationships.len(), 1);

                let objects = get_objects(&package).collect::<Vec<_>>();
                assert_eq!(objects.len(), 4);

                let mesh_objects = get_mesh_objects(&package).collect::<Vec<_>>();
                assert_eq!(mesh_objects.len(), 3);

                let composedpart_objects = get_composedpart_objects(&package).collect::<Vec<_>>();
                assert_eq!(composedpart_objects.len(), 1);

                let object_by_id = get_object_ref_from_id(1, &package, None, None);
                assert!(object_by_id.0.is_some());

                assert_eq!(2, package.root.build.item.len());

                let ns = package.get_namespaces_on_model(None).unwrap();
                assert_eq!(ns.len(), 3);
            }
            Err(err) => {
                panic!("read failed {:?}", err);
            }
        }
    }

    #[cfg(feature = "io-lazy-read")]
    #[test]
    fn read_threemf_package_lazy_memory_optimized() {
        use std::collections::HashSet;

        use amrust_3mf::io::{CachePolicy, ThreemfPackageLazyReader, XmlNamespace};

        let path = PathBuf::from("./tests/data/mesh-composedpart.3mf");
        let reader = File::open(path).unwrap();

        let result = ThreemfPackageLazyReader::from_reader_with_memory_optimized_deserializer(
            reader,
            CachePolicy::NoCache,
        );

        assert!(result.is_ok());

        let mut namespaces: HashSet<XmlNamespace> = HashSet::new();

        match result {
            Ok(package) => {
                assert_eq!(package.relationships().len(), 1);

                // Count total objects using with_model pattern
                let mut total_objects = 0;
                for model_path in package.model_paths() {
                    package
                        .with_model(model_path, |(model, ns)| {
                            total_objects += model.resources.object.len();

                            ns.iter().all(|ns| namespaces.insert(ns.clone()));
                        })
                        .unwrap();
                }
                assert_eq!(total_objects, 4);

                // Count mesh objects using with_model pattern
                let mut mesh_objects = 0;
                for model_path in package.model_paths() {
                    package
                        .with_model(model_path, |(model, ns)| {
                            mesh_objects += model
                                .resources
                                .object
                                .iter()
                                .filter(|o| o.mesh.is_some())
                                .count();

                            ns.iter().all(|ns| namespaces.insert(ns.clone()));
                        })
                        .unwrap();
                }
                assert_eq!(mesh_objects, 3);

                // Count composedpart objects using with_model pattern
                let mut composedpart_objects = 0;
                for model_path in package.model_paths() {
                    package
                        .with_model(model_path, |(model, ns)| {
                            composedpart_objects += model
                                .resources
                                .object
                                .iter()
                                .filter(|o| o.components.is_some())
                                .count();

                            ns.iter().all(|ns| namespaces.insert(ns.clone()));
                        })
                        .unwrap();
                }
                assert_eq!(composedpart_objects, 1);

                let (root_model, root_ns) = package.root_model().unwrap();
                let object_by_id = root_model.resources.object.iter().find(|o| o.id == 1);
                assert!(object_by_id.is_some());

                assert_eq!(2, root_model.build.item.len());

                root_ns.iter().all(|ns| namespaces.insert(ns.clone()));
                assert_eq!(namespaces.len(), 3);
            }
            Err(err) => {
                panic!("read failed {:?}", err);
            }
        }
    }
}
