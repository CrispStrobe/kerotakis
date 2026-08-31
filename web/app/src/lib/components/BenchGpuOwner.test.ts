import { readFileSync } from "node:fs";
import { describe, expect, it } from "vitest";

describe("Bench WebGPU ownership", () => {
  const source = readFileSync(new URL("./Bench.svelte", import.meta.url), "utf8");

  it("owns one policy and passes one atomic snapshot to every vessel", () => {
    expect(source.match(/createWebGpuEnvironmentPolicy\(/g)).toHaveLength(1);
    expect(source.match(/createWebGpuMetricsRegistry\(/g)).toHaveLength(1);
    expect(source).toContain("{gpuMetricsRegistry}");
    expect(source.match(/browserGpuEnvironment\(\)/g)).toHaveLength(1);
    expect(source).toContain("gpuIgnition={gpuSnapshot}");
    expect(source).not.toContain("requestAdapter(");
    expect(source).not.toContain("requestDevice(");
  });

  it("gates acquisition on live authoritative ignition", () => {
    expect(source).toContain("benchIgnitionApproved(effects, gpuClock)");
    expect(source).toContain("gpuPolicy?.setEffectApproved(gpuApproved)");
  });
});
