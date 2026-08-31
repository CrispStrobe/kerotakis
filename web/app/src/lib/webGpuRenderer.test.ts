import { describe, expect, it, vi } from "vitest";

import type { IgnitionFlameUniforms } from "./ignitionFlameUniforms";
import type { WebGpuDeviceLike, WebGpuLifecycleState } from "./webGpuLifecycle";
import {
  browserAnimationScheduler,
  createBrowserIgnitionFlameSurface,
  createWebGpuRendererAdapter,
  type AnimationSchedulerLike,
  type WebGpuFrameSurface,
} from "./webGpuRenderer";
import { createWebGpuPresentationMetrics } from "./webGpuMetrics";

const uniforms: IgnitionFlameUniforms = {
  active: true,
  intensity: 0.75,
  colour: [1, 0.5, 0.25],
  seed: 0.125,
};
const enabled = { backend: "webgpu", reason: "enabled" } as const;
const deferred = <T>() => {
  let resolve!: (value: T) => void;
  const promise = new Promise<T>((yes) => { resolve = yes; });
  return { promise, resolve };
};

function harness(presented: boolean[] = [true]) {
  const callbacks = new Map<number, () => void>();
  let nextHandle = 1;
  const scheduler: AnimationSchedulerLike = {
    request: vi.fn((callback) => {
      const handle = nextHandle++;
      callbacks.set(handle, callback);
      return handle;
    }),
    cancel: vi.fn((handle) => { callbacks.delete(handle); }),
  };
  const writes: Float32Array[] = [];
  const surface: WebGpuFrameSurface = {
    configure: vi.fn(),
    width: () => 160,
    height: () => 96,
    writeIgnitionUniforms: vi.fn((values) => { writes.push(values); }),
    present: vi.fn(() => presented.shift() ?? true),
    reset: vi.fn(),
  };
  const fallback = vi.fn();
  const adapter = createWebGpuRendererAdapter({
    surface,
    scheduler,
    setFallbackVisible: fallback,
    nowSeconds: () => 2,
  });
  const runFrame = (): void => {
    const entry = callbacks.entries().next().value as [number, () => void] | undefined;
    if (!entry) throw new Error("no scheduled frame");
    callbacks.delete(entry[0]);
    entry[1]();
  };
  return { adapter, callbacks, fallback, runFrame, scheduler, surface, writes };
}

function readyDevice(): { lifecycle: WebGpuLifecycleState; device: WebGpuDeviceLike } {
  const device = { lost: new Promise(() => undefined) };
  return { lifecycle: { status: "ready", device }, device };
}

describe("WebGPU renderer adapter", () => {
  it("keeps SVG visible while WebGPU is absent, requesting, or headless", () => {
    const h = harness();
    expect(h.fallback).toHaveBeenCalledWith(true);
    h.adapter.sync({ status: "fallback", reason: "adapter-unavailable" }, enabled, uniforms);
    h.adapter.sync({ status: "requesting" }, enabled, uniforms);
    h.adapter.sync(readyDevice().lifecycle, { backend: "lightweight", reason: "headless" }, uniforms);
    expect(h.adapter.fallbackVisible()).toBe(true);
    expect(h.surface.configure).not.toHaveBeenCalled();
    expect(h.scheduler.request).not.toHaveBeenCalled();
  });

  it("hides SVG only after the first successfully presented GPU frame", () => {
    const h = harness([true]);
    const { lifecycle, device } = readyDevice();
    h.adapter.sync(lifecycle, enabled, uniforms);
    expect(h.surface.configure).toHaveBeenCalledWith(device);
    expect(h.adapter.fallbackVisible()).toBe(true);
    h.runFrame();
    expect(h.adapter.fallbackVisible()).toBe(false);
    expect(h.fallback.mock.calls).toEqual([[true], [false]]);
  });

  it("records bounded configure-to-present and CPU submission telemetry", () => {
    const ticks = [10, 12, 15];
    const metrics = createWebGpuPresentationMetrics({ capacity: 2, now: () => ticks.shift() ?? 15 });
    const h = harness([true]);
    const instrumented = createWebGpuRendererAdapter({
      surface: h.surface, scheduler: h.scheduler, setFallbackVisible: () => undefined,
      nowSeconds: () => 0, metrics,
    });
    instrumented.sync(readyDevice().lifecycle, enabled, uniforms);
    h.runFrame();
    expect(metrics.snapshot()).toMatchObject({
      successfulPresentations: 1, presentationFailures: 0, submittedFrames: 1,
      firstPresentationLatencyMs: 5, frameCpuSubmissionP95Ms: 3,
    });
  });

  it("contains telemetry sink failures without changing GPU presentation", () => {
    const h = harness([true]);
    const metrics = createWebGpuPresentationMetrics();
    metrics.startSession = () => { throw new Error("telemetry unavailable"); };
    metrics.startFrame = () => { throw new Error("telemetry unavailable"); };
    metrics.recordPresentationSuccess = () => { throw new Error("telemetry unavailable"); };
    const instrumented = createWebGpuRendererAdapter({
      surface: h.surface, scheduler: h.scheduler, setFallbackVisible: h.fallback,
      nowSeconds: () => 0, metrics,
    });
    expect(() => instrumented.sync(readyDevice().lifecycle, enabled, uniforms)).not.toThrow();
    expect(() => h.runFrame()).not.toThrow();
    expect(instrumented.fallbackVisible()).toBe(false);
  });

  it("fails closed and stops when presentation fails after a visible frame", () => {
    const h = harness([true, false]);
    const { lifecycle } = readyDevice();
    h.adapter.sync(lifecycle, enabled, uniforms);
    h.runFrame();
    expect(h.adapter.fallbackVisible()).toBe(false);
    h.runFrame();
    expect(h.adapter.fallbackVisible()).toBe(true);
    expect(h.callbacks.size).toBe(0);
    expect(h.surface.reset).toHaveBeenCalledTimes(1);
  });

  it("ignores a canceled callback delivered after a replacement device starts", () => {
    const h = harness();
    const first = readyDevice();
    const second = readyDevice();
    h.adapter.sync(first.lifecycle, enabled, uniforms);
    const stale = h.callbacks.values().next().value as (() => void);
    h.adapter.sync({ status: "fallback", reason: "device-lost" }, { backend: "lightweight", reason: "device-lost" }, uniforms);
    h.adapter.sync(second.lifecycle, enabled, uniforms);
    expect(h.callbacks.size).toBe(1);
    stale();
    expect(h.surface.present).not.toHaveBeenCalled();
    expect(h.callbacks.size).toBe(1);
  });

  it("restores fallback synchronously on loss and cancels queued GPU work", () => {
    const h = harness();
    const { lifecycle } = readyDevice();
    h.adapter.sync(lifecycle, enabled, uniforms);
    h.runFrame();
    expect(h.adapter.fallbackVisible()).toBe(false);

    h.adapter.sync({ status: "fallback", reason: "device-lost" }, { backend: "lightweight", reason: "device-lost" }, uniforms);
    expect(h.adapter.fallbackVisible()).toBe(true);
    expect(h.callbacks.size).toBe(0);
    expect(h.surface.reset).toHaveBeenCalledTimes(1);
    expect(h.scheduler.cancel).toHaveBeenCalledTimes(1);
  });

  it.each(["uniform write", "presentation", "scheduling"] as const)(
    "fails closed when hot-path %s throws",
    (failure) => {
      const h = harness();
      const { lifecycle } = readyDevice();
      h.adapter.sync(lifecycle, enabled, uniforms);
      if (failure === "uniform write") {
        vi.mocked(h.surface.writeIgnitionUniforms).mockImplementationOnce(() => { throw new Error("write"); });
      } else if (failure === "presentation") {
        vi.mocked(h.surface.present).mockImplementationOnce(() => { throw new Error("present"); });
      } else {
        vi.mocked(h.scheduler.request).mockImplementationOnce(() => { throw new Error("schedule"); });
      }
      h.runFrame();
      expect(h.adapter.fallbackVisible()).toBe(true);
      expect(h.callbacks.size).toBe(0);
      expect(h.surface.reset).toHaveBeenCalledTimes(1);
    },
  );

  it("reuses one uniform view and one callback across frames and updates", () => {
    const h = harness([true, true]);
    const { lifecycle } = readyDevice();
    h.adapter.sync(lifecycle, enabled, uniforms);
    h.runFrame();
    h.adapter.sync(lifecycle, enabled, { ...uniforms, intensity: 0.25 });
    h.runFrame();
    expect(h.writes).toHaveLength(2);
    expect(h.writes[0]).toBe(h.writes[1]);
    expect([...h.writes[1]!]).toEqual([160, 96, 2, 0.25, 1, 0.5, 0.25, 0.125]);
    const callbacks = (h.scheduler.request as ReturnType<typeof vi.fn>).mock.calls.map(([callback]) => callback);
    expect(new Set(callbacks).size).toBe(1);
  });

  it("is SSR safe and disposal is idempotent", () => {
    expect(browserAnimationScheduler(undefined)).toBeNull();
    expect(browserAnimationScheduler({})).toBeNull();
    const h = harness();
    h.adapter.sync(readyDevice().lifecycle, enabled, uniforms);
    h.adapter.dispose();
    h.adapter.dispose();
    expect(h.surface.reset).toHaveBeenCalledTimes(1);
    expect(h.adapter.fallbackVisible()).toBe(true);
  });

  it("does not configure or schedule an inactive flame", () => {
    const h = harness();
    h.adapter.sync(readyDevice().lifecycle, enabled, { ...uniforms, active: false });
    expect(h.surface.configure).not.toHaveBeenCalled();
    expect(h.scheduler.request).not.toHaveBeenCalled();
    expect(h.adapter.fallbackVisible()).toBe(true);
  });

  it.each(["configure", "write", "present", "request"] as const)("contains a %s failure behind SVG", (failure) => {
    const h = harness();
    if (failure === "configure") vi.mocked(h.surface.configure).mockImplementation(() => { throw new Error("configure"); });
    if (failure === "write") vi.mocked(h.surface.writeIgnitionUniforms).mockImplementation(() => { throw new Error("write"); });
    if (failure === "present") vi.mocked(h.surface.present).mockImplementation(() => { throw new Error("present"); });
    if (failure === "request") vi.mocked(h.scheduler.request).mockImplementation(() => { throw new Error("request"); });
    expect(() => h.adapter.sync(readyDevice().lifecycle, enabled, uniforms)).not.toThrow();
    if (failure === "write" || failure === "present") expect(() => h.runFrame()).not.toThrow();
    expect(h.adapter.fallbackVisible()).toBe(true);
    expect(h.callbacks.size).toBe(0);
  });

  it("compiles, submits, and presents through the structural browser surface", async () => {
    const submit = vi.fn();
    const writeBuffer = vi.fn();
    const draw = vi.fn();
    const compilation = vi.fn(async () => ({ messages: [{ type: "warning", message: "portable warning" }] }));
    const pass = { setPipeline: vi.fn(), setBindGroup: vi.fn(), draw, end: vi.fn() };
    const encoder = { beginRenderPass: vi.fn(() => pass), finish: vi.fn(() => "commands") };
    const createRenderPipeline = vi.fn(() => ({ getBindGroupLayout: () => "layout" }));
    const device = {
      lost: new Promise(() => undefined),
      createShaderModule: vi.fn(() => ({ getCompilationInfo: compilation })),
      createBuffer: vi.fn(() => "buffer"),
      createRenderPipeline,
      createBindGroup: vi.fn(() => "group"),
      createCommandEncoder: vi.fn(() => encoder),
      queue: { writeBuffer, submit },
    };
    const context = {
      configure: vi.fn(),
      getCurrentTexture: vi.fn(() => ({ createView: () => "view" })),
    };
    const surface = createBrowserIgnitionFlameSurface(
      { width: 80, height: 48, getContext: () => context },
      "bgra8unorm",
    );
    await surface.configure(device);
    surface.writeIgnitionUniforms(new Float32Array([80, 48, 1, 0.5, 1, 0.5, 0, 0.25]));
    expect(surface.present()).toBe(true);
    expect(compilation).toHaveBeenCalledTimes(1);
    expect(writeBuffer).toHaveBeenCalledTimes(1);
    expect(draw).toHaveBeenCalledWith(3);
    expect(submit).toHaveBeenCalledWith(["commands"]);
    expect((createRenderPipeline.mock.calls as unknown[][])[0]![0]).toMatchObject({
      fragment: { targets: [{ blend: { color: { srcFactor: "one" } } }] },
    });
  });

  it.each([undefined, 42] as const)(
    "rejects a %s compilation-info gate before allocating GPU resources",
    async (getCompilationInfo) => {
      const createBuffer = vi.fn();
      const createRenderPipeline = vi.fn();
      const configure = vi.fn();
      const surface = createBrowserIgnitionFlameSurface(
        { width: 48, height: 56, getContext: () => ({ configure, getCurrentTexture: vi.fn() as never }) },
        "bgra8unorm",
      );
      const module = { getCompilationInfo };
      const device = {
        lost: new Promise(() => undefined),
        createShaderModule: () => module,
        createBuffer,
        createRenderPipeline,
      };
      await expect(surface.configure(device as WebGpuDeviceLike)).rejects.toThrow(
        "compilation information is unavailable",
      );
      expect(createBuffer).not.toHaveBeenCalled();
      expect(createRenderPipeline).not.toHaveBeenCalled();
      expect(configure).not.toHaveBeenCalled();
      expect(surface.present()).toBe(false);
    },
  );

  it("keeps SVG fallback when the browser compiler gate is missing", async () => {
    const surface = createBrowserIgnitionFlameSurface(
      { width: 48, height: 56, getContext: () => ({ configure: vi.fn(), getCurrentTexture: vi.fn() as never }) },
      "bgra8unorm",
    );
    const fallback = vi.fn();
    const request = vi.fn(() => 1);
    const adapter = createWebGpuRendererAdapter({
      surface,
      scheduler: { request, cancel: vi.fn() },
      setFallbackVisible: fallback,
      nowSeconds: () => 0,
    });
    const device = {
      lost: new Promise(() => undefined),
      createShaderModule: () => ({}),
    };
    adapter.sync({ status: "ready", device }, enabled, uniforms);
    await Promise.resolve();
    await Promise.resolve();
    expect(adapter.fallbackVisible()).toBe(true);
    expect(fallback).toHaveBeenCalledTimes(1);
    expect(request).not.toHaveBeenCalled();
  });

  it("rejects shader compilation errors before a frame can be presented", async () => {
    const surface = createBrowserIgnitionFlameSurface(
      { width: 1, height: 1, getContext: () => ({ configure: vi.fn(), getCurrentTexture: vi.fn() as never }) },
      "bgra8unorm",
    );
    const device = {
      lost: new Promise(() => undefined),
      createShaderModule: () => ({ getCompilationInfo: async () => ({ messages: [{ type: "error", message: "bad WGSL" }] }) }),
    };
    await expect(surface.configure(device as WebGpuDeviceLike)).rejects.toThrow("bad WGSL");
    expect(surface.present()).toBe(false);
  });

  it("cannot let stale asynchronous configuration replace the latest device", async () => {
    const firstInfo = deferred<{ messages: never[] }>();
    const firstPipeline = vi.fn();
    const first = {
      lost: new Promise(() => undefined),
      createShaderModule: () => ({ getCompilationInfo: () => firstInfo.promise }),
      createRenderPipeline: firstPipeline,
    };
    const secondPipeline = vi.fn(() => ({ getBindGroupLayout: () => "layout" }));
    const secondSubmit = vi.fn();
    const second = {
      lost: new Promise(() => undefined),
      createShaderModule: () => ({ getCompilationInfo: async () => ({ messages: [] }) }),
      createBuffer: () => ({ destroy: vi.fn() }),
      createRenderPipeline: secondPipeline,
      createBindGroup: () => "group",
      createCommandEncoder: () => ({
        beginRenderPass: () => ({ setPipeline() {}, setBindGroup() {}, draw() {}, end() {} }),
        finish: () => "second-commands",
      }),
      queue: { writeBuffer() {}, submit: secondSubmit },
    };
    const context = { configure: vi.fn(), getCurrentTexture: () => ({ createView: () => "view" }) };
    const surface = createBrowserIgnitionFlameSurface(
      { width: 4, height: 4, getContext: () => context },
      "bgra8unorm",
    );
    const stale = surface.configure(first as WebGpuDeviceLike);
    await surface.configure(second);
    firstInfo.resolve({ messages: [] });
    await stale;
    expect(firstPipeline).not.toHaveBeenCalled();
    expect(secondPipeline).toHaveBeenCalledTimes(1);
    expect(surface.present()).toBe(true);
    expect(secondSubmit).toHaveBeenCalledWith(["second-commands"]);
  });

  it("destroys partial surface resources when pipeline creation fails", async () => {
    const destroy = vi.fn();
    const unconfigure = vi.fn();
    const surface = createBrowserIgnitionFlameSurface(
      {
        width: 48,
        height: 56,
        getContext: () => ({ configure: vi.fn(), unconfigure, getCurrentTexture: vi.fn() as never }),
      },
      "bgra8unorm",
    );
    const device = {
      lost: new Promise(() => undefined),
      createShaderModule: () => ({ getCompilationInfo: async () => ({ messages: [] }) }),
      createBuffer: () => ({ destroy }),
      createRenderPipeline: () => { throw new Error("pipeline"); },
    };
    await expect(surface.configure(device as WebGpuDeviceLike)).rejects.toThrow("pipeline");
    expect(destroy).toHaveBeenCalledTimes(1);
    expect(surface.present()).toBe(false);
    surface.reset();
    expect(unconfigure).toHaveBeenCalledTimes(1);
  });
});
