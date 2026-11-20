// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! The shaders of the µcad viewers in WGSL, e.g. for Bevy, and converted to GLSL for WebGL.
//!
//! Each shader is a type with its [`Shader`] implementation, e.g. [`Lighting`]: the libraries
//! with `#define_import_path mucad_viewer_utils::…` without bindings, which the viewers import,
//! and the shaders for WebGL2 with their vertex and fragment shader, which import libraries.
//!
//! With the feature `glsl`, the shaders for WebGL2 are converted to GLSL ES 3.00, e.g. by
//! `cargo xtask web`.

#[cfg(feature = "glsl")]
mod glsl;

#[cfg(feature = "glsl")]
pub use glsl::{UniformBlock, WebGlShader};

/// A shader in WGSL with its name and the libraries, which it imports.
pub trait Shader {
    /// The file name, e.g. `lighting.wgsl`, e.g. the asset path of Bevy or the module name of
    /// naga_oil.
    const NAME: &'static str;

    /// The WGSL source.
    const SOURCE: &'static str;

    /// The libraries, which the source imports, e.g. `&[source::<Lighting>()]`.
    const IMPORTS: &'static [ShaderSource] = &[];

    /// The shader in GLSL ES 3.00 for WebGL2, see [`ShaderSource::to_webgl`].
    #[cfg(feature = "glsl")]
    fn to_webgl() -> Result<WebGlShader, Box<dyn std::error::Error>>
    where
        Self: Sized,
    {
        source::<Self>().to_webgl()
    }
}

/// The name, the source and the imports of a [`Shader`], e.g. for a list of shaders.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ShaderSource {
    /// See [`Shader::NAME`].
    pub name: &'static str,
    /// See [`Shader::SOURCE`].
    pub source: &'static str,
    /// See [`Shader::IMPORTS`].
    pub imports: &'static [ShaderSource],
}

impl ShaderSource {
    /// The shader with the entry points `vertex` and `fragment` in GLSL ES 3.00 for WebGL2,
    /// with the names, which naga generated, e.g. of its attributes and uniform blocks.
    ///
    /// The coordinate space is not adjusted, i.e. the clip space is the one of WebGL.
    #[cfg(feature = "glsl")]
    pub fn to_webgl(&self) -> Result<WebGlShader, Box<dyn std::error::Error>> {
        glsl::to_webgl(self)
    }
}

/// The name, the source and the imports of the shader `S`.
pub const fn source<S: Shader>() -> ShaderSource {
    ShaderSource {
        name: S::NAME,
        source: S::SOURCE,
        imports: S::IMPORTS,
    }
}

/// The libraries, which the viewers import, e.g. to register them all in Bevy.
pub const LIBRARIES: &[ShaderSource] = &[
    source::<Grid>(),
    source::<Lighting>(),
    source::<Silhouette>(),
];

/// The shaders for WebGL2, which `cargo xtask web` converts to GLSL.
pub const WEB_SHADERS: &[ShaderSource] = &[
    source::<GridWeb>(),
    source::<ModelWeb>(),
    source::<EdgeWeb>(),
];

/// The grid library with `#define_import_path mucad_viewer_utils::grid`: `GridConfig` and
/// `grid_color()`, see [`crate::GridConfig`].
pub struct Grid;

impl Shader for Grid {
    const NAME: &'static str = "grid.wgsl";
    const SOURCE: &'static str = include_str!("../shaders/grid.wgsl");
}

/// The lighting library with `#define_import_path mucad_viewer_utils::lighting`: `Lighting`
/// and `shade()` for the models, see [`crate::Lighting`].
pub struct Lighting;

impl Shader for Lighting {
    const NAME: &'static str = "lighting.wgsl";
    const SOURCE: &'static str = include_str!("../shaders/lighting.wgsl");
}

/// The silhouette library with `#define_import_path mucad_viewer_utils::silhouette`:
/// `is_silhouette()`, `to_viewer()`, `in_front()` and `HIDDEN` for the vertex shaders of the
/// edge lines, see [`crate::EdgeLines`].
pub struct Silhouette;

impl Shader for Silhouette {
    const NAME: &'static str = "silhouette.wgsl";
    const SOURCE: &'static str = include_str!("../shaders/silhouette.wgsl");
}

/// The grid for WebGL2, e.g. three.js.
pub struct GridWeb;

impl Shader for GridWeb {
    const NAME: &'static str = "grid_web.wgsl";
    const SOURCE: &'static str = include_str!("../shaders/grid_web.wgsl");
    const IMPORTS: &'static [ShaderSource] = &[source::<Grid>()];
}

/// The lit models for WebGL2, e.g. three.js.
pub struct ModelWeb;

impl Shader for ModelWeb {
    const NAME: &'static str = "model_web.wgsl";
    const SOURCE: &'static str = include_str!("../shaders/model_web.wgsl");
    const IMPORTS: &'static [ShaderSource] = &[source::<Lighting>()];
}

/// The edge lines for WebGL2, e.g. three.js.
pub struct EdgeWeb;

impl Shader for EdgeWeb {
    const NAME: &'static str = "edge_web.wgsl";
    const SOURCE: &'static str = include_str!("../shaders/edge_web.wgsl");
    const IMPORTS: &'static [ShaderSource] = &[source::<Silhouette>()];
}
