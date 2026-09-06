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

  it("stops both computed motions when reduced motion is requested", () => {
    const source = readFileSync(new URL("./Vessel.svelte", import.meta.url), "utf8");
    const reduced = source.slice(source.indexOf("@media (prefers-reduced-motion: reduce)"));
    expect(reduced).toMatch(/\.bubble,[\s\S]*\.foam-state,[\s\S]*animation: none/);
  });
});
