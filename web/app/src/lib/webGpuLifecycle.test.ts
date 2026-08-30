import { describe, expect, it, vi } from "vitest";
import {
  browserWebGpuProvider,
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
