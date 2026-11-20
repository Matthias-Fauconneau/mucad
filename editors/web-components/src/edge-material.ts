// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

import * as THREE from "three";
import type { EdgeData } from "./compiler/types";
import { edge } from "./generated/edge";
import { setUniforms, uniformsGroup } from "./uniforms";

/** The camera of the edge materials, which they all share. */
const cameraUniforms = uniformsGroup(edge.uniformBlocks.camera);

/**
 * Set the camera of all edge materials, e.g. in each frame after the camera is updated, with the
 * `height` of the render target in pixels.
 */
export function updateEdgeCamera(camera: THREE.Camera, height: number) {
  const clipFromWorld = new THREE.Matrix4().multiplyMatrices(
    camera.projectionMatrix,
    camera.matrixWorldInverse,
  );
  const world = camera.matrixWorld.elements;
  // The size of a pixel at the distance 1 or in an orthographic view, see `in_front`.
  const pixelSize = 2 / (camera.projectionMatrix.elements[5] * height);
  const orthographic = camera instanceof THREE.OrthographicCamera ? 1 : 0;
  setUniforms(cameraUniforms, edge.uniformBlocks.camera, [
    ...clipFromWorld.elements,
    ...[world[12], world[13], world[14]],
    pixelSize,
    ...[world[8], world[9], world[10]],
    orthographic,
  ]);
}

/**
 * The geometry of edge lines for a `THREE.LineSegments` with an {@link EdgeMaterial}, with the
 * attributes of the edge shader, whose names naga generated. The positions are world coordinates.
 */
export function edgeGeometry(lines: EdgeData): THREE.BufferGeometry {
  const { attributes } = edge;
  const positions = new THREE.BufferAttribute(lines.positions, 3);
  return (
    new THREE.BufferGeometry()
      // E.g. for the bounds of the lines.
      .setAttribute("position", positions)
      .setAttribute(attributes.position, positions)
      .setAttribute(attributes.normal_a, new THREE.BufferAttribute(lines.normalsA, 3))
      .setAttribute(attributes.normal_b, new THREE.BufferAttribute(lines.normalsB, 3))
      .setAttribute(attributes.midpoint, new THREE.BufferAttribute(lines.midpoints, 3))
  );
}

/**
 * The material of the edge lines of 3D models like in the Bevy viewer, from the WGSL shader
 * which `cargo xtask web` converts to GLSL: the contour edges are always drawn, the smooth edges
 * only on the silhouette, all moved in front of their faces.
 *
 * Its color is linear like the one of `ModelMaterial`.
 */
export class EdgeMaterial extends THREE.RawShaderMaterial {
  /** The color of this material. */
  private edgeUniforms = uniformsGroup(edge.uniformBlocks.edge);

  /** A material of `color`. */
  constructor(color: THREE.Color) {
    super({
      glslVersion: THREE.GLSL3,
      vertexShader: edge.vertexShader,
      fragmentShader: edge.fragmentShader,
    });
    setUniforms(this.edgeUniforms, edge.uniformBlocks.edge, [color.r, color.g, color.b, 1]);
    this.uniformsGroups = [cameraUniforms, this.edgeUniforms];
  }

  dispose() {
    this.edgeUniforms.dispose();
    super.dispose();
  }
}
