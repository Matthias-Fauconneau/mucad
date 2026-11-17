// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

import type { Compiler, CompilerSession, Geometry } from "../compiler/types";
import { errorMessage } from "../compiler/types";
import { wasmCompiler } from "../compiler/wasm";
import { decodeLink } from "../services/share";
import { Viewer } from "../viewer";

/**
 * The styles in the shadow root, with the colors of the µcad theme, also on pages without them,
 * e.g. in the documentation.
 */
const STYLE = `
:host {
  display: block;
  position: relative;
  min-height: 12rem;
  overflow: hidden;
  background: var(--primary, #020d1a);
}

:host([hidden]) {
  display: none;
}

.view {
  position: absolute;
  inset: 0;
}

/* A button in the view, e.g. to reset the camera. */
.view-button {
  position: absolute;
  top: 0.5rem;
  right: 0.5rem;
  height: 2.2rem;
  padding: 0 0.6rem;
  border: 1px solid var(--signal, #ff9f0a);
  border-radius: 4px;
  background: var(--primary, #020d1a);
  color: var(--signal, #ff9f0a);
  font: inherit;
  cursor: pointer;
}

/* The errors of the code of a standalone viewer. */
.message {
  position: absolute;
  left: 0.5rem;
  right: 0.5rem;
  bottom: 0.5rem;
  margin: 0;
  color: var(--complementary, #c5221f);
  font: 0.8rem monospace;
  white-space: pre-wrap;
}

.message:empty {
  display: none;
}
`;

/**
 * `<mucad-viewer>`: a three.js view of µcad models in a shadow root.
 *
 * The geometry is shown by an app with `show()`, e.g. of the code of an editor, or the viewer
 * is standalone with its code, e.g. in the documentation: in its text content, in the file of
 * its `src` attribute or in the shared link of its `link` attribute. A standalone viewer
 * loads the compiler, when it becomes visible, all viewers of a page share one compiler.
 *
 * ```html
 * <mucad-viewer style="height: 20rem">std::geo2d::Circle(r = 10mm);</mucad-viewer>
 * <mucad-viewer src="examples/brick.mu"></mucad-viewer>
 * <mucad-viewer link="wrUBQUQABENpcmNsZShyID0gMTBtbSk7CgM"></mucad-viewer>
 * ```
 */
export class MucadViewer extends HTMLElement {
  static observedAttributes = ["src", "link"];

  /** The compiler of a standalone viewer, by default the one in WebAssembly. */
  compiler?: Compiler;

  private viewer?: Viewer;
  private message?: HTMLPreElement;
  private session?: CompilerSession;
  private observer?: IntersectionObserver;
  /** Whether the code of a standalone viewer has been shown, e.g. after it became visible. */
  private loaded = false;

  connectedCallback() {
    // The element is created once, also when it is moved, e.g. into a panel of a dock.
    if (!this.viewer) {
      const shadow = this.attachShadow({ mode: "open" });
      const style = document.createElement("style");
      style.textContent = STYLE;
      const view = document.createElement("div");
      view.className = "view";
      this.message = document.createElement("pre");
      this.message.className = "message";
      shadow.append(style, view, this.message);
      this.viewer = new Viewer(view);
    }
    if (this.standalone && !this.loaded) this.loadWhenVisible();
  }

  disconnectedCallback() {
    this.observer?.disconnect();
    this.observer = undefined;
  }

  attributeChangedCallback() {
    if (this.loaded) void this.load();
  }

  /** Whether the viewer shows its own code, not the geometry of an app. */
  get standalone(): boolean {
    return this.hasAttribute("src") || this.hasAttribute("link") || !!this.textContent?.trim();
  }

  /** Show the geometry of a model, e.g. of the code of an editor. */
  show(geometry: Geometry) {
    this.viewer?.show(geometry);
  }

  /** Remove the model. */
  clear() {
    this.viewer?.clear();
  }

  /** Move the camera to see the whole model. */
  resetCamera() {
    this.viewer?.resetCamera();
  }

  /** Load the code, when the viewer becomes visible, e.g. when it is scrolled to. */
  private loadWhenVisible() {
    this.observer?.disconnect();
    this.observer = new IntersectionObserver((entries) => {
      if (!entries.some((entry) => entry.isIntersecting)) return;
      this.observer?.disconnect();
      this.observer = undefined;
      void this.load();
    });
    this.observer.observe(this);
  }

  /** Compile the code of a standalone viewer and show its model or its errors. */
  private async load() {
    this.loaded = true;
    try {
      const compiler = (this.compiler ??= await wasmCompiler());
      this.session ??= compiler.session();
      const result = await this.session.build(await this.code(compiler));
      if (!result) return;
      this.show(result.geometry);
      this.showMessage(result.success ? "" : result.report);
    } catch (error) {
      this.clear();
      this.showMessage(errorMessage(error));
    }
  }

  /** The code of a standalone viewer: of its shared link, its file or its text content. */
  private async code(compiler: Compiler): Promise<string> {
    const link = this.getAttribute("link");
    if (link) return (await decodeLink(compiler, link)).code;
    const src = this.getAttribute("src");
    if (src) {
      const response = await fetch(src);
      if (!response.ok) throw new Error(`Cannot load ${src}: ${response.status}`);
      return response.text();
    }
    return this.textContent ?? "";
  }

  private showMessage(message: string) {
    if (this.message) this.message.textContent = message;
  }
}

if (!customElements.get("mucad-viewer")) customElements.define("mucad-viewer", MucadViewer);

declare global {
  interface HTMLElementTagNameMap {
    "mucad-viewer": MucadViewer;
  }
}
