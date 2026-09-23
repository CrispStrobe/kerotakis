/**
 * One window per effect, in one place (GUI-132).
 *
 * The drawing window — how long a picture stays up when the engine did
 * not supply a duration — was a numeric literal at every call site in
 * `Vessel.svelte`: 85 of them across 55 kinds. Scattered, they had no way
 * to be compared with each other, and two kinds quietly disagreed with
 * themselves: `swirl` was written 8000, 2200 and 2000, and `vent` 4000
 * and 2600.
 *
 * Reading the call sites, three of those five were one thing said three
 * ways and two were a second thing — a stir READOUT outliving the
 * stirring, and wisps ABOVE the rim being shorter than the fizz inside
 * the liquid. Both survive as named constants, because a deliberate
 * exception and drift are indistinguishable until one of them is given a
 * name.
 *
 * This guard keeps that true. It reads `Vessel.svelte`, which is the only
 * consumer, so the table cannot grow rows nobody draws and the drawing
 * cannot ask for a kind the table has never heard of.
 */
import { describe, expect, it } from "vitest";
import { readFileSync } from "node:fs";
import { join } from "node:path";
import {
  EFFECT_WINDOW_MS, INSTRUMENT_READING_MS, STIR_READOUT_MS, VENT_WISP_MS, effectWindowMs,
} from "./magnitudes";

const VESSEL = readFileSync(join(import.meta.dirname, "components/Vessel.svelte"), "utf8");

/** Every kind the drawing asks for a window for. */
const asked = (): string[] => [
  ...new Set([...VESSEL.matchAll(/\b(?:latestEffect|active|mag)\(\s*"([a-z0-9_-]+)"/g)].map((m) => m[1]!)),
].sort();

describe("the effect window table", () => {
  it("finds the call sites at all, so nothing below is vacuous", () => {
    expect(asked().length).toBeGreaterThan(40);
    expect(Object.keys(EFFECT_WINDOW_MS).length).toBeGreaterThan(40);
  });

  it("carries a window for every kind the vessel draws", () => {
    const missing = asked().filter((kind) => !(kind in EFFECT_WINDOW_MS)).sort();
    expect(missing).toEqual([]);
  });

  it("has no row nobody draws", () => {
    // A window for a kind that is never asked for is a number that can
    // rot without anyone noticing — the same failure as an exception
    // outliving its defect.
    const drawn = new Set(asked());
    // `polymer` is drawn through its own `polymerEffect` derivation
    // rather than an inline call, so it is named here rather than being
    // reported as dead.
    const reachedElsewhere = new Set(["polymer"]);
    const orphans = Object.keys(EFFECT_WINDOW_MS)
      .filter((kind) => !drawn.has(kind) && !reachedElsewhere.has(kind)).sort();
    expect(orphans).toEqual([]);
  });

  it("leaves no numeric window behind at a call site", () => {
    // THE assertion. A literal here is a number that cannot be compared
    // with the other fifty-four, which is how `swirl` came to be written
    // three ways.
    const literals = [...VESSEL.matchAll(
      /\b(?:latestEffect|active|mag)\(\s*"[a-z0-9_-]+"\s*,\s*[\d_]+\s*\)/g,
    )].map((m) => m[0]);
    expect(literals).toEqual([]);
  });

  it("keeps the instrument readings a GROUP, not nine copies of 6000", () => {
    // `INSTRUMENT_READING_MS` predates this table and is the better
    // shape: a reading is a reading, and showing the thermometer's for
    // longer than the balance's would be a claim about instruments that
    // nobody makes. The table REFERENCES it rather than copying it, so
    // the group cannot come apart one row at a time.
    const instruments = ["balance", "calorimeter", "conductivity_meter", "geiger_counter",
      "ph_probe", "pressure_gauge", "thermometer", "uvvis", "volume_meter"];
    for (const kind of instruments) {
      expect({ kind, ms: EFFECT_WINDOW_MS[kind] }).toEqual({ kind, ms: INSTRUMENT_READING_MS });
    }
    expect(VESSEL).toContain("INSTRUMENT_READING_MS");
  });

  it("names its two deliberate exceptions rather than leaving them as drift", () => {
    expect(VESSEL).toContain("STIR_READOUT_MS");
    expect(VESSEL).toContain("VENT_WISP_MS");
    // Each must actually DIFFER from its kind's window, or it is not an
    // exception and the name is a lie.
    expect(STIR_READOUT_MS).not.toBe(EFFECT_WINDOW_MS.swirl);
    expect(VENT_WISP_MS).not.toBe(EFFECT_WINDOW_MS.vent);
    // And each must be the direction its reason claims.
    expect(STIR_READOUT_MS).toBeGreaterThan(EFFECT_WINDOW_MS.swirl!);
    expect(VENT_WISP_MS).toBeLessThan(EFFECT_WINDOW_MS.vent!);
  });

  it("gives every window a plausible length", () => {
    for (const [kind, ms] of Object.entries(EFFECT_WINDOW_MS)) {
      // Under a second is a flash nobody catches; over ten is a drawing
      // still up long after the step that caused it.
      expect({ kind, ms }).toEqual({ kind, ms: expect.any(Number) });
      expect(ms).toBeGreaterThanOrEqual(1000);
      expect(ms).toBeLessThanOrEqual(10_000);
    }
  });

  it("answers for a kind it has never heard of", () => {
    // A new event's effect draws for a middling time rather than zero: no
    // window is not the same claim as no picture.
    expect(effectWindowMs("some-new-kind")).toBeGreaterThan(0);
    expect(effectWindowMs("burst")).toBe(EFFECT_WINDOW_MS.burst);
  });
});
