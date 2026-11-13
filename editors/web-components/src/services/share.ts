// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

import type { Compiler, Preamble } from "../compiler/types";

/** The code and the name of the document of a shared link. */
export interface SharedLink {
  code: string;
  name?: string;
}

/**
 * The link data in `url`: in its fragment, e.g. `#wrUBQUQABENpcmNsZShyID0gMTBtbSk7CgM`, or as
 * query parameter `?code=`, `undefined` without one.
 */
export function linkInUrl(url: URL): string | undefined {
  if (url.hash.length > 1) return url.hash;
  if (url.searchParams.has("code")) return url.search;
  return undefined;
}

/** The preamble of a shared link with the name of its document. */
export function preamble(name?: string): Preamble {
  return name === undefined ? {} : { name };
}

/** The code and the name in the preamble of a shared link. */
export async function decodeLink(compiler: Compiler, link: string): Promise<SharedLink> {
  const { code, preamble } = await compiler.decodeLink(link);
  const { name } = preamble;
  return { code, name: typeof name === "string" ? name : undefined };
}

/** The number of modules of the quiet zone around a QR code. */
const QR_QUIET_ZONE = 4;

/**
 * Draw the modules of a QR code (row by row, 1 for dark) on `canvas`, dark on white with a quiet
 * zone, with at least 512 pixels for a sharp image.
 */
export function drawQrCode(canvas: HTMLCanvasElement, modules: Uint8Array) {
  const width = Math.sqrt(modules.length);
  const size = width + 2 * QR_QUIET_ZONE;
  const scale = Math.max(2, Math.ceil(512 / size));
  canvas.width = canvas.height = size * scale;
  const context = canvas.getContext("2d")!;
  context.fillStyle = "#ffffff";
  context.fillRect(0, 0, canvas.width, canvas.height);
  context.fillStyle = "#000000";
  modules.forEach((dark, index) => {
    if (!dark) return;
    const x = (index % width) + QR_QUIET_ZONE;
    const y = Math.floor(index / width) + QR_QUIET_ZONE;
    context.fillRect(x * scale, y * scale, scale, scale);
  });
}
