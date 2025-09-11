// use amrust_render::bounding_box::BoundingBox;
use amrust_render::camera::{CameraData, OrthographicCameraData};
use amrust_render::gpu_mesh::MeshBuilder;
use amrust_render::instance::InstanceDataBuilder;
use amrust_render::material::Material;
// use amrust_render::transformation::Transformation;
use amrust_render::vertex::Position;
use amrust_render::{RenderObject, renderer};
//use glam::{Mat4, Vec3};
use image::{ImageBuffer, Rgba};
use thiserror::Error;
use three_d::*;
use three_d::{Transform, prelude::*};
use three_d_asset::Camera;

use crate::core::component::Components;
use crate::core::transform::Transform as ThreemfTransform;
use crate::core::{Mesh, Triangles, Vertex};
use crate::query::get_object_ref_from_id;

pub use super::ThreemfPackage;
pub use super::error::Error;

use std::collections::HashMap;
use std::f32;

type ObjectId = usize;

enum PartRep {
    Mesh {
        vertices: Vec<Vec3>,
        triangles: Vec<u32>,
    },
    ComposedPart(Vec<ObjectId>),
}

struct Data {
    pub part_rep: PartRep, //optional data to gpu data in renderer
    pub transforms: Vec<Transformation>,
}

#[derive(Debug, Error)]
pub enum DbFrom3mfError {
    #[error("Object not found: {0}")]
    ObjectNotFound(usize),

    #[error("The Composed parts is empty {0}")]
    EmptyComposedPart(usize),

    #[error("Object does not contain data {0}")]
    EmptyObject(usize),
}

struct RenderDb {
    object_id_to_repdata: HashMap<usize, Data>,
    object_id_to_gpudata: HashMap<usize, usize>, //mesh object id to gpu_mesh_id
    cpu_mesh_collection: Vec<CpuMesh>,
    //gm_collection: Vec<Gm<InstancedMesh, ColorMaterial>>,
}

fn process_object_and_register_part_rep_data(
    db: &mut RenderDb,
    object_id: usize,
    package: &ThreemfPackage,
    context: &Context,
    parent_transformation: &Transformation,
    path: &Option<String>,
    parent_model: &Option<String>,
) -> Result<usize, DbFrom3mfError> {
    if let Some(data) = db.object_id_to_repdata.get_mut(&object_id) {
        data.transforms.push(*parent_transformation);
        Ok(object_id)
    } else {
        let (object, parent_model_path) =
            get_object_ref_from_id(object_id, package, &path, &parent_model);

        match object {
            Some(object) => {
                if let Some(m) = &object.mesh {
                    process_mesh_object(db, object.id, context, m, parent_transformation)
                } else if let Some(comps) = &object.components {
                    process_composed_object(
                        db,
                        object.id,
                        comps,
                        package,
                        context,
                        parent_transformation,
                        &parent_model_path,
                    )
                } else {
                    Err(DbFrom3mfError::EmptyObject(object_id))
                }
            }
            None => Err(DbFrom3mfError::ObjectNotFound(object_id)),
        }
    }
}

fn process_mesh_object(
    db: &mut RenderDb,
    object_id: usize,
    context: &Context,
    m: &Mesh,
    parent_transformation: &Transformation,
) -> Result<usize, DbFrom3mfError> {
    let vertices = convert_3mf_vertices_to_mesh_vertices(&m.vertices.vertex);
    let triangles = convert_triangles_to_indices(&m.triangles);

    let gpu_model = CpuMesh {
        positions: Positions::F32(vertices.clone()),
        indices: Indices::U32(triangles.clone()),
        colors: Some(convert_vertices_to_color(&vertices)),
        ..Default::default()
    };

    db.cpu_mesh_collection.push(gpu_model);
    let gpu_data = db.cpu_mesh_collection.len() - 1;

    let mesh = PartRep::Mesh {
        vertices,
        triangles,
    };

    db.object_id_to_repdata.insert(
        object_id,
        Data {
            part_rep: mesh,
            transforms: vec![*parent_transformation],
        },
    );

    db.object_id_to_gpudata.insert(object_id, gpu_data);

    Ok(object_id)
}

fn process_composed_object(
    db: &mut RenderDb,
    object_id: usize,
    comps: &Components,
    package: &ThreemfPackage,
    context: &Context,
    parent_transformation: &Transformation,
    parent_model: &Option<String>,
) -> Result<usize, DbFrom3mfError> {
    // let mut list_of_unique_part_id_per_component = vec![];
    let mut components = vec![];
    for comp in &comps.component {
        let transform = get_transformation(&comp.transform);
        let combined_transform = Transformation(parent_transformation.0 * transform.0);

        let id = process_object_and_register_part_rep_data(
            db,
            comp.objectid,
            package,
            context,
            &combined_transform,
            &comp.path,
            parent_model,
        )?;

        components.push(id);
    }

    db.object_id_to_repdata.insert(
        object_id,
        Data {
            part_rep: PartRep::ComposedPart(components),
            transforms: vec![*parent_transformation],
        },
    );

    Ok(object_id)
}

// pub async fn render_package_thumbnail(
//     package: &ThreemfPackage,
//     width: u32,
//     height: u32,
// ) -> Result<image::ImageBuffer<Rgba<u8>, Vec<u8>>, Error> {
//     let mut renderer = renderer::Renderer::from_new_device(width, height)
//         .await
//         .map_err(|e| Error::ThumbnailError(e.to_string()))?;

//     let mut db = RenderDb {
//         object_id_to_repdata: HashMap::new(),
//         object_id_to_gpudata: HashMap::new(),
//     };

//     let mut item_we_care_about = vec![];

//     for item in &package.root.build.item {
//         let transform = {
//             if let Some(transform) = &item.transform {
//                 Transformation(convert_transform_to_glam_matrix(transform))
//             } else {
//                 Transformation(glam::Mat4::IDENTITY)
//             }
//         };

//         let id = process_object_and_register_part_rep_data(
//             &mut db,
//             item.objectid,
//             package,
//             &mut renderer,
//             &transform,
//             &None,
//             &None,
//         )?;

//         item_we_care_about.push(id);
//     }

//     let mut total_bbox = BoundingBox::default();

//     //match instance data to the gpu data
//     for (id, data) in &db.object_id_to_repdata {
//         match &data.part_rep {
//             PartRep::Mesh {
//                 vertices,
//                 triangles: _,
//             } => {
//                 if let Some(gpu_mesh_id) = db.object_id_to_gpudata.get(id) {
//                     add_mesh_render_object(&mut renderer, *gpu_mesh_id, &data.transforms);
//                     for transform in &data.transforms {
//                         let bbox = compute_transformed_bounding_box_from_mesh(vertices, transform);
//                         total_bbox.unite(&bbox);
//                     }
//                 }
//             }
//             PartRep::ComposedPart(items) => {
//                 for transform in &data.transforms {
//                     process_composed_part_items(
//                         &mut renderer,
//                         &db,
//                         id,
//                         items,
//                         &mut total_bbox,
//                         transform,
//                     );
//                 }
//             }
//         }
//     }

//     add_bounding_box_wireframe(&mut renderer, &total_bbox);

//     let top_left_corner = Vec3::new(total_bbox.min.x, total_bbox.min.y, total_bbox.max.z);
//     let mut camera_data = OrthographicCameraData {
//         eye_position: top_left_corner,
//         target_position: total_bbox.center(),
//         up_vector: Vec3::Z,
//         aspect_ratio: width as f32 / height as f32,
//         ..Default::default()
//     };

//     let view_matrix = camera_data.get_view_matrix();
//     let max_extent = calculate_max_extent_in_camera_space(&total_bbox, &view_matrix);
//     // println!("max_extent: {}", max_extent);
//     if max_extent == f32::NAN {
//         return Err(Error::ThumbnailError("Maximum extent is NAN".to_owned()));
//     }

//     camera_data.zoom = 2.0 / max_extent;

//     renderer.update_camera(&camera_data);
//     renderer.render().await.unwrap();

//     let image_buffer = renderer.present().await;

//     Ok(image_buffer)
// }

pub async fn render_package_thumbnail(
    package: &ThreemfPackage,
    width: u32,
    height: u32,
) -> Result<image::ImageBuffer<Rgba<u8>, Vec<u8>>, Error> {
    let mut viewport = Viewport::new_at_origo(width, height);
    let context = HeadlessContext::new().unwrap();
    // let mut renderer = renderer::Renderer::from_new_device(width, height)
    //     .await
    //     .map_err(|e| Error::ThumbnailError(e.to_string()))?;

    let mut db = RenderDb {
        object_id_to_repdata: HashMap::new(),
        object_id_to_gpudata: HashMap::new(),
        cpu_mesh_collection: vec![],
        //gm_collection: vec![],
    };

    let mut item_we_care_about = vec![];
    let mut gm_collection: Vec<Gm<InstancedMesh, three_d::ColorMaterial>> = vec![];

    for item in &package.root.build.item {
        let transform = {
            if let Some(transform) = &item.transform {
                Transformation(convert_transform_to_matrix(transform))
            } else {
                Transformation(Mat4::identity())
            }
        };

        let id = process_object_and_register_part_rep_data(
            &mut db,
            item.objectid,
            package,
            &context,
            &transform,
            &None,
            &None,
        )?;

        item_we_care_about.push(id);
    }

    let mut total_bbox = BoundingBox::default();

    //match instance data to the gpu data
    for (id, data) in &db.object_id_to_repdata {
        match &data.part_rep {
            PartRep::Mesh {
                vertices,
                triangles: _,
            } => {
                if let Some(gpu_mesh_id) = db.object_id_to_gpudata.get(id) {
                    let gm = add_mesh_render_object(&context, &db, *gpu_mesh_id, &data.transforms);
                    gm_collection.push(gm);
                    for transform in &data.transforms {
                        let bbox = compute_transformed_bounding_box_from_mesh(vertices, transform);
                        total_bbox.unite(&bbox);
                    }
                }
            }
            PartRep::ComposedPart(items) => {
                for transform in &data.transforms {
                    process_composed_part_items(
                        &context,
                        &db,
                        id,
                        items,
                        &mut total_bbox,
                        transform,
                    );
                }
            }
        }
    }

    // add_bounding_box_wireframe(&mut renderer, &total_bbox);

    let center = total_bbox.center();

    let top_left_corner = Vec3::new(total_bbox.min.x, total_bbox.min.y, total_bbox.max.z);
    let mut camera = three_d::Camera::new_orthographic(
        viewport,
        top_left_corner,
        Vector3::new(center.x, center.y, center.z),
        Vec3::unit_z(),
        height as f32,
        0.01,
        5000.00,
    );
    // let mut camera_data = OrthographicCameraData {
    //     eye_position: top_left_corner,
    //     target_position: total_bbox.center(),
    //     up_vector: Vec3::Z,
    //     aspect_ratio: width as f32 / height as f32,
    //     ..Default::default()
    // };

    // let view_matrix = camera_data.get_view_matrix();
    let view_matrix = camera.view();
    let max_extent = calculate_max_extent_in_camera_space(&total_bbox, &view_matrix);
    // println!("max_extent: {}", max_extent);
    if max_extent == f32::NAN {
        return Err(Error::ThumbnailError("Maximum extent is NAN".to_owned()));
    }

    // camera_data.zoom = 2.0 / max_extent;
    camera.set_zoom_factor(2.0 / max_extent);

    // renderer.update_camera(&camera_data);
    // renderer.render().await.unwrap();

    // Create a color texture to render into
    let mut texture = Texture2D::new_empty::<[u8; 4]>(
        &context,
        viewport.width,
        viewport.height,
        Interpolation::Nearest,
        Interpolation::Nearest,
        None,
        Wrapping::ClampToEdge,
        Wrapping::ClampToEdge,
    );

    // Also create a depth texture to support depth testing
    let mut depth_texture = DepthTexture2D::new::<f32>(
        &context,
        viewport.width,
        viewport.height,
        Wrapping::ClampToEdge,
        Wrapping::ClampToEdge,
    );

    // Create a render target (a combination of a color and a depth texture) to write into
    let pixels = RenderTarget::new(
        texture.as_color_target(None),
        depth_texture.as_depth_target(),
    );
    // Clear color and depth of the render target
    pixels.clear(ClearState::color_and_depth(0.8, 0.8, 0.8, 1.0, 1.0));

    for gm in gm_collection {
        pixels.render(&camera, &gm, &[]);
    }

    let color = pixels.read_color();

    //let image_buffer = renderer.present().await;
    // Save the rendered image
    // use three_d_asset::io::Serialize;

    // three_d_asset::io::save(
    //     &CpuTexture {
    //         data: TextureData::RgbaU8(color),
    //         width: texture.width(),
    //         height: texture.height(),
    //         ..Default::default()
    //     }
    //     .serialize(format!("headless-{}.png", ""))
    //     .unwrap(),
    // )
    // .unwrap();

    let flat: Vec<u8> = color
        .iter()
        .flat_map(|px: &[u8; 4]| px.iter().copied())
        .collect();

    let image_buffer = ImageBuffer::from_vec(width, height, flat);

    match image_buffer {
        Some(image) => Ok(image),
        None => Err(Error::ThumbnailError("Failed to generate".to_owned())),
    }
    // Ok(image_buffer)
}

fn process_composed_part_items(
    context: &Context,
    db: &RenderDb,
    id: &usize,
    items: &Vec<usize>,
    bbox: &mut BoundingBox,
    parent_transformation: &Transformation,
) {
    for item in items {
        if let Some(data) = &db.object_id_to_repdata.get(item) {
            match &data.part_rep {
                PartRep::Mesh {
                    vertices,
                    triangles: _,
                } => {
                    if let Some(gpu_mesh_id) = db.object_id_to_gpudata.get(id) {
                        let gm =
                            add_mesh_render_object(context, db, *gpu_mesh_id, &data.transforms);
                        let part_bbox = compute_transformed_bounding_box_from_mesh(
                            vertices,
                            parent_transformation,
                        );
                        bbox.unite(&part_bbox);
                    }
                }
                PartRep::ComposedPart(items) => {}
            }
        }
    }

    // for item in items {
    //     if let Some(data) = &db.object_id_to_repdata.get(item) {
    //         match &data.part_rep {
    //             PartRep::Mesh {
    //                 vertices,
    //                 triangles: _,
    //             } => {
    //                 if let Some(gpu_mesh_id) = db.object_id_to_gpudata.get(id) {
    //                     let gm =
    //                         add_mesh_render_object(context, db, *gpu_mesh_id, &data.transforms);
    //                     db.gm_collection.push(gm);
    //                     let part_bbox = compute_transformed_bounding_box_from_mesh(
    //                         vertices,
    //                         parent_transformation,
    //                     );
    //                     bbox.unite(&part_bbox);
    //                 }
    //             }
    //             PartRep::ComposedPart(items) => {
    //                 process_composed_part_items(
    //                     context,
    //                     db,
    //                     id,
    //                     items,
    //                     bbox,
    //                     data.transforms.first().unwrap(),
    //                 );
    //             }
    //         }
    //     }
    // }
}

fn add_mesh_render_object(
    context: &Context,
    db: &RenderDb,
    gpu_mesh_id: usize,
    transforms: &[Transformation],
) -> Gm<InstancedMesh, three_d::ColorMaterial> {
    let cpu_mesh = &db.cpu_mesh_collection[gpu_mesh_id];

    let transformation_data = transforms.iter().map(|t| t.0).collect::<Vec<_>>();
    let color_data = vec![
        Srgba {
            r: 255,
            g: 255,
            b: 255,
            a: 255
        };
        transformation_data.len()
    ];

    let instances = InstancedMesh::new(
        context,
        &Instances {
            transformations: transformation_data,
            texture_transformations: None,
            colors: Some(color_data),
        },
        cpu_mesh,
    );

    let gm = Gm::new(
        instances,
        ColorMaterial::new_opaque(
            context,
            &CpuMaterial {
                albedo: Srgba {
                    r: 128,
                    g: 128,
                    b: 128,
                    a: 255,
                },
                ..Default::default()
            },
        ),
    );

    //db.gm_collection.push(gm);
    gm
}

// fn add_mesh_render_object(
//     renderer: &mut renderer::Renderer,
//     gpu_mesh_id: u32,
//     transforms: &[Transformation],
// ) {
//     let transformation_data = transforms.iter().map(|t| t.to_data()).collect::<Vec<_>>();
//     let material_data = vec![Material::new(1.0, 1.0, 1.0).to_data(); transformation_data.len()];
//     let object = RenderObject {
//         renderable: amrust_render::Renderable::ColoredMesh(gpu_mesh_id),
//         instance: InstanceDataBuilder::new()
//             .add_instance_stream(transformation_data.as_slice())
//             .add_instance_stream(material_data.as_slice())
//             .build(&renderer.device),
//     };
//     let _ = renderer.add_object(object);

//     // println!("Colored Object id {}", object_id);

//     let wireframe_object = RenderObject {
//         renderable: amrust_render::Renderable::WireframeMesh(gpu_mesh_id),
//         instance: InstanceDataBuilder::new()
//             .add_instance_stream(transformation_data.as_slice())
//             .add_instance_stream(material_data.as_slice())
//             .build(&renderer.device),
//     };
//     let _ = renderer.add_object(wireframe_object);

//     // println!("Wireframe Object id {}", wireframe_object_id);
// }

// fn create_gpu_data(renderer: &mut renderer::Renderer, vertices: &[Vec3], triangles: &[u32]) -> u32 {
//     let positions = convert_vertices_to_position(vertices);
//     let indices = triangles;
//     let color = convert_vertices_to_color(vertices);
//     let wireframe_indices = convert_triangles_to_wireframe_indices(triangles);

//     let gpu_mesh = MeshBuilder::new()
//         .add_vertex_stream(positions.as_slice())
//         .add_vertex_stream(color.as_slice())
//         .add_mesh_index_stream(indices)
//         .add_wireframe_index_stream(wireframe_indices.as_slice())
//         .build(&renderer.device);

//     renderer.add_mesh(gpu_mesh)
// }

fn calculate_max_extent_in_camera_space(bbox: &BoundingBox, view_matrix: &Mat4) -> f32 {
    let corners_in_camera_space = bbox.corners().map(|corner| {
        let corner_vector = corner.extend(1.0);
        let transformed = view_matrix * corner_vector;
        transformed.truncate()
    });
    let mut min = corners_in_camera_space[0];
    let mut max = corners_in_camera_space[0];

    for v in &corners_in_camera_space[1..] {
        min = three_d::Vector3::new(min.x.min(v.x), min.y.min(v.y), min.z.min(v.z));
        max = three_d::Vector3::new(max.x.max(v.x), max.y.max(v.y), max.z.max(v.z));
    }

    let extent = max - min;

    extent.x.max(extent.y).max(extent.z)
}

fn get_transformation(transform: &Option<ThreemfTransform>) -> Transformation {
    {
        if let Some(transform) = transform {
            Transformation(convert_transform_to_matrix(transform))
        } else {
            Transformation(Mat4::identity())
        }
    }
}

fn convert_3mf_vertices_to_mesh_vertices(vertices: &[Vertex]) -> Vec<Vec3> {
    vertices
        .iter()
        .map(|v| Vec3::new(v.x as f32, v.y as f32, v.z as f32))
        .collect()
}

fn convert_vertices_to_position(vertices: &[Vec3]) -> Vec<Position> {
    vertices.iter().map(|v| Position([v.x, v.y, v.z])).collect()
}

fn convert_triangles_to_indices(triangles: &Triangles) -> Vec<u32> {
    triangles
        .triangle
        .iter()
        .flat_map(|t| vec![t.v1 as u32, t.v2 as u32, t.v3 as u32])
        .collect()
}

fn convert_triangles_to_wireframe_indices(triangles: &[u32]) -> Vec<u32> {
    let mut indices = Vec::new();
    for t in triangles.chunks(3) {
        indices.push(t[0]);
        indices.push(t[1]);
        indices.push(t[1]);
        indices.push(t[2]);
        indices.push(t[2]);
        indices.push(t[0]);
    }

    indices
}

fn convert_vertices_to_color(vertices: &[Vec3]) -> Vec<Srgba> {
    vertices
        .iter()
        .map(|_| Srgba {
            r: 127,
            g: 127,
            b: 127,
            a: 255,
        })
        .collect()
}

fn convert_transform_to_matrix(transform: &ThreemfTransform) -> Mat4 {
    Mat4::from_cols(
        Vector4 {
            x: transform.0[0] as f32,
            y: transform.0[1] as f32,
            z: transform.0[2] as f32,
            w: 0.0,
        },
        Vector4 {
            x: transform.0[3] as f32,
            y: transform.0[4] as f32,
            z: transform.0[5] as f32,
            w: 0.0,
        },
        Vector4 {
            x: transform.0[6] as f32,
            y: transform.0[7] as f32,
            z: transform.0[8] as f32,
            w: 0.0,
        },
        Vector4 {
            x: transform.0[9] as f32,
            y: transform.0[10] as f32,
            z: transform.0[11] as f32,
            w: 1.0,
        },
    )
}

// fn add_bounding_box_wireframe(
//     renderer: &mut amrust_render::renderer::Renderer,
//     bbox: &BoundingBox,
// ) -> u32 {
//     let mesh = MeshBuilder::new()
//         .add_vertex_stream(convert_points_vec_to_position(&bbox.corners()).as_slice())
//         .add_wireframe_index_stream(&BoundingBox::wireframe_indices())
//         .build(&renderer.device);

//     let mesh_id = renderer.add_mesh(mesh);

//     let wireframe_object = RenderObject {
//         renderable: amrust_render::Renderable::WireframeMesh(mesh_id),
//         instance: InstanceDataBuilder::new()
//             .add_instance_stream(&[Transformation(Mat4::IDENTITY).to_data()])
//             .add_instance_stream(&[Material::new(1.0, 1.0, 1.0).to_data()])
//             .build(&renderer.device),
//     };

//     renderer.add_object(wireframe_object)
// }

fn convert_points_vec_to_position(points: &[Vec3]) -> Vec<Position> {
    points.iter().map(|p| Position([p.x, p.y, p.z])).collect()
}

fn compute_transformed_bounding_box_from_mesh(
    vertices: &Vec<Vec3>,
    transform: &Transformation,
) -> BoundingBox {
    let mut bbox = BoundingBox::default();
    for v in vertices {
        let v4 = transform.0 * v.extend(1.0);
        let transformed = Vec3::new(v4.x, v4.y, v4.z);
        bbox.expand_to_include(&transformed);
    }
    bbox
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Transformation(pub Mat4);

#[derive(Clone, Copy)]
pub struct BoundingBox {
    pub min: Point3<f32>,
    pub max: Point3<f32>,
}

impl Default for BoundingBox {
    fn default() -> Self {
        BoundingBox {
            min: Point3::new(f32::INFINITY, f32::INFINITY, f32::INFINITY),
            max: Point3::new(f32::NEG_INFINITY, f32::NEG_INFINITY, f32::NEG_INFINITY),
        }
    }
}

// impl Debug for BoundingBox {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         f.debug_struct("BoundingBox")
//             .field("min", &self.min)
//             .field("max", &self.max)
//             .field("delta", &self.delta())
//             .finish()
//     }
// }
//

impl BoundingBox {
    pub fn center(&self) -> Point3<f32> {
        self.min + (self.max - self.min) * 0.5
    }
    pub fn transform(&mut self, transform: &Transformation) {
        self.min = transform.0.transform_point(self.min);
        self.max = transform.0.transform_point(self.max);
    }

    pub fn unite(&mut self, other: &BoundingBox) {
        self.min = three_d::Point3::new(
            self.min.x.min(other.min.x),
            self.min.y.min(other.min.y),
            self.min.z.min(other.min.z),
        );
        self.max = three_d::Point3::new(
            self.max.x.max(other.max.x),
            self.max.y.max(other.max.y),
            self.max.z.max(other.max.z),
        );
    }

    pub fn expand_to_include(&mut self, point: &Vec3) {
        self.min = three_d::Point3::new(
            self.min.x.min(point.x),
            self.min.y.min(point.y),
            self.min.z.min(point.z),
        );
        self.max = three_d::Point3::new(
            self.max.x.max(point.x),
            self.max.y.max(point.y),
            self.max.z.max(point.z),
        );
    }

    pub fn corners(&self) -> [Vec3; 8] {
        [
            Vec3::new(self.min.x, self.min.y, self.min.z),
            Vec3::new(self.max.x, self.min.y, self.min.z),
            Vec3::new(self.min.x, self.max.y, self.min.z),
            Vec3::new(self.max.x, self.max.y, self.min.z),
            Vec3::new(self.min.x, self.min.y, self.max.z),
            Vec3::new(self.max.x, self.min.y, self.max.z),
            Vec3::new(self.min.x, self.max.y, self.max.z),
            Vec3::new(self.max.x, self.max.y, self.max.z),
        ]
    }

    pub fn wireframe_indices() -> [u32; 24] {
        // Each pair is a line segment between two corners
        let box_wireframe_indices: [u32; 24] = [
            // Bottom face (min z)
            0, 1, 1, 3, 3, 2, 2, 0, // Top face (max z)
            4, 5, 5, 7, 7, 6, 6, 4, // Vertical edges
            0, 4, 1, 5, 2, 6, 3, 7,
        ];

        box_wireframe_indices
    }

    pub fn delta(&self) -> Vec3 {
        self.max - self.min
    }
}

mod tests {
    use std::{
        cmp::Ordering,
        path::{Path, PathBuf},
    };

    fn test_data_path(rel: &str) -> PathBuf {
        let this_file = Path::new(file!());
        let dir = this_file.parent().unwrap();
        dir.join(rel)
    }

    #[test]
    fn test_thumbnail() {
        let threemf = std::fs::File::open(test_data_path(
            "../../tests/data/third-party/meshmixer-bunny.3mf",
        ))
        .unwrap();
        let golden_thumbnail_path = test_data_path(
            "../../tests/data/third-party/golden_thumbnails/meshmixer-bunny.3mf_golden_thumbnail.png",
        );

        let package = super::ThreemfPackage::from_reader(threemf, true).unwrap();

        const FLIP_MEAN_ERROR: f32 = 0.021;
        pollster::block_on(async {
            let ref_image_data = image::open(&golden_thumbnail_path).unwrap().into_rgba8();

            let thumbnail = super::render_package_thumbnail(&package, 1280, 1080)
                .await
                .unwrap();

            let ref_image = nv_flip::FlipImageRgb8::with_data(1280, 1080, &ref_image_data);
            let test_image = nv_flip::FlipImageRgb8::with_data(1280, 1080, &thumbnail);

            let error_map =
                nv_flip::flip(ref_image, test_image, nv_flip::DEFAULT_PIXELS_PER_DEGREE);
            let pool = nv_flip::FlipPool::from_image(&error_map);
            if let Some(Ordering::Greater) = pool.mean().partial_cmp(&FLIP_MEAN_ERROR) {
                println!("Mean error {}", pool.mean());
                thumbnail
                    .save(format!("meshmixer-bunny.3mf_golden_thumbnail.png"))
                    .unwrap();

                panic!("Something is wrong with the thumbnail");
            }
        });
    }
}
