import { afterEach, describe, expect, it } from "vitest";
import { readFileSync } from "node:fs";
import { render } from "svelte/server";
import type { SceneVessel } from "../host/EngineHost";
import { i18n } from "../i18n.svelte";
import Vessel from "./Vessel.svelte";

const base: SceneVessel = {
  id: 2, label: "beaker",
  liquid: { volume_l: 0.25, srgb: [120, 150, 210], colour_word: "blue", cloudiness: 0, path_length_cm: 4 },
  solids: [], bubbling: false, boundary: "open", temperature_k: 298.15,
  pressure_pa: 101325, elapsed_s: 0, mass_g: 250,
  words: "a beaker of blue solution", badges: [],
};
const phGap: SceneVessel = { ...base, liquid: { ...base.liquid!, optical_ph_gaps: ["methyl_orange"] } };
const draw = (vessel: SceneVessel): string => render(Vessel, { props: { vessel, register: "lv2", effects: [] } }).body;
afterEach(() => { i18n.locale = "en"; });

describe("known indicator spectra with a missing pH prerequisite", () => {
  it("does not hatch complete or older scene data", () => {
    const html = draw(base);
    expect(html).not.toMatch(/fill="url\(#vgap-2\)"/);
    expect(html).not.toContain("optical-ph-gap-readout");
  });

  it("marks the liquid and names the actual missing prerequisite", () => {
    const html = draw(phGap);
    expect(html).toMatch(/class="spectral-gap-hatch[^>]*fill="url\(#vgap-2\)"/);
    expect(html).toContain('data-ph-gaps="methyl_orange"');
    expect(html).toContain("pH is missing or invalid");
    expect(html).toContain("optical-ph-gap-readout");
    expect(html).not.toContain("no absorption spectrum for");
  });

  it("retains the computed partial colour rather than inventing a tint", () => {
    const colour = (html: string) => html.match(/class="scene-liquid-layer[^>]*fill="([^"]+)"/)?.[1];
    expect(colour(draw(phGap))).toBe(colour(draw(base)));
    expect(colour(draw(phGap))).toBe("rgb(120,150,210)");
  });

  it("keeps spectrum and pH gaps distinct while painting one coverage hatch", () => {
    const html = draw({ ...phGap, liquid: { ...phGap.liquid!, spectral_gaps: ["Cu(NH3)4+2"] } });
    expect(html).toContain('data-gaps="Cu(NH3)4+2"');
    expect(html).toContain('data-ph-gaps="methyl_orange"');
    expect(html).toContain("no absorption spectrum for");
    expect(html).toContain("pH is missing or invalid");
    expect(html.match(/class="spectral-gap-hatch/g)).toHaveLength(1);
  });

  it.each(["de", "fr"] as const)("localizes the pH explanation in %s", (locale) => {
    i18n.locale = locale;
    const html = draw(phGap);
    expect(html).not.toContain("pH is missing or invalid");
    expect(html).not.toContain("colour needs pH");
    expect(html).toContain(locale === "de" ? "gültiger pH-Wert" : "absent ou invalide");
  });

  it("keeps complete placeholder-compatible catalogue entries", () => {
    const keys = ["the computed colour is incomplete: pH is missing or invalid for {species}", "colour needs pH"];
    for (const locale of ["de", "fr", "_template"]) {
      const catalog = JSON.parse(readFileSync(new URL(`../../locales/${locale}.json`, import.meta.url), "utf8"));
      for (const key of keys) {
        const message = catalog.messages[key];
        if (locale === "_template") expect(message).toBe("");
        else {
          expect(message.length).toBeGreaterThan(0);
          expect((message.match(/\{species\}/g) ?? []).length).toBe(key.includes("{species}") ? 1 : 0);
        }
      }
    }
  });
});
