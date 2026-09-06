import { readFileSync } from "node:fs";
import { describe, expect, it } from "vitest";
import type { ComponentProps } from "svelte";
import { render } from "svelte/server";
import Vessel from "./Vessel.svelte";
import type { Effect } from "../magnitudes";
import { INSTRUMENT_READING_MS } from "../magnitudes";
import type { SceneVessel } from "../host/EngineHost";

/**
 * The instrument reading, at a size a phone can read.
 *
 * Owner, from the German deploy: clicking the thermometer shows a
 * thermometer on the vessel "too shortly and too tiny". Both halves are
 * measurable, so both are measured here — the SVG instruments live in a
 * 100-unit viewBox that `clamp(64px, 14vw, …)` gives 64 px on a 390 px
 * phone, which turns their 4.5-unit screen type into under 3 real pixels.
 */
const beaker: SceneVessel = {
  id: 1, label: "beaker", liquid: null, solids: [], bubbling: false,
  boundary: "open", temperature_k: 294.96, pressure_pa: 101325,
  elapsed_s: 0, mass_g: 104.2, words: "a beaker", badges: [],
};

function draw(effects: Effect[], props: Partial<ComponentProps<typeof Vessel>> = {}): string {
  return render(Vessel, { props: { vessel: beaker, register: "lv2", effects, ...props } }).body;
}

const reading = (kind: string, value: number, unit: string): Effect =>
  ({ kind, at: Date.now(), durationMs: INSTRUMENT_READING_MS, magnitude: 0.6, reading: value, unit });

describe("how long an instrument reading stays up", () => {
  it("is one value, long enough to read", () => {
    // 2500 ms is long enough to notice something flickered and not long
    // enough to read it. Anything under six seconds fails this on purpose.
    expect(INSTRUMENT_READING_MS).toBeGreaterThanOrEqual(6000);
  });

  it("is the value the vessel draws with and the session stamps", () => {
    // Split, these disagree and the shorter one silently wins: the drawing
    // window was 2500 ms while the effect itself lived 4000 ms, so no
    // amount of raising one alone would have fixed anything.
    const vesselSource = readSource("./Vessel.svelte");
    for (const instrument of [
      "thermometer", "ph_probe", "balance", "pressure_gauge",
      "volume_meter", "conductivity_meter", "uvvis", "calorimeter",
    ]) {
      expect([instrument, vesselSource.includes(`latestEffect("${instrument}", INSTRUMENT_READING_MS)`)])
        .toEqual([instrument, true]);
    }
    expect(readSource("../session.svelte.ts")).toContain("durationMs: INSTRUMENT_READING_MS");
  });
});

describe("the reading a phone can actually read", () => {
  it("prints the number in the page's own type, not in viewBox units", () => {
    const drawn = draw([reading("thermometer", 21.81, "°C")]);
    expect(drawn).toContain('class="instrument-readout');
    expect(drawn).toContain('data-instrument="thermometer"');
    expect(drawn).toContain('data-reading="21.8100"');
    expect(drawn).toContain("21.8");
    expect(drawn).toContain("°C");
  });

  it("sizes the glyph and the value in px, above the legibility floor", () => {
    // px, not em: this is the one place on the bench where the number has
    // to survive a 64px-wide vessel, and an em there is whatever the
    // caption's 0.78rem happens to be.
    const source = readSource("./Vessel.svelte");
    expect(source).toMatch(/\.readout-glyph \{ font-size: (2[4-9]|[3-9]\d)px;/);
    expect(source).toMatch(/\.readout-value \{ font-size: (1[1-9]|[2-9]\d)px;/);
    expect(source).toMatch(/\.readout-unit \{[^}]*font-size: (1[1-9]|[2-9]\d)px;/);
  });

  it("shows every instrument the bench can read with", () => {
    const cases: [string, Effect, string][] = [
      ["ph_probe", reading("ph_probe", 3.42, ""), "3.42"],
      ["balance", reading("balance", 104.2, "g"), "104.20"],
      ["pressure_gauge", reading("pressure_gauge", 101.3, "kPa"), "101.3"],
      ["volume_meter", reading("volume_meter", 245, "mL"), "245.0"],
      ["conductivity_meter", reading("conductivity_meter", 1280, "µS/cm"), "1,280.0"],
      ["calorimeter", reading("calorimeter", -1.42, "kJ"), "-1.42"],
      ["uvvis", reading("uvvis", 0.482, "AU"), "0.482"],
    ];
    for (const [kind, effect, expected] of cases) {
      const drawn = draw([effect]);
      expect([kind, drawn.includes(`data-instrument="${kind}"`)]).toEqual([kind, true]);
      expect([kind, drawn.includes(expected)]).toEqual([kind, true]);
    }
  });

  it("replaces the reading rather than stacking a second one", () => {
    const older = { ...reading("thermometer", 21.81, "°C"), at: Date.now() - 400 };
    const newer = { ...reading("ph_probe", 3.42, ""), at: Date.now() };
    const drawn = draw([older, newer]);
    expect(drawn).toContain('data-instrument="ph_probe"');
    expect(drawn).not.toContain('data-instrument="thermometer"');
    expect(drawn.match(/class="instrument-readout/g)?.length ?? 0).toBe(1);
  });

  it("draws nothing when no instrument is live", () => {
    expect(draw([])).not.toContain("instrument-readout");
  });

  it("keeps the reading, and its duration, under reduced motion", () => {
    // Duration is information here, not decoration: what reduced motion
    // switches off is the slide-in, not the six seconds.
    const source = readSource("./Vessel.svelte");
    const reduced = source.slice(source.indexOf("@media (prefers-reduced-motion: reduce)"));
    expect(reduced).toContain(".instrument-readout { animation: none; }");
  });
});

function readSource(relative: string): string {
  return readFileSync(new URL(relative, import.meta.url), "utf8");
}
