import { describe, expect, it } from "vitest";
import { KINDS, innerFloor, solidLayer } from "./glassware";

describe("glassware geometry", () => {
  it("rests contents on the floor of the silhouette they are clipped to", () => {
    // A `by` above the silhouette's floor leaves an unpainted strip under the
    // liquid or deposit; below it, the fill is silently clipped away.
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
});

describe("settled deposit layers", () => {
  const by = 127;
  const solidH = 18;

  it("tiles the deposit band exactly, whatever the layer count", () => {
    for (const n of [1, 2, 3]) {
      const layers = Array.from({ length: n }, (_, i) => solidLayer(i, n, solidH, by));
      // Topmost layer starts at the top of the band...
      expect(layers[0]!.y).toBeCloseTo(by - solidH, 10);
      // ...the bottom layer ends exactly on the floor — no black strip...
      const last = layers[n - 1]!;
      expect(last.y + last.h).toBeCloseTo(by, 10);
      // ...and each layer starts where the one above it ended.
      for (let i = 1; i < n; i++) {
        expect(layers[i]!.y).toBeCloseTo(layers[i - 1]!.y + layers[i - 1]!.h, 10);
      }
    }
  });

  it("never hangs a layer below the floor", () => {
    for (const n of [1, 2, 3]) {
      for (let i = 0; i < n; i++) {
        const { y, h } = solidLayer(i, n, solidH, by);
        expect(y + h).toBeLessThanOrEqual(by + 1e-9);
      }
    }
  });
});
