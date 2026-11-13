// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

import { defineConfig } from "vite";

/** The frontend of the desktop app, which Tauri loads from `dist` or from the dev server. */
export default defineConfig({
  // The output of Tauri stays visible.
  clearScreen: false,
  server: {
    // The port of `devUrl` in `src-tauri/tauri.conf.json`.
    port: 1420,
    strictPort: true,
    // The examples are loaded from the repository's `examples` folder.
    fs: { allow: ["../.."] },
    // The Rust code is rebuilt by Tauri.
    watch: { ignored: ["**/src-tauri/**"] },
  },
  resolve: {
    // Only the UMD build of dockview-core includes its stylesheet.
    alias: [{ find: /^dockview-core$/, replacement: "dockview-core/dist/dockview-core.js" }],
  },
  build: {
    // The libraries are in their own chunks like in the web viewer. Monaco alone is about 3 MB.
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
