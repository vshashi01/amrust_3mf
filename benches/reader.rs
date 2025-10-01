use criterion::{Criterion, criterion_group, criterion_main};
// use lib3mf_rs::*;
// use roxmltree::Document;

use amrust_3mf::io::ThreemfPackage;

use std::path::PathBuf;

fn amrust_3mf_reader(
    path: &PathBuf,
    expected_mesh_count: usize,
    expected_triangle_indices_count: usize,
    expected_composed_part_count: usize,
    expected_build_items_count: usize,
) {
    let reader = std::fs::File::open(path).unwrap();
    let package = ThreemfPackage::from_reader(reader, true).unwrap();
    let mut number_of_objects = 0;
    let mut number_of_triangles = 0;
    let mut number_of_components = 0;
    package.root.resources.object.iter().for_each(|o| {
        if let Some(mesh) = &o.mesh {
            number_of_objects += 1;
            number_of_triangles += mesh.triangles.triangle.len();
        } else if o.components.is_some() {
            number_of_components += 1;
        }
    });
    if number_of_objects != expected_mesh_count {
        panic!("Number of objects did not match");
    }

    if number_of_triangles != expected_triangle_indices_count {
        //println!("Number of triangles is: {number_of_triangles}");
        panic!("Number of triangle indices did not match");
    }

    if number_of_components != expected_composed_part_count {
        panic!("Number of Composed parts did not match");
    }

    if package.root.build.item.len() != expected_build_items_count {
        panic!("Number of Build items did not match");
    }
}

// fn panic_if_not_success(result: Lib3MFResult, reason: &str) {
//     if result != 0 {
//         panic!("{}", reason);
//     }
// }

// fn lib3mf_rs_read_model(
//     file: &Path,
//     expected_number_of_objects: u64,
//     expected_number_of_triangles: u64,
// ) {
//     let mut model: lib3mf_rs::Lib3MF_Model = std::ptr::null_mut();
//     let res = lib3mf_rs::get_3mf_model_from_file(file.to_path_buf(), &mut model);
//     panic_if_not_success(res, "Model not loaded");

//     let mut number_of_objects: lib3mf_rs::Lib3MF_uint64 = 0;
//     let mut number_of_triangles = 0;
//     unsafe {
//         let mut mesh_obj_it: lib3mf_rs::Lib3MF_MeshObjectIterator = std::ptr::null_mut();
//         let res = lib3mf_model_getmeshobjects(model, &mut mesh_obj_it);
//         panic_if_not_success(res, "Get Mesh objects failed");

//         let res = lib3mf_resourceiterator_count(mesh_obj_it, &mut number_of_objects);
//         panic_if_not_success(res, "Get objects count failed");

//         let mut has_next = false;
//         loop {
//             let res = lib3mf_resourceiterator_movenext(mesh_obj_it, &mut has_next);
//             panic_if_not_success(res, "Failed to move the iterator");
//             if !has_next {
//                 break;
//             }

//             let mut object: Lib3MF_Object = std::ptr::null_mut();
//             let res = lib3mf_meshobjectiterator_getcurrentmeshobject(mesh_obj_it, &mut object);
//             panic_if_not_success(res, "Mesh object not acquired");

//             let mut indices_buffer = Vec::with_capacity(100);
//             let mut needed_buffer_length: Lib3MF_uint64 = 0;
//             let res = lib3mf_meshobject_gettriangleindices(
//                 object,
//                 100,
//                 &mut needed_buffer_length,
//                 indices_buffer.as_mut_ptr(),
//             );
//             panic_if_not_success(res, "Getting triangles indices failed");
//             number_of_triangles += needed_buffer_length;

//             lib3mf_release(object);
//         }

//         lib3mf_release(mesh_obj_it);
//     }

//     if number_of_objects != expected_number_of_objects {
//         panic!("Number of objects did not match");
//     }

//     if number_of_triangles != expected_number_of_triangles {
//         // println!("Number of triangles is: {number_of_triangles}");
//         panic!("Number of triangles did not match");
//     }

//     unsafe {
//         lib3mf_release(model);
//     }
// }

// pub fn read_lib3mf_rs(c: &mut Criterion) {
//     let path = PathBuf::from("tests/data/third-party/mgx-iron_giant_single.3mf")
//         .canonicalize()
//         .unwrap();

//     let mut c = c.benchmark_group("read_group");
//     c.sample_size(10);
//     c.measurement_time(std::time::Duration::from_secs(70));
//     c.bench_function("Lib3mf Reader", |b| {
//         b.iter(|| lib3mf_rs_read_model(&path, 1, 1204183))
//     });
// }

pub fn reader_amrust_3mf(c: &mut Criterion) {
    let path = PathBuf::from("tests/data/third-party/mgx-iron_giant_single.3mf")
        .canonicalize()
        .unwrap();

    let mut c = c.benchmark_group("read_group");
    c.sample_size(10);
    c.measurement_time(std::time::Duration::from_secs(70));
    c.bench_function("ThreemfPackage reader", |b| {
        b.iter(|| amrust_3mf_reader(&path, 1, 1204183, 0, 1))
    });
}

// pub fn read_roxmltree(c: &mut Criterion) {
//     let path = PathBuf::from("benches/3dmodel.model")
//         .canonicalize()
//         .unwrap();
//     let text = std::fs::read_to_string(path).unwrap();
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

criterion_group!(benches, reader_amrust_3mf /* read_lib3mf_rs */,);
criterion_main!(benches);
