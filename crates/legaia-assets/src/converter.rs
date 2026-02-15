//! TMD to glTF converter

use anyhow::Result;
use gltf_json as json;
use gltf_json::validation::USize64;
use psxutils::formats::tmd::{Tmd, TmdPrimitive};
use std::fs;
use std::path::Path;

/// Convert a TMD model to glTF 2.0 format
pub fn tmd_to_gltf(tmd: &Tmd, output_path: &Path) -> Result<()> {
    let mut root = json::Root::default();
    let mut buffer_data = Vec::new();
    let mut buffer_views = Vec::new();
    let mut accessors = Vec::new();
    let mut meshes = Vec::new();

    for (_obj_idx, object) in tmd.objects.iter().enumerate() {
        // Skip empty objects
        if object.vertices.is_empty() {
            continue;
        }

        // Calculate scale factor
        let scale = if object.scale == 0 {
            1.0
        } else {
            object.scale as f32
        };

        // Convert vertices to f32 positions
        let mut positions: Vec<f32> = Vec::new();
        let mut pos_min = [f32::MAX, f32::MAX, f32::MAX];
        let mut pos_max = [f32::MIN, f32::MIN, f32::MIN];

        for vertex in &object.vertices {
            let x = vertex.x as f32 / scale;
            let y = vertex.y as f32 / scale;
            let z = vertex.z as f32 / scale;

            positions.push(x);
            positions.push(y);
            positions.push(z);

            pos_min[0] = pos_min[0].min(x);
            pos_min[1] = pos_min[1].min(y);
            pos_min[2] = pos_min[2].min(z);

            pos_max[0] = pos_max[0].max(x);
            pos_max[1] = pos_max[1].max(y);
            pos_max[2] = pos_max[2].max(z);
        }

        // Convert normals to f32
        let mut normals: Vec<f32> = Vec::new();
        for normal in &object.normals {
            let nx = normal.nx as f32 / 4096.0;
            let ny = normal.ny as f32 / 4096.0;
            let nz = normal.nz as f32 / 4096.0;
            let len = (nx * nx + ny * ny + nz * nz).sqrt();

            if len > 0.0 {
                normals.push(nx / len);
                normals.push(ny / len);
                normals.push(nz / len);
            } else {
                normals.push(0.0);
                normals.push(1.0);
                normals.push(0.0);
            }
        }

        // Build index buffer from primitives
        let mut indices: Vec<u16> = Vec::new();

        for primitive in &object.primitives {
            match primitive {
                TmdPrimitive::Triangle { vertices, .. } => {
                    // Add triangle indices
                    indices.push(vertices[0]);
                    indices.push(vertices[1]);
                    indices.push(vertices[2]);
                }
                TmdPrimitive::Quad { vertices, .. } => {
                    // Split quad into two triangles (0-1-2, 0-2-3)
                    indices.push(vertices[0]);
                    indices.push(vertices[1]);
                    indices.push(vertices[2]);

                    indices.push(vertices[0]);
                    indices.push(vertices[2]);
                    indices.push(vertices[3]);
                }
            }
        }

        // Skip objects with no primitives
        if indices.is_empty() {
            continue;
        }

        // --- Create position buffer and accessor ---
        let position_bytes: Vec<u8> = positions.iter().flat_map(|f| f.to_le_bytes()).collect();
        let position_offset = buffer_data.len();
        buffer_data.extend_from_slice(&position_bytes);

        let position_view_idx = buffer_views.len();
        buffer_views.push(json::buffer::View {
            buffer: json::Index::new(0),
            byte_length: USize64::from(position_bytes.len()),
            byte_offset: Some(USize64::from(position_offset)),
            byte_stride: None,
            extensions: None,
            extras: Default::default(),
            name: None,
            target: Some(json::validation::Checked::Valid(
                json::buffer::Target::ArrayBuffer,
            )),
        });

        let position_accessor_idx = accessors.len();
        accessors.push(json::Accessor {
            buffer_view: Some(json::Index::new(position_view_idx as u32)),
            byte_offset: Some(USize64(0)),
            count: USize64::from(object.vertices.len()),
            component_type: json::validation::Checked::Valid(json::accessor::GenericComponentType(
                json::accessor::ComponentType::F32,
            )),
            extensions: None,
            extras: Default::default(),
            name: None,
            type_: json::validation::Checked::Valid(json::accessor::Type::Vec3),
            min: Some(json::Value::from(vec![pos_min[0], pos_min[1], pos_min[2]])),
            max: Some(json::Value::from(vec![pos_max[0], pos_max[1], pos_max[2]])),
            normalized: false,
            sparse: None,
        });

        // --- Create normal buffer and accessor (if normals exist) ---
        let normal_accessor_idx = if !normals.is_empty() {
            let normal_bytes: Vec<u8> = normals.iter().flat_map(|f| f.to_le_bytes()).collect();
            let normal_offset = buffer_data.len();
            buffer_data.extend_from_slice(&normal_bytes);

            let normal_view_idx = buffer_views.len();
            buffer_views.push(json::buffer::View {
                buffer: json::Index::new(0),
                byte_length: USize64::from(normal_bytes.len()),
                byte_offset: Some(USize64::from(normal_offset)),
                byte_stride: None,
                extensions: None,
                extras: Default::default(),
                name: None,
                target: Some(json::validation::Checked::Valid(
                    json::buffer::Target::ArrayBuffer,
                )),
            });

            let idx = accessors.len();
            accessors.push(json::Accessor {
                buffer_view: Some(json::Index::new(normal_view_idx as u32)),
                byte_offset: Some(USize64(0)),
                count: USize64::from(object.normals.len()),
                component_type: json::validation::Checked::Valid(
                    json::accessor::GenericComponentType(json::accessor::ComponentType::F32),
                ),
                extensions: None,
                extras: Default::default(),
                name: None,
                type_: json::validation::Checked::Valid(json::accessor::Type::Vec3),
                min: None,
                max: None,
                normalized: false,
                sparse: None,
            });

            Some(idx)
        } else {
            None
        };

        // --- Create index buffer and accessor ---
        let index_bytes: Vec<u8> = indices.iter().flat_map(|i| i.to_le_bytes()).collect();
        let index_offset = buffer_data.len();
        buffer_data.extend_from_slice(&index_bytes);

        let index_view_idx = buffer_views.len();
        buffer_views.push(json::buffer::View {
            buffer: json::Index::new(0),
            byte_length: USize64::from(index_bytes.len()),
            byte_offset: Some(USize64::from(index_offset)),
            byte_stride: None,
            extensions: None,
            extras: Default::default(),
            name: None,
            target: Some(json::validation::Checked::Valid(
                json::buffer::Target::ElementArrayBuffer,
            )),
        });

        let index_accessor_idx = accessors.len();
        accessors.push(json::Accessor {
            buffer_view: Some(json::Index::new(index_view_idx as u32)),
            byte_offset: Some(USize64(0)),
            count: USize64::from(indices.len()),
            component_type: json::validation::Checked::Valid(json::accessor::GenericComponentType(
                json::accessor::ComponentType::U16,
            )),
            extensions: None,
            extras: Default::default(),
            name: None,
            type_: json::validation::Checked::Valid(json::accessor::Type::Scalar),
            min: None,
            max: None,
            normalized: false,
            sparse: None,
        });

        // --- Create mesh primitive with triangles ---
        let mut attributes = std::collections::BTreeMap::new();
        attributes.insert(
            json::validation::Checked::Valid(json::mesh::Semantic::Positions),
            json::Index::new(position_accessor_idx as u32),
        );

        if let Some(normal_idx) = normal_accessor_idx {
            attributes.insert(
                json::validation::Checked::Valid(json::mesh::Semantic::Normals),
                json::Index::new(normal_idx as u32),
            );
        }

        let primitive = json::mesh::Primitive {
            attributes,
            extensions: None,
            extras: Default::default(),
            indices: Some(json::Index::new(index_accessor_idx as u32)),
            material: None,
            mode: json::validation::Checked::Valid(json::mesh::Mode::Triangles),
            targets: None,
        };

        // Create mesh
        meshes.push(json::Mesh {
            extensions: None,
            extras: Default::default(),
            name: None,
            primitives: vec![primitive],
            weights: None,
        });
    }

    // If no meshes were created, return an error
    if meshes.is_empty() {
        return Err(anyhow::anyhow!("No valid meshes found in TMD file"));
    }

    // Create nodes for each mesh
    let nodes: Vec<json::Node> = meshes
        .iter()
        .enumerate()
        .map(|(i, _)| json::Node {
            camera: None,
            children: None,
            extensions: None,
            extras: Default::default(),
            matrix: None,
            mesh: Some(json::Index::new(i as u32)),
            name: None,
            rotation: None,
            scale: None,
            translation: None,
            skin: None,
            weights: None,
        })
        .collect();

    // Create scene
    let scene = json::Scene {
        extensions: None,
        extras: Default::default(),
        name: None,
        nodes: nodes
            .iter()
            .enumerate()
            .map(|(i, _)| json::Index::new(i as u32))
            .collect(),
    };

    // Build root
    root.accessors = accessors;
    root.buffers = vec![json::Buffer {
        byte_length: USize64::from(buffer_data.len()),
        extensions: None,
        extras: Default::default(),
        name: None,
        uri: Some(format!(
            "{}.bin",
            output_path.file_stem().unwrap().to_string_lossy()
        )),
    }];
    root.buffer_views = buffer_views;
    root.meshes = meshes;
    root.nodes = nodes;
    root.scenes = vec![scene];
    root.scene = Some(json::Index::new(0));

    // Write glTF JSON
    let gltf_json = json::serialize::to_string_pretty(&root)?;
    fs::write(output_path, gltf_json)?;

    // Write binary buffer
    let bin_path = output_path.with_extension("bin");
    fs::write(bin_path, buffer_data)?;

    Ok(())
}
