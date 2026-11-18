// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

import * as THREE from "three";
import { grid } from "./generated/grid";
import { member, setUniforms, uniformsGroup } from "./uniforms";

/**
 * The grid of the µcad viewers in the plane Z = 0, from the WGSL shader which
 * `cargo xtask web` converts to GLSL: a zoom-adaptive grid with colored axes, which fades out.
 */
export class Grid {
  /** The plane of the grid, which is added to the scene. */
  readonly mesh: THREE.Mesh;
  private cameraUniforms = uniformsGroup(grid.uniformBlocks.camera);
  private configUniforms = uniformsGroup(grid.uniformBlocks.config);
  private positions: THREE.BufferAttribute;
  /** The radius, which the plane covers. */
  private radius = 0;

  constructor() {
    // The shader has no model matrix, the positions are world coordinates.
    this.positions = new THREE.BufferAttribute(new Float32Array(4 * 3), 3);
    const geometry = new THREE.BufferGeometry();
    geometry.setAttribute("position", this.positions);
    geometry.setAttribute(grid.attributes[0], this.positions);
    geometry.setIndex([0, 1, 2, 0, 2, 3]);
    const material = new THREE.RawShaderMaterial({
      glslVersion: THREE.GLSL3,
      vertexShader: grid.vertexShader,
      fragmentShader: grid.fragmentShader,
      transparent: true,
      depthWrite: false,
      side: THREE.DoubleSide,
    });
    material.uniformsGroups = [this.cameraUniforms, this.configUniforms];
    this.mesh = new THREE.Mesh(geometry, material);
    // The bounds of the plane change with the radius, it is always drawn.
    this.mesh.frustumCulled = false;
  }

  /**
   * Update the grid to `camera` and `config`, the floats of the members of `GridConfig`, e.g. of
   * `CameraNavigation.gridConfig()`, after the camera is updated.
   */
  update(camera: THREE.Camera, config: ArrayLike<number>) {
    const block = grid.uniformBlocks.config;
    setUniforms(this.configUniforms, block, config);

    // The plane covers the grid, which is faded out at the double radius.
    const radius = 2 * member(block, config, "radius");
    if (radius !== this.radius) {
      this.radius = radius;
      this.positions.copyArray([-radius, -radius, 0, radius, -radius, 0, radius, radius, 0, -radius, radius, 0]);
      this.positions.needsUpdate = true;
    }

    // Like in Bevy, the plane is a little below Z = 0, so that 2D models on it stay visible.
    const clipFromWorld = new THREE.Matrix4().multiplyMatrices(
      camera.projectionMatrix,
      camera.matrixWorldInverse,
    );
    const zOffset = -0.05 / member(block, config, "zoom_level");
    setUniforms(this.cameraUniforms, grid.uniformBlocks.camera, [...clipFromWorld.elements, zOffset]);
  }

  /** Free the GPU resources. */
  dispose() {
    this.mesh.geometry.dispose();
    (this.mesh.material as THREE.Material).dispose();
  }
}
