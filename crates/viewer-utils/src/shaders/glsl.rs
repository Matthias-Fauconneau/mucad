// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Convert WGSL shaders with imports to GLSL ES 3.00 for WebGL2, e.g. for three.js, with the
//! feature `glsl`.

use std::collections::BTreeMap;
use std::error::Error;

use naga::back::glsl;
use naga_oil::compose::{
    ComposableModuleDescriptor, Composer, NagaModuleDescriptor, ShaderLanguage, ShaderType,
};

use super::ShaderSource;

/// A shader in GLSL ES 3.00 with the names, which naga generated, e.g. for three.js.
pub struct WebGlShader {
    /// The vertex shader.
    pub vertex: String,
    /// The fragment shader.
    pub fragment: String,
    /// The names of the vertex attributes in GLSL by their names in WGSL, e.g.
    /// `_p2vs_location0` of `position`.
    pub attributes: BTreeMap<String, String>,
    /// The uniform blocks by the names of their variables in WGSL, e.g. `config`.
    pub uniform_blocks: BTreeMap<String, UniformBlock>,
}

/// A uniform block, e.g. of `var<uniform> config: GridConfig;`.
pub struct UniformBlock {
    /// The name of the block in GLSL, e.g. `GridConfig_block_1Fragment`.
    pub name: String,
    /// The names and the number of floats of the members of the struct in the order of their
    /// layout, e.g. `("view_angle", 3)` of a `vec3<f32>`.
    pub members: Vec<(String, u32)>,
}

/// Convert `shader` with the entry points `vertex` and `fragment`, which imports its libraries,
/// e.g. with `#import mucad_viewer_utils::grid::{…}`, to GLSL ES 3.00 for WebGL2.
///
/// The coordinate space is not adjusted, i.e. the clip space is the one of WebGL.
pub(super) fn to_webgl(shader: &ShaderSource) -> Result<WebGlShader, Box<dyn Error>> {
    let mut composer = Composer::default();
    for library in shader.imports {
        let added = composer.add_composable_module(ComposableModuleDescriptor {
            source: library.source,
            file_path: library.name,
            language: ShaderLanguage::Wgsl,
            ..Default::default()
        });
        if let Err(error) = added {
            return Err(error.emit_to_string(&composer).into());
        }
    }
    let module = composer.make_naga_module(NagaModuleDescriptor {
        source: shader.source,
        file_path: shader.name,
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
    // `layout(location = 0) in vec3 _p2vs_location0;`, the names in WGSL are the ones of the
    // arguments of the vertex shader.
    let attribute = regex::Regex::new(r"^layout\(location = (\d+)\) in \w+ (\w+);$")?;
    let wgsl_attributes = vertex_attributes(&module)?;
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
                    .map(|member| {
                        let member_name = member.name.clone().ok_or("A member without name")?;
                        let floats = floats(&module.types[member.ty].inner)
                            .ok_or(format!("The member `{member_name}` is no float type"))?;
                        Ok::<_, Box<dyn Error>>((member_name, floats))
                    })
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
                    let location: u32 = captures[1].parse()?;
                    let name = wgsl_attributes
                        .get(&location)
                        .ok_or(format!("No vertex attribute at location {location}"))?;
                    shader
                        .attributes
                        .insert(name.clone(), captures[2].to_string());
                }
            }
            shader.vertex = out;
        } else {
            shader.fragment = out;
        }
    }
    Ok(shader)
}

/// The names of the vertex attributes in WGSL by their locations, the arguments of the vertex
/// shader with `@location`.
fn vertex_attributes(module: &naga::Module) -> Result<BTreeMap<u32, String>, Box<dyn Error>> {
    let vertex = module
        .entry_points
        .iter()
        .find(|entry_point| entry_point.stage == naga::ShaderStage::Vertex)
        .ok_or("No vertex shader")?;
    vertex
        .function
        .arguments
        .iter()
        .filter_map(|argument| match argument.binding {
            Some(naga::Binding::Location { location, .. }) => Some((location, argument)),
            _ => None,
        })
        .map(|(location, argument)| {
            let name = argument.name.clone().ok_or(format!(
                "The vertex attribute at location {location} has no name"
            ))?;
            Ok((location, name))
        })
        .collect()
}

/// The number of floats of a float, a float vector or a float matrix, e.g. 3 of `vec3<f32>`.
fn floats(ty: &naga::TypeInner) -> Option<u32> {
    let float = |scalar: &naga::Scalar| scalar.kind == naga::ScalarKind::Float;
    match ty {
        naga::TypeInner::Scalar(scalar) if float(scalar) => Some(1),
        naga::TypeInner::Vector { size, scalar } if float(scalar) => Some(*size as u32),
        naga::TypeInner::Matrix {
            columns,
            rows,
            scalar,
        } if float(scalar) => Some(*columns as u32 * *rows as u32),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn grid() {
        use crate::shaders::{GridWeb, Shader};
        let shader = GridWeb::to_webgl().expect("The grid in GLSL");
        assert!(shader.vertex.starts_with("#version 300 es\n"));
        assert!(shader.fragment.contains("fwidth("));
        assert_eq!(shader.attributes["position"], "_p2vs_location0");
        let config = &shader.uniform_blocks["config"];
        assert!(config.name.ends_with("Fragment"), "{}", config.name);
        fn members(block: &UniformBlock) -> Vec<(&str, u32)> {
            block
                .members
                .iter()
                .map(|(name, floats)| (name.as_str(), *floats))
                .collect()
        }
        assert_eq!(
            members(config),
            [
                ("radius", 1),
                ("zoom_level", 1),
                ("view_angle", 3),
                ("grid_color", 3),
                ("x_axis_color", 3),
                ("y_axis_color", 3)
            ]
        );
        assert_eq!(
            members(&shader.uniform_blocks["camera"]),
            [("clip_from_world", 16), ("z_offset", 1)]
        );
    }
}
