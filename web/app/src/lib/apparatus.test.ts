import { describe, expect, it } from "vitest";
import { APPARATUS } from "./apparatus";

const spec = (verb: string) => APPARATUS.find((s) => s.verb === verb)!;
const defaults = (verb: string) =>
  Object.fromEntries(spec(verb).fields.map((f) => [f.name, f.default]));

describe("apparatus forms compile to the grammar", () => {
  it("every spec's defaults build a line starting with its verb", () => {
    for (const s of APPARATUS) {
      const values = defaults(s.verb);
      if (s.verb === "grind") values.species = "NaCl"; // species has no default
      const line = s.build(0, values);
      expect(line, s.verb).not.toBeNull();
      expect(line!.startsWith(`${s.verb} v1 `), line!).toBe(true);
    }
  });

  it("the exact lines match the grammar's shapes", () => {
    expect(spec("dilute").build(1, { volume: 250 })).toBe("dilute v2 250mL");
    expect(spec("evaporate").build(0, { fraction: 0.5 })).toBe("evaporate v1 0.5");
    expect(spec("electrolyse").build(0, { amps: 0.5, minutes: 30 })).toBe(
      "electrolyse v1 0.5A 30min",
    );
    expect(spec("grind").build(0, { species: "CaCO3", diameter: 50 })).toBe(
      "grind v1 CaCO3 50um",
    );
    expect(spec("irradiate").build(0, { wavelength: 254, irradiance: 10 })).toBe(
      "irradiate v1 254nm 10W/m2",
    );
    expect(spec("regulate").build(0, { pressure: 1.5, volume: 500 })).toBe(
      "regulate v1 1.5bar 500mL",
    );
    expect(spec("sweep").build(0, { pressure: 1 })).toBe("sweep v1 1bar");
  });

  it("nonsense is refused, never emitted", () => {
    expect(spec("dilute").build(0, { volume: -5 })).toBeNull();
    expect(spec("evaporate").build(0, { fraction: 1.5 })).toBeNull();
    expect(spec("grind").build(0, { species: "  ", diameter: 50 })).toBeNull();
    expect(spec("electrolyse").build(0, { amps: NaN, minutes: 30 })).toBeNull();
  });
});
