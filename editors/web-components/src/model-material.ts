// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

import * as THREE from "three";
import { model } from "./generated/model";
import { setUniforms, uniformsGroup } from "./uniforms";

/** The camera of the model materials, which they all share. */
const cameraUniforms = uniformsGroup(model.uniformBlocks.camera);

/** The lighting of the model materials, which they all share, the default of the Bevy viewer. */
const lightingUniforms = uniformsGroup(model.uniformBlocks.lighting);
setUniforms(lightingUniforms, model.uniformBlocks.lighting, model.lighting);

/** Set the camera of all model materials, e.g. in each frame after the camera is updated. */
export function updateModelCamera(camera: THREE.Camera) {
  const clipFromWorld = new THREE.Matrix4().multiplyMatrices(
    camera.projectionMatrix,
    camera.matrixWorldInverse,
  );
  setUniforms(cameraUniforms, model.uniformBlocks.camera, [
    ...clipFromWorld.elements,
    ...camera.matrixWorldInverse.elements,
  ]);
}

/**
 * `geometry` with its positions and normals also as the attributes of the model shader, whose
 * names naga generated. Its positions and normals are world coordinates.
 */
export function modelGeometry(geometry: THREE.BufferGeometry): THREE.BufferGeometry {
  const [position, normal] = model.attributes;
  geometry.setAttribute(position, geometry.getAttribute("position"));
  geometry.setAttribute(normal, geometry.getAttribute("normal"));
  return geometry;
}

/**
 * The material of lit 3D models with the lighting of the Bevy viewer, from the WGSL shader which
 * `cargo xtask web` converts to GLSL, for geometries of {@link modelGeometry}.
 *
 * Its color is linear and blended like in Bevy, if the scene is rendered into a linear render
 * target, which is converted to sRGB afterwards, e.g. with `OutputPass`.
 */
export class ModelMaterial extends THREE.RawShaderMaterial {
  /** The color of this material. */
  private surfaceUniforms = uniformsGroup(model.uniformBlocks.surface);

  /** A material of `color` with `opacity`, which is transparent below 1. */
  constructor(color: THREE.Color, opacity = 1, parameters: THREE.ShaderMaterialParameters = {}) {
    super({
      ...parameters,
      glslVersion: THREE.GLSL3,
      vertexShader: model.vertexShader,
      fragmentShader: model.fragmentShader,
      transparent: opacity < 1,
    });
    setUniforms(this.surfaceUniforms, model.uniformBlocks.surface, [
      color.r,
      color.g,
      color.b,
      opacity,
    ]);
    this.uniformsGroups = [cameraUniforms, lightingUniforms, this.surfaceUniforms];
  }

  dispose() {
    this.surfaceUniforms.dispose();
    super.dispose();
  }
}
