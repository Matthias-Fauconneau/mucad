// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

import { defineConfig, type Plugin } from "vite";

/** The source of the standalone bundle of `<mucad-viewer>`. */
const VIEWER = "/src/elements/mucad-viewer.ts";

/**
 * The demo page `public/demo.html` loads the bundle `mucad-viewer.js` like a page of the
 * documentation, the development server serves its source instead.
 */
function serveViewerSource(): Plugin {
  return {
    name: "mucad-viewer-source",
    configureServer(server) {
      // The request is a Node.js request, whose types are not installed.
      server.middlewares.use((request: { url?: string }, _response, next) => {
        if (request.url === "/mucad-viewer.js") request.url = VIEWER;
        next();
      });
    },
  };
}

/**
 * The standalone bundle of `<mucad-viewer>`, e.g. for the documentation: `dist/mucad-viewer.js`
 * with three.js and the compiler, whose WebAssembly binary in `dist/assets` is loaded relative
 * to the bundle.
 */
export default defineConfig({
  // Relative paths, so the bundle can be served from any directory.
  base: "./",
  plugins: [serveViewerSource()],
  server: {
    open: "/demo.html",
    // The npm workspace and the WebAssembly package, which is outside of it.
    fs: { allow: ["..", "../../crates/wasm/pkg"] },
  },
  preview: { open: "/demo.html" },
  build: {
    // The bundle is one module with three.js, which alone is about 500 kB.
    chunkSizeWarningLimit: 700,
    rolldownOptions: {
      input: { "mucad-viewer": VIEWER.slice(1) },
      // The bundle exports the element class, e.g. to set the compiler of a viewer.
      preserveEntrySignatures: "exports-only",
      output: { entryFileNames: "[name].js" },
    },
  },
  // The WebAssembly package loads its binary relative to its JavaScript module.
  optimizeDeps: { exclude: ["mucad-wasm"] },
});
