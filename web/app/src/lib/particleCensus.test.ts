/** A particle view that draws nothing must say so.
 *
 * Reported from an iPad: "clicking Teilchen does nothing". It did exactly
 * what it was written to do — iterate `census.populations` and draw each
 * one — over an empty array. No error, no message, no change on screen. A
 * dead button is what the reader concludes, and they are not wrong to.
 *
 * A beaker of plain water lands here too, which is the easy way to hit
 * it: `census` filters H2O out, so water alone has no populations. That
 * is very likely the first thing anyone tries.
 */
import { describe, expect, it } from "vitest";
import { censusView } from "./particleCensus";
import type { ParticleCensus } from "./host/EngineHost";

const census = (over: Partial<ParticleCensus> = {}): ParticleCensus =>
  ({
    populations: [],
    per_glyph: 1e-3,
    too_rare: [],
    source: "inventory",
    ...over,
  }) as ParticleCensus;

describe("censusView", () => {
  it("draws particles when there are some", () => {
    const c = census({
      populations: [{ label: "Na+", kind: "cation", drawn: 4 }],
    } as Partial<ParticleCensus>);
    expect(censusView(c)).toBe("particles");
  });

  it("says the vessel is empty rather than rendering nothing", () => {
    // The engine asserts this shape itself, in
    // an_empty_vessel_draws_nothing_and_does_not_panic.
    expect(censusView(census())).toBe("empty");
  });

  it("distinguishes too-dilute from empty", () => {
    // Opposite facts: everything IS there, below one glyph. Telling a
    // learner "nothing to draw" here would be a claim about their
    // chemistry, and a false one.
    expect(censusView(census({ too_rare: [["Ag+", 1e-9]] }))).toBe("too-dilute");
  });

  it("treats a water-only vessel as empty, since H2O is filtered out", () => {
    expect(censusView(census({ populations: [] }))).toBe("empty");
  });

  it("survives a census the host never sent", () => {
    expect(censusView(undefined)).toBe("empty");
    expect(censusView(null)).toBe("empty");
  });
});
