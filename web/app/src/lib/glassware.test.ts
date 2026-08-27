import { describe, expect, it } from "vitest";
import { KINDS, depositDisplayHeight, innerFloor, solidLayers, fillHeight, graduationTicks } from "./glassware";

describe("glassware geometry", () => {
  it("rests contents on the floor of the silhouette they are clipped to", () => {
    for (const [kind, g] of Object.entries(KINDS)) {
      expect(`${kind}: ${g.by}`).toBe(`${kind}: ${innerFloor(g.inner)}`);
    }
  });

  it("keeps a full fill inside the glass", () => {
    for (const [kind, g] of Object.entries(KINDS)) {
      const top = g.by - g.fh;
      expect(`${kind} full-fill top: ${top >= 0 && top < g.by}`).toBe(`${kind} full-fill top: true`);
    }
  });

  it("capacity_ml matches fullAtL × 1000 for every kind", () => {
    for (const [kind, g] of Object.entries(KINDS)) {
      expect(`${kind}: ${g.capacity_ml}`).toBe(`${kind}: ${g.fullAtL * 1000}`);
    }
  });
});

describe("volume-true fill profiles (GUI-061)", () => {
  it("profile endpoints: 0→0 and 1→1 for every kind", () => {
    for (const [kind, g] of Object.entries(KINDS)) {
      expect(`${kind} v2h(0)`).toBe(`${kind} v2h(0)`);
      expect(g.volumeToHeight(0)).toBe(0);
      expect(g.volumeToHeight(1)).toBe(1);
      expect(g.heightToVolume(0)).toBe(0);
      expect(g.heightToVolume(1)).toBe(1);
    }
  });

  it("profiles are monotonically increasing", () => {
    for (const [kind, g] of Object.entries(KINDS)) {
      let prev = 0;
      for (let i = 1; i <= 20; i++) {
        const v = i / 20;
        const h = g.volumeToHeight(v);
        expect(h).toBeGreaterThanOrEqual(prev);
        prev = h;
      }
    }
  });

  it("volumeToHeight and heightToVolume are inverses", () => {
    for (const [kind, g] of Object.entries(KINDS)) {
      for (const f of [0, 0.1, 0.25, 0.5, 0.75, 0.9, 1]) {
        const roundTrip = g.heightToVolume(g.volumeToHeight(f));
        expect(roundTrip).toBeCloseTo(f, 6);
      }
    }
  });

  it("50 mL in a 100 mL cylinder reads half (linear)", () => {
    const cyl = KINDS.cylinder!;
    const h = fillHeight(cyl, 0.05);
    expect(h).toBeCloseTo(cyl.fh / 2, 0);
  });

  it("50 mL in a 400 mL beaker reads 1/8 (linear)", () => {
    const bk = KINDS.beaker!;
    const h = fillHeight(bk, 0.05);
    const expected = bk.fh * 0.125;
    expect(h).toBeCloseTo(expected, 0);
  });

  it("flask half-full by volume is more than halfway up (conical)", () => {
    const fl = KINDS.flask!;
    const halfVolH = fillHeight(fl, fl.fullAtL / 2);
    const linearHalf = fl.fh / 2;
    expect(halfVolH).toBeGreaterThan(linearHalf);
  });

  it("flask fill at full capacity equals fh", () => {
    const fl = KINDS.flask!;
    const h = fillHeight(fl, fl.fullAtL);
    expect(h).toBeCloseTo(fl.fh, 1);
  });

  it("crucible half-full is above linear halfway (widening upward)", () => {
    const cr = KINDS.crucible!;
    const halfVolH = fillHeight(cr, cr.fullAtL / 2);
    const linearHalf = cr.fh / 2;
    expect(halfVolH).toBeGreaterThan(linearHalf);
  });

  it("fillHeight returns 0 for zero volume", () => {
    for (const g of Object.values(KINDS)) {
      expect(fillHeight(g, 0)).toBe(0);
    }
  });

  it("fillHeight clamps to fh at or above capacity", () => {
    for (const g of Object.values(KINDS)) {
      expect(fillHeight(g, g.fullAtL)).toBeCloseTo(g.fh, 1);
      expect(fillHeight(g, g.fullAtL * 2)).toBeCloseTo(g.fh, 1);
    }
  });
});

describe("graduation ticks (GUI-061)", () => {
  it("cylinder ticks are at equal volume intervals", () => {
    const cyl = KINDS.cylinder!;
    const ticks = graduationTicks(cyl, 5);
    expect(ticks).toHaveLength(5);
    expect(ticks[0]!.ml).toBe(20);
    expect(ticks[1]!.ml).toBe(40);
    expect(ticks[2]!.ml).toBe(60);
    expect(ticks[3]!.ml).toBe(80);
    expect(ticks[4]!.ml).toBe(100);
  });

  it("cylinder ticks are evenly spaced in y (linear profile)", () => {
    const cyl = KINDS.cylinder!;
    const ticks = graduationTicks(cyl, 5);
    const gaps = ticks.slice(1).map((t, i) => ticks[i]!.y - t.y);
    for (const g of gaps) {
      expect(g).toBeCloseTo(gaps[0]!, 0);
    }
  });

  it("tick y values stay within the vessel body", () => {
    for (const g of Object.values(KINDS)) {
      const ticks = graduationTicks(g, 5);
      for (const t of ticks) {
        expect(t.y).toBeGreaterThanOrEqual(g.by - g.fh - 1);
        expect(t.y).toBeLessThanOrEqual(g.by);
      }
    }
  });
});

describe("settled deposit layers", () => {
  const by = 127;
  const solidH = 18;

  it("tiles the deposit band exactly, whatever the layer count", () => {
    for (const n of [1, 2, 3]) {
      const layers = solidLayers(Array.from({ length: n }, () => 1), solidH, by);
      expect(layers[0]!.y).toBeCloseTo(by - solidH, 10);
      const last = layers[n - 1]!;
      expect(last.y + last.h).toBeCloseTo(by, 10);
      for (let i = 1; i < n; i++) {
        expect(layers[i]!.y).toBeCloseTo(layers[i - 1]!.y + layers[i - 1]!.h, 10);
      }
    }
  });

  it("never hangs a layer below the floor", () => {
    for (const n of [1, 2, 3]) {
      for (const { y, h } of solidLayers(Array.from({ length: n }, () => 1), solidH, by)) {
        expect(y + h).toBeLessThanOrEqual(by + 1e-9);
      }
    }
  });

  it("gives each species its computed share instead of equal bands", () => {
    const [dominant, trace] = solidLayers([0.003, 0.001], solidH, by);
    expect(dominant!.h).toBeCloseTo(13.5);
    expect(trace!.h).toBeCloseTo(4.5);
    expect(trace!.y + trace!.h).toBeCloseTo(by);
  });

  it("scales deposits monotonically from engine volume and caps the display", () => {
    const beaker = KINDS.beaker!;
    const oneDose = depositDisplayHeight(beaker, 0.0005);
    const fourDoses = depositDisplayHeight(beaker, 0.002);
    expect(oneDose).toBeGreaterThan(0);
    expect(fourDoses).toBeGreaterThan(oneDose);
    expect(depositDisplayHeight(beaker, 4)).toBe(beaker.fh * 0.28);
    expect(depositDisplayHeight(beaker, 0)).toBe(0);
  });
});
