import type { ParticleCensus } from "./host/EngineHost";

/** What a particle view has to say about a census.
 *
 * Extracted from the component so it can be tested: there is no component
 * test harness here, and the bug this guards against was the component
 * rendering *nothing*, which is the hardest thing to notice in a review
 * and the easiest to notice on a tablet.
 *
 * "particles" draws them. The other two are both "nothing to draw", and
 * they mean opposite things — an empty vessel, versus a vessel whose
 * every population is below one glyph — so a reader who is told the wrong
 * one is misled about their own chemistry.
 */
export type CensusView = "particles" | "too-dilute" | "empty";

export function censusView(census: ParticleCensus | undefined | null): CensusView {
  if (!census || census.populations.length === 0) {
    // `too_rare` present with no populations means everything IS there,
    // just below the scale — a different fact from an empty beaker.
    return census && census.too_rare.length > 0 ? "too-dilute" : "empty";
  }
  return "particles";
}
