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

// ── The pour a learner is composing ───────────────────────────────────
//
// Everything above is the pour once it is happening: droplets, flight,
// the handoff into the grid. This is the pour before it happens — which
// vessel it comes out of, how much of it, and which vessel may receive
// it — and it lives beside that for one reason: they are the same act,
// and a learner who has just chosen "75%" is looking at the stream that
// number produces.
//
// It used to live as a `{#if transfer}` banner between the top bar and
// the stage: "dekantieren — gieße 25% 50% 75% 100% · von v1 — jetzt das
// Ziel antippen". Every word of that is about two vessels that are drawn
// on the stage, and none of it was near them. The banner also pushed the
// stage down by its own height at the exact moment a learner needed to
// see the stage.
//
// So this is pure: the questions a chooser has to answer, with no view
// attached, so `PourOverlay.svelte` can be moved or replaced without the
// rules moving with it.

import type { TwoVesselAction } from "./directActions";

/** What a learner may pour: quarters, because a slider over a beaker is
 * a precision claim the operator does not honour. */
export const POUR_FRACTIONS: readonly number[] = [0.25, 0.5, 0.75, 1];

export interface TransferDraft {
  verb: TwoVesselAction;
  fraction: number;
  /** The source, once tapped. `null` while the chooser is still asking. */
  from: number | null;
}

/**
 * Whether this verb takes a fraction at all.
 *
 * `filter`, `drain`, `magnet` and `cell` move what they move — the whole
 * residue, the whole magnetic solid — so offering "25%" beside them would
 * be offering a control that changes nothing. Only decanting and
 * distilling read the number.
 */
export function choosesFraction(verb: TwoVesselAction): boolean {
  return verb === "decant" || verb === "distil";
}

/** Which vessel the chooser is anchored to, or `null` for none yet. */
export function anchorVessel(draft: TransferDraft | null): number | null {
  return draft?.from ?? null;
}

/**
 * Keep the chooser inside the bench it is standing on.
 *
 * A vessel may be placed at x = 0.97, and an overlay centred on it would
 * hang half its width over the edge of a work surface that clips its
 * children — the fraction chips would simply not be there. So the anchor
 * is clamped by the overlay's own half-width as a fraction of the
 * surface: the chooser stops travelling before it reaches the edge and
 * the vessel keeps its outline as the thing that says which one it is.
 *
 * The default is the narrowest the bench ever gets: the surface has a
 * `min-width` of 42rem and the chooser a `max-width` of 14rem, so half of
 * it is a sixth of the surface. On a wider bench the clamp is generous
 * rather than wrong — it holds the chooser a little further from the edge
 * than it needs to be, which no learner can see.
 */
export function clampAnchor(x: number, halfWidth = 1 / 6): number {
  if (!Number.isFinite(x)) return 0.5;
  return Math.min(1 - halfWidth, Math.max(halfWidth, x));
}

/**
 * Which side of the vessel the chooser stands on.
 *
 * Above, normally — a pour comes out of the top of the glass and the
 * label belongs where the learner is already looking. But a vessel in the
 * top half of the bench has nothing above it, and an overlay pushed off
 * the surface is clipped away entirely, so the chooser goes below it
 * instead. Half the bench each way, because that is the only boundary
 * that cannot itself be off the surface.
 */
export function anchorSide(y: number): "above" | "below" {
  return Number.isFinite(y) && y < 0.5 ? "below" : "above";
}

/**
 * Which vessels may be tapped next.
 *
 * Before a source is chosen, every vessel is a candidate; after, every
 * vessel except the source. Returned as ids rather than as a predicate so
 * a caller can count them: zero eligible targets means the learner is
 * being asked to tap something that does not exist, and the chooser says
 * so instead of waiting for a tap that can never come.
 */
export function eligibleVessels(
  draft: TransferDraft | null,
  vessels: readonly { id: number }[],
): number[] {
  if (!draft) return [];
  return vessels.map((vessel) => vessel.id).filter((id) => id !== draft.from);
}

/**
 * The one line the chooser says, as a translation key.
 *
 * A key rather than a sentence, because the chooser is a component and
 * the dictionary is the shell's — and because a test can assert on a key
 * without asserting on German.
 */
export function transferPrompt(draft: TransferDraft, eligible: number): string {
  if (draft.from === null) return "tap the source vessel";
  if (eligible === 0) return "add a second vessel to pour into";
  // Not "from v1 — now tap the target": the chooser IS on v1, and a label
  // naming the thing it is attached to is the banner's habit, not a
  // sentence a learner needs twice.
  return "now tap the target";
}
