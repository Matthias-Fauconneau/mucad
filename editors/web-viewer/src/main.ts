// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

import {
  Dock,
  Documents,
  Viewer,
  errorMessage,
  share,
  wasmCompiler,
  webPlatform,
  addCompileShortcut,
  addCompletion,
  addLiteralStepping,
  addSaveShortcut,
  createEditor,
  replaceCode,
  setDiagnostics,
} from "mucad-web-components";
import { examples } from "./examples";
import "./style.css";

/** The time after the last edit, before the code is compiled again. */
const COMPILE_DELAY_MS = 500;

const element = <T extends HTMLElement>(id: string) => document.getElementById(id) as T;
const documentsElement = element<HTMLSpanElement>("documents");
const documentName = element<HTMLInputElement>("document-name");
const documentListButton = element<HTMLButtonElement>("document-list-button");
const documentList = element<HTMLDivElement>("document-list");
const status = element<HTMLSpanElement>("status");
const exportButton = element<HTMLButtonElement>("export");
const exportButtonCaption = element<HTMLDivElement>("export-btn-caption");
const saveButton = element<HTMLButtonElement>("save");
const formatButton = element<HTMLButtonElement>("format");
const compileButton = element<HTMLButtonElement>("compile");
const autoUpdate = element<HTMLInputElement>("auto-update");
const diagnostics = element<HTMLPreElement>("diagnostics");
const shareButton = element<HTMLButtonElement>("share");
const sharePanel = element<HTMLDivElement>("share-panel");
const shareError = element<HTMLParagraphElement>("share-error");
const copyLinkButton = element<HTMLButtonElement>("copy-link");
const qrCanvas = element<HTMLCanvasElement>("qr-canvas");
const qrDownloadButton = element<HTMLButtonElement>("qr-download");
const stripComments = element<HTMLInputElement>("strip-comments");
const menu = element<HTMLDivElement>("menu");
const menuButton = element<HTMLButtonElement>("menu-button");

const dock = new Dock(element("dock"), [
  { id: "editor", title: "Code", element: element("editor") },
  { id: "view", title: "View", element: element("view") },
  { id: "diagnostics", title: "Diagnostics", element: diagnostics },
], "view");

const platform = webPlatform;
const compiler = await wasmCompiler();
const session = compiler.session();
const viewer = new Viewer(element("view"));
const documents = new Documents(platform, examples);

/** The model type of the last successful compilation, which decides the export format. */
let modelType: string | undefined;

autoUpdate.checked = platform.setting("autoUpdate") !== "false";

/** The pending compilation after an edit. */
let timer: number | undefined;

/** Mark that the code has changed since it was compiled, e.g. to compile it with the button. */
function setModified(modified: boolean) {
  compileButton.classList.toggle("btn-bright", modified);
}

/** Compile the code of the editor at once, instead of a pending compilation. */
function compileNow() {
  clearTimeout(timer);
  compile(editor.getValue());
}

/** Compile `code` and show its model, diagnostics and status, unless it is outdated. */
async function compile(code: string) {
  setModified(false);
  const result = await session.build(code);
  if (!result) return;
  diagnostics.textContent = result.report;
  setDiagnostics(editor, result.diagnostics);
  diagnostics.classList.toggle("error", !result.success);
  viewer.show(result.geometry);
  modelType = result.geometry.kind === "none" ? undefined : result.geometry.kind;

  status.textContent = modelType ? `${modelType.toUpperCase()} model` : "Errors";
  status.classList.toggle("error", !modelType);
  exportButton.disabled = modelType !== "3d" && modelType !== "2d";
  exportButtonCaption.textContent = modelType === "2d" ? "Export SVG" : "Export STL";
}

/** Format the code of the editor, unless it is changed meanwhile. */
async function format() {
  const model = editor.getModel();
  if (!model) return;
  const version = model.getVersionId();
  const formatted = await compiler.format(model.getValue());
  if (model.getVersionId() !== version) return;
  if (formatted.success && formatted.formatted) {
    replaceCode(editor, formatted.code);
  }
}

/** Show the document `key` in the editor with its unsaved changes. */
function showDocument(key: string) {
  const document = documents.show(key);
  documentName.value = document.name;
  editor.setValue(document.code);
}

/** Show the document `key` from the list, compile it at once and look at the whole model. */
function openDocument(key: string) {
  showDocument(key);
  compileNow();
  viewer.resetCamera();
}

/** Show an error of storing the own documents in the browser. */
function showStoreError(error: unknown) {
  showError(`Cannot store the documents in the browser: ${errorMessage(error)}`);
}

/**
 * Apply the name in the name field to the shown document: an own document is renamed, an
 * example or shared code is copied into a new own document, see `Documents::rename`.
 */
async function renameDocument() {
  try {
    await documents.rename(documentName.value);
  } catch (error) {
    showStoreError(error);
  }
  documentName.value = documents.shownDocument?.name ?? "";
}

/** Delete the own document `key` after a confirmation, the first example is shown instead. */
async function deleteDocument(key: string) {
  const document = documents.get(key);
  if (!(await platform.confirm(`Delete "${document.name}"? It cannot be restored.`))) return;
  const shown = key === documents.shown;
  try {
    await documents.delete(key);
  } catch (error) {
    showStoreError(error);
  }
  if (shown) openDocument("0");
}

/** Show the documents in groups: own documents, shared code and examples. */
function renderDocumentList() {
  const groups: [string, string[]][] = [
    ["Own documents", documents.keys("own")],
    ["Shared code", documents.keys("shared")],
    ["Examples", documents.keys("example")],
  ];
  documentList.replaceChildren();
  for (const [title, group] of groups) {
    if (!group.length) continue;
    const heading = document.createElement("p");
    heading.className = "document-group";
    heading.textContent = title;
    documentList.append(heading);
    for (const key of group) {
      const doc = documents.get(key);
      const row = document.createElement("div");
      row.className = "document-row";
      const button = document.createElement("button");
      button.setAttribute("role", "option");
      button.setAttribute("aria-selected", String(key === documents.shown));
      button.textContent = documents.isModified(doc) ? `${doc.name} ●` : doc.name;
      button.addEventListener("click", () => {
        showDocumentList(false);
        openDocument(key);
      });
      row.append(button);
      if (doc.kind === "own") {
        const remove = document.createElement("button");
        remove.className = "document-delete";
        remove.title = `Delete ${doc.name}`;
        remove.textContent = "✕";
        remove.addEventListener("click", (event) => {
          // The list stays open, also when it is rendered again.
          event.stopPropagation();
          void deleteDocument(key);
        });
        row.append(remove);
      }
      documentList.append(row);
    }
  }
}

/** Show or hide the list of documents. */
function showDocumentList(open: boolean) {
  if (open) renderDocumentList();
  documentList.hidden = !open;
  documentListButton.setAttribute("aria-expanded", String(open));
}

/** Show the modified flag of the shown document, and of each one in an open list. */
function updateModified() {
  const doc = documents.shownDocument;
  const modified = !!doc && documents.isModified(doc);
  saveButton.classList.toggle("btn-bright", modified);
  documentsElement.classList.toggle("modified", modified);
  if (!documentList.hidden) renderDocumentList();
}

/**
 * Save the shown document in the browser, which formats and compiles it and shows its link in
 * the URL. An example or shared code is copied into an own document.
 */
async function save() {
  await format();
  try {
    await documents.save(editor.getValue());
  } catch (error) {
    showStoreError(error);
  }
  documentName.value = documents.shownDocument?.name ?? "";
  compileNow();
  void updateUrl();
}

/** The code and the name of a shared link in the URL, without one neither code nor error. */
async function sharedLink(): Promise<{ code?: string; name?: string; error?: string }> {
  const link = share.linkInUrl(new URL(location.href));
  if (link === undefined) return {};
  try {
    return await share.decodeLink(compiler, link);
  } catch (error) {
    return { error: errorMessage(error) };
  }
}

/**
 * Show the code of a shared link formatted under its `name`, e.g. `brick.mu (shared)`, or the
 * own document with its name and code, compile it and look at the whole model.
 */
async function showSharedCode(code: string, name?: string) {
  const formatted = await compiler.format(code);
  code = formatted.success && formatted.formatted ? formatted.code : code;
  openDocument(documents.addShared(code, name));
}

/** Show an error, e.g. of a shared link, in front of the diagnostics of the shown code. */
function showError(error: string) {
  diagnostics.textContent = `${error}\n\n${diagnostics.textContent}`;
  diagnostics.classList.add("error");
}

/** Show the error of a shared link in front of the diagnostics of the shown code. */
function showLinkError(error: string) {
  showError(`Cannot open the shared link: ${error}`);
}

stripComments.checked = platform.setting("stripComments") === "true";

/** Show an error in the share menu, or no error. */
function showShareError(error?: string) {
  shareError.textContent = error ?? "";
  shareError.hidden = error === undefined;
}

/**
 * The link to the web viewer with the code of the editor and the name of the shown document in
 * its preamble, with `strip` without comments.
 *
 * Fails, e.g. for code, which is too long for a link.
 */
function encodeLink(strip: boolean): Promise<string> {
  const name = documents.shownDocument?.sharedName;
  return compiler.encodeLink(editor.getValue(), share.preamble(name), strip);
}

/**
 * Show the link of the code of the editor in the address bar, e.g. after saving, so that it can
 * be bookmarked and is opened again after a reload.
 *
 * The URL is replaced without a new entry in the history and without opening the link again.
 * Code, which is too long for a link, removes the link from the URL, which would be outdated.
 */
async function updateUrl() {
  const url = new URL(location.href);
  url.searchParams.delete("code");
  try {
    url.hash = (await encodeLink(false)).split("#")[1];
  } catch {
    url.hash = "";
  }
  history.replaceState(null, "", url);
}

/** The link to share the code of the editor, `undefined` after an error. */
async function shareLink(): Promise<string | undefined> {
  try {
    return await encodeLink(stripComments.checked);
  } catch (error) {
    showShareError(`Cannot share the code: ${errorMessage(error)}`);
    return undefined;
  }
}

/** Copy the link with the code to the clipboard. */
async function copyLink() {
  const link = await shareLink();
  if (link === undefined) return;
  try {
    await platform.copyText(link);
    copyLinkButton.textContent = "Copied";
    setTimeout(() => (copyLinkButton.textContent = "Copy link"), 1500);
  } catch (error) {
    showShareError(`Cannot copy the link: ${error}`);
  }
}

/** The number of the latest update of the share menu, earlier updates are not shown. */
let shareMenuUpdates = 0;

/**
 * Draw the QR code of the link with the code dark on white, returns if it is drawn, `undefined`
 * if a later update of the share menu has been started meanwhile.
 */
async function drawQrCode(update: number): Promise<boolean | undefined> {
  const link = await shareLink();
  if (link === undefined) return false;
  let modules: Uint8Array;
  try {
    modules = await compiler.qrCode(link);
  } catch (error) {
    showShareError(`Cannot create the QR code: ${errorMessage(error)}`);
    return false;
  }
  if (update !== shareMenuUpdates) return undefined;
  share.drawQrCode(qrCanvas, modules);
  return true;
}

/**
 * Update the share menu for the code of the editor: the QR code, or the error, e.g. of code,
 * which is too long for a link.
 */
async function updateShareMenu() {
  const update = ++shareMenuUpdates;
  showShareError();
  const shareable = await drawQrCode(update);
  if (shareable === undefined) return;
  for (const element of [copyLinkButton, qrCanvas, qrDownloadButton]) {
    element.hidden = !shareable;
  }
}

/** Show or hide the share menu. */
function toggleShareMenu() {
  sharePanel.hidden = !sharePanel.hidden;
  if (!sharePanel.hidden) void updateShareMenu();
}

/** Show or hide the menu of actions and links on phones. */
function showMenu(open: boolean) {
  menu.classList.toggle("open", open);
  menuButton.setAttribute("aria-expanded", String(open));
}

/** Download the QR code as PNG. */
function downloadQrCode() {
  qrCanvas.toBlob((blob) => {
    if (blob) void platform.saveFile(`${documents.modelName}-qr.png`, blob);
  }, "image/png");
}

/** Download the model as STL (3D) or SVG (2D). */
async function exportModel() {
  const [format, type] =
    modelType === "2d" ? (["svg", "image/svg+xml"] as const) : (["stl", "model/stl"] as const);
  try {
    const data = await session.export(format);
    const blob = new Blob([data as BlobPart], { type });
    await platform.saveFile(`${documents.modelName}.${format}`, blob);
  } catch (error) {
    showError(`Cannot export the model: ${errorMessage(error)}`);
  }
}

// Detect if the user is on macOS
const isMac = /macintosh|mac os x/i.test(navigator.userAgent);
const modifierSymbol = isMac ? '⌘' : 'Ctrl+';

// Update all shortcut displays dynamically
document.querySelectorAll('.shortcut-key').forEach((el) => {
    el.textContent = modifierSymbol + el.textContent;
});

document.addEventListener('keydown', (e) => {
    const isCtrlOrCmd = e.ctrlKey || e.metaKey;
    if (!isCtrlOrCmd) return;

    // Ctrl+E or Cmd+E -> Export
    if (e.key.toLowerCase() === 'e') {
      e.preventDefault();
      if (!exportButton.disabled) void exportModel();
    }

    // Ctrl+S or Cmd+S -> Save
    if (e.key.toLowerCase() === 's') {
        e.preventDefault();
        void save();
    }

    // Ctrl+Enter or Cmd+Enter -> Compile
    if (e.key === 'Enter') {
        e.preventDefault();
        compileNow();
    }

    // Ctrl+F or Cmd+F -> Format
    if (e.key.toLowerCase() === 'f') {
        e.preventDefault();
        void format();
    }
});

// Compile after each change with auto-update, otherwise with the compile button.
const editor = createEditor(element("editor"), (code) => {
  documents.setCode(code);
  clearTimeout(timer);
  if (autoUpdate.checked) {
    timer = window.setTimeout(() => compile(code), COMPILE_DELAY_MS);
  } else {
    setModified(true);
  }
  // An open share menu shows the link of the changed code.
  if (!sharePanel.hidden) void updateShareMenu();
});
addCompileShortcut(editor, compileNow);
addSaveShortcut(editor, save);
// Literals are stepped with Alt+↑/↓ by the digit at the cursor, a step is compiled at once with
// auto-update.
addLiteralStepping(
  editor,
  (code, lineNo, column) => compiler.literalAt(code, lineNo, column),
  (literal, cursor, factor) => compiler.stepLiteral(literal, cursor, factor),
  () => {
    if (autoUpdate.checked) compileNow();
  },
);
// Names are completed with the symbols of the last compilation of the editor.
addCompletion((code, lineNo, column) => session.complete(code, lineNo, column));

// Set version tag
const version = compiler.version;
const versionTag = document.getElementById('version-tag');

if (versionTag) {
    versionTag.textContent = "v" + version;
}

// A document is compiled at once, also without auto-update, and shown as a whole.
// Its unsaved changes are kept, when another document is shown.
documentListButton.addEventListener("click", () => showDocumentList(documentList.hidden));
// A new name is applied with Enter or by leaving the name field, Escape restores the name.
documentName.addEventListener("change", () => void renameDocument());
documentName.addEventListener("keydown", (event) => {
  if (event.key === "Escape" && documents.shownDocument) {
    documentName.value = documents.shownDocument.name;
  }
  if (event.key === "Enter" || event.key === "Escape") documentName.blur();
});
exportButton.addEventListener("click", () => void exportModel());
saveButton.addEventListener("click", () => void save());
formatButton.addEventListener("click", () => void format());
compileButton.addEventListener("click", compileNow);
shareButton.addEventListener("click", toggleShareMenu);
copyLinkButton.addEventListener("click", copyLink);
qrDownloadButton.addEventListener("click", downloadQrCode);
stripComments.addEventListener("change", () => {
  platform.setSetting("stripComments", String(stripComments.checked));
  // The shown QR code is of the link with the new setting.
  void updateShareMenu();
});
menuButton.addEventListener("click", () => showMenu(!menu.classList.contains("open")));
// The menu is hidden after an action or link, but not after switching auto-update.
menu.addEventListener("click", (event) => {
  if ((event.target as Element).closest("button, a")) showMenu(false);
});
// The list of documents, the share menu and the menu are hidden by a click outside of them.
document.addEventListener("click", (event) => {
  const target = event.target as Node;
  if (!documentList.contains(target) && !documentListButton.contains(target)) {
    showDocumentList(false);
  }
  if (!sharePanel.contains(target) && !shareButton.contains(target)) sharePanel.hidden = true;
  if (!menu.contains(target) && !menuButton.contains(target)) showMenu(false);
});
autoUpdate.addEventListener("change", () => {
  platform.setSetting("autoUpdate", String(autoUpdate.checked));
  // Changes made without auto-update are compiled, when it is switched on.
  if (autoUpdate.checked && compileButton.classList.contains("btn-bright")) {
    compileNow();
  }
});

// The own documents of the browser, the code of a shared link, otherwise the first example or
// the one given by the URL, e.g. `?example=1`.
documents.addEventListener("change", updateModified);
try {
  await documents.restore();
} catch (error) {
  showError(`Cannot restore the documents of the browser: ${errorMessage(error)}`);
}
const shared = await sharedLink();
if (shared.code !== undefined) {
  await showSharedCode(shared.code, shared.name);
} else {
  const example = Number(new URLSearchParams(location.search).get("example")) || 0;
  showDocument(String(examples[example] ? example : 0));
  compileNow();
  if (shared.error) showLinkError(shared.error);
}

// A shared link opened while the viewer runs, e.g. pasted into the address bar.
window.addEventListener("hashchange", async () => {
  const shared = await sharedLink();
  if (shared.code !== undefined) await showSharedCode(shared.code, shared.name);
  else if (shared.error) showLinkError(shared.error);
});

// Unsaved changes are not stored, the browser asks before they are lost by a reload or by
// closing the page.
platform.confirmClose(() => documents.hasUnsavedChanges());
