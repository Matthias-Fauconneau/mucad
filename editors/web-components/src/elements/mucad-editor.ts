// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

import type { Compiler, CompilerSession, Diagnostic } from "../compiler/types";
import {
  type CodeEditor,
  addCompileShortcut,
  addCompletion,
  addLiteralStepping,
  addSaveShortcut,
  createEditor,
  replaceCode,
  setDiagnostics,
} from "../editor";

/**
 * `<mucad-editor>`: a Monaco editor of µcad code with syntax highlighting, markers of
 * diagnostics and, when it is connected to a compiler, completion and stepping of literals.
 *
 * Events:
 * - `code-change` after each edit, with the code as `detail.code`,
 * - `save` for Ctrl+S, `compile` for Ctrl+Enter (⌘ on macOS),
 * - `literal-step` after a literal has been stepped, e.g. to compile at once.
 *
 * The editor is in the light DOM, because Monaco adds its styles and widgets to the document.
 */
export class MucadEditor extends HTMLElement {
  private codeEditor?: CodeEditor;
  private compiler?: Compiler;

  connectedCallback() {
    // The element is created once, also when it is moved, e.g. into a panel of a dock.
    if (this.codeEditor) return;
    const editor = createEditor(this, (code) =>
      this.dispatchEvent(new CustomEvent("code-change", { detail: { code } })),
    );
    addCompileShortcut(editor, () => this.dispatchEvent(new Event("compile")));
    addSaveShortcut(editor, () => this.dispatchEvent(new Event("save")));
    this.codeEditor = editor;
  }

  /** The Monaco editor, which exists, when the element is in the document. */
  get editor(): CodeEditor {
    if (!this.codeEditor) throw new Error("<mucad-editor> is not in the document");
    return this.codeEditor;
  }

  /** The code. */
  get value(): string {
    return this.editor.getValue();
  }

  /** Replace the code, e.g. of another document, without undo of the code before. */
  set value(code: string) {
    this.editor.setValue(code);
  }

  /** Replace the code as one edit, which can be undone, e.g. formatted code. */
  replaceCode(code: string) {
    replaceCode(this.editor, code);
  }

  /** Show diagnostics as markers, e.g. squiggles below errors. */
  setDiagnostics(diagnostics: Diagnostic[]) {
    setDiagnostics(this.editor, diagnostics);
  }

  /**
   * Complete names with the symbols of the last build of `session`, and step literals with
   * Alt+↑/↓ with `compiler`.
   */
  connect(compiler: Compiler, session: CompilerSession) {
    this.compiler = compiler;
    addCompletion(this.editor, (code, lineNo, column) => session.complete(code, lineNo, column));
    addLiteralStepping(
      this.editor,
      (code, lineNo, column) => compiler.literalAt(code, lineNo, column),
      (literal, cursor, factor) => compiler.stepLiteral(literal, cursor, factor),
      () => this.dispatchEvent(new Event("literal-step")),
    );
  }

  /** Format the code with the connected compiler, unless it is changed meanwhile. */
  async format() {
    const model = this.editor.getModel();
    if (!model || !this.compiler) return;
    const version = model.getVersionId();
    const formatted = await this.compiler.format(model.getValue());
    if (model.getVersionId() !== version) return;
    if (formatted.success && formatted.formatted) this.replaceCode(formatted.code);
  }
}

if (!customElements.get("mucad-editor")) customElements.define("mucad-editor", MucadEditor);

declare global {
  interface HTMLElementTagNameMap {
    "mucad-editor": MucadEditor;
  }
}
