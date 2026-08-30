import { describe, expect, it, vi } from "vitest";
import {
  browserWebGpuProvider,
  createWebGpuEnvironmentPolicy,
  createWebGpuLifecycle,
  type WebGpuAdapterLike,
  type WebGpuDeviceLike,
} from "./webGpuLifecycle";

const deferred = <T>() => {
  let resolve!: (value: T) => void;
  let reject!: (reason?: unknown) => void;
  const promise = new Promise<T>((yes, no) => { resolve = yes; reject = no; });
  return { promise, resolve, reject };
};

const device = () => {
  const lost = deferred<unknown>();
  const destroy = vi.fn();
  return { value: { lost: lost.promise, destroy } satisfies WebGpuDeviceLike, lost, destroy };
};

describe("WebGPU lifecycle", () => {
  it("is SSR safe and requires callable requestAdapter", () => {
    expect(browserWebGpuProvider(undefined)).toBeNull();
    expect(browserWebGpuProvider({})).toBeNull();
    expect(browserWebGpuProvider({ navigator: { gpu: {} } })).toBeNull();
    expect(browserWebGpuProvider({ navigator: { gpu: { requestAdapter: async () => null } } })).not.toBeNull();
  });

  it.each([
    ["bgra8unorm", "bgra8unorm"],
    ["rgba8unorm", "rgba8unorm"],
    ["", null],
    ["rgba16float", null],
    [42, null],
  ] as const)("resolves a bounded preferred canvas format from %j", (returned, expected) => {
    const gpu = {
      requestAdapter: async () => null,
      getPreferredCanvasFormat: vi.fn(() => returned),
    };
    const provider = browserWebGpuProvider({ navigator: { gpu } });
    expect(provider?.preferredCanvasFormat?.()).toBe(expected);
    expect(gpu.getPreferredCanvasFormat).toHaveBeenCalledWith();
  });

  it("fails closed when the preferred format resolver is absent or throws", () => {
    const absent = browserWebGpuProvider({ navigator: { gpu: { requestAdapter: async () => null } } });
    expect(absent?.preferredCanvasFormat?.()).toBeNull();
    const throwing = browserWebGpuProvider({
      navigator: { gpu: { requestAdapter: async () => null, getPreferredCanvasFormat: () => { throw new Error("format"); } } },
    });
    expect(throwing?.preferredCanvasFormat?.()).toBeNull();
    const hostile = Object.defineProperty({}, "navigator", { get: () => { throw new Error("navigator"); } });
    expect(browserWebGpuProvider(hostile)).toBeNull();
  });

  it.each([
    ["missing adapter", async () => null, "adapter-unavailable"],
    ["adapter rejection", async () => { throw new Error("adapter"); }, "adapter-request-failed"],
  ] as const)("fails closed on %s", async (_label, requestAdapter, reason) => {
    const lifecycle = createWebGpuLifecycle({ provider: { requestAdapter } });
    await expect(lifecycle.start()).resolves.toEqual({ status: "fallback", reason });
  });

  it("fails closed when device acquisition rejects", async () => {
    const adapter: WebGpuAdapterLike = { requestDevice: async () => { throw new Error("device"); } };
    const lifecycle = createWebGpuLifecycle({ provider: { requestAdapter: async () => adapter } });
    await expect(lifecycle.start()).resolves.toEqual({ status: "fallback", reason: "device-request-failed" });
  });

  it("coalesces concurrent starts and reuses a ready device", async () => {
    const acquired = device();
    const adapterRequest = deferred<WebGpuAdapterLike | null>();
    const requestAdapter = vi.fn(() => adapterRequest.promise);
    const requestDevice = vi.fn(async () => acquired.value);
    const lifecycle = createWebGpuLifecycle({ provider: { requestAdapter } });
    const first = lifecycle.start();
    const second = lifecycle.start();
    expect(first).toBe(second);
    adapterRequest.resolve({ requestDevice });
    await expect(first).resolves.toEqual({ status: "ready", device: acquired.value });
    await expect(lifecycle.start()).resolves.toEqual({ status: "ready", device: acquired.value });
    expect(requestAdapter).toHaveBeenCalledTimes(1);
    expect(requestDevice).toHaveBeenCalledTimes(1);
  });

  it("falls back immediately on device loss and destroys once", async () => {
    const acquired = device();
    const states: string[] = [];
    const lifecycle = createWebGpuLifecycle({
      provider: { requestAdapter: async () => ({ requestDevice: async () => acquired.value }) },
      onChange: (state) => states.push(state.status === "fallback" ? `${state.status}:${state.reason}` : state.status),
    });
    await lifecycle.start();
    acquired.lost.resolve(undefined);
    await acquired.value.lost;
    await Promise.resolve();
    expect(lifecycle.state()).toEqual({ status: "fallback", reason: "device-lost" });
    lifecycle.stop();
    expect(acquired.destroy).toHaveBeenCalledTimes(1);
    expect(states).toEqual(["requesting", "ready", "fallback:device-lost", "fallback:stopped"]);
  });

  it("ignores stale acquisition after stop and destroys the late device", async () => {
    const acquired = device();
    const deviceRequest = deferred<WebGpuDeviceLike>();
    const lifecycle = createWebGpuLifecycle({
      provider: { requestAdapter: async () => ({ requestDevice: () => deviceRequest.promise }) },
    });
    const starting = lifecycle.start();
    await Promise.resolve();
    lifecycle.stop();
    deviceRequest.resolve(acquired.value);
    await starting;
    expect(lifecycle.state()).toEqual({ status: "fallback", reason: "stopped" });
    expect(acquired.destroy).toHaveBeenCalledTimes(1);
    acquired.lost.resolve(undefined);
    await Promise.resolve();
    expect(acquired.destroy).toHaveBeenCalledTimes(1);
  });

  it("stop is idempotent for a ready device", async () => {
    const acquired = device();
    const lifecycle = createWebGpuLifecycle({
      provider: { requestAdapter: async () => ({ requestDevice: async () => acquired.value }) },
    });
    await lifecycle.start();
    lifecycle.stop();
    lifecycle.stop();
    expect(acquired.destroy).toHaveBeenCalledTimes(1);
    expect(lifecycle.state()).toEqual({ status: "fallback", reason: "stopped" });
  });
});

class FakeEventSource {
  listeners = new Set<() => void>();
  addCalls = 0;
  removeCalls = 0;

  addEventListener(_type: string, listener: () => void): void {
    this.addCalls += 1;
    this.listeners.add(listener);
  }

  removeEventListener(_type: string, listener: () => void): void {
    this.removeCalls += 1;
    this.listeners.delete(listener);
  }

  dispatch(): void {
    for (const listener of [...this.listeners]) listener();
  }
}

describe("dynamic WebGPU environment policy", () => {
  const environment = () => {
    const media = Object.assign(new FakeEventSource(), { matches: false });
    const document = Object.assign(new FakeEventSource(), { visibilityState: "visible" });
    return { media, document };
  };

  it("requires approval and restarts after reduced motion is disabled", async () => {
    const env = environment();
    const first = device();
    const second = device();
    const devices = [first, second];
    const requestAdapter = vi.fn(async () => ({ requestDevice: async () => devices.shift()!.value }));
    const decisions: string[] = [];
    const policy = createWebGpuEnvironmentPolicy({
      provider: { requestAdapter },
      reducedMotion: env.media,
      document: env.document,
      onChange: (decision) => decisions.push(`${decision.backend}:${decision.reason}`),
    });

    policy.start();
    expect(requestAdapter).not.toHaveBeenCalled();
    policy.setEffectApproved(true);
    await vi.waitFor(() => expect(policy.decision().backend).toBe("webgpu"));

    env.media.matches = true;
    env.media.dispatch();
    expect(policy.decision()).toEqual({ backend: "lightweight", reason: "reduced-motion" });
    expect(first.destroy).toHaveBeenCalledTimes(1);

    env.media.matches = false;
    env.media.dispatch();
    await vi.waitFor(() => expect(policy.decision().backend).toBe("webgpu"));
    expect(requestAdapter).toHaveBeenCalledTimes(2);
    expect(decisions).toEqual([
      "lightweight:device-lost",
      "webgpu:enabled",
      "lightweight:reduced-motion",
      "lightweight:device-lost",
      "webgpu:enabled",
    ]);
    policy.dispose();
  });

  it("stops in the background and only restarts while approval remains", async () => {
    const env = environment();
    const acquired = [device(), device()];
    const requestAdapter = vi.fn(async () => ({ requestDevice: async () => acquired.shift()!.value }));
    const policy = createWebGpuEnvironmentPolicy({
      provider: { requestAdapter }, reducedMotion: env.media, document: env.document, effectApproved: true,
    });
    policy.start();
    await vi.waitFor(() => expect(policy.decision().backend).toBe("webgpu"));

    env.document.visibilityState = "hidden";
    env.document.dispatch();
    expect(policy.decision()).toEqual({ backend: "lightweight", reason: "backgrounded" });
    policy.setEffectApproved(false);
    env.document.visibilityState = "visible";
    env.document.dispatch();
    expect(policy.decision()).toEqual({ backend: "lightweight", reason: "effect-not-approved" });
    expect(requestAdapter).toHaveBeenCalledTimes(1);

    policy.setEffectApproved(true);
    await vi.waitFor(() => expect(policy.decision().backend).toBe("webgpu"));
    expect(requestAdapter).toHaveBeenCalledTimes(2);
    policy.dispose();
  });

  it("cancels stale work and disposes listeners exactly once", async () => {
    const env = environment();
    const late = device();
    const deviceRequest = deferred<WebGpuDeviceLike>();
    const policy = createWebGpuEnvironmentPolicy({
      provider: { requestAdapter: async () => ({ requestDevice: () => deviceRequest.promise }) },
      reducedMotion: env.media,
      document: env.document,
      effectApproved: true,
    });
    policy.start();
    policy.start();
    await Promise.resolve();
    expect(env.media.listeners.size).toBe(1);
    expect(env.document.listeners.size).toBe(1);

    policy.dispose();
    policy.dispose();
    expect(env.media.listeners.size).toBe(0);
    expect(env.document.listeners.size).toBe(0);
    expect(env.media.addCalls).toBe(1);
    expect(env.media.removeCalls).toBe(1);
    expect(env.document.addCalls).toBe(1);
    expect(env.document.removeCalls).toBe(1);

    deviceRequest.resolve(late.value);
    await vi.waitFor(() => expect(late.destroy).toHaveBeenCalledTimes(1));
    env.media.matches = true;
    env.media.dispatch();
    expect(policy.decision()).toEqual({ backend: "lightweight", reason: "effect-not-approved" });
  });

  it("publishes atomic lifecycle and decision snapshots in transition order", async () => {
    const env = environment();
    const acquired = device();
    const snapshots: string[] = [];
    const legacy: string[] = [];
    const policy = createWebGpuEnvironmentPolicy({
      provider: { requestAdapter: async () => ({ requestDevice: async () => acquired.value }) },
      reducedMotion: env.media,
      document: env.document,
      effectApproved: true,
      onChange: (decision) => legacy.push(`${decision.backend}:${decision.reason}`),
      onSnapshot: ({ lifecycle, decision }) => snapshots.push(
        `${lifecycle.status === "fallback" ? `${lifecycle.status}:${lifecycle.reason}` : lifecycle.status}|${decision.backend}:${decision.reason}`,
      ),
    });

    expect(policy.snapshot()).toEqual({
      lifecycle: { status: "idle" },
      decision: { backend: "lightweight", reason: "effect-not-approved" },
      preferredCanvasFormat: null,
    });
    policy.start();
    await vi.waitFor(() => expect(policy.snapshot().decision.backend).toBe("webgpu"));
    acquired.lost.resolve(undefined);
    await vi.waitFor(() => expect(policy.snapshot().lifecycle).toEqual({ status: "fallback", reason: "device-lost" }));

    expect(snapshots).toEqual([
      "requesting|lightweight:device-lost",
      "ready|webgpu:enabled",
      "fallback:device-lost|lightweight:device-lost",
    ]);
    expect(legacy).toEqual([
      "lightweight:device-lost",
      "webgpu:enabled",
      "lightweight:device-lost",
    ]);
    expect(policy.decision()).toBe(policy.snapshot().decision);
    policy.dispose();
  });

  it("pins the safe preferred format into every atomic snapshot", async () => {
    const env = environment();
    const resolveFormat = vi.fn(() => "bgra8unorm");
    const policy = createWebGpuEnvironmentPolicy({
      provider: { requestAdapter: async () => null, preferredCanvasFormat: resolveFormat },
      reducedMotion: env.media,
      document: env.document,
      effectApproved: true,
    });
    expect(policy.snapshot().preferredCanvasFormat).toBe("bgra8unorm");
    policy.start();
    await vi.waitFor(() => expect(policy.snapshot().lifecycle.status).toBe("fallback"));
    expect(policy.snapshot().preferredCanvasFormat).toBe("bgra8unorm");
    expect(resolveFormat).toHaveBeenCalledTimes(1);

    const throwing = createWebGpuEnvironmentPolicy({
      provider: {
        requestAdapter: async () => null,
        preferredCanvasFormat: () => { throw new Error("format"); },
      },
      reducedMotion: env.media,
      document: env.document,
    });
    expect(throwing.snapshot().preferredCanvasFormat).toBeNull();
  });

  it("publishes constraints only with the stopped lifecycle and disposes fail-closed", async () => {
    const env = environment();
    const acquired = device();
    const observed: Array<{ lifecycle: string; decision: string }> = [];
    const policy = createWebGpuEnvironmentPolicy({
      provider: { requestAdapter: async () => ({ requestDevice: async () => acquired.value }) },
      reducedMotion: env.media,
      document: env.document,
      effectApproved: true,
      onSnapshot: ({ lifecycle, decision }) => observed.push({
        lifecycle: lifecycle.status === "fallback" ? `${lifecycle.status}:${lifecycle.reason}` : lifecycle.status,
        decision: `${decision.backend}:${decision.reason}`,
      }),
    });
    policy.start();
    await vi.waitFor(() => expect(policy.snapshot().decision.backend).toBe("webgpu"));

    env.media.matches = true;
    env.media.dispatch();
    expect(policy.snapshot()).toEqual({
      lifecycle: { status: "fallback", reason: "stopped" },
      decision: { backend: "lightweight", reason: "reduced-motion" },
      preferredCanvasFormat: null,
    });
    expect(observed.at(-1)).toEqual({
      lifecycle: "fallback:stopped",
      decision: "lightweight:reduced-motion",
    });

    policy.dispose();
    expect(policy.snapshot()).toEqual({
      lifecycle: { status: "fallback", reason: "stopped" },
      decision: { backend: "lightweight", reason: "effect-not-approved" },
      preferredCanvasFormat: null,
    });
    expect(observed.at(-1)).toEqual({
      lifecycle: "fallback:stopped",
      decision: "lightweight:effect-not-approved",
    });
  });

  it("commits snapshots even when observers throw", async () => {
    const env = environment();
    const acquired = device();
    const policy = createWebGpuEnvironmentPolicy({
      provider: { requestAdapter: async () => ({ requestDevice: async () => acquired.value }) },
      reducedMotion: env.media,
      document: env.document,
      effectApproved: true,
      onChange: () => { throw new Error("legacy observer"); },
      onSnapshot: () => { throw new Error("snapshot observer"); },
    });
    expect(() => policy.start()).not.toThrow();
    await vi.waitFor(() => expect(policy.snapshot().decision.backend).toBe("webgpu"));
    expect(policy.snapshot().lifecycle).toEqual({ status: "ready", device: acquired.value });
    expect(() => policy.dispose()).not.toThrow();
    expect(policy.snapshot()).toEqual({
      lifecycle: { status: "fallback", reason: "stopped" },
      decision: { backend: "lightweight", reason: "effect-not-approved" },
      preferredCanvasFormat: null,
    });
  });

  it("disposes to a stopped fallback even when device destruction throws", async () => {
    const env = environment();
    const lost = deferred<unknown>();
    const hostileDevice: WebGpuDeviceLike = {
      lost: lost.promise,
      destroy: () => { throw new Error("destroy"); },
    };
    const policy = createWebGpuEnvironmentPolicy({
      provider: { requestAdapter: async () => ({ requestDevice: async () => hostileDevice }) },
      reducedMotion: env.media,
      document: env.document,
      effectApproved: true,
    });
    policy.start();
    await vi.waitFor(() => expect(policy.snapshot().decision.backend).toBe("webgpu"));
    expect(() => policy.dispose()).not.toThrow();
    expect(policy.snapshot()).toEqual({
      lifecycle: { status: "fallback", reason: "stopped" },
      decision: { backend: "lightweight", reason: "effect-not-approved" },
      preferredCanvasFormat: null,
    });
  });

  it("supports disposable snapshot subscriptions without observer authority", async () => {
    const env = environment();
    const acquired = device();
    const policy = createWebGpuEnvironmentPolicy({
      provider: {
        requestAdapter: async () => ({ requestDevice: async () => acquired.value }),
        preferredCanvasFormat: () => "rgba8unorm",
      },
      reducedMotion: env.media,
      document: env.document,
      effectApproved: true,
    });
    const observed: string[] = [];
    expect(() => policy.subscribe(() => { throw new Error("observer"); })).not.toThrow();
    const unsubscribe = policy.subscribe(({ lifecycle, preferredCanvasFormat }) => {
      observed.push(`${lifecycle.status}:${preferredCanvasFormat}`);
    });
    policy.start();
    await vi.waitFor(() => expect(policy.snapshot().decision.backend).toBe("webgpu"));
    unsubscribe();
    unsubscribe();
    const beforeLoss = observed.length;
    acquired.lost.resolve(undefined);
    await vi.waitFor(() => expect(policy.snapshot().lifecycle.status).toBe("fallback"));
    expect(observed).toEqual(["idle:rgba8unorm", "requesting:rgba8unorm", "ready:rgba8unorm"]);
    expect(observed).toHaveLength(beforeLoss);
    policy.dispose();
    expect(() => policy.subscribe(() => { throw new Error("disposed observer"); })).not.toThrow();
  });
});
