#[cfg(any(
    feature = "package-read",
    feature = "package-lazy-read"
))]
#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use std::{fs::File, path::PathBuf};

    #[cfg(feature = "package-read")]
    #[test]
    fn read_threemf_package_memory_optimized() {
        use threemf2::package::ThreemfPackage;
        use threemf2::package::query::get_mesh_objects;

        let path = PathBuf::from("./tests/data/mesh-composedpart-beamlattice.3mf");
        let reader = File::open(path).unwrap();

        let result = ThreemfPackage::from_reader(reader, true);

        assert!(result.is_ok());

        match result {
            Ok(package) => {
                let mesh_obj = get_mesh_objects(&package).collect::<Vec<_>>();
                assert_eq!(mesh_obj.len(), 5);

                let beam_lattice_obj = get_mesh_objects(&package)
                    .filter(|mesh_ref| mesh_ref.view.has_beamlattice())
                    .count();
                assert_eq!(beam_lattice_obj, 2);

                let ns = package.get_namespaces_on_model(None).unwrap();
                assert_eq!(ns.len(), 4);
            }
            Err(err) => {
                panic!("read failed {:?}", err);
            }
        }
    }

    #[cfg(all(
        feature = "package-lazy-read",
        feature = "package-read"
    ))]
    #[test]
    fn read_threemf_package_lazy_memory_optimized() {
        use threemf2::package::{CachePolicy, ThreemfPackageLazyReader};

        let path = PathBuf::from("./tests/data/mesh-composedpart-beamlattice.3mf");
        let reader = File::open(path).unwrap();

        let result = ThreemfPackageLazyReader::from_reader(
            reader,
            CachePolicy::NoCache,
        );

        assert!(result.is_ok());

        let mut namespaces = vec![];

        match result {
            Ok(package) => {
                use threemf2::threemf_namespaces::ThreemfNamespace;

                let mut mesh_objects = 0;
                let mut beam_lattice_objects = 0;
                for model_path in package.model_paths() {
                    package
                        .with_model(model_path, |model| {
                            use threemf2::model::query;

                            mesh_objects = query::get_mesh_objects_from_model(model).count();

                            beam_lattice_objects = query::get_mesh_objects_from_model(model)
                                .filter(|mesh_rep| mesh_rep.has_beamlattice())
                                .count();

                            namespaces.extend_from_slice(&model.used_namespaces());
                        })
                        .unwrap();
                }
                assert_eq!(mesh_objects, 5);
                assert_eq!(beam_lattice_objects, 2);

                // core, prod, material, beam lattice
                assert_eq!(
                    namespaces,
                    [
                        ThreemfNamespace::Core,
                        ThreemfNamespace::Prod,
                        ThreemfNamespace::BeamLattice,
                        ThreemfNamespace::Material,
                    ]
                );
            }
            Err(err) => {
                panic!("read failed {:?}", err);
            }
        }
    }
}
