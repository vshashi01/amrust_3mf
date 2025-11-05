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
        use amrust_3mf::io::query::get_beam_lattice_objects;
        use amrust_3mf::io::query::get_mesh_objects;

        let path = PathBuf::from("./tests/data/mesh-composedpart-beamlattice.3mf");
        let reader = File::open(path).unwrap();

        let result = ThreemfPackage::from_reader_with_memory_optimized_deserializer(reader, true);

        assert!(result.is_ok());

        match result {
            Ok(package) => {
                let mesh_obj = get_mesh_objects(&package).collect::<Vec<_>>();
                assert_eq!(mesh_obj.len(), 5);

                let beam_lattice_obj = get_beam_lattice_objects(&package).collect::<Vec<_>>();
                assert_eq!(beam_lattice_obj.len(), 2);
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
        use amrust_3mf::io::query::get_beam_lattice_objects;
        use amrust_3mf::io::query::get_mesh_objects;

        let path = PathBuf::from("./tests/data/mesh-composedpart-beamlattice.3mf");
        let reader = File::open(path).unwrap();

        let result = ThreemfPackage::from_reader_with_speed_optimized_deserializer(reader, true);

        assert!(result.is_ok());

        match result {
            Ok(package) => {
                let mesh_obj = get_mesh_objects(&package).collect::<Vec<_>>();
                assert_eq!(mesh_obj.len(), 5);

                let beam_lattice_obj = get_beam_lattice_objects(&package).collect::<Vec<_>>();
                assert_eq!(beam_lattice_obj.len(), 2);
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

        let path = PathBuf::from("./tests/data/mesh-composedpart-beamlattice.3mf");
        let reader = File::open(path).unwrap();

        let result = ThreemfPackageLazyReader::from_reader_with_memory_optimized_deserializer(
            reader,
            CachePolicy::NoCache,
        );

        assert!(result.is_ok());

        match result {
            Ok(package) => {
                // Count mesh objects using with_model pattern
                let mut mesh_objects = 0;
                for model_path in package.model_paths() {
                    package
                        .with_model(model_path, |model| {
                            mesh_objects += model
                                .resources
                                .object
                                .iter()
                                .filter(|o| o.mesh.is_some())
                                .count();
                        })
                        .unwrap();
                }
                assert_eq!(mesh_objects, 5);

                // Count beam lattice objects using with_model pattern
                let mut beam_lattice_objects = 0;
                for model_path in package.model_paths() {
                    package
                        .with_model(model_path, |model| {
                            beam_lattice_objects += model
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
                                .count();
                        })
                        .unwrap();
                }
                assert_eq!(beam_lattice_objects, 2);
            }
            Err(err) => {
                panic!("read failed {:?}", err);
            }
        }
    }
}
