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

  it("translates the engine-computed electrolysis operating point and deposit", () => {
    i18n.setLocale("de");
    expect(engineText("0.32 g of copper builds up on the electrode in v1."))
      .toBe("0.32 g Kupfer scheiden sich an der Elektrode in v1 ab.");
    expect(engineText("v1: 0.500 A for 120 s = 60 C → 0.0006 mol e⁻ → 0.0003 mol copper = 0.019 g"))
      .toBe("v1: 0.500 A für 120 s = 60 C → 0.0006 mol e⁻ → 0.0003 mol Kupfer = 0.019 g");
    expect(engineText("v1: I = 0.500000 A; t = 120.000 s; Q = It = 60.0 C; n(e⁻) = Q/F = 0.000622 mol; n(copper) = n(e⁻)/2 = 0.000311 mol; m = 0.0198 g — only the 2 is chemistry. Inert anode assumed: the water is oxidised there, so the oxygen leaves and the acid stays"))
      .toBe("v1: I = 0.500000 A; t = 120.000 s; Q = It = 60.0 C; n(e⁻) = Q/F = 0.000622 mol; n(Kupfer) = n(e⁻)/2 = 0.000311 mol; m = 0.0198 g — nur die 2 stammt aus der Chemie. Inerte Anode angenommen: Dort wird Wasser oxidiert; der Sauerstoff entweicht und die Säure bleibt zurück");
  });

  it("translates delivered heat while preserving the missing-time boundary", () => {
    i18n.setLocale("de");
    expect(engineText("v1 receives 2.50 kJ of heat. This energy step has no elapsed-time model yet."))
      .toBe("v1 nimmt 2.50 kJ Wärme auf. Dieser Energieschritt hat noch kein Zeitmodell.");
    expect(engineText("v1: 5.00 kJ requested; 2.50 kJ removed — time model not yet coupled"))
      .toBe("v1: 5.00 kJ angefordert; 2.50 kJ entzogen — Zeitmodell noch nicht gekoppelt");
    expect(engineText("v1: thermal energy requested=5000.000000 J, delivered=2500.000000 J, heating=false, time_coupled=false"))
      .toBe("v1: Wärmeenergie angefordert=5000.000000 J, übertragen=2500.000000 J, Erwärmung=nein, Zeitmodell gekoppelt=nein");
  });
});
