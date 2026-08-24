/**
 * Types for kerotakis.mjs — the two-wasm bridge. TypeScript picks this up
 * for imports of the .mjs from the app (web/app) and anywhere else.
 */

/** The Emscripten module factory exported by iphreeqc.mjs. */
export type CreateIPhreeqc = (moduleArg?: object) => Promise<unknown>;

export class PhreeqcPool {
  static create(
    createIPhreeqc: CreateIPhreeqc,
    loadDatabase: (filename: string) => Promise<string>,
  ): Promise<PhreeqcPool>;
  /** The synchronous solver hook the Rust bench calls. */
  solve(dbTag: string, input: string): string;
}

export function openLab(
  Lab: new () => unknown,
  opts: {
    createIPhreeqc?: CreateIPhreeqc;
    loadDatabase?: (filename: string) => Promise<string>;
    results?: Uint8Array;
  },
): Promise<unknown>;
