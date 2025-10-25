use amrust_3mf::{core::model::Model, io::ThreemfUnpacked};

use std::{fs::File, path::PathBuf};

/// This is an example showing unpacking the package and manually deserializing the root model
/// run with
/// `cargo run --example unpack --no-default-features --features unpack-only io`
///
fn main() {
    let path = PathBuf::from("./tests/data/third-party/mgx-core-prod-beamlattice-material.3mf");
    let reader = File::open(path).unwrap();

    let result = ThreemfUnpacked::from_reader(reader, true);

    match result {
        Ok(unpacked) => {
            let model = serde_roxmltree::from_str::<Model>(&unpacked.root);
            match model {
                Ok(model) => println!("Number of build items: {}", model.build.item.len()),
                Err(err) => println!("Error deserializing the model: {:?}", err),
            }
        }
        Err(err) => println!("Error reading the file: {:?}", err),
    }
}
