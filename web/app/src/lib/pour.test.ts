import { describe, expect, it } from "vitest";
import type { SceneVessel } from "./host/EngineHost";
import { fieldTotal } from "./fluid";
import { simFromScene } from "./fluidScene";
import {
  POUR_FRACTIONS,
  anchorSide,
  anchorVessel,
  clampAnchor,
  choosesFraction,
  eligibleVessels,
  inFlight,
  mulberry32,
  pourDone,
  startPour,
  stepPour,
  transferPrompt,
  type TransferDraft,
} from "./pour";

const LOOKUP = (key: string) => ({
  key,
  srgb: [80, 60, 200] as [number, number, number],
  density: 1,
});

const sim = () =>
  simFromScene(
    {
      id: 0,
      label: "beaker",
      liquid: { volume_l: 0.1, srgb: [200, 200, 255], colour_word: "pale", cloudiness: 0, path_length_cm: 4 },
      layers: [{ species: "water", name: "water", volume_l: 0.1, srgb: [200, 200, 255], colour_word: "pale" }],
      solids: [],
      bubbling: false,
      boundary: "open",
      temperature_k: 298,
      pressure_pa: 101325,
      elapsed_s: 0,
      words: "",
      badges: [],
    } as unknown as SceneVessel,
    20,
    40,
    1,
    LOOKUP,
  )!;

describe("the pour hands off exactly what it emits", () => {
  it("mulberry32 is deterministic", () => {
    const a = mulberry32(7);
    const b = mulberry32(7);
    for (let i = 0; i < 10; i++) expect(a()).toBe(b());
  });

  it("THE LEDGER: emitted mass = deposited + in flight, and all lands eventually", () => {
    const v = sim();
    const rand = mulberry32(42);
    const before = fieldTotal(v.grid, 0);
    const p = startPour(0, 3.0, 0.5);
    let guard = 0;
    while (!pourDone(p) && guard++ < 2000) {
      stepPour(p, v, 0.03, rand);
      // Invariant at every tick, not just the end.
      expect(p.deposited + inFlight(p) + p.remaining).toBeCloseTo(3.0, 6);
    }
    expect(pourDone(p)).toBe(true);
    expect(p.deposited).toBeCloseTo(3.0, 6);
    expect(fieldTotal(v.grid, 0) - before).toBeCloseTo(3.0, 4);
  });

  it("droplets never rest below the surface — they convert on crossing", () => {
    const v = sim();
    const rand = mulberry32(9);
    const p = startPour(0, 1.0, 0.5);
    for (let k = 0; k < 300 && !pourDone(p); k++) {
      stepPour(p, v, 0.03, rand);
      for (const d of p.droplets) {
        // Ejecta may sit fractionally at the surface on their way out.
        expect(d.y).toBeLessThanOrEqual(v.liquidTopRow + 0.5);
      }
    }
  });

  it("a hard landing splashes: ejecta appear and then land too", () => {
    const v = sim();
    const rand = mulberry32(3);
    const p = startPour(0, 1.5, 0.5);
    let sawEjecta = false;
    let guard = 0;
    while (!pourDone(p) && guard++ < 2000) {
      stepPour(p, v, 0.03, rand);
      if (p.droplets.some((d) => d.ejecta)) sawEjecta = true;
    }
    expect(sawEjecta).toBe(true);
    expect(p.deposited).toBeCloseTo(1.5, 6);
  });

  it("mass never lands in a wall cell", () => {
    const v = sim();
    // Wall off the left third BEFORE seeding would matter: rebuild the
    // seeded fields to respect the mask, as simFromScene(solidMask) does.
    for (let y = 0; y < v.grid.h; y++) {
      for (let x = 0; x < 6; x++) {
        const i = y * v.grid.w + x;
        v.grid.solid[i] = 1;
        for (const f of v.grid.fields) f[i] = 0;
        v.targets.forEach((t) => (t[i] = 0));
      }
    }
    const rand = mulberry32(11);
    const p = startPour(0, 2.0, 0.1); // aimed at the walled side
    let guard = 0;
    while (!pourDone(p) && guard++ < 2000) stepPour(p, v, 0.03, rand);
    for (let i = 0; i < v.grid.fields[0]!.length; i++) {
      if (v.grid.solid[i]) expect(v.grid.fields[0]![i]).toBe(0);
    }
    expect(p.deposited).toBeCloseTo(2.0, 6);
  });
});


/**
 * GUI-473 — the pour a learner is composing, before any droplet exists.
 *
 * The chooser used to be a banner above the stage and the rules were
 * written inline in `App.svelte`'s markup: which verbs take a fraction was
 * `verb === "decant" || verb === "distil"` inside an `{#if}`, and which
 * vessels could be tapped was implied by a `!==` in a prop. Both are now
 * answerable without rendering anything, which is the only reason a
 * chooser can move onto the stage without the rules moving with it.
 */
describe("the pour being composed", () => {
  const draft = (over: Partial<TransferDraft> = {}): TransferDraft =>
    ({ verb: "decant", fraction: 0.5, from: null, ...over });

  it("offers quarters, and only quarters", () => {
    expect([...POUR_FRACTIONS]).toEqual([0.25, 0.5, 0.75, 1]);
    // Every one is a fraction the engine's `decant` grammar accepts.
    expect(POUR_FRACTIONS.every((f) => f > 0 && f <= 1)).toBe(true);
  });

  it("asks how much only for the verbs that read the number", () => {
    expect(choosesFraction("decant")).toBe(true);
    expect(choosesFraction("distil")).toBe(true);
    // Filtering moves the residue, magnets move the magnetic solid: a
    // fraction chip beside these would be a control that changes nothing.
    for (const verb of ["filter", "drain", "magnet", "cell"] as const) {
      expect(choosesFraction(verb), verb).toBe(false);
    }
  });

  it("anchors the chooser to the source, and nowhere until there is one", () => {
    expect(anchorVessel(null)).toBe(null);
    expect(anchorVessel(draft())).toBe(null);
    expect(anchorVessel(draft({ from: 2 }))).toBe(2);
  });

  it("makes every vessel but the source eligible", () => {
    const vessels = [{ id: 0 }, { id: 1 }, { id: 2 }];
    expect(eligibleVessels(null, vessels)).toEqual([]);
    expect(eligibleVessels(draft(), vessels)).toEqual([0, 1, 2]);
    expect(eligibleVessels(draft({ from: 1 }), vessels)).toEqual([0, 2]);
  });

  it("never offers the source as its own target", () => {
    expect(eligibleVessels(draft({ from: 0 }), [{ id: 0 }])).toEqual([]);
  });

  it("says which tap it is waiting for", () => {
    expect(transferPrompt(draft(), 3)).toBe("tap the source vessel");
    expect(transferPrompt(draft({ from: 0 }), 2)).toBe("now tap the target");
  });

  it("says so rather than waiting for a tap that can never come", () => {
    // One vessel on the bench, chosen as the source: there is nothing left
    // to pour into, and "now tap the target" would be a lie.
    expect(transferPrompt(draft({ from: 0 }), 0)).toBe("add a second vessel to pour into");
  });

  it("keeps the chooser inside the bench, whatever the vessel is standing on", () => {
    // The surface clips its children, so an overlay centred on a vessel at
    // the edge loses the half of itself that matters — the chips.
    expect(clampAnchor(0.5)).toBeCloseTo(0.5);
    expect(clampAnchor(0.97)).toBeCloseTo(1 - 1 / 6);
    expect(clampAnchor(0.01)).toBeCloseTo(1 / 6);
    expect(clampAnchor(Number.NaN)).toBe(0.5);
    for (const x of [0, 0.2, 0.5, 0.8, 1]) {
      const at = clampAnchor(x);
      expect(at).toBeGreaterThanOrEqual(1 / 6);
      expect(at).toBeLessThanOrEqual(1 - 1 / 6);
    }
  });

  it("stands below a vessel that has nothing above it", () => {
    expect(anchorSide(0.1)).toBe("below");
    expect(anchorSide(0.49)).toBe("below");
    expect(anchorSide(0.5)).toBe("above");
    expect(anchorSide(0.95)).toBe("above");
  });

  it("keeps the prompt a key, not a sentence", () => {
    // The chooser is a component; the dictionary is the shell's. A prompt
    // that came back translated could not be asserted on in one language.
    for (const eligible of [0, 1, 5]) {
      for (const from of [null, 0]) {
        expect(transferPrompt(draft({ from }), eligible)).toMatch(/^[a-z]/);
      }
    }
  });
});
