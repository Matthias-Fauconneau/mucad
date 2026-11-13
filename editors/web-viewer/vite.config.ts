// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

import { defineConfig } from "vite";

export default defineConfig({
  // Relative paths, so the build can be served from any directory.
  base: "./",
  server: {
    // The examples are loaded from the repository's `examples` folder.
    fs: { allow: ["../.."] },
  },
  resolve: {
    // Only the UMD build of dockview-core includes its stylesheet.
    alias: [{ find: /^dockview-core$/, replacement: "dockview-core/dist/dockview-core.js" }],
  },
  build: {
    // The libraries are in their own chunks, which stay cached, when the app changes. Monaco
    // alone is about 3 MB.
    chunkSizeWarningLimit: 3500,
    rolldownOptions: {
      output: {
        codeSplitting: {
          groups: [
            { name: "monaco", test: /node_modules[\\/]monaco-editor/ },
            { name: "three", test: /node_modules[\\/]three/ },
            { name: "dockview", test: /node_modules[\\/]dockview-core/ },
          ],
        },
      },
    },
  },
  // The WebAssembly package loads its binary relative to its JavaScript module.
  optimizeDeps: { exclude: ["mucad-wasm"] },
});
