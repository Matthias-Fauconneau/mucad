// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

import { invoke } from "@tauri-apps/api/core";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { writeText } from "@tauri-apps/plugin-clipboard-manager";
import { ask } from "@tauri-apps/plugin-dialog";
import { type Platform, webPlatform } from "mucad-web-components";

/** Ask the user with a native dialog. */
function confirm(message: string): Promise<boolean> {
  return ask(message, { title: "µcad", kind: "warning" });
}

/**
 * The platform of the desktop app: the settings and the own documents in the local storage of
 * the web view like in a browser, native dialogs, the clipboard of the system and files saved
 * by the Rust command `save_file`.
 */
export const desktopPlatform: Platform = {
  ...webPlatform,

  /** The user chooses the path in a save dialog, the file is not saved after canceling it. */
  async saveFile(name, data) {
    await invoke("save_file", new Uint8Array(await data.arrayBuffer()), {
      // Header values are ASCII.
      headers: { name: encodeURIComponent(name) },
    });
  },

  copyText: writeText,

  confirm,

  confirmClose(hasUnsavedChanges) {
    void getCurrentWindow().onCloseRequested(async (event) => {
      if (!hasUnsavedChanges()) return;
      if (!(await confirm("Close µcad? The unsaved changes will be lost."))) event.preventDefault();
    });
  },
};
