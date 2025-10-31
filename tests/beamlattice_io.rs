#[cfg(feature = "io")]
#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use amrust_3mf::io::ThreemfPackage;
    use amrust_3mf::io::query::{get_beam_lattice_objects, get_mesh_objects};

    use std::{fs::File, path::PathBuf};

    #[cfg(feature = "memory-optimized-read")]
    #[test]
    fn read_threemf_package_memory_optimized() {
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

    #[cfg(feature = "speed-optimized-read")]
    #[test]
    fn read_threemf_package_speed_optimized() {
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
}
