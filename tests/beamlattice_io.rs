use amrust_3mf::io::ThreemfPackage;

#[cfg(feature = "io")]
#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use amrust_3mf::io::ThreemfPackage;

    use std::{fs::File, path::PathBuf};

    #[cfg(feature = "memory-optimized-read")]
    #[test]
    fn read_threemf_package_memory_optimized() {
        let path = PathBuf::from("./tests/data/third-party/mgx-core-prod-beamlattice-material.3mf");
        let reader = File::open(path).unwrap();

        let result = ThreemfPackage::from_reader_with_memory_optimized_deserializer(reader, true);

        assert!(result.is_ok());

        match result {
            Ok(package) => {
                use amrust_3mf::query::{get_beam_lattice_objects, get_mesh_objects};

                let mesh_obj = get_mesh_objects(&package);

                for object in mesh_obj {
                    println!("Mesh Objects id: {}", object.id);
                }

                let beam_lattice_obj = get_beam_lattice_objects(&package);

                for object in beam_lattice_obj {
                    println!("Beam Lattice Objects id: {}", object.id);
                }
            }
            Err(err) => {
                panic!("read failed {:?}", err);
            }
        }
    }

    #[cfg(feature = "speed-optimized-read")]
    #[test]
    fn read_threemf_package_speed_optimized() {
        let path = PathBuf::from("./tests/data/third-party/mgx-core-prod-beamlattice-material.3mf");
        let reader = File::open(path).unwrap();

        let result = ThreemfPackage::from_reader_with_speed_optimized_deserializer(reader, true);

        assert!(result.is_ok());

        match result {
            Ok(package) => {
                assert_eq!(package.relationships.len(), 2);
                for rels in package.relationships.keys() {
                    println!("Relationship file at {}", rels);
                }
                assert!(package.relationships.contains_key("_rels/.rels"));
                assert!(
                    package
                        .relationships
                        .contains_key("/3D/_rels/3dmodel.model.rels")
                );

                let sub_rels = package.relationships.get("/3D/_rels/3dmodel.model.rels");

                assert!(package.unknown_parts.contains_key("/3D/Disp2D/lines.png"));
                assert!(sub_rels.is_some());
            }
            Err(err) => {
                panic!("read failed {:?}", err);
            }
        }
    }
}

fn package_contains_mesh_with_lattice(package: ThreemfPackage) {}
