// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

import { theme } from "../generated/theme";

/** The corners of the overlay, whose slots hold its widgets. */
export const CORNERS = ["top-left", "top-right", "bottom-left", "bottom-right"] as const;

/** A corner of the overlay, e.g. `top-right` for the camera navigation. */
export type Corner = (typeof CORNERS)[number];

/**
 * The styles in the shadow root: the overlay lets the events through to the view below it,
 * except on its widgets.
 */
const STYLE = `
:host {
  position: absolute;
  inset: 0;
  display: grid;
  grid-template:
    "top-left . top-right" auto
    ". . ." 1fr
    "bottom-left . bottom-right" auto
    / auto 1fr auto;
  gap: 0.5rem;
  padding: 0.5rem;
  pointer-events: none;
}

.corner {
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
}

.top-left { grid-area: top-left; align-items: flex-start; }
.top-right { grid-area: top-right; align-items: flex-end; }
.bottom-left { grid-area: bottom-left; align-items: flex-start; justify-content: flex-end; }
.bottom-right { grid-area: bottom-right; align-items: flex-end; justify-content: flex-end; }

.corner > *,
::slotted(*) {
  pointer-events: auto;
}

button {
  height: 2.2rem;
  padding: 0 0.6rem;
  border: 1px solid var(--signal, ${theme.signal});
  border-radius: 4px;
  background: var(--primary, ${theme.primary});
  color: var(--signal, ${theme.signal});
  font: inherit;
  cursor: pointer;
}
`;

/**
 * `<mucad-viewer-overlay>`: the widgets over a view, e.g. of a `<mucad-viewer>`, in its four
 * corners, whose slots are named after them, e.g. `slot="top-left"`.
 *
 * The overlay lets the mouse and touch events through to the view, except on the widgets, and
 * knows nothing about the view: its widgets send events, which the view handles.
 *
 * Events: `reset-camera`, when the camera should look at the whole model.
 */
export class MucadViewerOverlay extends HTMLElement {
  connectedCallback() {
    // The element is created once, also when it is moved with its view.
    if (this.shadowRoot) return;
    const shadow = this.attachShadow({ mode: "open" });
    const style = document.createElement("style");
    style.textContent = STYLE;
    const corners = CORNERS.map((corner) => {
      const element = document.createElement("div");
      element.className = `corner ${corner}`;
      const slot = document.createElement("slot");
      slot.name = corner;
      element.append(slot);
      return element;
    });
    shadow.append(style, ...corners);
    this.corner("top-right").prepend(this.resetButton());
  }

  /** The element of `corner` in the shadow root. */
  private corner(corner: Corner): HTMLElement {
    const element = this.shadowRoot?.querySelector<HTMLElement>(`.${corner}`);
    if (!element) throw new Error(`No corner ${corner}`);
    return element;
  }

  /** The button, which resets the camera. */
  private resetButton(): HTMLButtonElement {
    const button = document.createElement("button");
    button.textContent = "Reset camera";
    button.title = "Look at the whole model";
    button.addEventListener("click", () => this.dispatchEvent(new Event("reset-camera")));
    return button;
  }
}

if (!customElements.get("mucad-viewer-overlay")) {
  customElements.define("mucad-viewer-overlay", MucadViewerOverlay);
}

declare global {
  interface HTMLElementTagNameMap {
    "mucad-viewer-overlay": MucadViewerOverlay;
  }
  interface HTMLElementEventMap {
    "reset-camera": Event;
  }
}
