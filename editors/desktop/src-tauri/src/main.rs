// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! µcad desktop app: the µcad editor of the web viewer in a native window, see
//! `editors/web-components`, with native dialogs and files.

// No additional console window on Windows in release builds.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use percent_encoding::percent_decode_str;
use std::path::Path;
use tauri::ipc::{InvokeBody, Request};
use tauri_plugin_dialog::DialogExt;

/// Save the data in the body of the request as file, the user chooses the path in a save
/// dialog with the URI-encoded name of the header `name`, e.g. `brick.stl`.
///
/// Returns whether the file is saved, not after canceling the dialog.
#[tauri::command]
async fn save_file(app: tauri::AppHandle, request: Request<'_>) -> Result<bool, String> {
    let InvokeBody::Raw(data) = request.body() else {
        return Err("The data of the file is missing".into());
    };
    let name = request
        .headers()
        .get("name")
        .and_then(|name| name.to_str().ok())
        .map(|name| percent_decode_str(name).decode_utf8_lossy().into_owned())
        .unwrap_or_default();

    let mut dialog = app.dialog().file().set_file_name(&name);
    if let Some(extension) = Path::new(&name).extension().and_then(|e| e.to_str()) {
        dialog = dialog.add_filter(extension.to_uppercase(), &[extension]);
    }
    // The dialog blocks this thread of the async runtime, not the main thread.
    let Some(path) = dialog.blocking_save_file() else {
        return Ok(false);
    };
    let path = path.into_path().map_err(|error| error.to_string())?;
    std::fs::write(&path, data).map_err(|error| format!("{}: {error}", path.display()))?;
    Ok(true)
}

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![save_file])
        .run(tauri::generate_context!())
        .expect("Cannot run the µcad desktop app");
}
