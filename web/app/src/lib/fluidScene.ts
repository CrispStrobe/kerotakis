/**
 * GUI-065a bridge: the engine's scene, translated for the fluid grid.
 *
 * Everything here is pure and testable. The overlay component owns the
 * canvas and the clock; this module owns the meaning: which rows of the
 * grid each computed layer occupies (the relax target — the honesty
 * gate's input), how a vessel's contents seed the fields, and how
 * per-species concentrations become pixels.
 */

import type { SceneLayer, SceneVessel } from "./host/EngineHost";
import { layerTarget, makeGrid, type FluidGrid } from "./fluid";

export interface FluidSpecies {
  key: string;
  srgb: [number, number, number];
  /** Relative density vs the ambient bath (1 = neutral). */
  density: number;
}

/** The sim bound to one vessel: the grid plus what its fields mean. */
export interface VesselSim {
  grid: FluidGrid;
  species: FluidSpecies[];
  /** Relax targets per species field, rebuilt when the scene changes. */
  targets: Float32Array[];
  /** Rows (top, bottom) the whole liquid occupies in the grid. */
  liquidTopRow: number;
}

/**
 * The layers' row bands on a grid of `h` rows whose liquid fills rows
 * `topRow..h-1`: bottom-first volumes become bands proportionally, the
 * same arithmetic the SVG stack uses — the sim and the static render
 * agree by construction.
 */
export function layerBands(
  layers: { volume_l: number }[],
  topRow: number,
  h: number,
): { top: number; bottom: number }[] {
  const total = layers.reduce((s, l) => s + l.volume_l, 0);
  const rows = h - topRow;
  if (total <= 0 || rows <= 0) return layers.map(() => ({ top: h, bottom: h - 1 }));
  const bands: { top: number; bottom: number }[] = [];
  let bottom = h - 1;
  for (const layer of layers) {
    const band = (layer.volume_l / total) * rows;
    const top = Math.max(topRow, Math.round(bottom - band + 1));
    bands.push({ top, bottom });
    bottom = top - 1;
  }
  // Bands are bottom-first (matching SceneLayer order).
  return bands;
}

/**
 * Build (or rebuild) a vessel's sim from its scene: one field per
 * layer-species, seeded AT the settled bands, targets equal to those
 * bands. A fresh sim therefore starts already settled — motion only
 * enters through injections (add, stir), and relaxation brings it home.
 */
export function simFromScene(
  v: SceneVessel,
  w: number,
  h: number,
  dx: number,
  lookup: (species: string) => FluidSpecies,
  solidMask?: Uint8Array,
): VesselSim | null {
  if (!v.liquid || !v.layers || v.layers.length === 0) return null;
  const grid = makeGrid(w, h, dx, v.layers.length);
  if (solidMask) grid.solid.set(solidMask);

  // The liquid's share of the vessel height mirrors the SVG's fill
  // fraction: volume relative to the drawn-full volume is the caller's
  // business; here the liquid simply owns the bottom `fillRows`.
  const fillFraction = Math.min(1, Math.max(0.1, v.liquid.volume_l > 0 ? 1 : 0));
  const liquidTopRow = Math.max(0, h - Math.round(h * fillFraction));

  const bands = layerBands(v.layers, liquidTopRow, h);
  const species = v.layers.map((l) => lookup(l.species));
  const targets = v.layers.map((_, s) =>
    layerTarget(grid, s, bands[s]!.top, bands[s]!.bottom),
  );
  for (let s = 0; s < targets.length; s++) {
    grid.fields[s]!.set(targets[s]!);
  }
  return { grid, species, targets, liquidTopRow };
}

/**
 * Concentrations → pixels. Each cell mixes its species' colours
 * weighted by concentration; an empty cell is transparent. Alpha rises
 * with how much substance is present, so a thinning plume genuinely
 * fades. Writes RGBA into `out` (w*h*4).
 */
export function paint(
  sim: VesselSim,
  out: Uint8ClampedArray,
  baseAlpha = 200,
): void {
  const { grid, species } = sim;
  const n = grid.w * grid.h;
  for (let i = 0; i < n; i++) {
    const o = i * 4;
    if (grid.solid[i]) {
      out[o + 3] = 0;
      continue;
    }
    let r = 0;
    let g = 0;
    let b = 0;
    let c = 0;
    for (let s = 0; s < grid.fields.length; s++) {
      const w = grid.fields[s]![i]!;
      if (w <= 0) continue;
      r += species[s]!.srgb[0] * w;
      g += species[s]!.srgb[1] * w;
      b += species[s]!.srgb[2] * w;
      c += w;
    }
    if (c <= 0.001) {
      out[o + 3] = 0;
      continue;
    }
    out[o] = r / c;
    out[o + 1] = g / c;
    out[o + 2] = b / c;
    out[o + 3] = Math.min(255, baseAlpha * Math.min(1, c));
  }
}

/**
 * Inject an addition: a blob of species `s` enters at the surface,
 * with initial downward or upward drift by its density. This is what
 * the sim does when the bench says "add" — the plume's path is the
 * simulation's, its destination the solver's.
 */
export function injectAt(
  sim: VesselSim,
  s: number,
  xFrac: number,
  amount: number,
): void {
  const { grid } = sim;
  const cx = Math.round(xFrac * (grid.w - 1));
  const cy = Math.max(sim.liquidTopRow, 1);
  const radius = 2;
  for (let y = cy; y <= Math.min(grid.h - 1, cy + radius); y++) {
    for (let x = Math.max(0, cx - radius); x <= Math.min(grid.w - 1, cx + radius); x++) {
      const i = y * grid.w + x;
      if (!grid.solid[i]) grid.fields[s]![i] = grid.fields[s]![i]! + amount;
    }
  }
  // Entry momentum: falling in.
  const vi = (cy + 1) * grid.w + cx;
  if (vi < sim.grid.v.length) grid.v[vi] = grid.v[vi]! + 3;
}

/** Inject a stir: a horizontal shear band mid-liquid that the
 * projection turns into a vortex. */
export function injectStir(sim: VesselSim, strength: number): void {
  const { grid } = sim;
  const midTop = sim.liquidTopRow + Math.floor((grid.h - sim.liquidTopRow) / 3);
  const midBottom = sim.liquidTopRow + Math.floor((2 * (grid.h - sim.liquidTopRow)) / 3);
  for (let y = sim.liquidTopRow; y < grid.h; y++) {
    const dir = y < midTop ? 1 : y > midBottom ? -1 : 0;
    if (dir === 0) continue;
    for (let x = 0; x <= grid.w; x++) {
      const i = y * (grid.w + 1) + x;
      grid.u[i] = grid.u[i]! + dir * strength;
    }
  }
}
