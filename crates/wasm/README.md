# µcad for WebAssembly

The µcad toolchain compiled to WebAssembly: compile µcad code, render its model and export it,
e.g. in a browser. The standard library is embedded, there is no file system access.
It also contains the camera navigation of the µcad viewers, see [Camera navigation](#camera-navigation).

## Build

Install [`wasm-pack`](https://rustwasm.github.io/wasm-pack/) and build the package for the web:

```sh
wasm-pack build crates/wasm --target web --release --out-dir pkg
```

The package in `crates/wasm/pkg` contains the WebAssembly binary (about 5 MB),
the JavaScript bindings and TypeScript declarations.
Use `--target bundler` for bundlers like webpack or `--target nodejs` for Node.js.

## Usage

```js
import init, { Mucad } from "./pkg/mucad_wasm.js";

await init(); // Load the WebAssembly binary.

const mucad = new Mucad();
const result = mucad.compile(`Circle(radius = 10mm).extrude(5mm);`);

if (!result.success) {
  console.error(result.report);
} else if (result.modelType === "3d") {
  for (const mesh of mucad.renderMeshes()) {
    // mesh.positions: Float32Array, mesh.indices: Uint32Array, mesh.color: Float32Array | undefined
  }
} else if (result.modelType === "2d") {
  for (const sketch of mucad.renderSketches()) {
    // sketch.lines.positions: Float32Array, sketch.color: Float32Array | undefined
  }
}
```

`compile` keeps the model of the last successful compilation, the render and export methods use it.
They throw a string with the reason, e.g. render errors, if there is no model or it cannot be
rendered.

### Compile

| Method | Returns |
|---|---|
| `compile(code)` | `CompileResult` with `success`, `diagnostics` (see below), `report` (the diagnostics as text) and `modelType` (`"2d"`, `"3d"`, `"mixed"` or `"none"`) |
| `setResolution(mm)` | Sets the linear render resolution, e.g. `0.1` for smoother circles (default `0.25`) |

### Colors

A model is rendered into one mesh or sketch for each color, e.g. of `#[color = "red"]`.
`color` is `r, g, b, a` from `0.0` to `1.0` (sRGB), or `undefined` for the default color,
which is up to the viewer.

### 3D models

| Method | Returns |
|---|---|
| `renderMeshes()` | `Mesh[]`, each with `positions` (`x, y, z` triples), `indices` (triangles as index triples) and `color` |
| `exportStl()` | Binary STL as `Uint8Array` |

A mesh can be displayed with three.js:

```js
const geometry = new THREE.BufferGeometry();
geometry.setAttribute("position", new THREE.BufferAttribute(mesh.positions, 3));
geometry.setIndex(new THREE.BufferAttribute(mesh.indices, 1));
geometry.computeVertexNormals();

const material = new THREE.MeshStandardMaterial();
if (mesh.color) {
  const [r, g, b, a] = mesh.color;
  material.color.setRGB(r, g, b, THREE.SRGBColorSpace);
  material.transparent = a < 1;
  material.opacity = a;
}
scene.add(new THREE.Mesh(geometry, material));
```

### 2D models

| Method | Returns |
|---|---|
| `renderSketches()` | `Sketch[]`, each with `color` and its geometries in three forms, see below |
| `exportSvg()` | SVG as string, with the colors of the models |

| `Sketch` property | Content |
|---|---|
| `polygons` | `Polygons`: the polygons with holes, e.g. for a canvas or SVG |
| `lines` | `Lines` with `positions`: line segments as `x0, y0, x1, y1`, e.g. for a `THREE.LineSegments` |
| `triangles` | `Mesh` in the XY plane (`z = 0`), e.g. for a filled `THREE.Mesh` |

`Polygons` stores the points of all rings one after another in `coordinates` (`x, y` pairs).
`rings` and `polygons` contain start indices with an additional end index:
ring `i` consists of the points `rings[i]` to `rings[i + 1]`,
polygon `j` of the rings `polygons[j]` to `polygons[j + 1]`.
The first ring of a polygon is its outline, the others are its holes.

```js
const { coordinates, rings, polygons } = sketch.polygons;
for (let p = 0; p + 1 < polygons.length; p++) {
  for (let r = polygons[p]; r < polygons[p + 1]; r++) {
    const ring = coordinates.subarray(2 * rings[r], 2 * rings[r + 1]);
    // ring: x0, y0, x1, y1, ...
  }
}
```

### Export

An export can be downloaded with a `Blob`:

```js
const blob = new Blob([mucad.exportStl()], { type: "model/stl" });
const link = document.createElement("a");
link.href = URL.createObjectURL(blob);
link.download = "model.stl";
link.click();
```

### Camera navigation

`CameraNavigation` controls the camera of a viewer with the mouse, the keys and touch, e.g. for
Three.js: the camera controller and the configurable controls of
[mucad-viewer-utils](https://crates.io/crates/mucad-viewer-utils). It needs no `Mucad`.

The methods take the values of DOM events. Coordinates are pixels from the top left of the
element, e.g. `offsetX` and `offsetY`. Modifiers are bits: Ctrl 1, Shift 2, Alt 4 and Meta 8.

```js
import init, { CameraNavigation } from "./pkg/mucad_wasm.js";

await init(); // Load the WebAssembly binary.

const navigation = new CameraNavigation(element.clientWidth, element.clientHeight);
const modifiers = (e) => e.ctrlKey | (e.shiftKey << 1) | (e.altKey << 2) | (e.metaKey << 3);

// Mouse events, because pointer events are not sent for a second button, e.g. of a chord.
element.addEventListener("mousedown", (e) =>
  navigation.mouseDown(e.button, e.offsetX, e.offsetY, modifiers(e)));
element.addEventListener("mouseup", (e) => {
  const selection = navigation.mouseUp(e.button, e.offsetX, e.offsetY, modifiers(e));
  if (selection) {
    // Pick the object at selection.x and selection.y, add it with selection.multi.
  }
});

// After the events, apply the camera, e.g. to a camera of Three.js.
camera.matrixAutoUpdate = false;
camera.matrixWorld.fromArray(navigation.cameraMatrix());
camera.matrixWorldInverse.fromArray(navigation.viewMatrix());
camera.projectionMatrix.fromArray(navigation.projectionMatrix());
camera.projectionMatrixInverse.copy(camera.projectionMatrix).invert();
```

The projection is orthographic and maps to the normalized device coordinates of WebGL, from -1
to 1 in z, like the projections of Three.js with the `WebGLRenderer`.

| Method | Does |
|---|---|
| `new CameraNavigation(width, height, controlsToml?)` | The navigation with the controls in TOML or the defaults (see `Controls` of `mucad-viewer-utils`), throws an error for invalid controls |
| `setControls(toml)` | Sets the controls, missing entries are the defaults |

The events of the element: `elapsedMs` is the time since the last frame, which keeps the speed
of orbiting and zooming the same at any frame rate.

| Method | Event |
|---|---|
| `mouseDown(button, x, y, modifiers)` | `mousedown` with `MouseEvent.button` |
| `mouseUp(button, x, y, modifiers)` | `mouseup`, returns a `Selection` after a click |
| `mouseMove(dx, dy, elapsedMs)` | `mousemove` with the difference of `clientX` and `clientY` to the previous event, so that panning follows the pointer exactly |
| `wheel(deltaY, deltaMode, x, y, modifiers, elapsedMs)` | `wheel`, returns whether it is bound to prevent scrolling |
| `keyDown(key, modifiers)` | `keydown` with `KeyboardEvent.key`, returns whether it is bound to prevent e.g. zooming the page |
| `touchStart(id, x, y)` | `touchstart` with `Touch.identifier`, for each changed touch |
| `touchMove(touches, elapsedMs)` | `touchmove` with all changed touches together as `[id1, x1, y1, id2, x2, y2]` |
| `touchEnd(id, x, y)` | `touchend`, for each changed touch, returns a `Selection` after a tap |
| `touchCancel(id)` | `touchcancel`, for each changed touch |
| `resize(width, height)` | The element is resized |

One finger orbits (or pans, see `[touch]` of the controls), two fingers pinch to zoom and pan
with their center. The touches of one `touchmove` event are passed together, so that moving
two fingers in parallel pans without zooming.

`mouseAction()` is `select`, `orbit`, `pan`, `zoom` or undefined, e.g. for the cursor. While
`pointerPinned()`, the pointer is locked with `requestPointerLock`.

| Method | Returns or does |
|---|---|
| `cameraMatrix()`, `viewMatrix()`, `projectionMatrix()` | 16 numbers by columns, like `Matrix4.fromArray` |
| `position()`, `viewCenter()`, `up()`, `pivot()` | `[x, y, z]` |
| `setBounds(x, y, z, radius)` | Sets the center and the radius of what the camera shows, e.g. of the bounding sphere of the model |
| `setView(name)` | Shows `isometric`, `front`, `top`, `right`, `rear`, `bottom` or `left` |
| `projection()`, `setProjection(name)`, `toggleProjection()` | The projection `orthographic` or `perspective`, e.g. for a `THREE.OrthographicCamera` or `THREE.PerspectiveCamera`; switching keeps the visible area at the pivot |
| `showAll()` | Shows the whole scene in the middle of the view, e.g. to reset the camera |
| `zoomFit()`, `zoomLevel()`, `resetPivot()` | Zooms to level 1 around the center of the view, the zoom level, sets the pivot to the center of the bounds |
| `takeEvents()` | The changes since the last call: `zoomLevelChanged` and `pivotChanged` |

### Help

`mucad.actions` are the actions of the viewer as an `ActionRegistry`, e.g. for its help and
tooltips, with the bindings of the current controls of a `CameraNavigation`. A
`new ActionRegistry()` without a `Mucad` has the same actions.

```js
help.innerHTML = markdownToHtml(mucad.actions.helpMarkdown(navigation, true));
fitButton.title = mucad.actions.tooltip(navigation, "zoom_fit"); // "Zoom to fit (f)"
```

| Method | Does |
|---|---|
| `ids()` | The ids of the actions in the order of the help, e.g. `zoom_fit` |
| `tooltip(navigation, id)` | The label with the bindings, e.g. `Zoom to fit (f)`, throws an error for an unknown id |
| `helpMarkdown(navigation, basic)` | A Markdown table of the actions with their bindings and descriptions, with `basic` only of the basic ones |

The ids are the ones of the controls, e.g. `zoom_in`, and `select`, `orbit`, `pan` and `zoom`
for the mouse.

`Selection`, `CameraNavigation` and `ActionRegistry` are freed with `free()`.

## Scope

Only compiling, rendering, exporting and the camera navigation are available yet.
The model tree and the symbol library may be exposed in the future, e.g. for a model browser.

## Tests

The interface is tested natively:

```sh
cargo test -p mucad-wasm
```
