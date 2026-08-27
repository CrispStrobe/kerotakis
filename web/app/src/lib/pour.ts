/**
 * GUI-065b: the Lagrangian half — pour streams and splash, the one
 * thing a coarse grid does badly and particles do naturally.
 *
 * Droplets exist only ABOVE the liquid surface: ballistic flight under
 * gravity, then a handoff — crossing the surface deposits the droplet's
 * whole mass into the grid's species field and its momentum into the
 * velocity field (the mini-FLIP handoff), optionally throwing brief
 * splash ejecta that fall back and deposit in turn. Mass is conserved
 * across the handoff by construction and pinned by test: what pours in
 * is exactly what the grid receives.
 *
 * Deterministic: the caller passes a seeded `rand` (mulberry32 below)
 * so a pour replays identically — decorative, but never flaky.
 */

import type { VesselSim } from "./fluidScene";

export interface Droplet {
  /** Position in grid-cell coordinates (cell centres, y down). */
  x: number;
  y: number;
  vx: number;
  vy: number;
  /** Which species field the mass belongs to. */
  s: number;
  mass: number;
  /** Splash ejecta are smaller and drawn dimmer. */
  ejecta: boolean;
}

export interface PourState {
  droplets: Droplet[];
  /** Mass still to emit, per emitter tick. */
  remaining: number;
  /** Emitter position as a fraction of the grid width. */
  xFrac: number;
  s: number;
  /** Mass per emitted droplet. */
  dropMass: number;
  /** Total mass handed to the grid so far (the conservation ledger). */
  deposited: number;
}

/** Deterministic PRNG (mulberry32) — replayable decoration. */
export function mulberry32(seed: number): () => number {
  let a = seed >>> 0;
  return () => {
    a = (a + 0x6d2b79f5) | 0;
    let t = Math.imul(a ^ (a >>> 15), 1 | a);
    t = (t + Math.imul(t ^ (t >>> 7), 61 | t)) ^ t;
    return ((t ^ (t >>> 14)) >>> 0) / 4294967296;
  };
}

export function startPour(
  s: number,
  totalMass: number,
  xFrac: number,
  dropMass = 0.12,
): PourState {
  return { droplets: [], remaining: totalMass, xFrac, s, dropMass, deposited: 0 };
}

export function pourDone(p: PourState): boolean {
  return p.remaining <= 0 && p.droplets.length === 0;
}

/**
 * Advance the pour by `dt`: emit, fly, hand off. Droplets below the
 * surface never exist — crossing it converts them into field mass and
 * face momentum in the same tick.
 */
export function stepPour(
  p: PourState,
  sim: VesselSim,
  dt: number,
  rand: () => number,
  gravity = 60,
): void {
  const { grid } = sim;
  const surfaceY = sim.liquidTopRow;

  // Emit: a thin stream, a couple of droplets per tick with jitter.
  const perTick = 2;
  for (let k = 0; k < perTick && p.remaining > 0; k++) {
    const mass = Math.min(p.dropMass, p.remaining);
    p.remaining -= mass;
    p.droplets.push({
      x: p.xFrac * (grid.w - 1) + (rand() - 0.5) * 1.2,
      y: 0.5,
      vx: (rand() - 0.5) * 1.5,
      vy: 6 + rand() * 3,
      s: p.s,
      mass,
      ejecta: false,
    });
  }

  const survivors: Droplet[] = [];
  for (const d of p.droplets) {
    d.vy += gravity * dt;
    d.x += d.vx * dt;
    d.y += d.vy * dt;

    // Walls and floor are absorbing for strays (mass still deposits in
    // the nearest fluid cell — nothing vanishes).
    const crossed = d.y >= surfaceY || d.y >= grid.h - 1;
    if (!crossed) {
      survivors.push(d);
      continue;
    }
    // Handoff: resolve ONE fluid target cell — used for the deposit,
    // the splash sliver, and the momentum — so no path can touch a wall.
    let tx = Math.min(grid.w - 1, Math.max(0, Math.round(d.x)));
    let ty = Math.min(grid.h - 1, Math.max(0, Math.round(d.y)));
    while (ty > 0 && grid.solid[ty * grid.w + tx]) ty--;
    if (grid.solid[ty * grid.w + tx]) {
      // A column of wall (outside the glass): fall back to the surface
      // centre, walking down to fluid if even that is masked —
      // conservation beats geometry for a stray.
      tx = Math.floor(grid.w / 2);
      ty = surfaceY;
      while (ty < grid.h - 1 && grid.solid[ty * grid.w + tx]) ty++;
    }
    const ti = ty * grid.w + tx;
    grid.fields[d.s]![ti] = grid.fields[d.s]![ti]! + d.mass;
    const vi = Math.min(grid.h, ty + 1) * grid.w + tx;
    if (vi < grid.v.length && !grid.solid[ti]) grid.v[vi] = grid.v[vi]! + d.vy * 0.15;
    p.deposited += d.mass;

    // Splash: a real droplet (not ejecta) throws up to two short-lived
    // ejecta carrying a sliver of ITS mass — taken back from the SAME
    // cell the deposit landed in, so the ledger stays exact.
    if (!d.ejecta && d.vy > 8) {
      const nEjecta = 1 + Math.floor(rand() * 2);
      for (let e = 0; e < nEjecta; e++) {
        const sliver = d.mass * 0.06;
        p.deposited -= sliver;
        grid.fields[d.s]![ti] = grid.fields[d.s]![ti]! - sliver;
        survivors.push({
          x: tx + (rand() - 0.5),
          y: surfaceY - 0.5,
          vx: (rand() - 0.5) * 8,
          vy: -(4 + rand() * 4),
          s: d.s,
          mass: sliver,
          ejecta: true,
        });
      }
    }
  }
  p.droplets = survivors;
}

/** Total mass currently in flight — with `deposited`, the whole ledger. */
export function inFlight(p: PourState): number {
  return p.droplets.reduce((s, d) => s + d.mass, 0);
}
