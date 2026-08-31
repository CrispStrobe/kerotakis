import { describe, expect, it } from "vitest";
import { readFileSync } from "node:fs";
import {
  IGNITION_FLAME_LOGICAL_HEIGHT,
  IGNITION_FLAME_LOGICAL_WIDTH,
  IGNITION_FLAME_MAX_DPR,
  ignitionFlameCanvasSize,
  type IgnitionFlameGpuSnapshot,
} from "./IgnitionFlameCanvas.svelte";

describe("bounded ignition flame canvas host", () => {
  const source = readFileSync(new URL("./IgnitionFlameCanvas.svelte", import.meta.url), "utf8");

  it("uses the fixed logical target at ordinary pixel density", () => {
    expect(ignitionFlameCanvasSize(1)).toEqual({ width: 48, height: 56 });
    expect(IGNITION_FLAME_LOGICAL_WIDTH).toBe(48);
    expect(IGNITION_FLAME_LOGICAL_HEIGHT).toBe(56);
  });

  it("caps high-density backing stores at 96 by 112 physical pixels", () => {
    expect(IGNITION_FLAME_MAX_DPR).toBe(2);
    expect(ignitionFlameCanvasSize(2)).toEqual({ width: 96, height: 112 });
    expect(ignitionFlameCanvasSize(3)).toEqual({ width: 96, height: 112 });
    expect(ignitionFlameCanvasSize(Number.POSITIVE_INFINITY)).toEqual({ width: 48, height: 56 });
  });

  it("fails malformed and sub-unit density inputs to a bounded 1x target", () => {
    expect(ignitionFlameCanvasSize(undefined)).toEqual({ width: 48, height: 56 });
    expect(ignitionFlameCanvasSize(Number.NaN)).toEqual({ width: 48, height: 56 });
    expect(ignitionFlameCanvasSize(-4)).toEqual({ width: 48, height: 56 });
  });

  it("accepts lifecycle and policy as one atomic snapshot", () => {
    const snapshot = {
      lifecycle: { status: "requesting" },
      decision: { backend: "lightweight", reason: "device-lost" },
      preferredCanvasFormat: null,
    } satisfies IgnitionFlameGpuSnapshot;
    expect(snapshot.lifecycle.status).toBe("requesting");
    expect(snapshot.decision.backend).toBe("lightweight");
  });

  it("stays outside accessibility and pointer ownership", () => {
    expect(source).toContain('aria-hidden="true"');
    expect(source).toContain("pointer-events: none");
    expect(source).not.toContain("tabindex");
    expect(source).not.toMatch(/on(?:click|pointer|key)/i);
    expect(source).not.toContain("bgra8unorm");
  });

  it("passes a disposable registry session into the renderer", () => {
    expect(source).toContain("metricsRegistry.open(vesselIdentity)");
    expect(source).toContain("metrics: metricsSession?.metrics");
    expect(source).toContain("metricsSession?.dispose()");
  });
});
