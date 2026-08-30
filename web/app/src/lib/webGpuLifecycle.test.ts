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
});
