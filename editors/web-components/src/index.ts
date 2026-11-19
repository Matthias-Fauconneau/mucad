// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

/**
 * µcad web components, which the web viewer and the desktop app are made of: the app, the
 * custom elements `<mucad-navbar>`, `<mucad-editor>`, `<mucad-viewer>` with its
 * `<mucad-viewer-overlay>` and `<mucad-info-widget>`, the dock, the camera controls, the
 * compiler, the services of documents and shared links, the examples and the platform.
 */

import "./generated/theme.css";
import "./style.css";

export * from "./app/app";
export * from "./camera-controls";
export * from "./compiler/types";
export { wasmCompiler } from "./compiler/wasm";
export * from "./dock";
export * from "./editor";
export * from "./examples";
export { MucadEditor } from "./elements/mucad-editor";
export { MucadNavbar } from "./elements/mucad-navbar";
export { MucadInfoWidget, markdownTable } from "./elements/mucad-info-widget";
export { MucadViewer } from "./elements/mucad-viewer";
export { CORNERS, type Corner, MucadViewerOverlay } from "./elements/mucad-viewer-overlay";
export * from "./platform";
export * from "./services/documents";
export * as share from "./services/share";
export * from "./generated/theme";
export * from "./grid";
export * from "./model-material";
export * from "./viewer";
