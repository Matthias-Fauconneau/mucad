# µcad desktop app

Edit, display and export µcad models in a native window: the µcad editor of the web viewer,
made of the [web components](../web-components), in a [Tauri](https://tauri.app/) app.

- Native dialogs to save exported models and QR codes and to confirm deleting a document or
  closing the app with unsaved changes
- Links, e.g. to report a bug, are opened in the browser
- The settings and the own documents are stored in the web view

## Build

Requirements: [Rust](https://rustup.rs/) with the target `wasm32-unknown-unknown`,
[`wasm-pack`](https://rustwasm.github.io/wasm-pack/), [Node.js](https://nodejs.org/) and the
[system dependencies of Tauri](https://tauri.app/start/prerequisites/), on Debian and Ubuntu:

```sh
sudo apt install libwebkit2gtk-4.1-dev build-essential curl wget file libxdo-dev libssl-dev \
  libayatana-appindicator3-dev librsvg2-dev libjavascriptcoregtk-4.1-dev
```

```sh
cd editors
npm run wasm                     # Build the WebAssembly package into crates/wasm/pkg
npm install
npm run tauri -w mucad-desktop dev     # Start the app with the dev server
npm run tauri -w mucad-desktop build   # Build the app and its installers
```

The Rust crate `src-tauri` is a member of the Cargo workspace, but not a default member, so
`cargo build` in the repository does not need the system dependencies of Tauri.
After changes in the Rust code of the compiler, run `npm run wasm` again.
