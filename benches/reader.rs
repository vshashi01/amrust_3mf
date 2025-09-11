use criterion::{Criterion, criterion_group, criterion_main};
use instant_xml::from_str;
use roxmltree::Document;

use amrust_3mf::core::model::Model;

use std::path::PathBuf;


pub fn read_instant_xml(c: &mut Criterion) {
    let path = PathBuf::from("benches/3dmodel.model")
        .canonicalize()
        .unwrap();
    let text = std::fs::read_to_string(
        path,
    )
    .unwrap();
    let mut c = c.benchmark_group("read_group");
    c.sample_size(10);
    c.measurement_time(std::time::Duration::from_secs(70));
    c.bench_function("read instant_xml function", |b| {
        b.iter(|| from_str::<Model>(&text).unwrap())
    });
}

// pub fn read_roxmltree(c: &mut Criterion) {
//     let text = std::fs::read_to_string(
//         "C:/Users/thara/Development/amrust/amrust_3mf/benches/3dmodel.model",
//     )
//     .unwrap();
//     let mut c = c.benchmark_group("read_group");
//     c.sample_size(10);
//     c.measurement_time(std::time::Duration::from_secs(40));
//     c.bench_function("read roxmltree function", |b| {
//         b.iter(|| match Document::parse(&text) {
//             Ok(_) => {}
//             Err(_) => {
//                 panic!("Something went wrong with roxmltree")
//             }
//         })
//     });
// }

criterion_group!(benches, read_instant_xml);
criterion_main!(benches);
