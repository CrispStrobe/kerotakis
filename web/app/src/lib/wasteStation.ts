/**
 * What the disposal station's confirmed press means.
 *
 * The station had exactly one meaning before the engine had a verb for it:
 * empty the whole bench. `discard vN` (kerotakis-core::ops::Operator::Discard)
 * moves one vessel's condensed portions into the bench's shared waste
 * container, weighed and logged, so the press means that vessel whenever
 * there is a vessel holding something to mean.
 *
 * The decision is a function rather than a branch inside the mount, because
 * the station's own wording is chosen by the same fact: a station that says
 * "empty v2" and a press that clears the bench would be two different
 * answers to one question, and only one of them would be visible.
 */

/** The slice of the scene this decision reads. */
export type DisposableVessel = { id: number; mass_g: number };

export type WasteStationAction =
  /** Hand this line to the engine, exactly as typed at the command bar. */
  | { kind: "discard"; command: string }
  /** No vessel to empty: the older meaning, the whole bench. */
  | { kind: "clear" };

/** Below this a vessel is empty for display purposes — the same floor the
 *  remove-vessel dialog uses to decide a vessel can be taken off the bench. */
export const DISPOSABLE_MASS_G = 1e-9;

/** Whether the station can offer the selected vessel to the container. */
export function isDisposable(vessel: DisposableVessel | null | undefined): boolean {
  return (vessel?.mass_g ?? 0) > DISPOSABLE_MASS_G;
}

/**
 * The engine command, or the bench-clear fallback.
 *
 * A sealed vessel is deliberately still a `discard`: the engine refuses it
 * by name and in the reader's language (`vessel-sealed`), and being told
 * that the lid is the problem beats a control that has silently gone grey.
 */
export function wasteStationAction(
  vessel: DisposableVessel | null | undefined,
): WasteStationAction {
  if (!isDisposable(vessel)) return { kind: "clear" };
  // Vessels are addressed from 1 at the command bar and from 0 in the scene.
  return { kind: "discard", command: `discard v${vessel!.id + 1}` };
}
