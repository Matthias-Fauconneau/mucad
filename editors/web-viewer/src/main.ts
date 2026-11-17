// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

import { startApp, webPlatform } from "mucad-web-components";
import "./style.css";

// Shared links in the address bar, e.g. `https://play.mucad.org/#wrUBQUQABENp…`.
await startApp(document.body, { platform: webPlatform, urlLinks: true });
