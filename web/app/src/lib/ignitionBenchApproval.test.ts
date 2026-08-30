import { describe, expect, it } from "vitest";

import type { Effect } from "./magnitudes";
import { benchIgnitionApproved, liveIgnitionVessels } from "./ignitionBenchApproval";
import { selectVisualBackend } from "./visualBackend";

const effect = (kind: string, at: number, durationMs?: number): Effect => ({
  kind,
  at,
  durationMs,
  magnitude: 0.5,
});

describe("bench ignition GPU approval", () => {
  it("approves one shared device for simultaneous live vessels in stable order", () => {
    const effects = {
      8: [effect("ignite", 9_900)],
      2: [effect("ignite", 9_800)],
      5: [effect("heat", 9_950)],
    };
    expect(liveIgnitionVessels(effects, 10_000)).toEqual([2, 8]);
    expect(benchIgnitionApproved(effects, 10_000)).toBe(true);
  });

  it("withdraws approval exactly when the final ignition expires", () => {
    const effects = {
      0: [effect("ignite", 9_000, 1_000)],
      1: [effect("ignite", 9_500, 400)],
    };
    expect(benchIgnitionApproved(effects, 9_999)).toBe(true);
    expect(liveIgnitionVessels(effects, 10_000)).toEqual([]);
    expect(benchIgnitionApproved(effects, 10_000)).toBe(false);
  });

  it("cannot be approved by heat, temperature-like data, or a flame test", () => {
    const effects = {
      0: [effect("heat", 9_999)],
      1: [effect("flame_test", 9_999)],
    };
    expect(benchIgnitionApproved(effects, 10_000)).toBe(false);
    expect(liveIgnitionVessels(effects, 10_000)).toEqual([]);
  });

  it("is SSR-pure and headless policy still overrides valid approval", () => {
    const approved = benchIgnitionApproved({ 0: [effect("ignite", 9_999)] }, 10_000);
    expect(approved).toBe(true);
    expect(selectVisualBackend({
      effectApproved: approved,
      webGpuAvailable: true,
      deviceHealthy: true,
      reducedMotion: false,
      headless: true,
      backgrounded: false,
    })).toEqual({ backend: "lightweight", reason: "headless" });
    expect(benchIgnitionApproved({}, 10_000)).toBe(false);
  });
});
