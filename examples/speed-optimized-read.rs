use amrust_3mf::io::ThreemfPackage;

use std::{fs::File, path::PathBuf};

/// This is an example showing speed optimized reading
/// run with
/// `cargo run --example speed-optimized-read --features speed-optimized-read`
///
fn main() {
    let path = PathBuf::from("./tests/data/third-party/mgx-core-prod-beamlattice-material.3mf");
    let reader = File::open(path).unwrap();

    let result = ThreemfPackage::from_reader_with_speed_optimized_deserializer(reader, true);

    match result {
        Ok(package) => {
            println!("Number of build items: {}", package.root.build.item.len())
        }
        Err(err) => println!("Error reading the file: {:?}", err),
    }
}
