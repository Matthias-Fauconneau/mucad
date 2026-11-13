// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

import {
  Dock,
  Documents,
  type MucadEditor,
  type MucadNavbar,
  type MucadViewer,
  errorMessage,
  share,
  wasmCompiler,
  webPlatform,
} from "mucad-web-components";
import { examples } from "./examples";
import "./style.css";

/** The time after the last edit, before the code is compiled again. */
const COMPILE_DELAY_MS = 500;

const element = <T extends HTMLElement>(id: string) => document.getElementById(id) as T;
const navbar = document.querySelector("mucad-navbar") as MucadNavbar;
const editor = element<MucadEditor>("editor");
const viewer = element<MucadViewer>("view");
const diagnostics = element<HTMLPreElement>("diagnostics");
const exportButton = element<HTMLButtonElement>("export");
const exportButtonCaption = element<HTMLDivElement>("export-btn-caption");
const shareButton = element<HTMLButtonElement>("share");
const sharePanel = element<HTMLDivElement>("share-panel");
const shareError = element<HTMLParagraphElement>("share-error");
const copyLinkButton = element<HTMLButtonElement>("copy-link");
const qrCanvas = element<HTMLCanvasElement>("qr-canvas");
const qrDownloadButton = element<HTMLButtonElement>("qr-download");
const stripComments = element<HTMLInputElement>("strip-comments");

new Dock(
  element("dock"),
  [
    { id: "editor", title: "Code", element: editor },
    { id: "view", title: "View", element: viewer },
    { id: "diagnostics", title: "Diagnostics", element: diagnostics },
  ],
  "view",
);

const platform = webPlatform;
const compiler = await wasmCompiler();
const session = compiler.session();
const documents = new Documents(platform, examples);
editor.connect(compiler, session);
navbar.documents = documents;
navbar.version = compiler.version;
navbar.autoUpdate = platform.setting("autoUpdate") !== "false";

/** The model type of the last successful compilation, which decides the export format. */
let modelType: string | undefined;

/** The pending compilation after an edit. */
let timer: number | undefined;

/** Whether the code has changed since it was compiled, e.g. to compile it with the button. */
let compileModified = false;

function setCompileModified(modified: boolean) {
  compileModified = modified;
  navbar.compileModified = modified;
}

/** Compile the code of the editor at once, instead of a pending compilation. */
function compileNow() {
  clearTimeout(timer);
  void compile(editor.value);
}

/** Compile `code` and show its model, diagnostics and status, unless it is outdated. */
async function compile(code: string) {
  setCompileModified(false);
  const result = await session.build(code);
  if (!result) return;
  diagnostics.textContent = result.report;
  editor.setDiagnostics(result.diagnostics);
  diagnostics.classList.toggle("error", !result.success);
  viewer.show(result.geometry);
  modelType = result.geometry.kind === "none" ? undefined : result.geometry.kind;

  navbar.setStatus(modelType ? `${modelType.toUpperCase()} model` : "Errors", !modelType);
  exportButton.disabled = modelType !== "3d" && modelType !== "2d";
  exportButtonCaption.textContent = modelType === "2d" ? "Export SVG" : "Export STL";
}

/** Show the document `key` in the editor with its unsaved changes. */
function showDocument(key: string) {
  editor.value = documents.show(key).code;
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
 * Rename the shown document: an own document is renamed, an example or shared code is copied
 * into a new own document, see `Documents::rename`.
 */
async function renameDocument(name: string) {
  try {
    await documents.rename(name);
  } catch (error) {
    showStoreError(error);
  }
  navbar.refresh();
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

/**
 * Save the shown document in the browser, which formats and compiles it and shows its link in
 * the URL. An example or shared code is copied into an own document.
 */
async function save() {
  await editor.format();
  try {
    await documents.save(editor.value);
  } catch (error) {
    showStoreError(error);
  }
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
  return compiler.encodeLink(editor.value, share.preamble(name), strip);
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
    showShareError(`Cannot copy the link: ${errorMessage(error)}`);
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

// The shortcuts outside of the editor, which has its own ones for saving and compiling.
document.addEventListener("keydown", (event) => {
  if (!event.ctrlKey && !event.metaKey) return;
  switch (event.key.toLowerCase()) {
    case "e":
      event.preventDefault();
      if (!exportButton.disabled) void exportModel();
      break;
    case "s":
      event.preventDefault();
      void save();
      break;
    case "enter":
      event.preventDefault();
      compileNow();
      break;
    case "f":
      event.preventDefault();
      void editor.format();
      break;
  }
});

// Compile after each change with auto-update, otherwise with the compile button.
editor.addEventListener("code-change", (event) => {
  const { code } = (event as CustomEvent<{ code: string }>).detail;
  documents.setCode(code);
  clearTimeout(timer);
  if (navbar.autoUpdate) {
    timer = window.setTimeout(() => void compile(code), COMPILE_DELAY_MS);
  } else {
    setCompileModified(true);
  }
  // An open share menu shows the link of the changed code.
  if (!sharePanel.hidden) void updateShareMenu();
});
editor.addEventListener("save", () => void save());
editor.addEventListener("compile", compileNow);
// A stepped literal is compiled at once with auto-update.
editor.addEventListener("literal-step", () => {
  if (navbar.autoUpdate) compileNow();
});

// A document is compiled at once, also without auto-update, and shown as a whole.
// Its unsaved changes are kept, when another document is shown.
const detail = <T>(event: Event) => (event as CustomEvent<T>).detail;
navbar.addEventListener("open-document", (event) => {
  openDocument(detail<{ key: string }>(event).key);
});
navbar.addEventListener("rename-document", (event) => {
  void renameDocument(detail<{ name: string }>(event).name);
});
navbar.addEventListener("delete-document", (event) => {
  void deleteDocument(detail<{ key: string }>(event).key);
});
navbar.addEventListener("save", () => void save());
navbar.addEventListener("compile", compileNow);
navbar.addEventListener("format", () => void editor.format());
navbar.addEventListener("auto-update", () => {
  platform.setSetting("autoUpdate", String(navbar.autoUpdate));
  // Changes made without auto-update are compiled, when it is switched on.
  if (navbar.autoUpdate && compileModified) compileNow();
});

exportButton.addEventListener("click", () => void exportModel());
shareButton.addEventListener("click", toggleShareMenu);
copyLinkButton.addEventListener("click", () => void copyLink());
qrDownloadButton.addEventListener("click", downloadQrCode);
stripComments.addEventListener("change", () => {
  platform.setSetting("stripComments", String(stripComments.checked));
  // The shown QR code is of the link with the new setting.
  void updateShareMenu();
});
// The share menu is hidden by a click outside of it.
document.addEventListener("click", (event) => {
  const target = event.target as Node;
  if (!sharePanel.contains(target) && !shareButton.contains(target)) sharePanel.hidden = true;
});

// The own documents of the browser, the code of a shared link, otherwise the first example or
// the one given by the URL, e.g. `?example=1`.
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
