// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

/**
 * The µcad compiler for the components, independent of where it runs: in WebAssembly in the
 * browser, or natively, e.g. in the desktop app.
 *
 * All results are plain data, which can also be sent between processes, and all functions are
 * asynchronous.
 */

/** A color with red, green, blue and alpha from 0 to 1. */
export type Color = [number, number, number, number];

/** A triangle mesh: positions of `x, y, z` and three indices of positions per triangle. */
export interface Triangles {
  positions: Float32Array;
  indices: Uint32Array;
}

/** A 3D mesh with the pairs of indices of its sharp edges, e.g. to draw contours. */
export interface MeshData extends Triangles {
  edges: Uint32Array;
  color?: Color;
}

/** A 2D sketch: filled triangles and outlines as pairs of points of `x, y`. */
export interface SketchData {
  triangles: Triangles;
  lines: Float32Array;
  color?: Color;
}

/** The geometry of a model: 3D meshes, 2D sketches or nothing, e.g. after errors. */
export type Geometry =
  | { kind: "3d"; meshes: MeshData[] }
  | { kind: "2d"; sketches: SketchData[] }
  | { kind: "none" };

/** The severity of a diagnostic. */
export type Severity = "error" | "warning" | "advice";

/**
 * An error, warning or advice with its range in the code, e.g. for an editor marker.
 *
 * Lines and columns start at 1, columns are counted in UTF-16 code units like in JavaScript.
 */
export interface Diagnostic {
  message: string;
  severity: Severity;
  startLineNo: number;
  startColumn: number;
  endLineNo: number;
  endColumn: number;
  /** Whether this is the use of a deprecated symbol, which editors show crossed out. */
  deprecated: boolean;
}

/** The result of building code: its diagnostics and the geometry of its model. */
export interface BuildResult {
  /** Whether the code was compiled and rendered without errors. */
  success: boolean;
  /** The diagnostics as text, e.g. for a panel. */
  report: string;
  diagnostics: Diagnostic[];
  geometry: Geometry;
}

/** Formatted code, which is `formatted`, if it differs from the code. */
export interface FormatResult {
  success: boolean;
  formatted: boolean;
  code: string;
}

/** A symbol to complete a path with, e.g. `Circle` for `std::geo2d::Ci`. */
export interface Completion {
  name: string;
  /** e.g. `sketch`, `fn` or `mod`. */
  kind: string;
  /** The documentation in Markdown. */
  doc?: string;
  /** `stable`, `experimental` or `deprecated`. */
  stability: string;
}

/** The range of a literal in a line, e.g. under the cursor. */
export interface LiteralRange {
  lineNo: number;
  startColumn: number;
  endColumn: number;
}

/** A literal after a step, with its stepped digit in UTF-16 code units from its start. */
export interface SteppedLiteral {
  text: string;
  digitStart?: number;
  digitEnd?: number;
}

/** The preamble of a shared link, e.g. the name of the shared document. */
export type Preamble = Record<string, unknown>;

/** The code of a shared link with its preamble. */
export interface SharedCode {
  code: string;
  preamble: Preamble;
}

/** The export formats: STL of 3D models and SVG of 2D models. */
export type ExportFormat = "stl" | "svg";

/** An error of the compiler, e.g. of an invalid link, with the message of the compiler. */
export class CompilerError extends Error {
  override name = "CompilerError";
}

/** The message of an error, e.g. of a rejected call of the compiler. */
export function errorMessage(error: unknown): string {
  return error instanceof Error ? error.message : String(error);
}

/**
 * Compiling of code of an editor or a viewer, which keeps the model of its last build, e.g. to
 * export it or to complete names with its symbols.
 */
export interface CompilerSession {
  /**
   * Compile `code` and render its model.
   *
   * Resolves with `undefined`, if a later build of the session has been started meanwhile, so
   * that only the result of the latest code is shown.
   */
  build(code: string): Promise<BuildResult | undefined>;
  /** The symbols to complete the path at a line and column, of the last build. */
  complete(code: string, lineNo: number, column: number): Promise<Completion[]>;
  /** The model of the last build as STL (`Uint8Array`) or SVG (`string`). */
  export(format: ExportFormat): Promise<Uint8Array | string>;
  /** Release the session, e.g. of a removed viewer. */
  dispose(): void;
}

/** The µcad compiler, which is shared by the sessions of the components. */
export interface Compiler {
  /** The version of µcad, e.g. `0.6.3`. */
  readonly version: string;
  /** A new session, e.g. of an editor or a viewer. */
  session(): CompilerSession;
  format(code: string): Promise<FormatResult>;
  /** The range of the literal at a line and column, e.g. under the cursor. */
  literalAt(code: string, lineNo: number, column: number): Promise<LiteralRange | undefined>;
  /** The literal stepped by `factor` times the place of the digit at `cursor`. */
  stepLiteral(literal: string, cursor: number, factor: number): Promise<SteppedLiteral | undefined>;
  /** The link to the web viewer with `code` and `preamble`, with `strip` without comments. */
  encodeLink(code: string, preamble: Preamble, strip: boolean): Promise<string>;
  /** The code and the preamble of a shared link. */
  decodeLink(link: string): Promise<SharedCode>;
  /** The modules of the QR code of `text`, row by row, 1 for dark. */
  qrCode(text: string): Promise<Uint8Array>;
}
