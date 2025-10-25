use amrust_3mf::io::ThreemfPackage;

use std::{fs::File, path::PathBuf};

/// This is an example to show how to do a memory-optimized-read
/// run with
/// `cargo run --example memory-optimized-read`
/// memory-optimized-read is part of the default features
///
fn main() {
    let path = PathBuf::from("./tests/data/third-party/mgx-core-prod-beamlattice-material.3mf");
    let reader = File::open(path).unwrap();

    let result = ThreemfPackage::from_reader_with_memory_optimized_deserializer(reader, true);

    match result {
        Ok(package) => {
            println!("Number of build items: {}", package.root.build.item.len())
        }
        Err(err) => println!("Error reading the file: {:?}", err),
    }
}
