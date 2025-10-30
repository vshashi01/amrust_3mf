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
                // for rels in package.relationships.keys() {
                //     println!("Relationship file at {}", rels);
                // }
                // assert!(package.relationships.contains_key("_rels/.rels"));
                // assert!(
                //     package
                //         .relationships
                //         .contains_key("/3D/_rels/3dmodel.model.rels")
                // );

                // let sub_rels = package.relationships.get("/3D/_rels/3dmodel.model.rels");

                //assert!(package.unknown_parts.contains_key("/3D/Disp2D/lines.png"));
                //assert!(sub_rels.is_some());
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

                // for rels in package.relationships.keys() {
                //     println!("Relationship file at {}", rels);
                // }
                // assert!(package.relationships.contains_key("_rels/.rels"));
                // assert!(
                //     package
                //         .relationships
                //         .contains_key("/3D/_rels/3dmodel.model.rels")
                // );

                // let sub_rels = package.relationships.get("/3D/_rels/3dmodel.model.rels");

                // assert!(package.unknown_parts.contains_key("/3D/Disp2D/lines.png"));
                // assert!(sub_rels.is_some());
            }
            Err(err) => {
                panic!("read failed {:?}", err);
            }
        }
    }
}
