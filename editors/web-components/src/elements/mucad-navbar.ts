// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

import logo from "../assets/logo.svg";
import type { Documents } from "../services/documents";
import "./navbar.css";

/** The navigation bar, the children of the app are moved into `.actions` and `.links`. */
const TEMPLATE = `
<header class="navbar">
  <a href="https://mucad.org" target="_blank" rel="noopener noreferrer" class="logo">
    <img alt="µcad" />
    <span class="version"></span>
  </a>
  <span class="documents dropdown">
    <input
      class="document-name"
      aria-label="Document name"
      title="The name of the document, a new name of an example creates an own document"
      spellcheck="false"
      autocomplete="off"
    />
    <button class="document-list-button" title="Show the documents" aria-expanded="false">▾</button>
    <div class="document-list dropdown-panel" role="listbox" hidden></div>
  </span>
  <span class="actions"></span>
  <span class="status"></span>
  <div class="menu">
    <button class="save" title="Save, format and compile the code">
      Save <kbd class="shortcut-key">S</kbd>
    </button>
    <button class="compile">Compile <kbd class="shortcut-key">Enter</kbd></button>
    <label class="toggle" title="Compile after each change">
      <input type="checkbox" class="auto-update" checked />
      Auto-update
    </label>
    <button class="format">Format <kbd class="shortcut-key">F</kbd></button>
    <nav class="links"></nav>
  </div>
  <button class="menu-button" title="Menu" aria-expanded="false">☰</button>
</header>
`;

/** The modifier of shortcuts: ⌘ on macOS, otherwise Ctrl. */
const MODIFIER = /macintosh|mac os x/i.test(navigator.userAgent) ? "⌘" : "Ctrl+";

/**
 * `<mucad-navbar>`: the navigation bar of an editor with the logo and the version, the name
 * field and the list of the documents, the actions Save, Compile, Auto-update and Format and
 * the status, e.g. `3D model`. On phones, the actions and links are a menu on the right.
 *
 * The children of the app with `slot="actions"`, e.g. a share menu, are behind Format, the ones
 * with `slot="links"` are on the right.
 *
 * Events:
 * - `save`, `compile`, `format` of the buttons,
 * - `auto-update` when it is switched, see `autoUpdate`,
 * - `open-document`, `delete-document` with the key of the document as `detail.key`,
 * - `rename-document` with the new name as `detail.name`, see `refresh()`.
 */
export class MucadNavbar extends HTMLElement {
  private documentsService?: Documents;
  private element = <T extends Element>(selector: string) => this.querySelector(selector) as T;

  connectedCallback() {
    if (this.querySelector(":scope > .navbar")) return;
    const actions = [...this.querySelectorAll(':scope > [slot="actions"]')];
    const links = [...this.querySelectorAll(':scope > [slot="links"]')];
    this.innerHTML = TEMPLATE;
    this.element<HTMLImageElement>(".logo img").src = logo;
    this.element(".actions").append(...actions);
    this.element(".links").append(...links);
    for (const key of this.querySelectorAll(".shortcut-key")) {
      key.textContent = MODIFIER + key.textContent;
    }
    this.listen();
  }

  /** The documents, whose shown one, list and modified flags are shown. */
  set documents(documents: Documents) {
    this.documentsService = documents;
    documents.addEventListener("change", () => this.refresh());
    this.refresh();
  }

  /** The version of µcad, e.g. `0.6.3`. */
  set version(version: string) {
    this.element(".version").textContent = `v${version}`;
  }

  /** Whether to compile after each change. */
  get autoUpdate(): boolean {
    return this.element<HTMLInputElement>(".auto-update").checked;
  }

  set autoUpdate(autoUpdate: boolean) {
    this.element<HTMLInputElement>(".auto-update").checked = autoUpdate;
  }

  /** Mark that the code has changed since it was compiled, e.g. without auto-update. */
  set compileModified(modified: boolean) {
    this.element(".compile").classList.toggle("btn-bright", modified);
  }

  /** Show the status, e.g. `3D model` or `Errors`. */
  setStatus(status: string, error: boolean) {
    const element = this.element(".status");
    element.textContent = status;
    element.classList.toggle("error", error);
  }

  /** Show the name and the modified flag of the shown document, and the list, if it is open. */
  refresh() {
    const documents = this.documentsService;
    const document = documents?.shownDocument;
    const name = this.element<HTMLInputElement>(".document-name");
    if (globalThis.document.activeElement !== name) name.value = document?.name ?? "";
    const modified = !!document && !!documents?.isModified(document);
    this.element(".save").classList.toggle("btn-bright", modified);
    this.element(".documents").classList.toggle("modified", modified);
    if (!this.element<HTMLElement>(".document-list").hidden) this.renderDocumentList();
  }

  private event(type: string, detail?: object) {
    this.dispatchEvent(new CustomEvent(type, { detail }));
  }

  private listen() {
    const name = this.element<HTMLInputElement>(".document-name");
    const listButton = this.element<HTMLButtonElement>(".document-list-button");
    const list = this.element<HTMLElement>(".document-list");
    const menu = this.element<HTMLElement>(".menu");
    const menuButton = this.element<HTMLButtonElement>(".menu-button");

    this.element(".save").addEventListener("click", () => this.event("save"));
    this.element(".compile").addEventListener("click", () => this.event("compile"));
    this.element(".format").addEventListener("click", () => this.event("format"));
    this.element(".auto-update").addEventListener("change", () => this.event("auto-update"));

    // A new name is applied with Enter or by leaving the name field, Escape restores the name.
    name.addEventListener("change", () => this.event("rename-document", { name: name.value }));
    name.addEventListener("keydown", (event) => {
      if (event.key === "Escape") name.value = this.documentsService?.shownDocument?.name ?? "";
      if (event.key === "Enter" || event.key === "Escape") name.blur();
    });
    name.addEventListener("blur", () => this.refresh());

    listButton.addEventListener("click", () => this.showDocumentList(list.hidden));
    menuButton.addEventListener("click", () => this.showMenu(!menu.classList.contains("open")));
    // The menu is hidden after an action or link, but not after switching auto-update.
    menu.addEventListener("click", (event) => {
      if ((event.target as Element).closest("button, a")) this.showMenu(false);
    });
    // The list of documents and the menu are hidden by a click outside of them.
    document.addEventListener("click", (event) => {
      const target = event.target as Node;
      if (!list.contains(target) && !listButton.contains(target)) this.showDocumentList(false);
      if (!menu.contains(target) && !menuButton.contains(target)) this.showMenu(false);
    });
  }

  /** Show or hide the list of documents. */
  private showDocumentList(open: boolean) {
    if (open) this.renderDocumentList();
    this.element<HTMLElement>(".document-list").hidden = !open;
    this.element(".document-list-button").setAttribute("aria-expanded", String(open));
  }

  /** Show or hide the menu of actions and links on phones. */
  private showMenu(open: boolean) {
    this.element(".menu").classList.toggle("open", open);
    this.element(".menu-button").setAttribute("aria-expanded", String(open));
  }

  /** Show the documents in groups: own documents, shared code and examples. */
  private renderDocumentList() {
    const documents = this.documentsService;
    const list = this.element<HTMLElement>(".document-list");
    list.replaceChildren();
    if (!documents) return;
    const groups: [string, string[]][] = [
      ["Own documents", documents.keys("own")],
      ["Shared code", documents.keys("shared")],
      ["Examples", documents.keys("example")],
    ];
    for (const [title, keys] of groups) {
      if (!keys.length) continue;
      const heading = document.createElement("p");
      heading.className = "document-group";
      heading.textContent = title;
      list.append(heading);
      for (const key of keys) {
        const shown = documents.get(key);
        const row = document.createElement("div");
        row.className = "document-row";
        const button = document.createElement("button");
        button.setAttribute("role", "option");
        button.setAttribute("aria-selected", String(key === documents.shown));
        button.textContent = documents.isModified(shown) ? `${shown.name} ●` : shown.name;
        button.addEventListener("click", () => {
          this.showDocumentList(false);
          this.event("open-document", { key });
        });
        row.append(button);
        if (shown.kind === "own") {
          const remove = document.createElement("button");
          remove.className = "document-delete";
          remove.title = `Delete ${shown.name}`;
          remove.textContent = "✕";
          remove.addEventListener("click", (event) => {
            // The list stays open, also when it is rendered again.
            event.stopPropagation();
            this.event("delete-document", { key });
          });
          row.append(remove);
        }
        list.append(row);
      }
    }
  }
}

if (!customElements.get("mucad-navbar")) customElements.define("mucad-navbar", MucadNavbar);

declare global {
  interface HTMLElementTagNameMap {
    "mucad-navbar": MucadNavbar;
  }
}
