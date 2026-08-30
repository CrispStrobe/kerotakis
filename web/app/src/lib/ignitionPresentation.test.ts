import { describe, expect, it } from "vitest";
import type { Effect } from "./magnitudes";
import { liveIgnitionEffect } from "./ignitionPresentation";

const effect = (kind: string, at: number, durationMs?: number): Effect => ({
  kind,
  at,
  durationMs,
  magnitude: 0.5,
});

describe("authoritative ignition presentation", () => {
  it("selects the latest live ignite and ignores unrelated hot presentation", () => {
    const effects = [effect("heat", 9_900), effect("ignite", 9_800), effect("ignite", 9_900)];
    expect(liveIgnitionEffect(effects, 10_000)).toBe(effects[2]);
    expect(liveIgnitionEffect([effect("heat", 9_900)], 10_000)).toBeUndefined();
  });

  it("keeps flame tests separate and expires ignite deterministically", () => {
    expect(liveIgnitionEffect([effect("flame_test", 9_900)], 10_000)).toBeUndefined();
    expect(liveIgnitionEffect([effect("ignite", 7_000)], 10_000)).toBeUndefined();
    expect(liveIgnitionEffect([effect("ignite", 7_001)], 10_000)?.kind).toBe("ignite");
  });

  it("honours bounded event lifetime and rejects future events", () => {
    const short = effect("ignite", 9_500, 400);
    expect(liveIgnitionEffect([short], 10_000)).toBeUndefined();
    expect(liveIgnitionEffect([effect("ignite", 10_001)], 10_000)).toBeUndefined();
  });
});
