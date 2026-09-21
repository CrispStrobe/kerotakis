import { readFileSync } from "node:fs";
import { describe, expect, it } from "vitest";
import { render } from "svelte/server";
import type { SceneVessel } from "../host/EngineHost";
import Vessel from "./Vessel.svelte";

const base: SceneVessel = {
  id: 3,
  label: "beaker",
  liquid: { volume_l: 0.25, srgb: [180, 210, 240], colour_word: "blue", cloudiness: 0, path_length_cm: 4 },
  solids: [],
  bubbling: false,
  boundary: "open",
  temperature_k: 298.15,
  pressure_pa: 101325,
  elapsed_s: 0,
  mass_g: 250,
  words: "a beaker of blue solution",
  badges: [],
};

const deposit: SceneVessel["solids"] = [{
  species: "AgCl",
  name: "silver chloride",
  moles: 0.02,
  volume_l: 1.1e-4,
  srgb: [242, 242, 238],
  colour_word: "white",
  metallic: false,
  settled_fraction: 1,
}];

const draw = (vessel: SceneVessel): string =>
  render(Vessel, { props: { vessel, register: "lv2", effects: [] } }).body;

const source = readFileSync(new URL("./Vessel.svelte", import.meta.url), "utf8");

describe("turbidity is the engine's number, drawn", () => {
  // Preconditions first, each named: a veil measured on a vessel that has
  // no liquid or no deposit measures nothing at all.
  it("draws a liquid column to carry any of this", () => {
    expect(draw(base)).toContain('class="scene-liquid-layer');
  });

  it("draws a deposit when one is present", () => {
    expect(draw({ ...base, solids: deposit })).toContain('class="scene-solid');
  });

  it("draws no suspension wash when the engine computed a clear liquid", () => {
    expect(draw(base)).not.toContain("turbidity-veil");
  });

  it("carries the computed cloudiness into the wash it draws", () => {
    const html = draw({ ...base, liquid: { ...base.liquid!, cloudiness: 0.4 } });
    expect(html).toContain('data-cloudiness="0.4000"');
    expect(html).toContain('opacity="0.34"');
  });

  it("scales the wash with the cloudiness rather than drawing one fixed haze", () => {
    const faint = draw({ ...base, liquid: { ...base.liquid!, cloudiness: 0.1 } });
    const milky = draw({ ...base, liquid: { ...base.liquid!, cloudiness: 0.9 } });
    const opacityOf = (html: string) => Number(html.match(/class="turbidity-veil[^>]*opacity="([\d.]+)"/)?.[1]);
    expect(opacityOf(faint)).toBeGreaterThan(0);
    expect(opacityOf(milky)).toBeGreaterThan(opacityOf(faint) * 5);
  });

  it("veils the deposit that the suspension is standing over", () => {
    const html = draw({ ...base, solids: deposit, liquid: { ...base.liquid!, cloudiness: 0.6 } });
    expect(html).toContain('data-veils-deposit="true"');
  });

  it("leaves the deposit unveiled when the liquid above it is clear", () => {
    expect(draw({ ...base, solids: deposit })).not.toContain("data-veils-deposit");
  });
});

describe("glass optics survive both themes and the larger vessel", () => {
  it("takes its glass colours from theme tokens, never a fixed hex", () => {
    const glass = source.slice(source.indexOf("vglass-"), source.lastIndexOf("</linearGradient>"));
    expect(glass).not.toMatch(/stop-color="#/);
    expect(source).not.toContain("#bcd6e4");
  });

  it("defines every glass stop class against a token", () => {
    for (const name of ["glass-wall", "glass-core", "glass-specular", "glass-depth"]) {
      expect(source).toMatch(new RegExp(`\\.${name}\\s*\\{\\s*stop-color: var\\(--${name}\\)`));
    }
  });

  it("samples the wall falloff closely enough to stay a curve when enlarged", () => {
    const html = draw(base);
    const gradient = html.slice(html.indexOf("vglass-"));
    const stops = gradient.slice(0, gradient.indexOf("</linearGradient>")).match(/<stop/g) ?? [];
    expect(stops.length).toBeGreaterThanOrEqual(9);
  });

  it("keeps the drawing resolution-independent: one fixed viewBox at both sizes", () => {
    // The lone vessel and the 64 px one differ only by CSS width, so every
    // gradient stop and stroke above is in the same user units at both.
    expect(source).toContain('viewBox="0 0 100 140"');
    expect(source).toMatch(/clamp\(64px/);
  });
});
