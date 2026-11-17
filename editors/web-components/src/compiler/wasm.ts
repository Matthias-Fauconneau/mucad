// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

import init, {
  Mucad,
  Severity as WasmSeverity,
  type Diagnostic as WasmDiagnostic,
  type Mesh,
  type Sketch,
} from "mucad-wasm";

import {
  type BuildResult,
  type Color,
  type Compiler,
  CompilerError,
  type CompilerSession,
  type Completion,
  type Diagnostic,
  type ExportFormat,
  type FormatResult,
  type Geometry,
  type LiteralRange,
  type MeshData,
  type Preamble,
  type Severity,
  type SharedCode,
  type SketchData,
  type SteppedLiteral,
} from "./types";

/** The WebAssembly module of the page, which is loaded once, e.g. for the compiler and viewers. */
let wasmModule: Promise<void> | undefined;

/** Load the WebAssembly module at the first call, e.g. for the compiler or the camera controls. */
export function loadWasm(): Promise<void> {
  wasmModule ??= init().then(() => undefined);
  return wasmModule;
}

/** The compiler of the page, which is loaded once, e.g. for several viewers. */
let compiler: Promise<Compiler> | undefined;

/**
 * The µcad compiler in WebAssembly, which is loaded at the first call.
 *
 * All sessions share one instance of the compiler, whose calls are queued.
 */
export function wasmCompiler(): Promise<Compiler> {
  compiler ??= loadWasm().then(() => new WasmCompiler());
  return compiler;
}

/** The severities of the diagnostics of WebAssembly. */
const SEVERITIES: Record<WasmSeverity, Severity> = {
  [WasmSeverity.Error]: "error",
  [WasmSeverity.Warning]: "warning",
  [WasmSeverity.Advice]: "advice",
};

/** The result of `call`, with errors of the compiler, e.g. strings, as `CompilerError`. */
function call<T>(call: () => T): T {
  try {
    return call();
  } catch (error) {
    throw error instanceof Error ? error : new CompilerError(String(error));
  }
}

/** A color of WebAssembly. */
function color(color: Float32Array | number[] | undefined): Color | undefined {
  return color ? [color[0], color[1], color[2], color[3]] : undefined;
}

/** A mesh of WebAssembly as plain data, the mesh is freed. */
function meshData(mesh: Mesh): MeshData {
  const data = {
    positions: mesh.positions,
    indices: mesh.indices,
    edges: mesh.edges(),
    color: color(mesh.color),
  };
  mesh.free();
  return data;
}

/** A sketch of WebAssembly as plain data, the sketch is freed. */
function sketchData(sketch: Sketch): SketchData {
  const triangles = sketch.triangles;
  const lines = sketch.lines;
  const data = {
    triangles: { positions: triangles.positions, indices: triangles.indices },
    lines: new Float32Array(lines.positions),
    color: color(sketch.color),
  };
  triangles.free();
  lines.free();
  sketch.free();
  return data;
}

/** A diagnostic of WebAssembly as plain data, the diagnostic is freed. */
function diagnostic(diagnostic: WasmDiagnostic): Diagnostic {
  const data = {
    message: diagnostic.message,
    severity: SEVERITIES[diagnostic.severity],
    startLineNo: diagnostic.startLineNo,
    startColumn: diagnostic.startColumn,
    endLineNo: diagnostic.endLineNo,
    endColumn: diagnostic.endColumn,
    deprecated: diagnostic.deprecated,
  };
  diagnostic.free();
  return data;
}

/**
 * The compiler in WebAssembly with one instance of `Mucad`, which keeps the model of the last
 * compilation, so that a session, whose model it is not, compiles its code again, e.g. to
 * export it.
 */
class WasmCompiler implements Compiler {
  readonly version = Mucad.versionString();
  private mucad = new Mucad();
  /** The calls of the sessions, which are run one after another. */
  private queue: Promise<unknown> = Promise.resolve();
  /** The session of the last compilation, whose model the instance has. */
  private owner?: WasmSession;

  session(): CompilerSession {
    return new WasmSession(this);
  }

  /** Run `task` after the calls before it. */
  run<T>(task: () => T): Promise<T> {
    const result = this.queue.then(() => call(task));
    this.queue = result.catch(() => undefined);
    return result;
  }

  /** Compile and render `code` of `session` at once. */
  build(session: WasmSession, code: string): BuildResult {
    const result = this.mucad.compile(code);
    this.owner = session;
    session.code = code;
    const built: BuildResult = {
      success: result.success,
      report: result.report,
      diagnostics: result.diagnostics.map(diagnostic),
      geometry: { kind: "none" },
    };
    const modelType = result.success ? result.modelType : undefined;
    result.free();
    try {
      built.geometry = this.render(modelType);
    } catch (error) {
      built.report += String(error);
      built.success = false;
    }
    return built;
  }

  private render(modelType: string | undefined): Geometry {
    switch (modelType) {
      case "3d":
        return { kind: "3d", meshes: this.mucad.renderMeshes().map(meshData) };
      case "2d":
        return { kind: "2d", sketches: this.mucad.renderSketches().map(sketchData) };
      default:
        return { kind: "none" };
    }
  }

  /** Compile the code of `session` again, if the instance has the model of another one. */
  own(session: WasmSession) {
    if (this.owner === session || session.code === undefined) return;
    this.mucad.compile(session.code).free();
    this.owner = session;
  }

  complete(session: WasmSession, code: string, lineNo: number, column: number): Completion[] {
    this.own(session);
    return this.mucad.complete(code, lineNo, column).map((completion) => {
      const { name, kind, doc, stability } = completion;
      completion.free();
      return { name, kind, doc, stability };
    });
  }

  export(session: WasmSession, format: ExportFormat): Uint8Array | string {
    this.own(session);
    return format === "svg" ? this.mucad.exportSvg() : new Uint8Array(this.mucad.exportStl());
  }

  forget(session: WasmSession) {
    if (this.owner === session) this.owner = undefined;
  }

  format(code: string): Promise<FormatResult> {
    return this.run(() => {
      const result = this.mucad.format(code);
      const { success, formatted, code: text } = result;
      result.free();
      return { success, formatted, code: text };
    });
  }

  literalAt(code: string, lineNo: number, column: number): Promise<LiteralRange | undefined> {
    return this.run(() => {
      const range = Mucad.literalAt(code, lineNo, column);
      if (range === undefined) return undefined;
      const { lineNo: line, startColumn, endColumn } = range;
      range.free();
      return { lineNo: line, startColumn, endColumn };
    });
  }

  stepLiteral(
    literal: string,
    cursor: number,
    factor: number,
  ): Promise<SteppedLiteral | undefined> {
    return this.run(() => {
      const stepped = Mucad.stepLiteralAt(literal, cursor, factor);
      if (stepped === undefined) return undefined;
      const { text, digitStart, digitEnd } = stepped;
      stepped.free();
      return { text, digitStart, digitEnd };
    });
  }

  encodeLink(code: string, preamble: Preamble, strip: boolean): Promise<string> {
    return this.run(() => Mucad.encodeLinkWithPreamble(code, JSON.stringify(preamble), strip));
  }

  decodeLink(link: string): Promise<SharedCode> {
    return this.run(() => {
      const shared = Mucad.decodeLinkWithPreamble(link);
      const { code, preamble } = shared;
      shared.free();
      return { code, preamble: JSON.parse(preamble) as Preamble };
    });
  }

  qrCode(text: string): Promise<Uint8Array> {
    return this.run(() => Mucad.qrCode(text));
  }
}

/** A session of the compiler in WebAssembly. */
class WasmSession implements CompilerSession {
  /** The code of the last build. */
  code?: string;
  /** The number of the latest build, earlier builds are superseded. */
  private latest = 0;

  constructor(private compiler: WasmCompiler) {}

  async build(code: string): Promise<BuildResult | undefined> {
    const build = ++this.latest;
    // A build, which is superseded while it waits in the queue, is not run.
    const result = await this.compiler.run(() =>
      build === this.latest ? this.compiler.build(this, code) : undefined,
    );
    return build === this.latest ? result : undefined;
  }

  complete(code: string, lineNo: number, column: number): Promise<Completion[]> {
    return this.compiler.run(() => this.compiler.complete(this, code, lineNo, column));
  }

  export(format: ExportFormat): Promise<Uint8Array | string> {
    return this.compiler.run(() => this.compiler.export(this, format));
  }

  dispose() {
    this.compiler.forget(this);
  }
}
