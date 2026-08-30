import { describe, expect, it } from "vitest";
import { hasWebGpu, selectVisualBackend, type VisualBackendCapabilities } from "./visualBackend";

const capable = (overrides: Partial<VisualBackendCapabilities> = {}): VisualBackendCapabilities => ({
  effectApproved: true,
  webGpuAvailable: true,
  deviceHealthy: true,
  reducedMotion: false,
  headless: false,
  backgrounded: false,
  ...overrides,
});

describe("optional WebGPU presentation tier", () => {
  it("requires an explicitly approved effect", () => {
    expect(selectVisualBackend(capable())).toEqual({ backend: "webgpu", reason: "enabled" });
    expect(selectVisualBackend(capable({ effectApproved: false }))).toEqual({
      backend: "lightweight",
      reason: "effect-not-approved",
    });
  });

  it.each([
    ["reducedMotion", "reduced-motion"],
    ["headless", "headless"],
    ["backgrounded", "backgrounded"],
  ] as const)("keeps the baseline when %s is active", (key, reason) => {
    expect(selectVisualBackend(capable({ [key]: true }))).toEqual({
      backend: "lightweight",
      reason,
    });
  });

  it("falls back when WebGPU is absent or its device is lost", () => {
    expect(selectVisualBackend(capable({ webGpuAvailable: false })).reason).toBe("webgpu-unavailable");
    expect(selectVisualBackend(capable({ deviceHealthy: false })).reason).toBe("device-lost");
  });

  it("detects support without requiring browser globals in tests or SSR", () => {
    expect(hasWebGpu(undefined)).toBe(false);
    expect(hasWebGpu({})).toBe(false);
    expect(hasWebGpu({ navigator: {} })).toBe(false);
    expect(hasWebGpu({ navigator: { gpu: {} } })).toBe(true);
  });
});
