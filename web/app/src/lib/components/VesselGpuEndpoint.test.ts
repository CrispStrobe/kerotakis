import { readFileSync } from "node:fs";
import { describe, expect, it } from "vitest";

describe("Vessel ignition GPU endpoint", () => {
  const source = readFileSync(new URL("./Vessel.svelte", import.meta.url), "utf8");

  it("mounts only the latest live ignite behind an optional atomic snapshot", () => {
    expect(source).toContain("gpuIgnition?: WebGpuEnvironmentSnapshot | null");
    expect(source).toContain("{#if ignitionEffect && gpuIgnition}");
    expect(source).toContain("effect={ignitionEffect}");
    expect(source).not.toContain("effect={flameTestEffect}");
  });

  it("does not infer combustion from heat and keeps SVG until GPU presentation", () => {
    expect(source).toContain("const burning = $derived(ignitionEffect !== undefined)");
    expect(source).not.toContain("temperature_k > 600");
    expect(source).toContain("burning && ignitionFallbackVisible");
    expect(source).toContain("{#if ignitionEffect && gpuIgnition}");
    expect(source).toContain("onfallbackchange={(visible) => (ignitionFallbackVisible = visible)}");
  });

  it("does not claim pointer or accessibility ownership", () => {
    expect(source).toContain('class="ignition-flame-gpu" aria-hidden="true"');
    expect(source).toMatch(/\.ignition-flame-gpu\s*\{[^}]*pointer-events: none;/s);
    expect(source).not.toMatch(/class="ignition-flame-gpu"[^>]*tabindex/);
  });
});
