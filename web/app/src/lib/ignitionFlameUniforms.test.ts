import { describe, expect, it } from "vitest";
import { ignitionFlameUniforms } from "./ignitionFlameUniforms";
import type { Effect } from "./magnitudes";

const ignite = (magnitude = 0.5, flameColour?: string): Effect => ({
  kind: "ignite",
  at: 100,
  magnitude,
  flameColour,
});

describe("ignition flame uniform authority boundary", () => {
  it.each([undefined, null, { ...ignite(0.8), kind: "flame_test" }, { ...ignite(0.8), kind: "heat" }])(
    "disables absent and non-ignite effects (case %#)",
    (effect) => {
      const uniforms = ignitionFlameUniforms({ effect, vesselIdentity: 4 });
      expect(uniforms.active).toBe(false);
      expect(uniforms.intensity).toBe(0);
    },
  );

  it("activates only authoritative ignite and disables reduced motion", () => {
    expect(ignitionFlameUniforms({ effect: ignite(), vesselIdentity: 4 }).active).toBe(true);
    expect(ignitionFlameUniforms({ effect: ignite(), vesselIdentity: 4, reducedMotion: true })).toMatchObject({
      active: false,
      intensity: 0,
    });
  });

  it.each([
    [Number.NEGATIVE_INFINITY, 0],
    [Number.NaN, 0],
    [-1, 0],
    [0, 0],
    [0.25, 0.25],
    [0.75, 0.75],
    [1, 1],
    [2, 1],
    [Number.POSITIVE_INFINITY, 0],
  ])("maps magnitude %s to finite bounded intensity %s", (magnitude, expected) => {
    const result = ignitionFlameUniforms({ effect: ignite(magnitude), vesselIdentity: 1 });
    expect(result.intensity).toBe(expected);
    expect(Number.isFinite(result.intensity)).toBe(true);
    expect(result.intensity).toBeGreaterThanOrEqual(0);
    expect(result.intensity).toBeLessThanOrEqual(1);
  });

  it("is monotone across the authoritative bounded magnitude range", () => {
    const values = [0, 0.1, 0.5, 0.9, 1].map((magnitude) =>
      ignitionFlameUniforms({ effect: ignite(magnitude), vesselIdentity: 1 }).intensity,
    );
    expect(values).toEqual([...values].sort((a, b) => a - b));
  });

  it.each([
    ["#c8a2c8", [200 / 255, 162 / 255, 200 / 255]],
    ["#9b30ff", [155 / 255, 48 / 255, 1]],
    ["#ffd700", [1, 215 / 255, 0]],
    ["#ff8c00", [1, 140 / 255, 0]],
    ["#cb4154", [203 / 255, 65 / 255, 84 / 255]],
    ["#ff2400", [1, 36 / 255, 0]],
    ["#00e676", [0, 230 / 255, 118 / 255]],
    ["#0dbf8c", [13 / 255, 191 / 255, 140 / 255]],
    ["#1e90ff", [30 / 255, 144 / 255, 1]],
    ["#dc143c", [220 / 255, 20 / 255, 60 / 255]],
    ["#ffffff", [1, 1, 1]],
  ] as const)("maps curated colour %s exactly", (colour, expected) => {
    expect(ignitionFlameUniforms({ effect: ignite(0.5, colour), vesselIdentity: 1 }).colour).toEqual(expected);
  });

  it.each([undefined, "", "chartreuse", "#000000"])("uses a safe deterministic fallback for %s", (colour) => {
    const result = ignitionFlameUniforms({ effect: ignite(0.5, colour), vesselIdentity: 1 });
    expect(result.colour).toEqual([1, 140 / 255, 0]);
    expect(result.colour.every((channel) => Number.isFinite(channel) && channel >= 0 && channel <= 1)).toBe(true);
  });

  it("derives a deterministic, finite, bounded seed only from vessel identity", () => {
    const first = ignitionFlameUniforms({ effect: ignite(0.1, "#ffffff"), vesselIdentity: "v4" });
    const repeated = ignitionFlameUniforms({ effect: ignite(0.9, "#1e90ff"), vesselIdentity: "v4" });
    const other = ignitionFlameUniforms({ effect: ignite(0.1), vesselIdentity: "v5" });
    expect(repeated.seed).toBe(first.seed);
    expect(other.seed).not.toBe(first.seed);
    expect(Number.isFinite(first.seed)).toBe(true);
    expect(first.seed).toBeGreaterThanOrEqual(0);
    expect(first.seed).toBeLessThanOrEqual(1);
  });
});
