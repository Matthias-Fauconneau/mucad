// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

import * as THREE from "three";
import { EffectComposer } from "three/examples/jsm/postprocessing/EffectComposer.js";
import { OutputPass } from "three/examples/jsm/postprocessing/OutputPass.js";
import { RenderPass } from "three/examples/jsm/postprocessing/RenderPass.js";
import { LineMaterial } from "three/examples/jsm/lines/LineMaterial.js";
import { LineSegments2 } from "three/examples/jsm/lines/LineSegments2.js";
import { LineSegmentsGeometry } from "three/examples/jsm/lines/LineSegmentsGeometry.js";
import { toCreasedNormals } from "three/examples/jsm/utils/BufferGeometryUtils.js";
import { creaseAngle } from "mucad-wasm";
import { CameraControls, type StandardView } from "./camera-controls";
import type { Color, Geometry, MeshData, SketchData, Triangles } from "./compiler/types";
import { theme } from "./generated/theme";
import { Grid } from "./grid";
import { EdgeMaterial, edgeGeometry, updateEdgeCamera } from "./edge-material";
import { ModelMaterial, modelGeometry, updateModelCamera } from "./model-material";


/** The width of the outlines of 2D sketches in pixels outside of them, like in the Bevy viewer. */
const OUTLINE_WIDTH = 4;

/** The views of 3D models (from the front, the right and the top) and 2D models (from above). */
const VIEWS = {
  "3d": "isometric",
  "2d": "top",
} as const satisfies Record<string, StandardView>;

type ModelKind = keyof typeof VIEWS;

/**
 * A three.js view of µcad models, the Z axis points up like in µcad.
 *
 * The orthographic or perspective camera is navigated with the mouse and the keys by
 * {@link CameraControls}, the key `p` toggles the projection.
 *
 * The scene is rendered into a linear render target and converted to sRGB afterwards, so that
 * transparent objects are blended in linear colors like in the Bevy viewer, e.g. the grid.
 */
export class Viewer {
  private renderer: THREE.WebGLRenderer;
  private composer: EffectComposer;
  private scene = new THREE.Scene();
  /** The pass, which renders the scene with the camera of the controls. */
  private renderPass: RenderPass;
  /** The camera navigation, e.g. to listen to its `select` events or to set its controls. */
  readonly controls: CameraControls;
  /** The radius of the bounds of the camera, which a bigger model increases. */
  private boundsRadius = 0;
  private model = new THREE.Group();
  /** The grid of the Bevy viewer, which is shown with a model. */
  private grid = new Grid();
  /** The kind of the shown model, whose direction the camera was reset to. */
  private kind?: ModelKind;

  constructor(private element: HTMLElement) {
    this.renderer = new THREE.WebGLRenderer();
    this.renderer.setPixelRatio(window.devicePixelRatio);
    element.appendChild(this.renderer.domElement);
    // A linear float render target, which is antialiased, and its conversion to sRGB.
    const target = new THREE.WebGLRenderTarget(1, 1, { type: THREE.HalfFloatType, samples: 4 });
    this.composer = new EffectComposer(this.renderer, target);
    this.controls = new CameraControls(this.renderer.domElement);
    this.renderPass = new RenderPass(this.scene, this.controls.camera);
    this.composer.addPass(this.renderPass);
    this.composer.addPass(new OutputPass());

    this.scene.background = new THREE.Color(theme.primary);
    this.scene.add(this.model);
    // The models are lit by their material like in the Bevy viewer, the scene has no lights.
    this.grid.mesh.visible = false;
    this.scene.add(this.grid.mesh);

    THREE.Object3D.DEFAULT_UP.set(0, 0, 1);
    // The right button orbits, the view has no context menu.
    element.addEventListener("contextmenu", (event) => event.preventDefault());

    const resetButton = document.createElement("button");
    resetButton.className = "view-button";
    resetButton.textContent = "Reset camera";
    resetButton.title = "Look at the whole model";
    resetButton.addEventListener("click", () => this.resetCamera());
    element.appendChild(resetButton);

    new ResizeObserver(() => this.resize()).observe(element);
    this.renderer.setAnimationLoop(() => this.render());
  }

  /** Render the scene with the camera of the controls and the grid for it. */
  private render() {
    this.controls.update();
    const camera = this.controls.camera;
    this.renderPass.camera = camera;
    updateModelCamera(camera);
    // The widths of the outlines are pixels of the render target.
    const size = this.renderer.getDrawingBufferSize(new THREE.Vector2());
    updateEdgeCamera(camera, size.y);
    this.model.traverse((object) => {
      if (object instanceof LineSegments2) object.material.resolution.copy(size);
    });
    const gridConfig = this.controls.gridConfig();
    if (gridConfig) this.grid.update(camera, gridConfig);
    this.composer.render();
  }

  /** Show the geometry of a model, or remove the model without geometry. */
  show(geometry: Geometry) {
    switch (geometry.kind) {
      case "3d":
        return this.show3d(geometry.meshes);
      case "2d":
        return this.show2d(geometry.sketches);
      case "none":
        return this.clear();
    }
  }

  /** Show 3D triangle meshes in their colors with contours and silhouettes. */
  show3d(meshes: MeshData[]) {
    this.replace(
      meshes.flatMap((mesh) => {
        const { color, opacity = 1 } = colorParameters(mesh.color, theme.brighter);
        // The lighting of the Bevy viewer.
        const material = new ModelMaterial(new THREE.Color(color), opacity);
        const surface = new THREE.Mesh(
          // Sharp at the contour edges of the mesh, with the crease angle of the viewers.
          modelGeometry(toCreasedNormals(geometry(mesh), THREE.MathUtils.degToRad(creaseAngle()))),
          material,
        );
        // The contour edges and the smooth edges on the silhouette like in the Bevy viewer.
        const edges = new THREE.LineSegments(
          edgeGeometry(mesh.edges),
          new EdgeMaterial(new THREE.Color(theme.primary)),
        );
        return [surface, edges];
      }),
    );
    this.update("3d");
  }

  /**
   * Show 2D sketches as filled triangles with outlines.
   *
   * Sketches without a color are filled in the brighter color. Their outlines are outside of them
   * in the darker color with half opacity, like `bevy_mod_outline` in the Bevy viewer: the lines
   * are drawn before the fill, which covers their inner half.
   */
  show2d(sketches: SketchData[]) {
    this.replace(
      sketches.flatMap((sketch) => {
        const fill = new THREE.Mesh(
          geometry(sketch.triangles),
          new THREE.MeshBasicMaterial({
            ...colorParameters(sketch.color, theme.brighter),
            side: THREE.DoubleSide,
            // Drawn after the outline in the transparent objects, also when it is opaque.
            transparent: true,
          }),
        );
        fill.renderOrder = 1;
        const positions = sketch.lines;
        const points = new Float32Array((positions.length / 2) * 3);
        positions.forEach(
          (value, index) => (points[Math.floor(index / 2) * 3 + (index % 2)] = value),
        );
        // The lines are twice as wide, the fill covers their inner half.
        const outline = new LineSegments2(
          new LineSegmentsGeometry().setPositions(points),
          new LineMaterial({
            color: theme.darker,
            transparent: true,
            opacity: 0.5,
            depthWrite: false,
            linewidth: 2 * OUTLINE_WIDTH * this.renderer.getPixelRatio(),
          }),
        );
        return [fill, outline];
      }),
    );
    this.update("2d");
  }

  /** Remove the model. */
  clear() {
    this.replace([]);
  }

  private replace(objects: THREE.Object3D[]) {
    this.model.children.forEach((child) => disposeObject(child));
    this.model.clear();
    objects.forEach((object) => this.model.add(object));
  }

  /**
   * Show the grid for a new model of `kind`, the camera stays, unless it is the first model or
   * the kind changes, e.g. from a 3D to a 2D model.
   */
  private update(kind: ModelKind) {
    const box = this.bounds();
    if (!box) return;
    if (this.kind !== kind) {
      this.kind = kind;
      this.resetCamera();
    } else {
      // A bigger model is not cut off by the near plane of the camera, which is outside of the
      // scene.
      const { center, radius } = box.getBoundingSphere(new THREE.Sphere());
      if (radius > this.boundsRadius) {
        this.boundsRadius = radius;
        this.controls.setBounds(center, radius);
      }
    }
    this.grid.mesh.visible = true;
  }

  /** Move the camera to see the whole model in the standard view of its kind. */
  resetCamera() {
    const box = this.bounds();
    if (!box || !this.kind) return;
    const { center, radius } = box.getBoundingSphere(new THREE.Sphere());
    this.boundsRadius = radius || 1;
    this.controls.setBounds(center, this.boundsRadius);
    this.controls.setView(VIEWS[this.kind]);
    this.controls.showAll();
    this.controls.resetPivot();
  }

  /** The bounds of the model, `undefined` without model. */
  private bounds(): THREE.Box3 | undefined {
    const box = new THREE.Box3().setFromObject(this.model);
    return box.isEmpty() ? undefined : box;
  }

  private resize() {
    const { clientWidth: width, clientHeight: height } = this.element;
    this.renderer.setSize(width, height);
    this.composer.setSize(width, height);
    this.controls.resize(width, height);
    // Resizing clears the canvas, render at once to avoid flickering while a panel is resized.
    this.render();
  }
}

/**
 * Material parameters of a µcad color (`r, g, b, a` from 0 to 1), e.g. of `#[color = "red"]`,
 * or of the `fallback` color, if there is none.
 */
function colorParameters(color: Color | undefined, fallback: string) {
  if (!color) return { color: fallback };
  const [r, g, b, a] = color;
  return {
    color: new THREE.Color().setRGB(r, g, b, THREE.SRGBColorSpace),
    transparent: a < 1,
    opacity: a,
  };
}

/** A three.js geometry of µcad triangles. */
function geometry(mesh: Triangles): THREE.BufferGeometry {
  const geometry = new THREE.BufferGeometry();
  geometry.setAttribute("position", new THREE.BufferAttribute(mesh.positions, 3));
  geometry.setIndex(new THREE.BufferAttribute(mesh.indices, 1));
  geometry.computeVertexNormals();
  return geometry;
}

function disposeObject(object: THREE.Object3D) {
  if (object instanceof THREE.Mesh || object instanceof THREE.LineSegments) {
    object.geometry.dispose();
    (object.material as THREE.Material).dispose();
  }
}
