use amrust_3mf::core::model::Unit;
use amrust_3mf::core::object::ObjectType;
use amrust_3mf::io::ModelBuilder;

/// This example shows how to build 3MF Model using ModelBuilder.
/// Use this to reduce the boilerplate needed to setup 3MF Models.
///
/// Run with:
/// `cargo run --example builder_example --no-default-features --features io-write`
///
fn main() {
    // Create a simple cube model using the builder
    let mut builder = ModelBuilder::new(Unit::Millimeter, true);

    // Set model properties
    builder
        .unit(Unit::Millimeter)
        .add_metadata("Application", Some("Builder Example"))
        .add_metadata(
            "Description",
            Some("A simple cube created with ModelBuilder"),
        );

    // Add a cube object
    let cube_id = builder.add_object(|obj| {
        if obj
            .name("Cube")
            .object_type(ObjectType::Model)
            .part_number("CUBE-001")
            .mesh(|mesh| {
                // Define vertices for a cube
                mesh.add_vertex(&[0.0, 0.0, 0.0]) // 0: bottom-front-left
                    .add_vertex(&[10.0, 0.0, 0.0]) // 1: bottom-front-right
                    .add_vertex(&[10.0, 10.0, 0.0]) // 2: bottom-back-right
                    .add_vertex(&[0.0, 10.0, 0.0]) // 3: bottom-back-left
                    .add_vertex(&[0.0, 0.0, 10.0]) // 4: top-front-left
                    .add_vertex(&[10.0, 0.0, 10.0]) // 5: top-front-right
                    .add_vertex(&[10.0, 10.0, 10.0]) // 6: top-back-right
                    .add_vertex(&[0.0, 10.0, 10.0]); // 7: top-back-left

                // Define triangles for the cube faces
                // Bottom face
                mesh.add_triangle(&[0, 1, 2]).add_triangle(&[0, 2, 3]);
                // Top face
                mesh.add_triangle(&[4, 5, 6]).add_triangle(&[4, 6, 7]);
                // Front face
                mesh.add_triangle(&[0, 1, 5]).add_triangle(&[0, 5, 4]);
                // Back face
                mesh.add_triangle(&[3, 2, 6]).add_triangle(&[3, 6, 7]);
                // Left face
                mesh.add_triangle(&[0, 3, 7]).add_triangle(&[0, 7, 4]);
                // Right face
                mesh.add_triangle(&[1, 2, 6]).add_triangle(&[1, 6, 5]);
            })
            .is_ok()
        {};
    });

    match cube_id {
        Ok(id) => {
            // Add the cube to the build
            if let Err(err) = builder.add_build_item(id) {
                panic!("{err:?}");
            }
        }
        Err(err) => panic!("{err:?}"),
    }

    // Build the final model
    let model = builder.build();

    match model {
        Ok(model) => {
            // Verify the model
            println!("Model created successfully!");
            println!("Unit: {:?}", model.unit);
            println!("Metadata count: {}", model.metadata.len());
            println!("Objects count: {}", model.resources.object.len());
            println!("Build items count: {}", model.build.item.len());

            if let Some(obj) = model.resources.object.first() {
                println!("First object name: {:?}", obj.name);
                if let Some(mesh) = &obj.mesh {
                    println!("Vertices: {}", mesh.vertices.vertex.len());
                    println!("Triangles: {}", mesh.triangles.triangle.len());
                }
            }
        }
        Err(err) => panic!("{err:?}"),
    }
}
