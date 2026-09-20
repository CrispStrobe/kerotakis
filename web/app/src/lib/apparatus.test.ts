import { describe, expect, it } from "vitest";
import { readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { APPARATUS, BUNSEN_MAX_KJ, ENERGY_ENTRY, HEAT_SOURCES, heatSource } from "./apparatus";

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
      expect(line!.startsWith(`${s.commandVerb ?? s.verb} v1 `), line!).toBe(true);
    }
  });

  it("the exact lines match the grammar's shapes", () => {
    expect(spec("bunsen").build(0, { flame: 50, seconds: 30 })).toBe("heat v1 7.5kJ on burner");
    expect(spec("bunsen").build(0, { flame: 50, air: 0, seconds: 30 })).toBe("heat v1 4.125kJ on burner");
    expect(spec("bunsen").build(0, { flame: 50, air: 100, seconds: 30 })).toBe("heat v1 7.5kJ on burner");
    expect(spec("bunsen").secondary!.build(1, { flame: 50, seconds: 30 })).toBe("ignite v2");
    expect(spec("stir").build(0, { rpm: 600, seconds: 30 })).toBe("stir v1 600rpm 30s");
    expect(spec("heat").build(0, { watts: 250, seconds: 30 })).toBe("heat v1 7500J on hotplate");
    expect(spec("cool").build(0, { watts: 100, seconds: 30 })).toBe("cool v1 3000J");
    expect(spec("centrifuge").build(0, { rpm: 3000, seconds: 60, radius: 8, counterbalance: 100 })).toBe(
      "centrifuge v1 3000rpm 60s 8cm 100g",
    );
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
    expect(spec("bunsen").build(0, { flame: 101, seconds: 30 })).toBeNull();
    expect(spec("bunsen").build(0, { flame: 0, air: 70, seconds: 30 })).toBeNull();
    expect(spec("bunsen").build(0, { flame: 50, air: 101, seconds: 30 })).toBeNull();
    expect(spec("bunsen").secondary!.build(0, { flame: 0, air: 70 })).toBeNull();
    expect(spec("bunsen").build(0, { flame: 50, seconds: 0 })).toBeNull();
  });

  it("blocks an unsafe centrifuge imbalance", () => {
    expect(spec("centrifuge").warning?.({ sampleMass: 100, counterbalance: 99 })).toContain("out of balance");
    expect(spec("centrifuge").warning?.({ sampleMass: 100, counterbalance: 99.95 })).toBeNull();
  });

  it("previews physical consequences from the same settings sent to the engine", () => {
    expect(spec("stir").readouts?.({ rpm: 500, seconds: 10 })).toEqual([
      { label: "stir-bar tip speed", value: Math.PI * 0.025 * 500 / 60, unit: "m/s", digits: 3 },
    ]);
    expect(spec("heat").readouts?.({ watts: 250, seconds: 30 })).toEqual([
      { label: "delivered energy", value: 7.5, unit: "kJ", digits: 2 },
      { label: "plate ceiling", value: 550, unit: "°C", digits: 0 },
    ]);
    expect(spec("cool").readouts?.({ watts: 100, seconds: 30 })).toEqual([
      { label: "removed energy", value: 3, unit: "kJ", digits: 2 },
    ]);
    expect(spec("bunsen").readouts?.({ flame: 50, air: 0, seconds: 30 })).toEqual([
      { label: "delivered energy", value: 4.125, unit: "kJ", digits: 3 },
      { label: "flame ceiling", value: 1500, unit: "°C", digits: 0 },
    ]);
    // The picker is visible before the run, not only in the temperature
    // that comes back: switching to the candle lowers the stated ceiling.
    expect(spec("bunsen").readouts?.({ flame: 50, air: 0, seconds: 30, source: "candle" })).toEqual([
      { label: "delivered energy", value: 4.125, unit: "kJ", digits: 3 },
      { label: "flame ceiling", value: 1400, unit: "°C", digits: 0 },
    ]);
    const [centrifuge] = spec("centrifuge").readouts?.({ rpm: 3000, radius: 8 }) ?? [];
    expect(centrifuge).toMatchObject({ label: "relative centrifugal force", unit: "× g", digits: 0 });
    expect(centrifuge?.value).toBeCloseTo(805.136, 3);
    expect(spec("electrolyse").readouts?.({ amps: 0.5, minutes: 30 })).toEqual([
      { label: "electrical charge", value: 900, unit: "C", digits: 0 },
      { label: "electron amount", value: 900 / 96_485.332_12, unit: "mol e⁻", digits: 5 },
    ]);
    const [photon] = spec("irradiate").readouts?.({ wavelength: 254, irradiance: 10 }) ?? [];
    expect(photon).toMatchObject({ label: "photon energy", unit: "eV", digits: 3 });
    expect(photon?.value).toBeCloseTo(4.881, 3);
  });

  /**
   * GUI: every heating apparatus names what is under the vessel.
   *
   * The engine caps a vessel at its heat source, and a `heat` line that
   * omits `on <source>` means "the bench default, a laboratory burner".
   * That default was silently claiming a burner for the hotplate and for
   * the kids' candle — both of which reach hundreds of degrees less. The
   * fix is a clause, so the check is that the clause is actually there.
   */
  describe("a heating apparatus names its own flame", () => {
    const heating = APPARATUS.filter((s) => (s.commandVerb ?? s.verb) === "heat");

    it("finds both heaters, so the sweep below is not vacuous", () => {
      expect(heating.map((s) => s.verb).sort()).toEqual(["bunsen", "heat"]);
    });

    it("compiles a source into every heat command it can build", () => {
      const named = HEAT_SOURCES.map((source) => source.value);
      for (const s of heating) {
        const values = defaults(s.verb);
        const line = s.build(0, values)!;
        const [clause, source] = line.split(" ").slice(-2);
        expect(clause, line).toBe("on");
        // The token has to be one the engine's `HeatSource::by_name`
        // answers to; an unknown word is a refused command, not a default.
        expect(named, line).toContain(source);
      }
    });

    it("sends the hotplate's own ceiling, never the burner's", () => {
      expect(spec("heat").build(0, { watts: 100, seconds: 10 })).toBe("heat v1 1000J on hotplate");
      expect(heatSource("hotplate").ceilingC).toBe(550);
    });

    it("sends a candle when the flame is a candle", () => {
      expect(spec("bunsen").build(0, { flame: 50, seconds: 30, source: "candle" }))
        .toBe("heat v1 7.5kJ on candle");
      expect(heatSource("candle").ceilingC).toBe(1400);
    });

    it("falls back to the burner for a source it does not know", () => {
      // A save written before the picker existed carries no source, and a
      // hand-edited one could carry anything. Both land on the engine's
      // own default rather than on a command the bench would refuse.
      expect(heatSource(undefined).value).toBe("burner");
      expect(heatSource("blowtorch").value).toBe("burner");
      expect(spec("bunsen").build(0, { flame: 50, seconds: 30, source: "blowtorch" }))
        .toBe("heat v1 7.5kJ on burner");
    });

    it("offers the flame as a choice the panel can render", () => {
      const field = spec("bunsen").fields.find((f) => f.name === "source")!;
      expect(field.type).toBe("choice");
      expect(field.default).toBe("burner");
      expect(field.options?.map((o) => o.value)).toEqual(["burner", "candle"]);
    });
  });

  /**
   * GUI-113 — the owner: "we should be able to directly change, for
   * Erhitzen, what amount of 'zugeführte Energie' we add".
   *
   * The panel already PRINTED that number, computed from flame power,
   * collar and exposure. Here it can be the input instead, and the three
   * things that can go wrong with a typed number each say so in words.
   */
  describe("the flame's energy, typed", () => {
    const typed = (values: Record<string, number | string>) => ({
      ...defaults("bunsen"),
      entry: ENERGY_ENTRY.typed,
      ...values,
    });

    it("has both entry modes as a choice the panel can render", () => {
      const field = spec("bunsen").fields.find((f) => f.name === "entry")!;
      expect(field.type).toBe("choice");
      expect(field.default).toBe(ENERGY_ENTRY.derived);
      expect(field.options?.map((o) => o.value)).toEqual([ENERGY_ENTRY.derived, ENERGY_ENTRY.typed]);
    });

    it("hides each mode's controls from the other, and hides nothing by default", () => {
      const fields = spec("bunsen").fields;
      const shown = (values: Record<string, number | string>) =>
        fields.filter((f) => f.when?.(values) ?? true).map((f) => f.name);
      expect(shown(defaults("bunsen"))).toEqual(["entry", "flame", "air", "seconds", "source"]);
      expect(shown(typed({}))).toEqual(["entry", "energy", "source"]);
    });

    it("sends the number the reader wrote, not one derived from a flame", () => {
      expect(spec("bunsen").build(0, typed({ energy: 40 }))).toBe("heat v1 40kJ on burner");
      expect(spec("bunsen").build(0, typed({ energy: 0.25, source: "candle" })))
        .toBe("heat v1 0.25kJ on candle");
    });

    /**
     * The reader of this app is German and writes 7,5. A `type="number"`
     * input parses against the BROWSER's locale, so on an English-locale
     * Chrome that value arrives as the empty string; the field is text and
     * the parsing is here, which is why this test can exist at all.
     */
    it("reads a German decimal comma", () => {
      expect(spec("bunsen").build(0, typed({ energy: "7,5" }))).toBe("heat v1 7.5kJ on burner");
      expect(spec("bunsen").build(0, typed({ energy: "7.5" }))).toBe("heat v1 7.5kJ on burner");
      expect(spec("bunsen").warning?.(typed({ energy: "7,5" }))).toBeNull();
    });

    it("reads a lone comma as the decimal point, and refuses a second separator", () => {
      // `1,234` is 1.234 kJ and not 1234 kJ. Safe to decide rather than
      // ask only because this field's ceiling is 150: there is nothing
      // here for a thousands separator to group.
      expect(spec("bunsen").build(0, typed({ energy: "1,234" }))).toBe("heat v1 1.234kJ on burner");
      expect(spec("bunsen").build(0, typed({ energy: "1.2,3" }))).toBeNull();
      expect(spec("bunsen").build(0, typed({ energy: "1,2,3" }))).toBeNull();
      expect(spec("bunsen").build(0, typed({ energy: "ganz viel" }))).toBeNull();
    });

    it("says which thing is wrong rather than greying the button in silence", () => {
      expect(spec("bunsen").warning?.(typed({ energy: "ganz viel" })))
        .toBe("that is not a number — write it as 7.5 or 7,5");
      expect(spec("bunsen").warning?.(typed({ energy: 0 })))
        .toBe("an energy of nothing heats nothing — write a number above zero");
      expect(spec("bunsen").warning?.(typed({ energy: BUNSEN_MAX_KJ + 1 })))
        .toContain("more than this flame can deliver");
      // An empty field is not yet a mistake: it is somebody mid-typing.
      expect(spec("bunsen").warning?.(typed({ energy: "" }))).toBeNull();
      expect(spec("bunsen").build(0, typed({ energy: "" }))).toBeNull();
      // And the derived mode is never scolded about a field it does not show.
      expect(spec("bunsen").warning?.({ ...defaults("bunsen"), energy: "nonsense" })).toBeNull();
    });

    it("judges a typed energy against this panel's own bounds, not a round number", () => {
      // The ceiling IS the derived path at full flame, open collar, the
      // longest exposure the exposure field takes — so it cannot drift
      // away from the fields it is the limit of.
      const flatOut = { ...defaults("bunsen"), flame: 100, air: 100, seconds: 300 };
      expect(spec("bunsen").build(0, flatOut)).toBe(`heat v1 ${BUNSEN_MAX_KJ}kJ on burner`);
      expect(spec("bunsen").build(0, typed({ energy: BUNSEN_MAX_KJ }))).not.toBeNull();
      expect(spec("bunsen").build(0, typed({ energy: BUNSEN_MAX_KJ + 0.001 }))).toBeNull();
      // The sentence quotes the number; a test, so the two cannot part.
      expect(spec("bunsen").warning?.(typed({ energy: 10_000 })))
        .toContain(`${BUNSEN_MAX_KJ} kJ`);
    });

    it("keeps the flame touch available, because igniting carries no energy", () => {
      expect(spec("bunsen").secondary!.build(0, typed({ energy: 40 }))).toBe("ignite v1");
    });

    it("echoes the ceiling and not a delivered-energy readout it did not compute", () => {
      expect(spec("bunsen").readouts?.(typed({ energy: 40 }))).toEqual([
        { label: "flame ceiling", value: 1500, unit: "°C", digits: 0 },
      ]);
    });

    /** I18N: a label with no catalogue row shows the reader English. */
    it("names every new control in German", () => {
      const DIR = join(dirname(fileURLToPath(import.meta.url)), "../locales");
      const de = JSON.parse(readFileSync(join(DIR, "de.json"), "utf8")).messages as Record<string, string>;
      const template = JSON.parse(readFileSync(join(DIR, "_template.json"), "utf8")).messages as Record<string, string>;
      const strings = [
        ...spec("bunsen").fields.flatMap((f) => [f.label, ...(f.options ?? []).map((o) => o.label)]),
        spec("bunsen").warning!({ ...defaults("bunsen"), entry: ENERGY_ENTRY.typed, energy: "x" })!,
        spec("bunsen").warning!({ ...defaults("bunsen"), entry: ENERGY_ENTRY.typed, energy: 0 })!,
        spec("bunsen").warning!({ ...defaults("bunsen"), entry: ENERGY_ENTRY.typed, energy: 1e6 })!,
      ];
      expect(strings.length).toBeGreaterThan(8);
      for (const text of strings) {
        expect(de[text], text).toBeTruthy();
        // A language added later must find the row waiting, empty.
        expect(Object.hasOwn(template, text), text).toBe(true);
      }
    });
  });
});
