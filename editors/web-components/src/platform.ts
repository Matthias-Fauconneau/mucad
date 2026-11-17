// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

/**
 * What differs between the platforms of the components, e.g. the browser and the desktop app:
 * settings, storing documents, saving files, the clipboard and dialogs.
 */

/** An own document in a storage: its name and its saved code, no unsaved changes. */
export interface StoredDocument {
  name: string;
  code: string;
}

/** The platform of the components. */
export interface Platform {
  /** A setting, e.g. whether to compile after each change, `undefined` if it is not set. */
  setting(key: string): string | undefined;
  /** Change a setting. */
  setSetting(key: string, value: string): void;
  /** The stored own documents. */
  loadDocuments(): Promise<StoredDocument[]>;
  /** Store the own documents, e.g. after saving one. */
  storeDocuments(documents: StoredDocument[]): Promise<void>;
  /** Save `data` as file `name`, e.g. an exported model. */
  saveFile(name: string, data: Blob): Promise<void>;
  /** Copy `text` to the clipboard. */
  copyText(text: string): Promise<void>;
  /** Ask the user to confirm `message`, e.g. to delete a document. */
  confirm(message: string): Promise<boolean>;
  /** Ask before the app is closed or reloaded, while `hasUnsavedChanges` is true. */
  confirmClose(hasUnsavedChanges: () => boolean): void;
}

/** The prefix of the keys of the local storage. */
const PREFIX = "mucad.";

/** The local storage key of the own documents. */
const DOCUMENTS_KEY = `${PREFIX}documents`;

/** The platform of a browser: the local storage, downloads and the dialogs of the browser. */
export const webPlatform: Platform = {
  setting(key) {
    return localStorage.getItem(PREFIX + key) ?? undefined;
  },

  setSetting(key, value) {
    localStorage.setItem(PREFIX + key, value);
  },

  async loadDocuments() {
    return JSON.parse(localStorage.getItem(DOCUMENTS_KEY) ?? "[]") as StoredDocument[];
  },

  /** The browser is asked once to keep the storage, also when it runs out of space. */
  async storeDocuments(documents) {
    localStorage.setItem(DOCUMENTS_KEY, JSON.stringify(documents));
    if (documents.length) navigator.storage?.persist?.();
  },

  async saveFile(name, data) {
    const link = document.createElement("a");
    link.href = URL.createObjectURL(data);
    link.download = name;
    link.click();
    URL.revokeObjectURL(link.href);
  },

  copyText(text) {
    return navigator.clipboard.writeText(text);
  },

  async confirm(message) {
    return window.confirm(message);
  },

  /** The browser asks with its own dialog, whose text cannot be changed. */
  confirmClose(hasUnsavedChanges) {
    window.addEventListener("beforeunload", (event) => {
      if (hasUnsavedChanges()) event.preventDefault();
    });
  },
};
