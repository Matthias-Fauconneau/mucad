// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Binary glTF 2.0 (GLB) exporter.

use std::io::Write;

use mucad_core::{Geometry, TriangleMesh};
use mucad_lang_types::{ModelTree, ModelType, Value};
use mucad_render::{GeometryOutput, Renderer};
use serde_json::{Value as JsonValue, json};

use crate::{Export, ExportError};

/// Exports rendered 3D geometry to a binary glTF 2.0 file.
#[derive(Debug, Default)]
pub struct GlbExporter {
    /// The render resolution.
    pub resolution: mucad_render::RenderResolution,
}

#[derive(Default)]
struct MeshData {
    positions: Vec<[f32; 3]>,
    indices: Vec<u32>,
    color: Option<[f32; 4]>,
    name: String,
}

fn mesh_data(output: &GeometryOutput, name: String) -> Option<MeshData> {
    let Geometry::Geometry3D(geometry) = &output.geometry else {
        return None;
    };
    let mesh = TriangleMesh::from(geometry);
    if mesh.is_empty() {
        return None;
    }

    // glTF uses meters and a right-handed, Y-up coordinate system. µcad uses mm and Z-up.
    let positions: Vec<_> = mesh
        .positions
        .iter()
        .map(|p| [p.x * 0.001, p.z * 0.001, -p.y * 0.001])
        .collect();
    let indices = mesh
        .triangle_indices
        .iter()
        .flat_map(|triangle| [triangle.0, triangle.1, triangle.2])
        .collect();

    Some(MeshData {
        positions,
        indices,
        color: output.attr.fill_color.map(Into::into),
        name,
    })
}

fn append_f32(buffer: &mut Vec<u8>, value: f32) {
    buffer.extend_from_slice(&value.to_le_bytes());
}

fn pad(buffer: &mut Vec<u8>, alignment: usize, byte: u8) {
    while !buffer.len().is_multiple_of(alignment) {
        buffer.push(byte);
    }
}

impl Export for GlbExporter {
    fn export(&self, model: &ModelTree, writer: &mut dyn Write) -> Result<Value, ExportError> {
        let renderer = Renderer::new().with_resolution(self.resolution.clone());
        let tree = crate::render(renderer, model)?;
        let outputs = &tree.root().get().outputs;
        let meshes: Vec<_> = outputs
            .iter()
            .enumerate()
            .filter_map(|(index, output)| {
                let name = output
                    .id
                    .clone()
                    .unwrap_or_else(|| format!("Part {}", index + 1));
                mesh_data(&output, name)
            })
            .collect();

        let mut binary = Vec::new();
        let mut buffer_views = Vec::new();
        let mut accessors = Vec::new();
        let mut gltf_meshes = Vec::new();
        let mut nodes = Vec::new();
        let mut materials = Vec::new();

        for (mesh_index, mesh) in meshes.iter().enumerate() {
            pad(&mut binary, 4, 0);
            let position_offset = binary.len();
            for position in &mesh.positions {
                for coordinate in position {
                    append_f32(&mut binary, *coordinate);
                }
            }
            let position_length = binary.len() - position_offset;
            buffer_views.push(json!({
                "buffer": 0, "byteOffset": position_offset, "byteLength": position_length,
                "target": 34962
            }));
            let (min, max) = bounds(&mesh.positions);
            let position_accessor = accessors.len();
            accessors.push(json!({
                "bufferView": buffer_views.len() - 1, "componentType": 5126,
                "count": mesh.positions.len(), "type": "VEC3", "min": min, "max": max
            }));

            pad(&mut binary, 4, 0);
            let index_offset = binary.len();
            for index in &mesh.indices {
                binary.extend_from_slice(&index.to_le_bytes());
            }
            let index_length = binary.len() - index_offset;
            buffer_views.push(json!({
                "buffer": 0, "byteOffset": index_offset, "byteLength": index_length,
                "target": 34963
            }));
            let index_accessor = accessors.len();
            accessors.push(json!({
                "bufferView": buffer_views.len() - 1, "componentType": 5125,
                "count": mesh.indices.len(), "type": "SCALAR"
            }));

            let mut primitive = json!({
                "attributes": {"POSITION": position_accessor},
                "indices": index_accessor,
                "mode": 4
            });
            if let Some(color) = mesh.color {
                let material_index = materials.len();
                materials.push(json!({
                    "name": format!("{} material", mesh.name),
                    "pbrMetallicRoughness": {
                        "baseColorFactor": color,
                        "metallicFactor": 0.0,
                        "roughnessFactor": 0.8
                    },
                    "alphaMode": if color[3] < 1.0 { "BLEND" } else { "OPAQUE" }
                }));
                primitive["material"] = json!(material_index);
            }
            gltf_meshes.push(json!({"name": mesh.name, "primitives": [primitive]}));
            nodes.push(json!({"name": mesh.name, "mesh": mesh_index}));
        }

        let mut document: JsonValue = json!({
            "asset": {"version": "2.0", "generator": "µcad"},
            "scene": 0,
            "scenes": [{"nodes": (0..nodes.len()).collect::<Vec<_>>()}],
            "nodes": nodes,
            "meshes": gltf_meshes,
            "bufferViews": buffer_views,
            "accessors": accessors
        });
        if !binary.is_empty() {
            document["buffers"] = json!([{"byteLength": binary.len()}]);
        }
        if !materials.is_empty() {
            document["materials"] = json!(materials);
        }

        let mut json_bytes = serde_json::to_vec(&document)
            .map_err(|error| ExportError::Custom(format!("Could not encode GLB JSON: {error}")))?;
        pad(&mut json_bytes, 4, b' ');
        pad(&mut binary, 4, 0);

        let binary_chunk_length = if binary.is_empty() {
            0
        } else {
            8 + binary.len()
        };
        let total_length = 12 + 8 + json_bytes.len() + binary_chunk_length;
        let total_length = u32::try_from(total_length)
            .map_err(|_| ExportError::Custom("GLB file exceeds the 4 GiB format limit".into()))?;

        writer.write_all(b"glTF")?;
        writer.write_all(&2u32.to_le_bytes())?;
        writer.write_all(&total_length.to_le_bytes())?;
        writer.write_all(&(json_bytes.len() as u32).to_le_bytes())?;
        writer.write_all(&0x4E4F_534Au32.to_le_bytes())?;
        writer.write_all(&json_bytes)?;
        if !binary.is_empty() {
            writer.write_all(&(binary.len() as u32).to_le_bytes())?;
            writer.write_all(&0x004E_4942u32.to_le_bytes())?;
            writer.write_all(&binary)?;
        }
        Ok(Value::None)
    }

    fn model_type(&self) -> ModelType {
        ModelType::Geometry3D
    }
}

fn bounds(positions: &[[f32; 3]]) -> ([f32; 3], [f32; 3]) {
    let mut min = [f32::INFINITY; 3];
    let mut max = [f32::NEG_INFINITY; 3];
    for position in positions {
        for axis in 0..3 {
            min[axis] = min[axis].min(position[axis]);
            max[axis] = max[axis].max(position[axis]);
        }
    }
    (min, max)
}
