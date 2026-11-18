// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

import * as THREE from "three";
import { CameraNavigation } from "mucad-wasm";
import { loadWasm } from "./compiler/wasm";

/** A standard view of the scene with Z upwards. */
export type StandardView = "isometric" | "front" | "top" | "right" | "rear" | "bottom" | "left";

/** The detail of a `select` event: a click at `x` and `y` in pixels from the top left. */
export interface SelectDetail {
  x: number;
  y: number;
  /** Whether the object is added to the selection, instead of replacing it. */
  multi: boolean;
}

/** The near and far plane of the projection of the camera controller. */
const NEAR = 0.1;
const FAR = 100000;

/** The time in milliseconds after a drag with the right button, in which no context menu opens. */
const CONTEXT_MENU_DELAY = 500;

/** The cursors of the actions of the mouse. */
const CURSORS: Record<string, string> = { pan: "move", zoom: "ns-resize", orbit: "grabbing" };

/** The modifiers of an event as bits: Ctrl 1, Shift 2, Alt 4 and Meta 8. */
function modifiers(event: MouseEvent | KeyboardEvent): number {
  return (
    Number(event.ctrlKey) |
    (Number(event.shiftKey) << 1) |
    (Number(event.altKey) << 2) |
    (Number(event.metaKey) << 3)
  );
}

/**
 * The camera navigation of the µcad viewers for three.js cameras: orbiting, panning and zooming
 * an orthographic or a perspective view with the mouse, the keys and touch, configurable in
 * TOML. The projection is toggled with the key `p`, {@link CameraControls.camera} is the camera
 * of the current one.
 *
 * The controls listen to the events of `element`, e.g. the canvas of the renderer, which gets
 * the focus for the keys by a click. `update()` applies the camera in each frame, a click or a
 * tap dispatches a `select` event with a {@link SelectDetail}.
 *
 * The context menu of the element is disabled, so that the right button orbits, also when it
 * is released outside of the element.
 *
 * Touch: one finger orbits, two fingers pinch to zoom and pan, the page does not scroll or
 * zoom with them.
 *
 * The WebAssembly module is loaded at the first call or event, which are applied after it.
 *
 * ```ts
 * const controls = new CameraControls(renderer.domElement);
 * controls.setBounds(center, radius);
 * controls.setView("isometric");
 * controls.addEventListener("select", (event) => pick((event as CustomEvent<SelectDetail>).detail));
 * renderer.setAnimationLoop(() => {
 *   controls.update();
 *   renderer.render(scene, controls.camera);
 * });
 * ```
 */
export class CameraControls extends EventTarget {
  private navigation?: CameraNavigation;
  /** The calls before the WebAssembly module is loaded. */
  private pending: ((navigation: CameraNavigation) => void)[] = [];
  private loading = false;
  private disposed = false;
  /** The buttons of `MouseEvent.button`, which are pressed on the element. */
  private pressed = new Set<number>();
  /** Whether the context menu after a drag with the right button is prevented. */
  private noContextMenu = false;
  private contextMenuTimer?: number;
  /** The start of the frame, the time since it scales the movements, like a frame timer. */
  private frameStart = performance.now();
  /** The last pointer position in client coordinates, whose movement drags. */
  private lastPointer = { x: 0, y: 0 };
  private width = 1;
  private height = 1;

  /** The camera of the orthographic projection, whose matrices the controls set. */
  readonly orthographicCamera = new THREE.OrthographicCamera();
  /** The camera of the perspective projection, whose matrices the controls set. */
  readonly perspectiveCamera = new THREE.PerspectiveCamera();

  /**
   * Controls with the events of `element` and the controls in TOML or the defaults, see
   * `CameraNavigation.defaultControls()`.
   */
  constructor(
    private element: HTMLElement,
    private controlsToml?: string,
  ) {
    super();
    this.orthographicCamera.near = NEAR;
    this.orthographicCamera.far = FAR;
    // The canvas gets the focus for the keys, without an outline.
    element.tabIndex = 0;
    element.style.outline = "none";
    element.addEventListener("mousedown", this.onMouseDown);
    element.addEventListener("wheel", this.onWheel, { passive: false });
    element.addEventListener("keydown", this.onKeyDown);
    // The browser neither scrolls nor zooms the page with the fingers on the element.
    element.style.touchAction = "none";
    element.addEventListener("touchstart", this.onTouchStart, { passive: false });
    element.addEventListener("touchmove", this.onTouchMove, { passive: false });
    element.addEventListener("touchend", this.onTouchEnd, { passive: false });
    element.addEventListener("touchcancel", this.onTouchCancel, { passive: false });
    // The right button orbits: no context menu on the element, and none after a drag, which
    // opens on some systems where the button is released, e.g. outside of the element.
    element.addEventListener("contextmenu", this.preventDefault);
    window.addEventListener("contextmenu", this.onContextMenu, { capture: true });
    window.addEventListener("blur", this.releaseAll);
  }

  /**
   * Set the bounds of what the camera shows: their center and their radius, e.g. of the bounding
   * sphere of the model.
   */
  setBounds(center: THREE.Vector3, radius: number) {
    this.run((navigation) => navigation.setBounds(center.x, center.y, center.z, radius));
  }

  /** Show a standard view. */
  setView(view: StandardView) {
    this.run((navigation) => navigation.setView(view));
  }

  /** Show the whole scene in the middle of the view. */
  showAll() {
    this.run((navigation) => navigation.showAll());
  }

  /** Set the pivot, which the camera orbits around, to the center of the bounds. */
  resetPivot() {
    this.run((navigation) => navigation.resetPivot());
  }

  /**
   * Set the controls in TOML, missing entries are the defaults.
   *
   * Throws an error with the reason for invalid controls, once the WebAssembly module is
   * loaded.
   */
  setControls(toml: string) {
    this.controlsToml = toml;
    this.navigation?.setControls(toml);
  }

  /**
   * The parameters of the grid shader for the camera, the floats of the members of `GridConfig`,
   * or `undefined`, until the WebAssembly module is loaded.
   */
  gridConfig(): Float32Array | undefined {
    return this.navigation?.gridConfig();
  }

  /** Set the size of the element in pixels, e.g. after resizing it. */
  resize(width: number, height: number) {
    this.width = width;
    this.height = height;
    this.navigation?.resize(width, height);
  }

  /**
   * The camera of the projection, orthographic or perspective, e.g. to render the scene and to
   * pick objects. It changes, when the projection is changed, e.g. with the key `p`.
   */
  get camera(): THREE.OrthographicCamera | THREE.PerspectiveCamera {
    return this.navigation?.projection() === "perspective"
      ? this.perspectiveCamera
      : this.orthographicCamera;
  }

  /** Change between the orthographic and the perspective projection, like the key `p`. */
  toggleProjection() {
    this.run((navigation) => navigation.toggleProjection());
  }

  /** Apply the camera of the projection, e.g. in each frame before rendering. */
  update() {
    this.frameStart = performance.now();
    const navigation = this.navigation;
    if (!navigation) return;
    const camera = this.camera;
    camera.matrix.fromArray(navigation.cameraMatrix());
    camera.matrix.decompose(camera.position, camera.quaternion, camera.scale);
    camera.updateMatrixWorld(true);
    camera.projectionMatrix.fromArray(navigation.projectionMatrix());
    camera.projectionMatrixInverse.copy(camera.projectionMatrix).invert();
  }

  /** Remove the event listeners and free the WebAssembly memory. */
  dispose() {
    this.disposed = true;
    this.releaseAll();
    this.element.removeEventListener("mousedown", this.onMouseDown);
    this.element.removeEventListener("wheel", this.onWheel);
    this.element.removeEventListener("keydown", this.onKeyDown);
    this.element.removeEventListener("touchstart", this.onTouchStart);
    this.element.removeEventListener("touchmove", this.onTouchMove);
    this.element.removeEventListener("touchend", this.onTouchEnd);
    this.element.removeEventListener("touchcancel", this.onTouchCancel);
    this.element.removeEventListener("contextmenu", this.preventDefault);
    window.removeEventListener("contextmenu", this.onContextMenu, { capture: true });
    window.removeEventListener("blur", this.releaseAll);
    clearTimeout(this.contextMenuTimer);
    this.navigation?.free();
    this.navigation = undefined;
  }

  /** Call `action` with the navigation, after loading the WebAssembly module. */
  private run(action: (navigation: CameraNavigation) => void) {
    if (this.navigation) {
      action(this.navigation);
      return;
    }
    this.pending.push(action);
    this.load();
  }

  /** Load the WebAssembly module once and apply the pending calls. */
  private load() {
    if (this.loading) return;
    this.loading = true;
    loadWasm()
      .then(() => {
        if (this.disposed) return;
        this.navigation = this.createNavigation();
        for (const action of this.pending) action(this.navigation);
        this.pending = [];
        this.update();
      })
      .catch((error) => console.error("Cannot load the camera controls:", error));
  }

  /** The navigation with the controls, or with the default ones, if they are invalid. */
  private createNavigation(): CameraNavigation {
    try {
      return new CameraNavigation(this.width, this.height, this.controlsToml);
    } catch (error) {
      console.error(error);
      return new CameraNavigation(this.width, this.height);
    }
  }

  /** The position of `event` in pixels from the top left of the element. */
  private position(event: MouseEvent | Touch): [number, number] {
    const rect = this.element.getBoundingClientRect();
    return [event.clientX - rect.left, event.clientY - rect.top];
  }

  /** The time since the start of the frame in milliseconds. */
  private elapsed(): number {
    return performance.now() - this.frameStart;
  }

  private updateCursor() {
    const action = this.navigation?.mouseAction();
    this.element.style.cursor = (action && CURSORS[action]) ?? "";
  }

  private onMouseDown = (event: MouseEvent) => {
    const navigation = this.navigation;
    if (!navigation) {
      this.load();
      return;
    }
    // No autoscroll with the middle button and no text selection, but the focus for the keys.
    event.preventDefault();
    this.element.focus({ preventScroll: true });
    if (this.pressed.size === 0) {
      // Movements outside of the element, e.g. while orbiting, are followed.
      window.addEventListener("mousemove", this.onMouseMove);
      window.addEventListener("mouseup", this.onMouseUp);
    }
    this.pressed.add(event.button);
    // Like the timer at the start of an action.
    this.frameStart = performance.now();
    this.lastPointer = { x: event.clientX, y: event.clientY };
    navigation.mouseDown(event.button, ...this.position(event), modifiers(event));
    this.updateCursor();
  };

  /**
   * Move by the movement of the pointer, so that panning follows it exactly, instead of
   * `movementX`, which some browsers scale by the device pixel ratio.
   */
  private onMouseMove = (event: MouseEvent) => {
    const dx = event.clientX - this.lastPointer.x;
    const dy = event.clientY - this.lastPointer.y;
    this.lastPointer = { x: event.clientX, y: event.clientY };
    this.navigation?.mouseMove(dx, dy, this.elapsed());
  };

  /** Release the button and dispatch a `select` event after a click. */
  private onMouseUp = (event: MouseEvent) => {
    if (!this.pressed.delete(event.button)) return;
    if (event.button === 2) this.preventContextMenu();
    const detail = this.release(event.button, ...this.position(event), modifiers(event));
    if (detail) this.dispatchEvent(new CustomEvent<SelectDetail>("select", { detail }));
  };

  /** Release `button`, returns the selection after a click. */
  private release(
    button: number,
    x: number,
    y: number,
    modifierBits: number,
  ): SelectDetail | undefined {
    if (this.pressed.size === 0) {
      window.removeEventListener("mousemove", this.onMouseMove);
      window.removeEventListener("mouseup", this.onMouseUp);
    }
    const selection = this.navigation?.mouseUp(button, x, y, modifierBits);
    this.updateCursor();
    if (!selection) return undefined;
    const detail = { x: selection.x, y: selection.y, multi: selection.multi };
    selection.free();
    return detail;
  }

  /**
   * Release all buttons without selecting, e.g. when the window loses the focus while
   * dragging.
   */
  private releaseAll = () => {
    for (const button of [...this.pressed]) {
      this.pressed.delete(button);
      this.release(button, 0, 0, 0);
    }
  };

  /** Prevent the context menu, which opens shortly after releasing the right button. */
  private preventContextMenu() {
    this.noContextMenu = true;
    clearTimeout(this.contextMenuTimer);
    this.contextMenuTimer = window.setTimeout(
      () => (this.noContextMenu = false),
      CONTEXT_MENU_DELAY,
    );
  }

  private onContextMenu = (event: MouseEvent) => {
    if (this.pressed.size === 0 && !this.noContextMenu) return;
    event.preventDefault();
    this.noContextMenu = false;
  };

  private onTouchStart = (event: TouchEvent) => {
    // No mouse events are emulated for the touches, e.g. no second selection after a tap.
    event.preventDefault();
    const navigation = this.navigation;
    if (!navigation) {
      this.load();
      return;
    }
    // Like the timer at the start of an action.
    this.frameStart = performance.now();
    for (const touch of event.changedTouches) {
      navigation.touchStart(touch.identifier, ...this.position(touch));
    }
  };

  private onTouchMove = (event: TouchEvent) => {
    event.preventDefault();
    const touches = [...event.changedTouches].flatMap((touch) => [
      touch.identifier,
      ...this.position(touch),
    ]);
    this.navigation?.touchMove(new Float64Array(touches), this.elapsed());
  };

  private onTouchEnd = (event: TouchEvent) => {
    event.preventDefault();
    for (const touch of event.changedTouches) {
      const selection = this.navigation?.touchEnd(touch.identifier, ...this.position(touch));
      if (!selection) continue;
      const detail = { x: selection.x, y: selection.y, multi: selection.multi };
      selection.free();
      this.dispatchEvent(new CustomEvent<SelectDetail>("select", { detail }));
    }
  };

  private onTouchCancel = (event: TouchEvent) => {
    for (const touch of event.changedTouches) this.navigation?.touchCancel(touch.identifier);
  };

  private onWheel = (event: WheelEvent) => {
    const navigation = this.navigation;
    if (!navigation) return;
    const [x, y] = this.position(event);
    const bound = navigation.wheel(
      event.deltaY,
      event.deltaMode,
      x,
      y,
      modifiers(event),
      this.elapsed(),
    );
    if (bound) event.preventDefault();
  };

  private onKeyDown = (event: KeyboardEvent) => {
    if (this.navigation?.keyDown(event.key, modifiers(event))) event.preventDefault();
  };

  private preventDefault = (event: Event) => event.preventDefault();
}
