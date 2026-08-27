import { afterEach, describe, expect, it } from "vitest";
import { engineText } from "./engineText";
import { i18n } from "./i18n.svelte";

afterEach(() => i18n.setLocale("en"));

describe("engine text localization", () => {
  it("keeps the canonical engine prose in English", () => {
    i18n.setLocale("en");
    expect(engineText("You add water to v1.")).toBe("You add water to v1.");
  });

  it("translates additions including domain vocabulary", () => {
    i18n.setLocale("de");
    expect(engineText("You add water to v1.")).toBe("Du gibst Wasser in v1.");
    expect(engineText("v1: +0.0312 mol sulfur — 0.0624 mol now in vessel"))
      .toBe("v1: +0.0312 mol Schwefel — jetzt 0.0624 mol im Gefäß");
  });

  it("translates computed stirring and solvent observations", () => {
    i18n.setLocale("de");
    expect(engineText("v4: magnetic stirrer 500 rpm for 10 s — bar tip 0.262 m/s; 81% resuspension"))
      .toBe("v4: Magnetrührer 500 U/min für 10 s — Rührstabspitze 0.262 m/s; 81 % wieder aufgeschwemmt");
    expect(engineText("v1: sulfur in ethanol — 0.0021 mol dissolved (handbook limit), 0.0291 mol left as solid"))
      .toBe("v1: Schwefel in Ethanol — 0.0021 mol gelöst (Handbuchgrenze), 0.0291 mol bleiben als Feststoff");
  });

  it("translates applied irradiation without hiding the uncoupled model boundary", () => {
    i18n.setLocale("de");
    expect(engineText("The lamp shines on v1. The light is applied, but photolysis is not connected yet."))
      .toBe("Die Lampe bestrahlt v1. Das Licht wirkt ein, aber die Photolyse ist noch nicht gekoppelt.");
    expect(engineText("v1: lamp 254 nm at 12.50 W/m² — photolysis not yet coupled"))
      .toBe("v1: Lampe 254 nm bei 12.50 W/m² — Photolyse noch nicht gekoppelt");
    expect(engineText("v1: irradiate λ=254.000 nm, Ė/A=12.500000 W/m²; photolysis_coupled=false"))
      .toBe("v1: Bestrahlung λ=254.000 nm, Ė/A=12.500000 W/m²; Photolyse gekoppelt=nein");
  });
});
