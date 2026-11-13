// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

import type { Platform, StoredDocument } from "../platform";

/** An example, e.g. of the repository's `examples` folder. */
export interface Example {
  name: string;
  code: string;
}

/** Whether a document is an example, shared code or an own document, which can be changed. */
export type DocumentKind = "example" | "shared" | "own";

/** A document: an example, the code of a shared link or an own document of the user. */
export interface OpenDocument {
  kind: DocumentKind;
  /** The name in the list of documents. */
  name: string;
  /** The name in the preamble of a shared link, e.g. of the example. */
  sharedName?: string;
  /** The code when it was last saved. */
  saved: string;
  /** The code in the editor, also when another document is shown. */
  code: string;
}

/** The key of the code of a shared link. */
const SHARED_CODE = "shared";

/**
 * The documents of an editor, which are kept for the session, and the shown one of them: the
 * examples, the code of a shared link and the own documents, which are stored by the platform.
 *
 * Only own documents can be changed: saving or renaming an example or shared code copies it
 * into an own document. A `change` event is dispatched after each change, e.g. to show the
 * modified flags.
 *
 * The keys of the documents are the indices of the examples, `shared` for the code of a shared
 * link and `own-1`, `own-2`… for own documents.
 */
export class Documents extends EventTarget {
  private documents = new Map<string, OpenDocument>();
  /** The number of own documents, which have been created, for their keys. */
  private ownDocuments = 0;
  /** The key of the shown document. */
  private shownKey?: string;

  constructor(
    private platform: Platform,
    readonly examples: Example[],
  ) {
    super();
  }

  private changed() {
    this.dispatchEvent(new Event("change"));
  }

  /** Restore the own documents of the platform. */
  async restore() {
    for (const { name, code } of await this.platform.loadDocuments()) {
      const document: OpenDocument = { kind: "own", name, sharedName: name, saved: code, code };
      this.documents.set(`own-${++this.ownDocuments}`, document);
    }
    this.changed();
  }

  /** Store the saved code of the own documents, e.g. after saving or deleting one. */
  private store(): Promise<void> {
    const own: StoredDocument[] = [...this.documents.values()]
      .filter((document) => document.kind === "own")
      .map((document) => ({ name: document.name, code: document.saved }));
    return this.platform.storeDocuments(own);
  }

  /** The document `key`, an example is added at its first use. */
  get(key: string): OpenDocument {
    let document = this.documents.get(key);
    if (!document) {
      const example = this.examples[Number(key)];
      const name = example?.name ?? key;
      const code = example?.code ?? "";
      document = { kind: "example", name, sharedName: name, saved: code, code };
      this.documents.set(key, document);
    }
    return document;
  }

  /** The keys of the documents of `kind`, of examples in their order. */
  keys(kind: DocumentKind): string[] {
    if (kind === "example") return this.examples.map((_, index) => String(index));
    return [...this.documents].filter(([, document]) => document.kind === kind).map(([key]) => key);
  }

  /** The key of the shown document. */
  get shown(): string | undefined {
    return this.shownKey;
  }

  /** The shown document. */
  get shownDocument(): OpenDocument | undefined {
    return this.shownKey === undefined ? undefined : this.get(this.shownKey);
  }

  /** Show the document `key`, with its unsaved changes. */
  show(key: string): OpenDocument {
    this.shownKey = key;
    this.changed();
    return this.get(key);
  }

  /** The code of the shown document changed in the editor. */
  setCode(code: string) {
    const document = this.shownDocument;
    if (!document || document.code === code) return;
    document.code = code;
    this.changed();
  }

  /** Whether `document` has unsaved changes. */
  isModified(document: OpenDocument): boolean {
    return document.code !== document.saved;
  }

  /** Whether any document has unsaved changes, e.g. before the app is closed. */
  hasUnsavedChanges(): boolean {
    return [...this.documents.values()].some((document) => this.isModified(document));
  }

  /** The name of the shown document, e.g. for exported files. */
  get modelName(): string {
    return this.shownDocument?.sharedName ?? "model";
  }

  /** `name` or, if another document than `except` has it, a unique name like `gear (2)`. */
  private uniqueName(name: string, except?: OpenDocument): string {
    const names = new Set(
      [...this.documents.values()].filter((d) => d !== except).map((d) => d.name),
    );
    let unique = name;
    for (let number = 2; names.has(unique); number++) unique = `${name} (${number})`;
    return unique;
  }

  /**
   * Copy the shown example or shared code into a new own document with `name` and show it.
   *
   * The own document has the unsaved changes, the example or shared code keeps its saved code.
   */
  private copyShown(name: string): OpenDocument {
    const source = this.shownDocument!;
    const unique = this.uniqueName(name);
    const key = `own-${++this.ownDocuments}`;
    const document: OpenDocument = {
      kind: "own",
      name: unique,
      sharedName: unique,
      saved: source.saved,
      code: source.code,
    };
    this.documents.set(key, document);
    source.code = source.saved;
    this.shownKey = key;
    return document;
  }

  /**
   * Rename the shown document: an own document is renamed, an example or shared code is copied
   * into a new own document with the name.
   *
   * The name is made unique, e.g. `gear (2)`. Returns whether it is applied, not e.g. an empty
   * name.
   */
  async rename(name: string): Promise<boolean> {
    const document = this.shownDocument;
    name = name.trim();
    if (!document || !name || name === document.name) return false;
    if (document.kind === "own") {
      document.name = document.sharedName = this.uniqueName(name, document);
    } else {
      this.copyShown(name);
    }
    this.changed();
    await this.store();
    return true;
  }

  /**
   * Save `code` as the shown document. An example or shared code is copied into an own
   * document, e.g. `brick` for `brick.mu`.
   */
  async save(code: string) {
    let document = this.shownDocument;
    if (!document) return;
    if (document.kind !== "own") {
      document = this.copyShown((document.sharedName ?? "Untitled").replace(/\.mu$/, ""));
    }
    document.saved = document.code = code;
    this.changed();
    await this.store();
  }

  /** Delete the own document `key`, which may be the shown one. */
  async delete(key: string) {
    this.documents.delete(key);
    this.changed();
    await this.store();
  }

  /**
   * Add the formatted `code` of a shared link with `name` as shared code and return its key,
   * or the key of an own document with the same name and code, e.g. after a reload.
   */
  addShared(code: string, name?: string): string {
    const own = [...this.documents].find(
      ([, document]) =>
        document.kind === "own" && document.sharedName === name && document.saved === code,
    );
    if (own) return own[0];
    // Another shared link replaces the shared code, also its unsaved changes.
    const label = name === undefined ? "Shared code" : `${name} (shared)`;
    const shared: OpenDocument = {
      kind: "shared",
      name: label,
      sharedName: name,
      saved: code,
      code,
    };
    this.documents.set(SHARED_CODE, shared);
    return SHARED_CODE;
  }
}
