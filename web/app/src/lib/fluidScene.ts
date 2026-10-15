/**
 * GUI-065a bridge: the engine's scene, translated for the fluid grid.
 *
 * Everything here is pure and testable. The overlay component owns the
 * canvas and the clock; this module owns the meaning: which rows of the
 * grid each computed layer occupies (the relax target — the honesty
 * gate's input), how a vessel's contents seed the fields, and how
 * per-species concentrations become pixels.
 */

import { fillHeight, KINDS } from "./glassware";
import type { SceneLayer, SceneVessel } from "./host/EngineHost";
import { layerTarget, makeGrid, type FluidGrid } from "./fluid";

export interface FluidSpecies {
  key: string;
  srgb: [number, number, number];
  /** Relative density vs the ambient bath (1 = neutral). */
  density: number;
  /** Solver/provenance-owned dynamic viscosity in Pa s, when known. */
  dynamicViscosityPaS?: number;
  /** Solver/provenance-owned surface tension in N/m, when known. */
  surfaceTensionNM?: number;
}

export interface FluidVisualPlan {
  animate: boolean;
  /** Velocity retained per lightweight solver step. */
  damping: number;
  /** Particle size is presentation only; transferred mass remains authoritative. */
  dropMass: number;
  acceptedMass: number;
}

/**
 * BRD-072 lightweight visual adapter. Physical parameters alter only the
 * presentation between accepted scene states. Reduced motion creates no
 * particle run at all, and transfer mass comes solely from the accepted event.
 */
export function fluidVisualPlan(
  species: FluidSpecies[],
  acceptedTransferFraction: number | undefined,
  reducedMotion: boolean,
  fullVisualMass = 1.5,
): FluidVisualPlan {
  const fraction = acceptedTransferFraction === undefined
    ? 1
    : Number.isFinite(acceptedTransferFraction) && acceptedTransferFraction >= 0 && acceptedTransferFraction <= 1
      ? acceptedTransferFraction
      : 0;
  const viscosity = Math.max(0.000_001, ...species.map((item) => item.dynamicViscosityPaS ?? 0.001));
  const tension = Math.max(0.001, ...species.map((item) => item.surfaceTensionNM ?? 0.072));
  // Water (~1 mPa s) remains lively; syrup-like values rapidly dissipate.
  const damping = Math.max(0.72, Math.min(0.96, 0.96 - Math.log10(Math.max(1, viscosity / 0.001)) * 0.08));
  // Higher surface tension forms larger coherent droplets. This changes only
  // particle count: startPour still conserves acceptedMass exactly.
  const dropMass = Math.max(0.06, Math.min(0.24, 0.12 * Math.sqrt(tension / 0.072)));
  return {
    animate: !reducedMotion && fraction > 0,
    damping,
    dropMass,
    acceptedMass: fullVisualMass * fraction,
  };
}

/** The sim bound to one vessel: the grid plus what its fields mean. */
export interface VesselSim {
  grid: FluidGrid;
  species: FluidSpecies[];
  /** Relax targets per species field, rebuilt when the scene changes. */
  targets: Float32Array[];
  /** Rows (top, bottom) the whole liquid occupies in the grid. */
  liquidTopRow: number;
  /** Last row occupied by liquid, above the SVG vessel bottom. */
  liquidBottomRow: number;
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
  if (!v.liquid || !Number.isFinite(v.liquid.volume_l) || v.liquid.volume_l <= 0
    || !v.layers || v.layers.length === 0) return null;
  const grid = makeGrid(w, h, dx, v.layers.length);
  if (solidMask) grid.solid.set(solidMask);

  // This grid spans the same 0..140 SVG viewBox as Vessel.svelte and
  // glassMask. Seed only the rows inside the actual static liquid body.
  const geometry = KINDS[v.label] ?? KINDS.beaker!;
  const topY = geometry.by - fillHeight(geometry, v.liquid.volume_l);
  const liquidTopRow = Math.max(0, Math.ceil(topY * h / 140 - 0.5));
  const liquidBottomRow = Math.min(h - 1, Math.ceil(geometry.by * h / 140 - 0.5) - 1);
  if (liquidTopRow > liquidBottomRow) return null;
  // Air and the region below the vessel are outside this fixed liquid
  // domain. In-flight droplets are rendered separately above its surface.
  for (let y = 0; y < h; y++) {
    if (y < liquidTopRow || y > liquidBottomRow) grid.solid.fill(1, y * w, (y + 1) * w);
  }

  if (!grid.solid.some((cell) => cell === 0)) return null;
  const bands = layerBands(v.layers, liquidTopRow, liquidBottomRow + 1);
  // A layer can be a computed mixture named "solution", not a shelf item.
  // Keep lookup transport properties, but colour belongs to the scene.
  const species = v.layers.map((layer) => ({
    ...lookup(layer.species), key: layer.species, srgb: [...layer.srgb] as [number, number, number],
  }));
  const targets = v.layers.map((_, s) =>
    layerTarget(grid, s, bands[s]!.top, bands[s]!.bottom),
  );
  for (let s = 0; s < targets.length; s++) {
    grid.fields[s]!.set(targets[s]!);
  }
  return { grid, species, targets, liquidTopRow, liquidBottomRow };
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
  const cy = sim.liquidTopRow;
  const radius = 2;
  for (let y = cy; y <= Math.min(sim.liquidBottomRow, cy + radius); y++) {
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
  const midTop = sim.liquidTopRow + Math.floor((sim.liquidBottomRow - sim.liquidTopRow + 1) / 3);
  const midBottom = sim.liquidTopRow + Math.floor((2 * (sim.liquidBottomRow - sim.liquidTopRow + 1)) / 3);
  for (let y = sim.liquidTopRow; y <= sim.liquidBottomRow; y++) {
    const dir = y < midTop ? 1 : y > midBottom ? -1 : 0;
    if (dir === 0) continue;
    for (let x = 0; x <= grid.w; x++) {
      const i = y * (grid.w + 1) + x;
      grid.u[i] = grid.u[i]! + dir * strength;
    }
  }
}
