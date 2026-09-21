import { readFileSync } from "node:fs";
import { describe, expect, it } from "vitest";
import { render } from "svelte/server";
import type { SceneVessel } from "../host/EngineHost";
import Vessel from "./Vessel.svelte";

/* GUI-119. `Appearance.spectral_gaps` is the engine saying which species it
 * could not give a colour to. It reached the prose and not the picture, so
 * the drawn vessel painted a confident colour over an admitted gap.
 *
 * Both directions, always: a whole colour must carry no mark, and the same
 * vessel with a gap must carry one. A test that only checked the marked
 * case would pass against a vessel that hatched everything. */

const base: SceneVessel = {
  id: 2,
  label: "beaker",
  liquid: { volume_l: 0.25, srgb: [120, 150, 210], colour_word: "blue", cloudiness: 0, path_length_cm: 4 },
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

const gapped: SceneVessel = {
  ...base,
  liquid: { ...base.liquid!, spectral_gaps: ["Cu(NH3)4+2", "Fe(SCN)+2"] },
};

const draw = (vessel: SceneVessel): string =>
  render(Vessel, { props: { vessel, register: "lv2", effects: [] } }).body;

const source = readFileSync(new URL("./Vessel.svelte", import.meta.url), "utf8");

describe("a colour the model could not finish is drawn as unfinished", () => {
  // The precondition, said out loud: a mark measured on a vessel with no
  // liquid column measures nothing.
  it("draws a liquid column for the mark to sit on", () => {
    expect(draw(gapped)).toContain('class="scene-liquid-layer');
  });

  it("leaves a colour the engine computed whole unmarked", () => {
    const html = draw(base);
    expect(html).not.toContain("spectral-gap");
    // The hatch is DEFINED on every vessel — a `<defs>` entry paints
    // nothing, and a browser can only measure a pattern that exists — so
    // the absence that matters is the absence of anything PAINTED with it.
    expect(html).toContain("vgap-2");
    expect(html).not.toMatch(/fill="url\(#vgap-2\)"/);
  });

  it("marks the liquid when the engine reported a gap in its colour", () => {
    const html = draw(gapped);
    expect(html).toContain('class="spectral-gap ');
    expect(html).toMatch(/class="spectral-gap-hatch[^>]*fill="url\(#vgap-2\)"/);
  });

  it("names what is missing rather than only that something is", () => {
    const html = draw(gapped);
    expect(html).toContain('data-gaps="Cu(NH3)4+2,Fe(SCN)+2"');
  });

  it("says it in the caption the reader already reads, not only on hover", () => {
    const html = draw(gapped);
    expect(html).toContain('class="persistent-readout spectral-gap-readout ');
    expect(draw(base)).not.toContain("spectral-gap-readout");
  });

  it("treats an engine that predates the field as nothing known to be missing", () => {
    const { spectral_gaps, ...older } = gapped.liquid!;
    expect(spectral_gaps).toHaveLength(2);
    expect(draw({ ...gapped, liquid: older })).not.toContain("spectral-gap");
  });

  it("does not repaint the liquid it marks", () => {
    const colourOf = (html: string) => html.match(/class="scene-liquid-layer[^>]*fill="([^"]+)"/)?.[1];
    expect(colourOf(draw(gapped))).toBe(colourOf(draw(base)));
    expect(colourOf(draw(gapped))).toBe("rgb(120,150,210)");
  });
});

describe("the mark is coverage, not chemistry and not an alarm", () => {
  const pattern = source.slice(source.indexOf("id={`vgap-"), source.indexOf("</pattern>"));

  it("invents no colour for the species it could not compute", () => {
    // Anything hue-bearing inside the hatch would be the engine's declined
    // claim, made by the renderer instead.
    expect(pattern).not.toMatch(/#[0-9a-fA-F]{3,6}/);
    expect(pattern).not.toMatch(/\brgb\(/);
    expect(pattern).not.toMatch(/fill=/);
  });

  it("takes both hatch strokes from theme tokens, so it reads on every bench", () => {
    for (const [name, token] of [["gap-hatch-under", "glass-depth"], ["gap-hatch-over", "glass-specular"]]) {
      expect(source).toMatch(new RegExp(`\\.${name}\\s*\\{\\s*stroke: var\\(--${token}\\)`));
    }
  });

  it("tiles in the fixed viewBox's own units, so 64 px and 257 px are one drawing", () => {
    expect(pattern).toContain('patternUnits="userSpaceOnUse"');
  });

  it("uses no warning colour and no alarm animation", () => {
    const style = source.slice(source.indexOf(".gap-hatch-under"), source.indexOf(".turbidity-veil"));
    expect(style).not.toContain("--danger");
    expect(style).not.toContain("--warning");
    expect(style).not.toContain("animation");
  });
});

describe("every sentence the mark says has a catalogue row", () => {
  const de = JSON.parse(readFileSync(new URL("../../locales/de.json", import.meta.url), "utf8"));
  const template = JSON.parse(readFileSync(new URL("../../locales/_template.json", import.meta.url), "utf8"));
  const keys = [
    "the computed colour is incomplete: no absorption spectrum for {species}",
    "colour incomplete",
    "The hatching on the liquid marks the part of the colour the model could not compute. It says something about the model, not about the liquid.",
  ];

  it("is in the component, so a new language is one json and no code", () => {
    for (const key of keys) expect(source).toContain(key);
  });

  it("is translated in de.json", () => {
    for (const key of keys) expect(de.messages[key]?.length ?? 0).toBeGreaterThan(0);
  });

  it("has an empty row waiting in the template", () => {
    for (const key of keys) expect(template.messages[key]).toBe("");
  });
});
