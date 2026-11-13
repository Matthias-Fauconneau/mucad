// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

import { openUrl } from "@tauri-apps/plugin-opener";
import { startApp } from "mucad-web-components";
import { desktopPlatform } from "./platform";
import "./style.css";

// Links, e.g. to report a bug, are opened in the browser instead of the window of the app.
document.addEventListener("click", (event) => {
  const link = (event.target as Element).closest<HTMLAnchorElement>("a[href^='http']");
  if (!link) return;
  event.preventDefault();
  void openUrl(link.href);
});

await startApp(document.body, { platform: desktopPlatform });
