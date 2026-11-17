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

`CameraNavigation` controls the camera of a viewer with the mouse and the keys, e.g. for
Three.js: the camera controller and the configurable controls of
[mucad-viewer-utils](https://crates.io/crates/mucad-viewer-utils). It needs no `Mucad`.

The methods take the values of DOM events. Coordinates are pixels from the top left of the
element, e.g. `offsetX` and `offsetY`. Modifiers are bits: Ctrl 1, Shift 2, Alt 4 and Meta 8.

```js
import init, { CameraNavigation } from "./pkg/mucad_wasm.js";

await init(); // Load the WebAssembly binary.

const navigation = new CameraNavigation(element.clientWidth, element.clientHeight);
const modifiers = (e) => e.ctrlKey | (e.shiftKey << 1) | (e.altKey << 2) | (e.metaKey << 3);

element.addEventListener("pointerdown", (e) =>
  navigation.mouseDown(e.button, e.offsetX, e.offsetY, modifiers(e)));
element.addEventListener("pointerup", (e) => {
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
| `new CameraNavigation(width, height, controlsToml?)` | The navigation with the controls in TOML or the defaults, throws an error for invalid controls |
| `CameraNavigation.defaultControls()` | The default controls as TOML with comments |
| `setControls(toml)` | Sets the controls, missing entries are the defaults |

The events of the element: `elapsedMs` is the time since the last frame, which keeps the speed
of orbiting and zooming the same at any frame rate.

| Method | Event |
|---|---|
| `mouseDown(button, x, y, modifiers)` | `pointerdown` with `MouseEvent.button` |
| `mouseUp(button, x, y, modifiers)` | `pointerup`, returns a `Selection` after a click |
| `mouseMove(dx, dy, elapsedMs)` | `pointermove` with `movementX` and `movementY` |
| `wheel(deltaY, deltaMode, x, y, modifiers, elapsedMs)` | `wheel`, returns whether it is bound to prevent scrolling |
| `keyDown(key, modifiers)` | `keydown` with `KeyboardEvent.key`, returns whether it is bound to prevent e.g. zooming the page |
| `resize(width, height)` | The element is resized |

`mouseAction()` is `select`, `orbit`, `pan`, `zoom` or undefined, e.g. for the cursor. While
`pointerPinned()`, the pointer is locked with `requestPointerLock`.

| Method | Returns or does |
|---|---|
| `cameraMatrix()`, `viewMatrix()`, `projectionMatrix()` | 16 numbers by columns, like `Matrix4.fromArray` |
| `position()`, `viewCenter()`, `up()`, `pivot()` | `[x, y, z]` |
| `setScene(x, y, z, radius)` | Sets the center and the radius of the scene, e.g. of the bounding sphere of the model |
| `setView(name)` | Shows `isometric`, `front`, `top`, `right`, `rear`, `bottom` or `left` |
| `showAll()` | Shows the whole scene in the middle of the view, e.g. to reset the camera |
| `zoomFit()`, `zoomLevel()`, `resetPivot()` | Zooms to level 1 around the center of the view, the zoom level, sets the pivot to the scene center |
| `takeEvents()` | The changes since the last call: `zoomLevelChanged` and `pivotChanged` |

`Selection` and `CameraNavigation` are freed with `free()`.

## Scope

Only compiling, rendering, exporting and the camera navigation are available yet.
The model tree and the symbol library may be exposed in the future, e.g. for a model browser.

## Tests

The interface is tested natively:

```sh
cargo test -p mucad-wasm
```
