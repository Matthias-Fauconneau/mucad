// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

import type { Example } from "./services/documents";

/** The `.mu` files of the repository's `examples` folder, imported as text when building. */
const modules = import.meta.glob<string>("../../../examples/*.mu", {
  eager: true,
  query: "?raw",
  import: "default",
});

/** The examples of the repository's `examples` folder, e.g. `brick.mu`. */
export const examples: Example[] = Object.entries(modules).map(([path, code]) => ({
  name: path.split("/").pop() ?? "unknown",
  code,
}));
