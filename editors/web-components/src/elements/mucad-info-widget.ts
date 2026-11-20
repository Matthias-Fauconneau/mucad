// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

import { theme } from "../generated/theme";

/** The styles in the shadow root, with the colors of the µcad theme. */
const STYLE = `
:host {
  display: flex;
  flex-direction: column;
  align-items: flex-end;
  gap: 0.5rem;
  max-width: 100%;
  font-size: 0.8rem;
}

button {
  height: 2.2rem;
  min-width: 2.2rem;
  padding: 0 0.6rem;
  border: 1px solid var(--signal, ${theme.signal});
  border-radius: 4px;
  background: var(--primary, ${theme.primary});
  color: var(--signal, ${theme.signal});
  font: inherit;
  cursor: pointer;
}

.panel {
  max-width: min(32rem, 100%);
  max-height: 16rem;
  overflow: auto;
  padding: 0.5rem;
  border: 1px solid var(--signal, ${theme.signal});
  border-radius: 4px;
  background: var(--primary, ${theme.primary});
  color: var(--bright, ${theme.bright});
  /* The scroll bars of the dark background. */
  color-scheme: dark;
}

.panel[hidden] {
  display: none;
}

.header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  gap: 1rem;
  margin-bottom: 0.5rem;
  color: var(--signal, ${theme.signal});
  font-weight: bold;
}

.header button {
  height: 1.6rem;
  font-weight: normal;
}

table {
  border-collapse: collapse;
}

th, td {
  padding: 0.2rem 0.4rem;
  text-align: left;
  vertical-align: top;
}

th {
  color: var(--signal, ${theme.signal});
}

kbd {
  padding: 0 0.25rem;
  border: 1px solid var(--signal, ${theme.signal});
  border-radius: 3px;
  font: 0.75rem monospace;
  white-space: nowrap;
}
`;

/**
 * `<mucad-info-widget>`: a help of the bindings in a `<mucad-viewer-overlay>`, which a `?`
 * button opens, with the basic actions or all of them.
 *
 * The help comes from {@link MucadInfoWidget.help}, e.g. `help` of `CameraControls`, as a
 * Markdown table of `helpMarkdown` of `ActionRegistry`.
 */
export class MucadInfoWidget extends HTMLElement {
  /** The help as a Markdown table, with `basic` only of the basic actions. */
  help?: (basic: boolean) => Promise<string>;

  private panel?: HTMLElement;
  private content?: HTMLElement;
  private toggleAll?: HTMLButtonElement;
  private button?: HTMLButtonElement;
  /** Whether the help has all actions, not only the basic ones. */
  private all = false;

  connectedCallback() {
    // The element is created once, also when it is moved with its overlay.
    if (this.shadowRoot) return;
    const shadow = this.attachShadow({ mode: "open" });
    const style = document.createElement("style");
    style.textContent = STYLE;

    this.panel = document.createElement("div");
    this.panel.className = "panel";
    this.panel.id = "help";
    this.panel.hidden = true;
    const header = document.createElement("div");
    header.className = "header";
    const title = document.createElement("span");
    title.textContent = "Controls";
    this.toggleAll = document.createElement("button");
    this.toggleAll.addEventListener("click", () => {
      this.all = !this.all;
      void this.update();
    });
    header.append(title, this.toggleAll);
    this.content = document.createElement("div");
    this.panel.append(header, this.content);

    this.button = document.createElement("button");
    this.button.textContent = "?";
    this.button.title = "Show the controls";
    this.button.setAttribute("aria-controls", "help");
    this.button.setAttribute("aria-expanded", "false");
    this.button.addEventListener("click", () => this.setOpen(this.panel?.hidden ?? false));
    this.addEventListener("keydown", (event) => {
      if (event.key === "Escape") this.setOpen(false);
    });

    shadow.append(style, this.panel, this.button);
  }

  /** Open or close the help. */
  setOpen(open: boolean) {
    if (!this.panel || !this.button) return;
    this.panel.hidden = !open;
    this.button.setAttribute("aria-expanded", String(open));
    this.button.title = open ? "Hide the controls" : "Show the controls";
    if (open) void this.update();
  }

  /** Show the help of the basic actions or of all of them. */
  private async update() {
    if (!this.content || !this.toggleAll) return;
    this.toggleAll.textContent = this.all ? "Basic" : "All";
    this.toggleAll.title = this.all ? "Show the basic controls" : "Show all controls";
    const markdown = (await this.help?.(!this.all)) ?? "";
    this.content.replaceChildren(markdownTable(markdown));
  }
}

/**
 * The table of `markdown`, e.g. of `helpMarkdown` of `ActionRegistry`: its first row is the
 * header, code is shown as keys. The cells are text, they cannot contain HTML.
 */
export function markdownTable(markdown: string): HTMLTableElement {
  const table = document.createElement("table");
  const rows = markdown
    .split("\n")
    .map((line) => line.trim())
    .filter((line) => line.startsWith("|") && line.endsWith("|"))
    .map((line) => line.slice(1, -1).split("|").map((cell) => cell.trim()))
    // The row between the header and the body, e.g. `|---|---|`.
    .filter((cells) => !cells.every((cell) => /^:?-+:?$/.test(cell)));
  rows.forEach((cells, index) => {
    const row = table.insertRow();
    for (const cell of cells) {
      const element = document.createElement(index === 0 ? "th" : "td");
      element.append(...inlineCode(cell));
      row.append(element);
    }
  });
  return table;
}

/** The text of `cell` with its code in backticks as keys, e.g. `` `Ctrl+Plus` ``. */
function inlineCode(cell: string): Node[] {
  return cell.split("`").map((part, index) => {
    if (index % 2 === 0) return document.createTextNode(part);
    const key = document.createElement("kbd");
    key.textContent = part;
    return key;
  });
}

if (!customElements.get("mucad-info-widget")) {
  customElements.define("mucad-info-widget", MucadInfoWidget);
}

declare global {
  interface HTMLElementTagNameMap {
    "mucad-info-widget": MucadInfoWidget;
  }
}
