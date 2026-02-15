//! TMD to glTF converter

use anyhow::Result;
use gltf_json as json;
use gltf_json::validation::USize64;
use psxutils::formats::Tmd;
use std::fs;
use std::path::Path;

/// Convert a TMD model to glTF 2.0 format
pub fn tmd_to_gltf(tmd: &Tmd, output_path: &Path) -> Result<()> {
    let mut root = json::Root::default();
    let mut buffers = Vec::new();
    let mut buffer_views = Vec::new();
    let mut accessors = Vec::new();
    let mut meshes = Vec::new();

    // For now, export only vertices as a simple point cloud
    // Full primitive export would require implementing the primitive parser in TMD

    for (_obj_idx, object) in tmd.objects.iter().enumerate() {
        // Skip empty objects
        if object.vertices.is_empty() {
            continue;
        }

        // Convert vertices to f32 positions
        let scale = if object.scale == 0 {
            1.0
        } else {
            object.scale as f32
        };

        let mut positions: Vec<f32> = Vec::new();
        let mut min = [f32::MAX, f32::MAX, f32::MAX];
        let mut max = [f32::MIN, f32::MIN, f32::MIN];

        for vertex in &object.vertices {
            let x = vertex.x as f32 / scale;
            let y = vertex.y as f32 / scale;
            let z = vertex.z as f32 / scale;

            positions.push(x);
            positions.push(y);
            positions.push(z);

            min[0] = min[0].min(x);
            min[1] = min[1].min(y);
            min[2] = min[2].min(z);

            max[0] = max[0].max(x);
            max[1] = max[1].max(y);
            max[2] = max[2].max(z);
        }

        // Convert to bytes
        let position_bytes: Vec<u8> = positions.iter().flat_map(|f| f.to_le_bytes()).collect();

        let buffer_length = position_bytes.len();

        // Create buffer
        let buffer_index = buffers.len();
        buffers.push(position_bytes);

        // Create buffer view
        let buffer_view_index = buffer_views.len();
        buffer_views.push(json::buffer::View {
            buffer: json::Index::new(buffer_index as u32),
            byte_length: USize64::from(buffer_length),
            byte_offset: None,
            byte_stride: Some(json::buffer::Stride(12)), // 3 floats * 4 bytes
            extensions: None,
            extras: Default::default(),
            name: Some(format!("positions_view_{}", _obj_idx)),
            target: Some(json::validation::Checked::Valid(
                json::buffer::Target::ArrayBuffer,
            )),
        });

        // Create accessor
        let accessor_index = accessors.len();
        accessors.push(json::Accessor {
            buffer_view: Some(json::Index::new(buffer_view_index as u32)),
            byte_offset: Some(USize64(0)),
            count: USize64::from(object.vertices.len()),
            component_type: json::validation::Checked::Valid(json::accessor::GenericComponentType(
                json::accessor::ComponentType::F32,
            )),
            extensions: None,
            extras: Default::default(),
            type_: json::validation::Checked::Valid(json::accessor::Type::Vec3),
            min: Some(json::Value::from(vec![min[0], min[1], min[2]])),
            max: Some(json::Value::from(vec![max[0], max[1], max[2]])),
            name: Some(format!("positions_accessor_{}", _obj_idx)),
            normalized: false,
            sparse: None,
        });

        // Create mesh primitive (as POINTS for now, since we haven't parsed indices)
        let primitive = json::mesh::Primitive {
            attributes: {
                let mut map = std::collections::BTreeMap::new();
                map.insert(
                    json::validation::Checked::Valid(json::mesh::Semantic::Positions),
                    json::Index::new(accessor_index as u32),
                );
                map
            },
            extensions: None,
            extras: Default::default(),
            indices: None, // TODO: Add when primitive parsing is implemented
            material: None,
            mode: json::validation::Checked::Valid(json::mesh::Mode::Points),
            targets: None,
        };

        // Create mesh
        meshes.push(json::Mesh {
            extensions: None,
            extras: Default::default(),
            name: Some(format!("mesh_{}", _obj_idx)),
            primitives: vec![primitive],
            weights: None,
        });
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
            name: Some(format!("node_{}", i)),
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
        name: Some("Scene".to_string()),
        nodes: nodes
            .iter()
            .enumerate()
            .map(|(i, _)| json::Index::new(i as u32))
            .collect(),
    };

    // Build root
    root.accessors = accessors;
    root.buffers = buffers
        .iter()
        .enumerate()
        .map(|(i, b)| json::Buffer {
            byte_length: USize64::from(b.len()),
            extensions: None,
            extras: Default::default(),
            name: Some(format!("buffer_{}", i)),
            uri: Some(format!(
                "{}.bin",
                output_path.file_stem().unwrap().to_string_lossy()
            )),
        })
        .collect();
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
    let all_buffers = buffers.concat();
    fs::write(bin_path, all_buffers)?;

    Ok(())
}
