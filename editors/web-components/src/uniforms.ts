// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

import * as THREE from "three";

/**
 * A uniform block of a generated shader, e.g. of `generated/grid.ts`: its name in GLSL and its
 * members with their numbers of floats in the order of their std140 layout.
 */
export type UniformBlock = {
  readonly name: string;
  readonly members: readonly (readonly [string, number])[];
};

/** The value of a uniform of `floats` floats: a number, a `Vector3`, a `Vector4` or a `Matrix4`. */
function uniformValue(floats: number): number | THREE.Vector3 | THREE.Vector4 | THREE.Matrix4 {
  switch (floats) {
    case 1:
      return 0;
    case 3:
      return new THREE.Vector3();
    case 4:
      return new THREE.Vector4();
    case 16:
      return new THREE.Matrix4();
    default:
      throw new Error(`A uniform of ${floats} floats is not supported`);
  }
}

/** The uniforms group of `block`, whose uniforms are in the order of its std140 layout. */
export function uniformsGroup(block: UniformBlock): THREE.UniformsGroup {
  const group = new THREE.UniformsGroup();
  group.setName(block.name);
  for (const [, floats] of block.members) group.add(new THREE.Uniform(uniformValue(floats)));
  return group;
}

/** Set the uniforms of `group` of `block` to `values`, the floats of its members one after another. */
export function setUniforms(
  group: THREE.UniformsGroup,
  block: UniformBlock,
  values: ArrayLike<number>,
) {
  let offset = 0;
  block.members.forEach(([, floats], index) => {
    const uniform = group.uniforms[index] as THREE.Uniform;
    if (typeof uniform.value === "number") uniform.value = values[offset];
    else uniform.value.fromArray(values, offset);
    offset += floats;
  });
}

/** The float of `member` of `block` in `values`, the floats of its members one after another. */
export function member(block: UniformBlock, values: ArrayLike<number>, name: string): number {
  let offset = 0;
  for (const [memberName, floats] of block.members) {
    if (memberName === name) return values[offset];
    offset += floats;
  }
  throw new Error(`No member ${name} in ${block.name}`);
}
