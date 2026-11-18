// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

import * as THREE from "three";
import { grid } from "./generated/grid";

/** A uniform block of the generated grid shader: its name and its members with their floats. */
type UniformBlock = { readonly name: string; readonly members: readonly (readonly [string, number])[] };

/** The value of a uniform of `floats` floats: a number, a `Vector3` or a `Matrix4`. */
function uniformValue(floats: number): number | THREE.Vector3 | THREE.Matrix4 {
  switch (floats) {
    case 1:
      return 0;
    case 3:
      return new THREE.Vector3();
    case 16:
      return new THREE.Matrix4();
    default:
      throw new Error(`A uniform of ${floats} floats is not supported`);
  }
}

/** The uniforms group of `block`, whose uniforms are in the order of its std140 layout. */
function uniformsGroup(block: UniformBlock): THREE.UniformsGroup {
  const group = new THREE.UniformsGroup();
  group.setName(block.name);
  for (const [, floats] of block.members) group.add(new THREE.Uniform(uniformValue(floats)));
  return group;
}

/** Set the uniforms of `group` of `block` to `values`, the floats of its members one after another. */
function setUniforms(group: THREE.UniformsGroup, block: UniformBlock, values: ArrayLike<number>) {
  let offset = 0;
  block.members.forEach(([, floats], index) => {
    const uniform = group.uniforms[index] as THREE.Uniform;
    if (typeof uniform.value === "number") uniform.value = values[offset];
    else uniform.value.fromArray(values, offset);
    offset += floats;
  });
}

/** The float of `member` of `block` in `values`, the floats of its members one after another. */
function member(block: UniformBlock, values: ArrayLike<number>, name: string): number {
  let offset = 0;
  for (const [memberName, floats] of block.members) {
    if (memberName === name) return values[offset];
    offset += floats;
  }
  throw new Error(`No member ${name} in ${block.name}`);
}

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
    geometry.setAttribute(grid.positionAttribute, this.positions);
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
