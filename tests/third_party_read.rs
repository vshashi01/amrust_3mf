#[cfg(feature = "io")]
#[cfg(test)]
pub mod tests {

    pub mod test_utilities;

    use thiserror::Error;

    use amrust_3mf::io::ThreemfPackage;
    use amrust_3mf::io::ThreemfUnpacked;
    use amrust_3mf::io::error::Error;

    use std::cmp::Ordering;
    use std::fs::File;
    use std::path::PathBuf;

    #[derive(Debug, Error)]
    enum ImageTestError {
        #[error("Failed to generate image for file {0}")]
        ThumbnailGenerationFailed(#[from] Error),

        #[cfg(feature = "thumbnail")]
        #[error("Image dont match for file {0} with a Mean error {1}")]
        ThumbnailComparisonFailed(PathBuf, f32),
    }

    #[test]
    pub fn can_load_thirdparty_3mf_package() {
        let folder_path = PathBuf::from("./tests/data/third-party/");
        let fixtures = test_utilities::get_test_fixtures();
        let mut failed_conditions: Vec<ImageTestError> = vec![];

        for fixture in fixtures {
            if fixture.skip_test || fixture.large_test {
                continue;
            }

            let filepath = folder_path.join(fixture.filepath.clone());
            println!("{:?}", filepath);
            let file = File::open(&filepath).unwrap();

            let package = ThreemfPackage::from_reader(file, true);

            let golden_thumbnail_path = folder_path.join(format!(
                "golden_thumbnails/{}",
                fixture.golden_thumbnail_path
            ));

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

        if !failed_conditions.is_empty() {
            panic!("Some thumbnail generations failed {:?}", failed_conditions)
        }
    }

    #[test]
    pub fn unpack_thirdparty_3mf_package() {
        let folder_path = PathBuf::from("./tests/data/third-party/");
        let fixtures = test_utilities::get_test_fixtures();

        for fixture in fixtures {
            if fixture.skip_test {
                continue;
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
