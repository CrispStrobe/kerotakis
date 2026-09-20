/**
 * Where a piece of bench equipment meets the bench top.
 *
 * GUI-114. The bench pane already carried a `.work-surface` and a
 * `.vessel-position`, but neither of them drew a surface: `.work-surface`
 * is a bare positioning box, and `.bench` painted its counter as a 2.6 rem
 * lip along the very bottom edge of the scroll area. Everything therefore
 * stood in the transparent band between the wall and that lip — which is
 * the owner's "devices floating not on a desk", exactly.
 *
 * Two numbers fix it. `BENCH_DECK_TOP` is the far edge of the bench top,
 * as a fraction of the work surface's height: above it is wall, below it
 * is counter, and nothing that stands on the bench may be placed above it.
 * `footingY` is where a vessel's own drawing touches that counter.
 *
 * The second one is the honest half. A vessel on its own stands on its
 * glass base. A vessel on a hotplate, a stirrer, a cooling bath or a
 * burner does NOT touch the bench at all — the appliance does, and the
 * glass stands on the appliance. Those tools are the ones `DeployedApparatus`
 * draws with a base under the glass, and their base is the bottom of the
 * vessel's own `0 0 100 140` viewBox. Every other tool either hangs over
 * the glass (a lamp, a burette on its stand), goes into it (electrodes) or
 * replaces its contents (a mortar): those leave the glass standing on its
 * own base, and this module says so rather than guessing a lift.
 */

/** Far edge of the drawn bench top, as a fraction of the work surface. */
export const BENCH_DECK_TOP = 0.24;

/**
 * Tools drawn standing on the bench with the vessel carried on top of
 * them, and the y in the vessel viewBox at which their own base rests.
 * These are the `DeployedApparatus` groups whose base rect reaches 133–134.
 */
export const CARRYING_TOOL_FOOTING: Readonly<Record<string, number>> = {
  heat: 134,
  stir: 134,
  cool: 134,
  bunsen: 133,
};

/** True when the tool carries the vessel rather than standing beside it. */
export function toolCarriesVessel(tool: string | null | undefined): boolean {
  return typeof tool === "string" && tool in CARRYING_TOOL_FOOTING;
}

/**
 * The y, in the vessel's own viewBox, where this stack meets the bench.
 * `glassBottomY` is the glassware's base (`geom.by`); a carrying tool
 * moves the contact down to its own base.
 */
export function footingY(glassBottomY: number, tool: string | null | undefined): number {
  if (typeof tool !== "string") return glassBottomY;
  const carried = CARRYING_TOOL_FOOTING[tool];
  return carried === undefined ? glassBottomY : Math.max(glassBottomY, carried);
}
