import { readFileSync } from "node:fs";
import { describe, expect, it } from "vitest";
import { render } from "svelte/server";
import type { SceneVessel } from "../host/EngineHost";
import type { Effect } from "../magnitudes";
import Vessel from "./Vessel.svelte";

const vessel: SceneVessel = {
  id: 0,
  label: "beaker",
  liquid: { volume_l: 0.25, srgb: [180, 210, 240], colour_word: "blue", cloudiness: 0, path_length_cm: 4 },
  solids: [],
  bubbling: true,
  foam: { trapped_gas_liters: 0.06, volume_liters: 0.08, height_cm: 4, overflow_liters: 0, srgb: [245, 245, 245], colour_word: "white" },
  boundary: "open",
  temperature_k: 298.15,
  pressure_pa: 101325,
  elapsed_s: 0,
  mass_g: 250,
  words: "a foaming beaker",
  badges: [],
};

function draw(effects: Effect[]): string {
  return render(Vessel, { props: { vessel, register: "lv2", effects } }).body;
}

describe("computed gas and foam motion", () => {
  it("exposes the engine gas rate and its derived visible-bubble cadence", () => {
    const html = draw([{
      kind: "vent", at: Date.now(), magnitude: 0.5,
      gasProduction: { molesPerSecond: 4.1e-5 },
    }]);
    expect(html).toContain('data-gas-rate-mol-s="4.100e-5"');
    expect(html).toContain('data-bubble-period-s="1.00"');
    expect(html).toContain("mol per second");
  });

  it("exposes the engine foam half-life and uses it as the collapse duration", () => {
    const html = draw([{
      kind: "foam", at: Date.now(), durationMs: 12_000, magnitude: 0.5,
      foam: { halfLifeSeconds: 12 },
    }]);
    expect(html).toContain('data-foam-half-life-s="12.00"');
    expect(html).toContain("--foam-half-life:12s");
    expect(html).toContain("half-life 12.0 s");
  });

  /**
   * GUI-128. The foam head was a coloured rectangle with 5-16 cells on a
   * modulo lattice: every foam in the app had the same bubbles in the
   * same places, in rows, and none of them moved. Three things are read
   * off the engine now, and each is asserted against the value it is read
   * from rather than against a number written down here twice.
   */
  describe("the foam head is drawn from the foam", () => {
    const withFoam = (volumeLiters: number, halfLifeSeconds?: number): string =>
      render(Vessel, {
        props: {
          vessel: { ...vessel, foam: { ...vessel.foam!, volume_liters: volumeLiters } },
          register: "lv2",
          effects: halfLifeSeconds === undefined ? [] : [{
            kind: "foam", at: Date.now(), durationMs: 12_000, magnitude: 0.5,
            foam: { halfLifeSeconds },
          }],
        },
      }).body;

    const cells = (html: string): number => (html.match(/class="foam-cell/g) ?? []).length;

    it("puts more bubbles in more foam", () => {
      const thin = cells(withFoam(0.01));
      const thick = cells(withFoam(0.6));
      expect(thin).toBeGreaterThan(0);
      expect(thick).toBeGreaterThan(thin);
    });

    it("breaks the top edge with a crown rather than ruling a line across it", () => {
      expect(withFoam(0.08)).toContain('class="foam-cell crown');
    });

    it("churns at the engine's own half-life, not at a rate of its own", () => {
      // A head that halves in 1.2 s boils; one that halves in 30 s is
      // nearly still. Same number that drives `foam-collapse`, said as
      // motion instead of as height.
      const fast = /--pop-period:([\d.]+)s/.exec(withFoam(0.08, 1.2))?.[1];
      const slow = /--pop-period:([\d.]+)s/.exec(withFoam(0.08, 30))?.[1];
      expect(Number(fast)).toBeGreaterThan(0);
      expect(Number(slow)).toBeGreaterThan(Number(fast));
    });

    it("scatters the bubbles instead of ruling them into rows", () => {
      // The lattice put every cell at `(i * 17) % width`, so the same
      // handful of x positions repeated. A scatter has many.
      const xs = [...withFoam(0.6).matchAll(/class="foam-cell[^"]*"[^>]*?cx="([\d.]+)"/g)]
        .map((m) => m[1]);
      expect(xs.length).toBeGreaterThan(10);
      expect(new Set(xs).size).toBeGreaterThan(xs.length * 0.7);
    });

    it("fills the head rather than ruling a diagonal through it", () => {
      // Caught in a photograph, not in a test. The first draft salted ONE
      // golden-ratio sequence with an additive offset per axis — and an
      // additive offset of a sequence is the same sequence, so x and y
      // were perfectly correlated and every bubble sat on one diagonal
      // band through the middle. A scatter that is a line is a lattice
      // wearing a different hat, which is what it replaced.
      // The head's own cells, NOT the crown: the crown sits in a row of
      // its own along the top, and including it fills buckets the head
      // did not — which is how a first draft of this assertion passed
      // against the very scatter it was written to reject.
      const cells = [...withFoam(0.6).matchAll(
        /class="foam-cell svelte[^"]*"[^>]*?cx="([\d.]+)"[^>]*?cy="([\d.]+)"/g,
      )].map((m) => [Number(m[1]), Number(m[2])] as const);
      expect(cells.length).toBeGreaterThan(20);
      const span = (at: 0 | 1) => {
        const values = cells.map((c) => c[at]);
        return [Math.min(...values), Math.max(...values)] as const;
      };
      const [x0, x1] = span(0);
      const [y0, y1] = span(1);
      const bucket = new Set(cells.map(([x, y]) =>
        `${Math.min(2, Math.floor(((x - x0) / (x1 - x0 || 1)) * 3))},`
        + `${Math.min(2, Math.floor(((y - y0) / (y1 - y0 || 1)) * 3))}`));
      // Measured both ways: the correlated pair reaches 6 of the 9 cells
      // of a 3x3 grid (two diagonal bands), and the R2 pair reaches all
      // 9. The bound is 8, which separates them with a cell to spare.
      expect(bucket.size).toBeGreaterThanOrEqual(8);
    });

    it("draws the same picture twice — the scatter is the index, not a die", () => {
      // Not decoration: a `Math.random()` scatter re-rolls on every
      // reactive redraw and the foam twitches, and no server-rendered
      // test could assert anything about it.
      expect(withFoam(0.3)).toBe(withFoam(0.3));
    });
  });

  /**
   * GUI-128. The seal failing drew eight identical shards at eight fixed
   * angles however hard it failed; only the distance and the ring radius
   * moved with the magnitude.
   */
  describe("a burst is as big as the burst", () => {
    const burst = (magnitude: number): string =>
      render(Vessel, {
        props: { vessel, register: "lv2", effects: [{ kind: "burst", at: Date.now(), magnitude }] },
      }).body;
    const shards = (html: string): number =>
      (html.match(/<path[^>]*--angle:/g) ?? []).length;

    it("throws more shards for a bigger failure", () => {
      expect(shards(burst(0.1))).toBeGreaterThan(0);
      expect(shards(burst(1))).toBeGreaterThan(shards(burst(0.1)));
    });

    it("gives every shard its own reach and spin", () => {
      const html = burst(1);
      expect(new Set([...html.matchAll(/--reach:([\d.]+)/g)].map((m) => m[1])).size)
        .toBeGreaterThan(3);
      expect(new Set([...html.matchAll(/--spin:(\d+)deg/g)].map((m) => m[1])).size)
        .toBeGreaterThan(3);
    });

    it("draws a front with depth and a flash, not one ruled circle", () => {
      const html = burst(0.8);
      expect(html).toContain('class="flash');
      expect(html).toContain('class="second');
    });
  });

  /**
   * GUI-131. `chains-slide-networks-do-not` is ABOUT the difference
   * between two materials at one temperature, and the bench drew the same
   * heated block for both. So what is asserted is the DIFFERENCE, not the
   * presence of a drawing.
   */
  describe("chains slide and networks do not", () => {
    const block = (material: string, over: number, crossLinked: boolean): string =>
      render(Vessel, {
        props: {
          vessel: {
            ...vessel,
            bulk_objects: [{
              material, recipe_id: material.replace(/\s+/g, "-"), amount_g: 20,
              bulk_density_g_per_ml: 0.95, position: "sunk" as const,
              srgb: [200, 190, 175] as [number, number, number],
            }],
          },
          register: "lv2",
          effects: [{
            kind: "polymer", at: Date.now(), durationMs: 4200, magnitude: 0.6,
            polymer: {
              state: crossLinked ? "rigid" as const : (over > 0 ? "softened" as const : "rigid" as const),
              material, crossLinked, reversible: !crossLinked && over > 0,
              temperatureK: 420 + over, thresholdK: 420, overK: over,
            },
          }],
        },
      }).body;

    it("draws the structure, because the structure is the reason", () => {
      expect(block("thermoplastic sheet", 60, false)).toContain("polymer-strand");
    });

    it("a network gets its cross-links; loose chains do not", () => {
      // The cross-links ARE the explanation: a thing tied to itself has
      // nothing to slide.
      expect(block("cured thermoset resin", 60, true)).toContain("polymer-link");
      expect(block("thermoplastic sheet", 60, false)).not.toContain("polymer-link");
    });

    it("the same heat draws two different pictures", () => {
      // The one assertion this whole item exists for.
      expect(block("cured thermoset resin", 60, true))
        .not.toBe(block("thermoplastic sheet", 60, false));
    });

    it("only the chains get the sliding class", () => {
      expect(block("thermoplastic sheet", 60, false)).toContain("softened");
      expect(block("cured thermoset resin", 60, true)).not.toContain("softened");
    });

    it("says what happened in words as well as in line work", () => {
      // The drawing is not the only reader of this bench.
      expect(block("thermoplastic sheet", 60, false)).toMatch(/aria-label="[^"]*480/);
    });

    it("draws for a vessel whose polymer is a SOLID, not a bulk object", () => {
      // The first draft anchored this to `bulk_objects` and covered
      // exactly one of the two materials: the engine files the thermoset
      // as a bulk object and the thermoplastic as a solid, so the one
      // that actually softens drew nothing. Caught in a browser — the
      // scene reported `scene-solid: 1, bulk-object: 0` — and not by any
      // test, which is why this one exists.
      const html = render(Vessel, {
        props: {
          vessel: { ...vessel, bulk_objects: [] },
          register: "lv2",
          effects: [{
            kind: "polymer", at: Date.now(), magnitude: 0.6,
            polymer: {
              state: "softened" as const, material: "thermoplastic sheet",
              crossLinked: false, reversible: true,
              temperatureK: 480, thresholdK: 420, overK: 60,
            },
          }],
        },
      }).body;
      expect(html).toContain("polymer-strand");
      expect(html).toContain("softened");
    });

    it("draws nothing when no polymer was heated", () => {
      expect(render(Vessel, { props: { vessel, register: "lv2", effects: [] } }).body)
        .not.toContain("polymer-strand");
    });
  });

  it("stops both computed motions when reduced motion is requested", () => {
    const source = readFileSync(new URL("./Vessel.svelte", import.meta.url), "utf8");
    const reduced = source.slice(source.indexOf("@media (prefers-reduced-motion: reduce)"));
    expect(reduced).toMatch(/\.bubble,[\s\S]*\.foam-state,[\s\S]*animation: none/);
  });
});
