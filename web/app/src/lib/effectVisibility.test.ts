/**
 * Does the bench DRAW what the engine computes — or say, once, why not?
 *
 * GUI-128 found one instance by tracing the DOM at 120 ms through a whole
 * catalogue run: the fizz was gated on a scene STATE that was already
 * false, so an open beaker evolved 32 mmol of CO2 and the bench drew
 * nothing. GUI-129 asked whether that was one bug or a class, and it is a
 * class — a sweep of fifteen catalogue entries, one per phenomenon the
 * codex promises, found `equilibrium-can-run-backward` running an
 * esterification on a bench that never moved.
 *
 * A coverage count could not have found either. What it takes is a list of
 * the engine's events and an answer for every one of them, which is what
 * this is. Every `Event` variant must be in exactly one bucket:
 *
 *   - handled in `magnitudes.ts` or `session.svelte.ts` — it becomes a
 *     transient effect the vessel draws;
 *   - `DRAWN_FROM_THE_SCENE` — no transient, but the result is a standing
 *     field of the scene and the vessel draws THAT;
 *   - `REPORTED_NOT_DRAWN` — a claim in words, with the reason;
 *   - `KNOWN_GAPS` — a phenomenon with no picture, recorded with what it
 *     would take. A recorded exception carries its reason; an unrecorded
 *     one fails, which is the shape `overlayStacking.test.ts` uses.
 *
 * The enum is read out of the Rust rather than restated here, because a
 * hand-kept list would have to be remembered at exactly the moment someone
 * adds an event — which is the moment it would be forgotten.
 */
import { describe, expect, it } from "vitest";
import { readFileSync } from "node:fs";
import { join } from "node:path";

const ROOT = join(import.meta.dirname, "../../../..");

/** Every `Event` variant, in the snake_case serde puts on the wire. */
function engineEvents(): string[] {
  const source = readFileSync(join(ROOT, "crates/kerotakis-core/src/ops.rs"), "utf8");
  const body = /pub enum Event \{([\s\S]*?)\n\}\n/.exec(source)?.[1] ?? "";
  const variants = [...body.matchAll(/^ {4}([A-Z][A-Za-z0-9]*)\s*[{(,]/gm)].map((m) => m[1]!);
  return [...new Set(variants.map((v) => v.replace(/(?<!^)(?=[A-Z])/g, "_").toLowerCase()))].sort();
}

/** Events some part of the shell turns into something on the stage. */
function handledEvents(): Set<string> {
  const found = new Set<string>();
  for (const file of ["magnitudes.ts", "session.svelte.ts"]) {
    const source = readFileSync(join(import.meta.dirname, file), "utf8");
    for (const m of source.matchAll(/case\s+"([a-z_0-9]+)"/g)) found.add(m[1]!);
    for (const m of source.matchAll(/event\??\.event\s*===\s*"([a-z_0-9]+)"/g)) found.add(m[1]!);
  }
  return found;
}

/**
 * No transient, but the RESULT is a standing field of the scene, and
 * `Vessel.svelte` draws that. The value names the field, so the claim can
 * be checked rather than believed.
 */
const DRAWN_FROM_THE_SCENE: Readonly<Record<string, string>> = {
  added: "liquid",
  material_added: "bulk_objects",
  browning_changed: "lemon_paper_mark",
  chemiluminescence_observed: "chemiluminescence",
  layers_formed: "layers",
  lemon_paper_browned: "lemon_paper_mark",
  lemon_paper_dried: "lemon_paper_mark",
  lemon_paper_marked: "lemon_paper_mark",
  material_layers_formed: "layers",
  polymer_swelled: "swelling",
  soap_scum_formed: "soap_scum",
  surface_colour_mixed: "surface_colours",
};

/** A claim in words. The value is the reason there is no picture. */
const REPORTED_NOT_DRAWN: Readonly<Record<string, string>> = {
  acid_metal_cell_voltage: "the cell rig is a BenchEffect operation; the volts are a readout",
  collision_withstood: "a claim that nothing broke — there is no picture of an absence",
  inert: "the claim IS that nothing happened",
  inert_in_solvent: "the claim IS that nothing happened",
  // Not a picture, but not silence either: `persistentReadouts.ts` keeps
  // the converted fraction beside the vessel for as long as it is true.
  // A hydrolysis has no look of its own — the liquid is the same liquid.
  enzyme_hydrolysed: "a persistent readout; the reaction has no appearance to draw",
  no_cell: "a refusal: these two metals make no cell",
  not_yet_modeled: "honesty about a boundary, and drawing it would imply a phenomenon",
  object_spill_boundary: "a boundary statement about what a spill model does not cover",
  particles_counted: "the particle view IS the drawing, and it is a separate surface",
  reaction_occurred: "the REAKTION rail (GUI-125) — the equation is the picture",
  sealed_cell: "the cell rig is a BenchEffect operation",
  shelf_stocked: "bench bookkeeping, not chemistry",
  solution_characterized: "provenance: which solver answered, not what happened",
  solution_routed: "provenance, and switchable at that (GUI-127)",
  solver_failed: "honesty about the engine, reported in the log",
  spill_recovered: "the recovery is bookkeeping; the spill itself is drawn",
  stock_exhausted: "a bottle running out is a shelf fact",
  transition_point_read: "a reading taken off a curve",
  vessel_created: "the vessel appearing IS the drawing",
  vessel_opened: "the boundary badge carries it",
  vessel_removed: "the vessel leaving IS the drawing",
};

/**
 * A phenomenon the bench computes and does not picture. Each entry says
 * what it would take, because "no picture yet" is only honest if the next
 * person can see what was intended.
 */
const KNOWN_GAPS: Readonly<Record<string, string>> = {};

describe("every computed event is drawn, or says why not", () => {
  const events = engineEvents();
  const handled = handledEvents();

  it("finds the enum at all, so nothing below is vacuous", () => {
    expect(events.length).toBeGreaterThan(90);
    expect(events).toContain("gas_evolved");
    expect(handled.size).toBeGreaterThan(50);
  });

  it("accounts for every event exactly once", () => {
    const unaccounted = events.filter((event) =>
      !handled.has(event)
      && !(event in DRAWN_FROM_THE_SCENE)
      && !(event in REPORTED_NOT_DRAWN)
      && !(event in KNOWN_GAPS));
    expect(unaccounted).toEqual([]);
  });

  it("puts no event in two buckets at once", () => {
    const doubled = events.filter((event) =>
      [handled.has(event), event in DRAWN_FROM_THE_SCENE,
       event in REPORTED_NOT_DRAWN, event in KNOWN_GAPS]
        .filter(Boolean).length > 1);
    expect(doubled).toEqual([]);
  });

  it("names no event the engine does not have", () => {
    const known = new Set(events);
    const stale = [...Object.keys(DRAWN_FROM_THE_SCENE), ...Object.keys(REPORTED_NOT_DRAWN),
                   ...Object.keys(KNOWN_GAPS)].filter((e) => !known.has(e)).sort();
    // A bucket entry for an event that has been renamed or removed is an
    // excuse still standing for a defect that is gone.
    expect(stale).toEqual([]);
  });

  it("every scene-drawn event names a field the scene actually has", () => {
    const host = readFileSync(join(import.meta.dirname, "host/EngineHost.ts"), "utf8");
    const missing = [...new Set(Object.values(DRAWN_FROM_THE_SCENE))]
      .filter((field) => !new RegExp(`\\b${field}\\??:`).test(host)).sort();
    expect(missing).toEqual([]);
  });

  it("every scene field named here is drawn by the vessel", () => {
    const vessel = readFileSync(join(import.meta.dirname, "components/Vessel.svelte"), "utf8");
    // `added` points at `liquid`, which the vessel draws as the liquid
    // column; the rest are the standing phenomena. A field nobody draws is
    // the same defect wearing the scene's clothes.
    const undrawn = [...new Set(Object.values(DRAWN_FROM_THE_SCENE))]
      .filter((field) => !vessel.includes(`vessel.${field}`)).sort();
    expect(undrawn).toEqual([]);
  });

  it("the two events GUI-129 fixed stay fixed", () => {
    // Both were found by tracing the bench in a browser, not by reading.
    expect(handled.has("org_reacted")).toBe(true);
    expect(handled.has("dissolved_in_solvent")).toBe(true);
  });

  it("the two GUI-129 RECORDED and GUI-131 closed stay closed", () => {
    // These were the gap list, and they were the whole gap list. An event
    // that falls back out of `handled` would be caught by "accounted for
    // exactly once" above only if someone also added it to a bucket —
    // which is exactly the edit this stops being silent.
    expect(handled.has("polymer_heated")).toBe(true);
    expect(handled.has("extracted")).toBe(true);
  });

  it("records what is still missing, and there is nothing", () => {
    // GUI-131 emptied it. The machinery stays, and an empty list is not a
    // dead check: it is the claim, asserted rather than assumed, that
    // every phenomenon this engine computes now reaches the screen or
    // says in one line why it does not.
    expect(Object.keys(KNOWN_GAPS)).toEqual([]);
  });
});
