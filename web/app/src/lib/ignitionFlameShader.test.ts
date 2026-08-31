import { describe, expect, it } from "vitest";
import type { IgnitionFlameUniforms } from "./ignitionFlameUniforms";
import {
  IGNITION_FLAME_UNIFORM_BYTES,
  IGNITION_FLAME_UNIFORM_FLOATS,
  IGNITION_FLAME_MAX_DIMENSION,
  IGNITION_FLAME_TIME_PERIOD_SECONDS,
  IGNITION_FLAME_WGSL,
  writeIgnitionFlameUniformBuffer,
} from "./ignitionFlameShader";

const active: IgnitionFlameUniforms = {
  active: true,
  intensity: 0.625,
  colour: [1, 140 / 255, 0],
  seed: 0.25,
};

function sourceFingerprint(source: string): string {
  let hash = 0x811c9dc5;
  for (let index = 0; index < source.length; index += 1) {
    hash ^= source.charCodeAt(index);
    hash = Math.imul(hash, 0x01000193);
  }
  return (hash >>> 0).toString(16).padStart(8, "0");
}

describe("ignition flame WGSL", () => {
  it("keeps a reviewed deterministic project-owned source", () => {
    expect(sourceFingerprint(IGNITION_FLAME_WGSL)).toBe("9fa6d80f");
    expect(IGNITION_FLAME_WGSL).toContain("Nguyen, Fedkiw & Jensen");
    expect(IGNITION_FLAME_WGSL).toContain("Independent Kerotakis implementation");
    expect(IGNITION_FLAME_WGSL).toContain("@vertex");
    expect(IGNITION_FLAME_WGSL).toContain("@fragment");
    expect(IGNITION_FLAME_WGSL).toContain("@group(0) @binding(0)");
    expect(IGNITION_FLAME_WGSL).toContain("clamp(colour * strength");
    expect(IGNITION_FLAME_WGSL).not.toMatch(/textureSample|storage|atomic|workgroup/);
  });

  it("packs the documented 32-byte WGSL layout exactly", () => {
    expect(IGNITION_FLAME_UNIFORM_FLOATS).toBe(8);
    expect(IGNITION_FLAME_UNIFORM_BYTES).toBe(32);
    const buffer = new Float32Array(IGNITION_FLAME_UNIFORM_FLOATS);
    writeIgnitionFlameUniformBuffer(buffer, active, 1.5, 160, 96);
    expect([...buffer]).toEqual([160, 96, 1.5, 0.625, 1, Math.fround(140 / 255), 0, 0.25]);
  });

  it("writes repeatedly into caller-owned storage and disables inactive flames", () => {
    const buffer = new Float32Array(IGNITION_FLAME_UNIFORM_FLOATS);
    writeIgnitionFlameUniformBuffer(buffer, active, 1, 80, 40);
    const sameBuffer = buffer;
    writeIgnitionFlameUniformBuffer(buffer, { ...active, active: false }, 2, 80, 40);
    expect(buffer).toBe(sameBuffer);
    expect(buffer[3]).toBe(0);
  });

  it("bounds malformed presentation values before they cross the GPU boundary", () => {
    const buffer = new Float32Array(IGNITION_FLAME_UNIFORM_FLOATS);
    writeIgnitionFlameUniformBuffer(
      buffer,
      { active: true, intensity: Number.POSITIVE_INFINITY, colour: [-1, Number.NaN, 4], seed: -2 },
      Number.NaN,
      0,
      Number.NEGATIVE_INFINITY,
    );
    expect([...buffer]).toEqual([1, 1, 0, 0, 0, 0, 1, 0]);
    expect(() => writeIgnitionFlameUniformBuffer(new Float32Array(7), active, 0, 1, 1)).toThrow(RangeError);
  });

  it("caps canvas dimensions and wraps huge finite animation times", () => {
    const buffer = new Float32Array(IGNITION_FLAME_UNIFORM_FLOATS);
    writeIgnitionFlameUniformBuffer(buffer, active, Number.MAX_VALUE, Number.MAX_VALUE, 0.25);
    expect(buffer[0]).toBe(IGNITION_FLAME_MAX_DIMENSION);
    expect(buffer[1]).toBe(1);
    expect(buffer[2]).toBeGreaterThanOrEqual(0);
    expect(buffer[2]).toBeLessThan(IGNITION_FLAME_TIME_PERIOD_SECONDS);
    expect([...buffer].every(Number.isFinite)).toBe(true);
  });

  it("is deterministic and leaves an oversized caller buffer tail untouched", () => {
    const first = new Float32Array(10).fill(99);
    const second = new Float32Array(10).fill(99);
    writeIgnitionFlameUniformBuffer(first, active, 4.25, 48, 56);
    writeIgnitionFlameUniformBuffer(second, active, 4.25, 48, 56);
    expect(first).toEqual(second);
    expect([...first.slice(8)]).toEqual([99, 99]);
  });
});
