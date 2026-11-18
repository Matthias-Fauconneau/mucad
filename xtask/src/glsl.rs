// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Convert WGSL shaders with imports to GLSL ES 3.00 for WebGL2, e.g. for three.js.

use std::collections::BTreeMap;
use std::error::Error;

use naga::back::glsl;
use naga_oil::compose::{
    ComposableModuleDescriptor, Composer, NagaModuleDescriptor, ShaderLanguage, ShaderType,
};

/// A shader in GLSL ES 3.00 with the names, which naga generated, e.g. for three.js.
pub struct WebGlShader {
    /// The vertex shader.
    pub vertex: String,
    /// The fragment shader.
    pub fragment: String,
    /// The names of the vertex attributes by their locations, e.g. `_p2vs_location0`.
    pub attributes: BTreeMap<u32, String>,
    /// The uniform blocks by the names of their variables in WGSL, e.g. `config`.
    pub uniform_blocks: BTreeMap<String, UniformBlock>,
}

/// A uniform block, e.g. of `var<uniform> config: GridConfig;`.
pub struct UniformBlock {
    /// The name of the block in GLSL, e.g. `GridConfig_block_1Fragment`.
    pub name: String,
    /// The names of the members of the struct in the order of their layout.
    pub members: Vec<String>,
}

/// Convert `source` with the entry points `vertex` and `fragment`, which imports `libraries`,
/// e.g. with `#import mucad_viewer_utils::grid::{…}`, to GLSL ES 3.00 for WebGL2.
///
/// The coordinate space is not adjusted, i.e. the clip space is the one of WebGL.
pub fn to_webgl(
    source: &str,
    file_path: &str,
    libraries: &[(&str, &str)],
) -> Result<WebGlShader, Box<dyn Error>> {
    let mut composer = Composer::default();
    for (library_path, library) in libraries {
        let added = composer.add_composable_module(ComposableModuleDescriptor {
            source: library,
            file_path: library_path,
            language: ShaderLanguage::Wgsl,
            ..Default::default()
        });
        if let Err(error) = added {
            return Err(error.emit_to_string(&composer).into());
        }
    }
    let module = composer.make_naga_module(NagaModuleDescriptor {
        source,
        file_path,
        shader_type: ShaderType::Wgsl,
        ..Default::default()
    });
    let module = match module {
        Ok(module) => module,
        Err(error) => return Err(error.emit_to_string(&composer).into()),
    };
    let info = naga::valid::Validator::new(
        naga::valid::ValidationFlags::all(),
        naga::valid::Capabilities::empty(),
    )
    .validate(&module)?;

    let options = glsl::Options {
        version: glsl::Version::Embedded {
            version: 300,
            is_webgl: true,
        },
        // three.js has the coordinate space of WebGL already, nothing is flipped.
        writer_flags: glsl::WriterFlags::empty(),
        ..Default::default()
    };

    let mut shader = WebGlShader {
        vertex: String::new(),
        fragment: String::new(),
        attributes: BTreeMap::new(),
        uniform_blocks: BTreeMap::new(),
    };
    // The reflection of naga has no attributes for GLSL ES, they are declared like
    // `layout(location = 0) in vec3 _p2vs_location0;`.
    let attribute = regex::Regex::new(r"^layout\(location = (\d+)\) in \w+ (\w+);$")?;
    for stage in [naga::ShaderStage::Vertex, naga::ShaderStage::Fragment] {
        let entry_point = match stage {
            naga::ShaderStage::Vertex => "vertex",
            _ => "fragment",
        };
        let mut out = String::new();
        let reflection = glsl::Writer::new(
            &mut out,
            &module,
            &info,
            &options,
            &glsl::PipelineOptions {
                shader_stage: stage,
                entry_point: entry_point.into(),
                multiview: None,
            },
            naga::proc::BoundsCheckPolicies::default(),
        )?
        .write()?;

        for (handle, block_name) in reflection.uniforms {
            let variable = &module.global_variables[handle];
            let name = variable.name.clone().ok_or("A uniform without name")?;
            let members = match &module.types[variable.ty].inner {
                naga::TypeInner::Struct { members, .. } => members
                    .iter()
                    .map(|member| member.name.clone().ok_or("A member without name"))
                    .collect::<Result<_, _>>()?,
                _ => return Err(format!("The uniform `{name}` is no struct").into()),
            };
            if shader.uniform_blocks.contains_key(&name) {
                // Its blocks would have different names in the stages.
                return Err(format!("The uniform `{name}` is used in both stages").into());
            }
            shader.uniform_blocks.insert(
                name,
                UniformBlock {
                    name: block_name,
                    members,
                },
            );
        }
        if stage == naga::ShaderStage::Vertex {
            for line in out.lines() {
                if let Some(captures) = attribute.captures(line) {
                    shader
                        .attributes
                        .insert(captures[1].parse()?, captures[2].to_string());
                }
            }
            shader.vertex = out;
        } else {
            shader.fragment = out;
        }
    }
    Ok(shader)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn grid() {
        let shader = to_webgl(
            mucad_viewer_utils::GRID_WEB_WGSL,
            "grid_web.wgsl",
            &[("grid.wgsl", mucad_viewer_utils::GRID_WGSL)],
        )
        .expect("The grid in GLSL");
        assert!(shader.vertex.starts_with("#version 300 es\n"));
        assert!(shader.fragment.contains("fwidth("));
        assert_eq!(shader.attributes[&0], "_p2vs_location0");
        let config = &shader.uniform_blocks["config"];
        assert!(config.name.ends_with("Fragment"), "{}", config.name);
        assert_eq!(
            config.members,
            [
                "radius",
                "zoom_level",
                "view_angle",
                "grid_color",
                "x_axis_color",
                "y_axis_color"
            ]
        );
        assert_eq!(
            shader.uniform_blocks["camera"].members,
            ["clip_from_world", "z_offset"]
        );
    }
}
