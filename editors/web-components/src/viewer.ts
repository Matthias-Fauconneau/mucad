// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

import * as THREE from "three";
import { EffectComposer } from "three/examples/jsm/postprocessing/EffectComposer.js";
import { OutputPass } from "three/examples/jsm/postprocessing/OutputPass.js";
import { RenderPass } from "three/examples/jsm/postprocessing/RenderPass.js";
import { toCreasedNormals } from "three/examples/jsm/utils/BufferGeometryUtils.js";
import { CameraControls, type StandardView } from "./camera-controls";
import type { Color, Geometry, MeshData, SketchData, Triangles } from "./compiler/types";
import { theme } from "./generated/theme";
import { Grid } from "./grid";

/** The angle between faces, above which an edge is shown sharp instead of smooth. */
const CREASE_ANGLE = THREE.MathUtils.degToRad(20);

/** The views of 3D models (from the front, the right and the top) and 2D models (from above). */
const VIEWS = {
  "3d": "isometric",
  "2d": "top",
} as const satisfies Record<string, StandardView>;

type ModelKind = keyof typeof VIEWS;

/**
 * A three.js view of µcad models, the Z axis points up like in µcad.
 *
 * The orthographic camera is navigated with the mouse and the keys by {@link CameraControls}.
 *
 * The scene is rendered into a linear render target and converted to sRGB afterwards, so that
 * transparent objects are blended in linear colors like in the Bevy viewer, e.g. the grid.
 */
export class Viewer {
  private renderer: THREE.WebGLRenderer;
  private composer: EffectComposer;
  private scene = new THREE.Scene();
  private camera = new THREE.OrthographicCamera();
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
    this.composer.addPass(new RenderPass(this.scene, this.camera));
    this.composer.addPass(new OutputPass());

    this.scene.background = new THREE.Color(theme.primary);
    this.scene.add(this.model);
    this.scene.add(new THREE.HemisphereLight(theme.brighter, theme.darker, 1.5));
    const light = new THREE.DirectionalLight(theme.brighter, 2);
    light.position.set(1, -2, 3);
    this.camera.add(light);
    this.scene.add(this.camera);
    this.grid.mesh.visible = false;
    this.scene.add(this.grid.mesh);

    THREE.Object3D.DEFAULT_UP.set(0, 0, 1);
    this.controls = new CameraControls(this.camera, this.renderer.domElement);
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
    const gridConfig = this.controls.gridConfig();
    if (gridConfig) this.grid.update(this.camera, gridConfig);
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

  /** Show 3D triangle meshes in their colors with contours at their sharp edges. */
  show3d(meshes: MeshData[]) {
    this.replace(
      meshes.flatMap((mesh) => {
        const material = new THREE.MeshStandardMaterial({
          ...colorParameters(mesh.color, theme.bright),
          metalness: 0.1,
          roughness: 0.6,
          // Move the faces back a little, so that the contours on their edges stay visible.
          polygonOffset: true,
          polygonOffsetFactor: 1,
          polygonOffsetUnits: 1,
        });
        const surface = new THREE.Mesh(toCreasedNormals(geometry(mesh), CREASE_ANGLE), material);
        const contours = new THREE.LineSegments(
          new THREE.BufferGeometry()
            .setAttribute("position", new THREE.BufferAttribute(mesh.positions, 3))
            .setIndex(new THREE.BufferAttribute(mesh.edges, 1)),
          new THREE.LineBasicMaterial({ color: theme.primary }),
        );
        return [surface, contours];
      }),
    );
    this.update("3d");
  }

  /**
   * Show 2D sketches as filled triangles with outlines.
   *
   * Sketches without a color are filled in the darker color with outlines in the signal color.
   */
  show2d(sketches: SketchData[]) {
    this.replace(
      sketches.flatMap((sketch) => {
        const fill = new THREE.Mesh(
          geometry(sketch.triangles),
          new THREE.MeshBasicMaterial({
            ...colorParameters(sketch.color, theme.darker),
            side: THREE.DoubleSide,
          }),
        );
        const positions = sketch.lines;
        const points = new Float32Array((positions.length / 2) * 3);
        positions.forEach(
          (value, index) => (points[Math.floor(index / 2) * 3 + (index % 2)] = value),
        );
        const outline = new THREE.LineSegments(
          new THREE.BufferGeometry().setAttribute("position", new THREE.BufferAttribute(points, 3)),
          new THREE.LineBasicMaterial(colorParameters(sketch.color, theme.signal)),
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
