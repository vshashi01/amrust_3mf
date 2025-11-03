#[cfg(feature = "io")]
#[cfg(test)]
pub mod tests {

    pub mod test_utilities;

    use amrust_3mf::io::ThreemfPackage;

    #[cfg(feature = "unpack-only")]
    use amrust_3mf::io::ThreemfUnpacked;

    use std::fs::File;
    use std::path::PathBuf;

    #[cfg(feature = "memory-optimized-read")]
    #[test]
    pub fn can_load_thirdparty_3mf_package() {
        let fixtures = test_utilities::get_test_fixtures();

        for fixture in fixtures {
            if fixture.skip_test || fixture.large_test {
                continue;
            }

            let mut folder_path = PathBuf::from("./tests/data/third-party/");
            if fixture.large_test {
                folder_path = folder_path.join("lfs");
            }
            let filepath = folder_path.join(fixture.filepath.clone());
            // println!("{:?}", filepath);
            let file = File::open(&filepath).unwrap();

            let package =
                ThreemfPackage::from_reader_with_memory_optimized_deserializer(file, true);

            match package {
                Ok(threemf) => {
                    assert!(!threemf.content_types.defaults.is_empty());
                    assert!(!threemf.relationships.is_empty());
                    assert!(!threemf.root.build.item.is_empty());
                }
                Err(err) => {
                    panic!(
                        "Failed to read the file: {:?} with err: {:?}",
                        &filepath, err
                    );
                }
            }
        }
    }

    #[cfg(feature = "unpack-only")]
    #[test]
    pub fn unpack_thirdparty_3mf_package() {
        let fixtures = test_utilities::get_test_fixtures();

        for fixture in fixtures {
            if fixture.skip_test {
                continue;
            }

            let mut folder_path = PathBuf::from("./tests/data/third-party/");
            if fixture.large_test {
                folder_path = folder_path.join("lfs");
            }

            let filepath = folder_path.join(fixture.filepath);
            let file = File::open(&filepath).unwrap();

            let package = ThreemfUnpacked::from_reader(file, true);

            match package {
                Ok(threemf) => {
                    assert!(!threemf.content_types.is_empty());
                    assert!(!threemf.relationships.is_empty());
                    assert!(!threemf.root.is_empty());
                }
                Err(err) => {
                    panic!(
                        "Failed to read the file: {:?} with err: {:?}",
                        &filepath, err
                    );
                }
            }
        }
    }
}
